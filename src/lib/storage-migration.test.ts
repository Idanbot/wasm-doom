import { test } from "node:test";
import assert from "node:assert/strict";
import { migrateGameStorage } from "./storage-migration.ts";

function memoryStorage(entries: [string, string][]) {
  const values = new Map(entries);
  return { values, getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => { values.set(key, value); },
    removeItem: (key: string) => { values.delete(key); } };
}
test("imports old settings, scores and checkpoints once without replacing current data", () => {
  const s = memoryStorage([["hellscan-vol", "old"], ["blacksite-vol", "current"],
    ["hellscan-board", "scores"], ["hellscan-checkpoint", "progress"], ["unrelated", "keep"]]);
  migrateGameStorage(s); migrateGameStorage(s);
  assert.equal(s.getItem("blacksite-vol"), "current");
  assert.equal(s.getItem("blacksite-board"), "scores");
  assert.equal(s.getItem("blacksite-checkpoint"), "progress");
  assert.equal(s.getItem("hellscan-board"), null);
  assert.equal(s.getItem("unrelated"), "keep");
});
test("retains old progress when storage cannot accept the migrated value", () => {
  const s = memoryStorage([["hellscan-checkpoint", "progress"]]);
  s.setItem = () => { throw new Error("quota exceeded"); };
  migrateGameStorage(s);
  assert.equal(s.getItem("hellscan-checkpoint"), "progress");
});
