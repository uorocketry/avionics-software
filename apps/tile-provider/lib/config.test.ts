import { expect, test } from "bun:test";

test("config defaults are sane", async () => {
  // Dynamically import to avoid capturing env from test runner incorrectly
  const mod = await import("./config");
  const cfg = mod.config;

  expect(cfg.address.length).toBeGreaterThan(0);
  expect(Number(cfg.port)).toBeGreaterThan(0);
  expect(cfg.database).toBeDefined();
  expect(cfg.tileSources.satellite).toContain("http");
  expect(cfg.tileSources.terrain).toContain("http");
});
