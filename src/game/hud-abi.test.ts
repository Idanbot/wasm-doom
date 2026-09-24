import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { HUD_OFFSETS, HUD_SIZE } from "./hud-abi.ts";

/**
 * Guards the WASM/TS wire contract. The Rust side asserts the same size at
 * compile time (`engine/src/hud.rs`) and reports it via `hs_hud_size`, which
 * `runtime.ts` checks on boot. These tests pin the TS copy so a local edit
 * that drifts from the engine fails here instead of in the browser.
 */
describe("HUD ABI", () => {
  it("struct size matches the Rust repr(C) layout", () => {
    assert.equal(HUD_SIZE, 132);
  });

  it("field offsets match engine/src/hud.rs HUD_OFFSETS", () => {
    assert.deepEqual(HUD_OFFSETS, {
      state: 24,
      reserve: 88,
      reloading: 92,
      weapFrame: 96,
      events: 108,
      evWeapon: 112,
      wave: 116,
      bossHealth: 120,
      bossMaxHealth: 124,
      bossPhase: 128,
    });
  });

  it("every offset lands inside the struct", () => {
    for (const [field, off] of Object.entries(HUD_OFFSETS)) {
      assert.ok(off >= 0 && off + 4 <= HUD_SIZE, `${field} @ ${off} out of range`);
    }
  });

  it("event flag offsets do not overlap other fields", () => {
    const offsets = Object.values(HUD_OFFSETS);
    assert.equal(new Set(offsets).size, offsets.length);
  });
});
