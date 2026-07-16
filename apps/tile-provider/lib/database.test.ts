import { expect, test } from "bun:test";
import { Database } from "bun:sqlite";
import { TileDatabase } from "./database";

function createMemoryDb() {
  return new Database(":memory:");
}

test("TileDatabase: save/get/has tile roundtrip", () => {
  const db = createMemoryDb();
  const tiles = new TileDatabase(db);

  const blob = new Uint8Array([1, 2, 3, 4]);
  const saved = tiles.saveTile("satellite", 5, 10, 20, blob);
  expect(saved).toBe(true);

  expect(tiles.hasTile("satellite", 5, 10, 20)).toBe(true);

  const read = tiles.getTile("satellite", 5, 10, 20);
  expect(read).not.toBeNull();
  expect(Array.from(read!)).toEqual(Array.from(blob));
});

test("TileDatabase: batchCheckTiles finds existing", () => {
  const db = createMemoryDb();
  const tiles = new TileDatabase(db);

  const existing = new Uint8Array([9]);
  tiles.saveTile("satellite", 3, 4, 5, existing);

  const set = tiles.batchCheckTiles("satellite", [
    { zoom: 3, x: 4, y: 5, blob: new Uint8Array() },
    { zoom: 7, x: 8, y: 9, blob: new Uint8Array() }
  ]);

  expect(set.has("3/4/5")).toBe(true);
  expect(set.has("7/8/9")).toBe(false);
});

test("TileDatabase: cleanupOldTiles removes outdated rows", () => {
  const db = createMemoryDb();
  const tiles = new TileDatabase(db);

  // Insert a tile and mark it as very old
  tiles.saveTile("satellite", 1, 1, 1, new Uint8Array([7]));
  db.exec("UPDATE tiles SET created_at = datetime('now', '-100 days') WHERE layer=0 AND zoom=1 AND x=1 AND y=1");

  const removed = tiles.cleanupOldTiles(30);
  expect(removed).toBeGreaterThan(0);
  expect(tiles.hasTile("satellite", 1, 1, 1)).toBe(false);
});
