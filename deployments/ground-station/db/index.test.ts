import { expect, test } from "bun:test";
import { Database } from "bun:sqlite";
const seed = await Bun.file(new URL("./seed.sql", import.meta.url)).text();
import { parseSqlStatements } from "./sql";

test("seed.sql applies cleanly to in-memory sqlite", () => {
  const db = new Database(":memory:");
  const stmts = parseSqlStatements(seed);
  let threwError = false;
  for (const s of stmts) {
    try {
      // Using .run for DDL/DML; will throw on error
      db.run(s);
    } catch (e) {
      threwError = true;
    }
  }
  expect(threwError).toBe(false);
});
