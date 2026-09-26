import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { SAVE_AMMO_BASE, SAVE_MAG_BASE, SAVE_SIZE, SAVE_SLOTS } from "./save-abi.ts";

/**
 * Guards the RunSave WASM/TS wire contract. The Rust side pins the same
 * layout in `run_save_layout_matches_ts_side` (engine/src/lib.rs). Update
 * both files together when WEP_N changes.
 */
describe("RunSave ABI", () => {
  it("covers all eleven weapons", () => {
    assert.equal(SAVE_SLOTS, 11);
  });

  it("struct size matches the Rust repr(C) layout", () => {
    assert.equal(SAVE_SIZE, 32 + SAVE_SLOTS * 4 * 2);
  });

  it("ammo and mag arrays sit back to back after the header", () => {
    assert.equal(SAVE_AMMO_BASE, 32);
    assert.equal(SAVE_MAG_BASE, SAVE_AMMO_BASE + SAVE_SLOTS * 4);
    assert.equal(SAVE_MAG_BASE + SAVE_SLOTS * 4, SAVE_SIZE);
  });
});
