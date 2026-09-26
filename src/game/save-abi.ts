/**
 * RunSave wire format shared with the Rust engine (`engine/src/lib.rs`).
 *
 * The struct is `#[repr(C)]`: eight header i32s (wave, health, armor,
 * weapon, flags, kills, secrets, elapsed_ms), then `ammo[WEP_N]` and
 * `mag[WEP_N]` with WEP_N = 11.
 *
 * Keep these in sync with the engine. `run_save_layout_matches_ts_side`
 * in `engine/src/lib.rs` pins the same numbers on the Rust side; if either
 * file drifts, a test fails instead of saves silently corrupting (the 8-vs-11
 * slot bug that ate boss-weapon ammo).
 */

/** Weapon slots persisted per save. Must match WEP_N in engine/src/consts.rs. */
export const SAVE_SLOTS = 11;
/** Byte size of the RunSave struct. */
export const SAVE_SIZE = 120;
/** Byte offset of the ammo array (8 header i32s). */
export const SAVE_AMMO_BASE = 32;
/** Byte offset of the mag array (ammo base + 11 i32s). */
export const SAVE_MAG_BASE = 76;
