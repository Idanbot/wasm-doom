/**
 * HUD wire format shared with the Rust engine (`engine/src/hud.rs`).
 *
 * The struct is `#[repr(C)]` and read from WASM memory with a DataView in
 * `runtime.ts`. `HUD_SIZE` is asserted at boot via the `hs_hud_size` export
 * so a Rust-side field reorder fails loudly instead of misreading memory.
 *
 * Keep `HUD_OFFSETS` in sync with `HUD_OFFSETS` in `engine/src/hud.rs`.
 */

/** Byte size of the Hud struct. Must match `HUD_SIZE` in engine/src/hud.rs. */
export const HUD_SIZE = 192;
/** Byte offsets of key fields inside the HUD struct. */
export const HUD_OFFSETS = {
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
  hasW6: 132,
  hasW7: 136,
  hasW8: 140,
  objective: 144,
  radioSeq: 148,
  splash: 188,
} as const;
