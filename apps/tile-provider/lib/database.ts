import { Database } from "bun:sqlite";

export interface Tile {
  zoom: number;
  x: number;
  y: number;
  blob: Uint8Array;
}

export interface TileQuery {
  get: (...args: any[]) => { blob: Uint8Array } | null | undefined;
  run: (...args: any[]) => void;
  all: (...args: any[]) => any[];
}

const LAYER_MAP: Record<string, number> = {
  satellite: 0,
  terrain: 1,
};

function getLayerId(layer: string): number {
  if (layer in LAYER_MAP) {
    return LAYER_MAP[layer];
  }
  return 0; // Default to satellite (0)
}

export class TileDatabase {
  private db: Database;
  private getTileQuery!: TileQuery;
  private saveTileQuery!: TileQuery;

  constructor(db: Database) {
    this.db = db;
    // Ensure schema exists before preparing queries that depend on it
    this.initializeSchema();
    this.initializeQueries();
  }

  private initializeSchema() {
    // Check if table exists and if it has the 'layer' column with INTEGER type
    let dropTable = false;
    try {
      const info = this.db.prepare("PRAGMA table_info(tiles)").all() as any[];
      const layerCol = info.find((col: any) => col.name === 'layer');
      if (!layerCol || layerCol.type !== 'INTEGER') {
        dropTable = true;
      }
    } catch (e) {
      // Table doesn't exist yet, which is fine
    }

    if (dropTable) {
      // If table exists but has incorrect type (like TEXT), drop it to migrate primary key and type cleanly
      try {
        this.db.exec("DROP TABLE IF EXISTS tiles;");
      } catch (e) {
        console.warn('Error dropping old tiles table:', e);
      }
    }

    // Create the schema with layer as INTEGER and part of the primary key
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS tiles (
        layer INTEGER,
        zoom INTEGER,
        x INTEGER,
        y INTEGER,
        blob BLOB,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (layer, zoom, x, y)
      );
    `);

    // Create the index if it doesn't exist
    this.db.exec(`
      CREATE INDEX IF NOT EXISTS idx_tiles_created_at ON tiles(created_at);
    `);
  }

  private initializeQueries() {
    this.getTileQuery = this.db.query(
      "SELECT blob FROM tiles WHERE layer = ? AND zoom = ? AND x = ? AND y = ?"
    ) as unknown as TileQuery;

    this.saveTileQuery = this.db.prepare(
      "INSERT OR REPLACE INTO tiles (layer, zoom, x, y, blob) VALUES (?, ?, ?, ?, ?)"
    ) as unknown as TileQuery;
  }

  getTile(layer: string, zoom: number, x: number, y: number): Uint8Array | null {
    try {
      const result = this.getTileQuery.get(getLayerId(layer), zoom, x, y);
      return result ? result.blob : null;
    } catch (error) {
      console.error(`Error getting tile ${layer}/${zoom}/${x}/${y}:`, error);
      return null;
    }
  }

  saveTile(layer: string, zoom: number, x: number, y: number, blob: Uint8Array): boolean {
    try {
      this.saveTileQuery.run(getLayerId(layer), zoom, x, y, blob);
      return true;
    } catch (error) {
      console.error(`Error saving tile ${layer}/${zoom}/${x}/${y}:`, error);
      return false;
    }
  }

  hasTile(layer: string, zoom: number, x: number, y: number): boolean {
    return this.getTile(layer, zoom, x, y) !== null;
  }

  batchCheckTiles(layer: string, tiles: Tile[]): Set<string> {
    const existingTiles = new Set<string>();
    const layerId = getLayerId(layer);

    // Process tiles in batches
    for (let i = 0; i < tiles.length; i += 100) {
      const batch = tiles.slice(i, i + 100);
      const placeholders = batch.map(() => '(?, ?, ?, ?)').join(',');

      try {
        const query = this.db.prepare(`
          SELECT zoom, x, y
          FROM tiles
          WHERE (layer, zoom, x, y) IN (${placeholders})
        `);

        const params = batch.flatMap(tile => [layerId, tile.zoom, tile.x, tile.y]);
        const results = query.all(...params) as Tile[];

        results.forEach(tile => {
          existingTiles.add(`${tile.zoom}/${tile.x}/${tile.y}`);
        });
      } catch (error) {
        console.error('Error in batch check:', error);
      }
    }

    return existingTiles;
  }

  cleanupOldTiles(daysOld: number = 30): number {
    try {
      const result = this.db.prepare(`
        DELETE FROM tiles
        WHERE created_at < datetime('now', '-' || ? || ' days')
      `).run(daysOld);

      return result.changes;
    } catch (error) {
      console.error('Error cleaning up old tiles:', error);
      return 0;
    }
  }
}