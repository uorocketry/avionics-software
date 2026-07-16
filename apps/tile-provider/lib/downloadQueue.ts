import { Database } from "bun:sqlite";
import { config } from "./config";
import { TileDatabase } from "./database";

export interface DownloadJob {
  id: string;
  layer: string;
  lat: number;
  lon: number;
  minZoom: number;
  maxZoom: number;
  radiusKm: number;
  status: 'pending' | 'processing' | 'completed' | 'cancelled' | 'failed';
  downloaded: number;
  total: number;
  error?: string;
}

interface TileCoordinates {
  zoom: number;
  x: number;
  y: number;
}

interface DownloadResult {
  success: boolean;
  error?: string;
  jobId?: string;
  downloaded: number;
  total: number;
}

function deg2tile(lat: number, lon: number, zoom: number): [number, number] {
  const n = 2 ** zoom;
  const xTile = Math.floor(((lon + 180) / 360) * n);
  const yTile = Math.floor(
    ((1 -
      Math.log(
        Math.tan((lat * Math.PI) / 180) + 1 / Math.cos((lat * Math.PI) / 180)
      ) /
      Math.PI) /
      2) *
    n
  );
  return [xTile, yTile];
}

function generateTileUrls(job: DownloadJob): TileCoordinates[] {
  const urls: TileCoordinates[] = [];

  for (let zoom = job.minZoom; zoom <= job.maxZoom; zoom++) {
    const [xCenter, yCenter] = deg2tile(job.lat, job.lon, zoom);
    const n = 2 ** zoom;

    // Calculate radius in tiles (approximate)
    const earthRadius = 6371; // km
    const tileSize = (2 * Math.PI * earthRadius) / n;
    const radiusTiles = Math.ceil(job.radiusKm / tileSize);

    // Calculate bounds
    const xMin = Math.max(0, xCenter - radiusTiles);
    const xMax = Math.min(n - 1, xCenter + radiusTiles);
    const yMin = Math.max(0, yCenter - radiusTiles);
    const yMax = Math.min(n - 1, yCenter + radiusTiles);

    // Generate URLs
    for (let x = xMin; x <= xMax; x++) {
      for (let y = yMin; y <= yMax; y++) {
        urls.push({ zoom, x, y });
      }
    }
  }

  return urls;
}

export class DownloadQueue {
  private jobs: Map<string, DownloadJob> = new Map();
  private isProcessing = false;
  private db: TileDatabase;
  private currentJobId: string | null = null;
  private shuttingDown = false;
  private inflightAbort: AbortController | null = null;

  constructor(db: Database) {
    this.db = new TileDatabase(db);
  }

  async queueDownload(job: Omit<DownloadJob, 'id' | 'status' | 'downloaded' | 'total'>): Promise<DownloadResult> {
    try {
      // Backend input validation and sanitization
      if (typeof job.minZoom !== 'number' || typeof job.maxZoom !== 'number') {
        return { success: false, error: "Zoom levels must be numbers", downloaded: 0, total: 0 };
      }

      const minZoom = Math.max(0, Math.min(19, Math.floor(job.minZoom)));
      const maxZoom = Math.max(0, Math.min(19, Math.floor(job.maxZoom)));

      if (minZoom > maxZoom) {
        return {
          success: false,
          error: "minZoom cannot be greater than maxZoom",
          downloaded: 0,
          total: 0
        };
      }

      if (typeof job.lat !== 'number' || isNaN(job.lat) || job.lat < -90 || job.lat > 90) {
        return { success: false, error: "Invalid latitude (-90 to 90)", downloaded: 0, total: 0 };
      }
      if (typeof job.lon !== 'number' || isNaN(job.lon) || job.lon < -180 || job.lon > 180) {
        return { success: false, error: "Invalid longitude (-180 to 180)", downloaded: 0, total: 0 };
      }
      if (typeof job.radiusKm !== 'number' || isNaN(job.radiusKm) || job.radiusKm <= 0 || job.radiusKm > 50) {
        return { success: false, error: "Invalid radius (must be between 0 and 50 km)", downloaded: 0, total: 0 };
      }

      const id = crypto.randomUUID();
      const newJob: DownloadJob = {
        ...job,
        minZoom,
        maxZoom,
        id,
        layer: "satellite & terrain",
        status: 'pending',
        downloaded: 0,
        total: 0
      };

      this.jobs.set(id, newJob);

      if (!this.isProcessing) {
        this.processQueue();
      }

      return {
        success: true,
        jobId: id,
        downloaded: 0,
        total: 0
      };
    } catch (error) {
      console.error('Error queueing download:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        downloaded: 0,
        total: 0
      };
    }
  }

