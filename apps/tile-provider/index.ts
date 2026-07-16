import { Database } from "bun:sqlite";
import { config } from "./lib/config";
import { TileDatabase } from "./lib/database";
import { DownloadQueue } from "./lib/downloadQueue";
import homepage from "./public/index.html";

// -----------------------
// Setup SQLite database
// -----------------------
console.log("Opening DB at:", config.database);
console.log("Using address:", config.address);
console.log("Using port:", config.port);

const db = new Database(config.database);
db.exec("PRAGMA journal_mode = WAL;");

// Create the tiles table if needed.
db.exec(`
  CREATE TABLE IF NOT EXISTS tiles (
    zoom INTEGER,
    x INTEGER,
    y INTEGER,
    blob BLOB,
    PRIMARY KEY (zoom, x, y)
  );
`);

// Initialize database and download queue
const tileDb = new TileDatabase(db);
const downloadQueue = new DownloadQueue(db);

// -----------------------
// CORS helper function
// -----------------------
function corsHeaders(): Record<string, string> {
  return {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type, Authorization",
    "Access-Control-Max-Age": "86400",
  };
}

// -----------------------
// HTTP handler
// -----------------------
async function handleTileRequest(
  layer: string,
  zoom: string,
  x: string,
  y: string
): Promise<Response> {
  const baseUrl = config.tileSources[layer as keyof typeof config.tileSources];
  if (!baseUrl) {
    return new Response(`Unknown layer: ${layer}`, {
      status: 400,
      headers: corsHeaders(),
    });
  }

  // Check the database.
  const dbBlob = tileDb.getTile(layer, Number(zoom), Number(x), Number(y));
  if (dbBlob) {
    return new Response(dbBlob, {
      status: 200,
      headers: {
        "Content-Type": "image/png",
        "Cache-Control": "public, max-age=31536000",
        ...corsHeaders(),
      },
    });
  }

  // Fetch from the remote source.
  const url = `${baseUrl}&x=${x}&y=${y}&z=${zoom}`;
  const response = await fetch(url);

  if (!response.ok) {
    return new Response(response.statusText, {
      status: 500,
      headers: {
        "Content-Type": "text/plain",
        ...corsHeaders(),
      },
    });
  }

  const buffer = new Uint8Array(await response.arrayBuffer());
  tileDb.saveTile(layer, Number(zoom), Number(x), Number(y), buffer);

  return new Response(buffer, {
    status: 200,
    headers: {
      "Content-Type": "image/png",
      "Cache-Control": "public, max-age=31536000",
      ...corsHeaders(),
    },
  });
}

// -----------------------
// Main HTTP server route handling
// -----------------------
const server = Bun.serve({
  development: process.env.NODE_ENV !== "production",
  routes: {
    "/health": () => new Response("OK", { headers: corsHeaders() }),
    "/tiles/:zoom/:x/:y": async (req) => {
      const { zoom, x, y } = req.params;
      return handleTileRequest("satellite", zoom, x, y);
    },
    "/tiles/:layer/:zoom/:x/:y": async (req) => {
      const { layer, zoom, x, y } = req.params;
      return handleTileRequest(layer, zoom, x, y);
    },
    "/api/download": async (req) => {
      // Handle OPTIONS preflight request
      if (req.method === "OPTIONS") {
        return new Response(null, {
          status: 200,
          headers: corsHeaders(),
        });
      }

      const body = await req.json();
      const result = await downloadQueue.queueDownload(body);
      return new Response(JSON.stringify(result), {
        status: result.success ? 200 : 400,
        headers: {
          "Content-Type": "application/json",
          ...corsHeaders(),
        },
      });
    },
    "/api/download/:jobId/status": async (req) => {
      // Handle OPTIONS preflight request
      if (req.method === "OPTIONS") {
        return new Response(null, {
          status: 200,
          headers: corsHeaders(),
        });
      }

      const { jobId } = req.params;
      const job = await downloadQueue.getJobStatus(jobId);
      if (!job) {
        return new Response(JSON.stringify({ error: 'Job not found' }), {
          status: 404,
          headers: {
            "Content-Type": "application/json",
            ...corsHeaders(),
          },
        });
      }
      return new Response(JSON.stringify(job), {
        headers: {
          "Content-Type": "application/json",
          ...corsHeaders(),
        },
      });
    },
    "/api/download/:jobId/cancel": async (req) => {
      // Handle OPTIONS preflight request
      if (req.method === "OPTIONS") {
        return new Response(null, {
          status: 200,
          headers: corsHeaders(),
        });
      }

      const { jobId } = req.params;
      const success = await downloadQueue.cancelDownload(jobId);
      return new Response(JSON.stringify({ success }), {
        status: success ? 200 : 404,
        headers: {
          "Content-Type": "application/json",
          ...corsHeaders(),
        },
      });
    },
    "/": homepage,
  },
  hostname: config.address,
  port: config.port,
});

console.log(`Starting server at ${config.address}:${config.port}`);

// -----------------------
// Graceful shutdown
// -----------------------
function shutdown(signal: string) {
  try {
    console.log(`[tile_provider] Received ${signal}, shutting down...`);
    // Stop accepting new requests
    try { server.stop(); } catch { }
    // Stop background work
    try { downloadQueue.shutdown(); } catch { }
  } finally {
    // Force exit if cleanup hangs
    setTimeout(() => process.exit(0), 2000).unref();
  }
}

try {
  process.on('SIGTERM', () => shutdown('SIGTERM'));
  process.on('SIGINT', () => shutdown('SIGINT'));
} catch { }
