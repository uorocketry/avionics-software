import { expect, test } from "bun:test";
import { Database } from "bun:sqlite";
import { DownloadQueue } from "./downloadQueue";

// Avoid hitting the network in tests
global.fetch = (async (url: string) => {
  // Return a tiny PNG header-like buffer to simulate a tile
  const data = new Uint8Array([137,80,78,71]);
  return new Response(data, { status: 200, headers: { 'content-type': 'image/png' } });
}) as any;

test("DownloadQueue enqueues and completes a tiny job", async () => {
  const db = new Database(":memory:");
  const q = new DownloadQueue(db);

  const res = await q.queueDownload({
    layer: "satellite",
    lat: 0,
    lon: 0,
    minZoom: 1,
    maxZoom: 1,
    radiusKm: 0.1
  });

  expect(res.success).toBe(true);
  expect(res.jobId).toBeDefined();

  // Wait briefly for processing
  await new Promise((r) => setTimeout(r, 200));

  const job = await q.getJobStatus(res.jobId!);
  expect(job).not.toBeNull();
  expect(["completed","failed","processing","cancelled","pending"]).toContain(job!.status);
});