  async cancelDownload(jobId: string): Promise<boolean> {
    const job = this.jobs.get(jobId);
    if (!job) return false;

    if (job.status === 'processing' && this.currentJobId === jobId) {
      job.status = 'cancelled';
      return true;
    }

    return false;
  }

  shutdown(): void {
    this.shuttingDown = true;
    if (this.inflightAbort) {
      try { this.inflightAbort.abort(); } catch {}
    }
  }

  async getJobStatus(jobId: string): Promise<DownloadJob | null> {
    return this.jobs.get(jobId) || null;
  }

  private async processQueue(): Promise<void> {
    if (this.jobs.size === 0 || this.shuttingDown) {
      this.isProcessing = false;
      return;
    }

    this.isProcessing = true;
    const job = Array.from(this.jobs.values()).find(j => j.status === 'pending');

    if (!job) {
      this.isProcessing = false;
      return;
    }

    this.currentJobId = job.id;
    job.status = 'processing';

    try {
      const urls = generateTileUrls(job);
      const targetLayers = ["satellite", "terrain"];
      const tasks: { layer: string; zoom: number; x: number; y: number }[] = [];

      // Generate task list for both layers
      for (const layer of targetLayers) {
        const existingTiles = this.db.batchCheckTiles(layer, urls.map(url => ({ ...url, blob: new Uint8Array() })));
        for (const url of urls) {
          if (!existingTiles.has(`${url.zoom}/${url.x}/${url.y}`)) {
            tasks.push({ layer, ...url });
          }
        }
      }

      job.total = tasks.length;

      console.log(`Processing download job ${job.id}: ${tasks.length} tiles to download across both layers`);

      for (const task of tasks) {
        if (job.status !== 'processing' || this.shuttingDown) {
          break;
        }

        try {
          const result = await this.fetchTileFromSource(task.layer, task.zoom, task.x, task.y);
          if (result.ok && result.buffer) {
            if (this.db.saveTile(task.layer, task.zoom, task.x, task.y, result.buffer)) {
              job.downloaded++;
            }
          }
          // Add small delay to avoid overwhelming the source; skip if shutting down
          if (this.shuttingDown) break;
          await new Promise(resolve => setTimeout(resolve, config.download.delayMs));
        } catch (error) {
          if ((error as any)?.name === 'AbortError') {
            // Graceful shutdown; mark as cancelled
            job.status = 'cancelled';
            break;
          }
          console.error(`Failed to download tile ${task.layer}/${task.zoom}/${task.x}/${task.y}:`, error);
          job.status = 'failed';
          job.error = error instanceof Error ? error.message : 'Unknown error';
          break;
        } finally {
          this.inflightAbort = null;
        }
      }

      if (job.status === 'processing') {
        job.status = 'completed';
      }
    } catch (error) {
      console.error('Error processing download job:', error);
      job.status = 'failed';
      job.error = error instanceof Error ? error.message : 'Unknown error';
    } finally {
      this.currentJobId = null;
      // Process next job
      if (!this.shuttingDown) this.processQueue();
    }
  }

  private async fetchTileFromSource(layer: string, zoom: number, x: number, y: number) {
    const baseUrl = config.tileSources[layer as keyof typeof config.tileSources];
    if (!baseUrl) {
      return { ok: false, error: `Unknown layer: ${layer}` };
    }
    const url = `${baseUrl}&x=${x}&y=${y}&z=${zoom}`;
    const controller = new AbortController();
    this.inflightAbort = controller;
    const response = await fetch(url, { signal: controller.signal });

    if (!response.ok) {
      return { ok: false, error: response.statusText };
    }

    const buffer = new Uint8Array(await response.arrayBuffer());
    return { ok: true, buffer };
  }
}
