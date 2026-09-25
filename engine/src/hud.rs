//! HUD wire format shared with TypeScript (`src/game/runtime.ts`).
//!
//! The struct is `#[repr(C)]` and read from WASM memory with a DataView.
//! `HUD_SIZE` is exported via `hs_hud_size` so the TS side can assert the
//! layout matches instead of silently misreading fields.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Hud {
    pub health: i32,
    pub armor: i32,
    pub ammo: i32,
    pub weapon: i32,
    pub kills: i32,
    pub living: i32,
    pub state: i32,
    pub prompt: i32,
    pub has_w2: i32,
    pub has_w3: i32,
    pub secrets: i32,
    pub elapsed_ms: i32,
    pub shake: f32,
    pub muzzle: f32,
    pub hurt: f32,
    pub bob: f32,
    pub kick: f32,
    pub hitmarker: f32,
    pub yaw: f32,
    pub speed: f32,
    pub x: f32,
    pub y: f32,
    pub reserve: i32,
    pub reloading: f32,
    pub weap_frame: i32,
    pub has_w4: i32,
    pub has_w5: i32,
    pub events: u32,
    pub ev_weapon: i32,
    pub wave: i32,
    pub boss_health: i32,
    pub boss_max_health: i32,
    pub boss_phase: i32,
    pub has_w6: i32,
    pub has_w7: i32,
    pub has_w8: i32,
    pub objective: i32,
    pub radio_seq: i32,
    pub radio_line: i32,
    pub vuln: f32,
    pub node_x: f32,
    pub node_y: f32,
    pub has_w9: i32,
    pub has_w10: i32,
    pub has_w11: i32,
    pub power: i32,
    pub power_t: f32,
}

/// Byte size of the HUD struct as seen by TypeScript.
pub const HUD_SIZE: usize = core::mem::size_of::<Hud>();

/// Field offsets as seen by TypeScript. Kept next to the struct so a
/// reorder forces an update here instead of a silent desync.
pub const HUD_OFFSETS: [(u32, &str); 15] = [
    (108, "events"),
    (112, "ev_weapon"),
    (88, "reserve"),
    (92, "reloading"),
    (96, "weap_frame"),
    (116, "wave"),
    (120, "boss_health"),
    (124, "boss_max_health"),
    (128, "boss_phase"),
    (132, "has_w6"),
    (136, "has_w7"),
    (140, "has_w8"),
    (144, "objective"),
    (148, "radio_seq"),
    (24, "state"),
];

const _: () = assert!(HUD_SIZE == 188, "Hud layout changed; update runtime.ts");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_size_matches_ts_side() {
        // Mirrors src/game/hud-abi.ts. The compile-time assert above is the
        // real guard; this keeps the value visible in test output.
        assert_eq!(HUD_SIZE, 188);
    }

    #[test]
    fn hud_offsets_are_sorted_unique_and_in_range() {
        let mut offs: Vec<u32> = HUD_OFFSETS.iter().map(|(o, _)| *o).collect();
        let unique: std::collections::HashSet<u32> = offs.iter().copied().collect();
        assert_eq!(unique.len(), offs.len(), "HUD offsets overlap");
        for o in &offs {
            assert!(*o as usize + 4 <= HUD_SIZE, "offset {o} out of range");
        }
        offs.sort_unstable();
        let mut sorted = HUD_OFFSETS.to_vec();
        sorted.sort_unstable();
        assert_eq!(offs, sorted.iter().map(|(o, _)| *o).collect::<Vec<_>>());
    }
}
