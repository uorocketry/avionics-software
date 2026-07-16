import process from "process";
import { parseArgs } from "util";


const { values } = parseArgs({
  args: Bun.argv,
  options: {
    address: { type: "string", default: process.env.ADDRESS ?? "0.0.0.0" },
    port: { type: "string", default: process.env.PORT ?? "6565" },
    database: {
      type: "string",
      default: process.env.DB_FILENAME ?? "tiles.db",
    },
  },
  strict: true,
  allowPositionals: true,
})

export const config = {
  address: values.address ?? "127.0.0.1",
  port: values.port ?? "6565",
  database: values.database ?? "tiles.db",
  tileSources: {
    satellite: "http://mt0.google.com/vt/lyrs=s,h",
    terrain: "http://mt0.google.com/vt/lyrs=p",
  },
  cors: {
    allowedOrigins: ["*"],
    allowedMethods: ["GET", "POST", "OPTIONS"],
    allowedHeaders: ["Content-Type"]
  },
  download: {
    batchSize: 100,
    delayMs: 100
  }
} as const;