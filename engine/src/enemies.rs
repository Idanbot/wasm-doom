//! Data-driven enemy roster.
//!
//! Every spawnable entity kind has exactly one row in [`ENEMY_DEFS`].
//! Adding a new enemy is a table edit, not a code hunt:
//!
//! 1. Author the seven BLACKSITE animation layers per
//!    `docs/monster-sprite-design.md` (256x256, 2x2 cells of 128x128) and
//!    register the files in `TEX_FILES` in `src/game/runtime.ts`.
//! 2. Add an `EK_*` kind constant in `consts.rs`.
//! 3. Add one `EnemyDef` row below (copy the TEMPLATE row).
//! 4. Place it in `map.rs` (`place_hub_spoke`, a spawn group, or an ambush).
//! 5. Run `cargo test` — the roster tests reject duplicate kinds, missing
//!    hostile flags, and zero-size entries.
//!
//! The sim never matches on kinds directly: `spawn()` reads hp/radius/zoff,
//! the renderer reads the skin animation layers, and wave cleanup reads
//! `hostile`/`cleared_on_wave`. See `docs/enemy-authoring.md`.

use crate::consts::*;

/// One row per spawnable entity kind.
pub(crate) struct EnemyDef {
    /// Matches the `EK_*` constant used in `Ent.kind`.
    pub kind: u8,
    /// Display name. Roster metadata for docs and tooling (asserted in
    /// tests); the sim itself keys everything off `kind`.
    #[allow(dead_code)]
    pub name: &'static str,
    /// Combat role: grunt / tank / skirmisher / boss / pickup / prop / fx.
    #[allow(dead_code)]
    pub role: &'static str,
    /// Hit points at spawn.
    pub hp: i32,
    /// Collision radius in world units (player is 0.22).
    pub radius: f32,
    /// Vertical screen offset in framebuffer px at depth 1
    /// (renderer computes `voff = zoff / depth`).
    pub zoff: f32,
    /// World-height multiplier (`sprite_h = screen_h / depth * scale`).
    pub scale: f32,
    /// `T_*` atlas slot the renderer samples.
    pub texture: usize,
    /// True for 2x2 animated sheets (`frame = (anim*4) % 4`);
    /// false samples the whole 256x256 image.
    pub sheet4: bool,
    /// Chases/shoots the player; counted as living; cleared per wave.
    pub hostile: bool,
    /// Removed when the next wave starts (hostiles plus their
    /// projectiles, beams and lingering fire).
    pub cleared_on_wave: bool,
}

/// Presentation-only BLACKSITE skin. Combat behavior remains attached to the
/// legacy archetype in `EnemyDef`; each skin owns seven atlas layers in the
/// order idle, move, pain, fire, reload/charge, dead, special.
#[derive(Clone, Copy)]
pub(crate) struct EnemySkin {
    pub id: u8,
    #[allow(dead_code)]
    pub name: &'static str,
    pub texture: usize,
    pub scale: f32,
    pub zoff: f32,
    #[allow(dead_code)]
    pub special: &'static str,
}

pub(crate) const ENEMY_SKINS: &[EnemySkin] = &[
    // The source renders are normalized to a 216px subject height before
    // packing. These scales keep the old world-space silhouette sizes while
    // leaving enough depth for the new high-resolution silhouettes.
    EnemySkin { id: SKIN_RIFLEMAN, name: "Directorate Rifleman", texture: ENEMY_TEX_BASE, scale: 0.90, zoff: 0.0, special: "tactical brace" },
    EnemySkin { id: SKIN_BREACHER, name: "Breacher", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT, scale: 0.95, zoff: 0.0, special: "breach rush" },
    EnemySkin { id: SKIN_SUBJECT, name: "Failed Augment Subject", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 2, scale: 0.90, zoff: 0.0, special: "augment surge" },
    EnemySkin { id: SKIN_HAZMAT, name: "Hazmat Security", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 3, scale: 1.12, zoff: 0.0, special: "purge charge" },
    EnemySkin { id: SKIN_GUNNER, name: "Heavy Gunner", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 4, scale: 1.14, zoff: 0.0, special: "stabilized burst" },
    EnemySkin { id: SKIN_LOADER, name: "Industrial Loader", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 5, scale: 1.18, zoff: 0.0, special: "hydraulic slam" },
    EnemySkin { id: SKIN_VATBRUTE, name: "Vat-grown Brute", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 6, scale: 1.22, zoff: 0.0, special: "bio-rage" },
    EnemySkin { id: SKIN_MARKSMAN, name: "Marksman", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 7, scale: 0.78, zoff: -18.0, special: "optic lock" },
    EnemySkin { id: SKIN_HORNET, name: "Hornet Drone", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 8, scale: 0.72, zoff: -70.0, special: "attack vector" },
    EnemySkin { id: SKIN_HOUND, name: "Hound", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 9, scale: 0.72, zoff: 22.0, special: "pounce" },
    EnemySkin { id: SKIN_SPITTER, name: "Spitter", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 10, scale: 0.76, zoff: -8.0, special: "acid sac" },
    EnemySkin { id: SKIN_MARTYR, name: "Martyr Drone", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 11, scale: 0.70, zoff: -48.0, special: "detonation" },
    EnemySkin { id: SKIN_VEYRAN, name: "VEYRAN // MALIK", texture: ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * 12, scale: 2.00, zoff: 8.0, special: "override surge" },
];

pub(crate) fn skin_def(id: u8) -> Option<&'static EnemySkin> {
    ENEMY_SKINS.iter().find(|skin| skin.id == id)
}

pub(crate) fn default_skin(kind: u8) -> u8 {
    match kind {
        EK_HUSK => SKIN_RIFLEMAN,
        EK_BRUTE => SKIN_HAZMAT,
        EK_WRAITH => SKIN_MARKSMAN,
        EK_BOSS => SKIN_VEYRAN,
        EK_MARTYR => SKIN_MARTYR,
        EK_BARREL => SKIN_NONE,
        _ => SKIN_NONE,
    }
}

// TEMPLATE — copy this row to add a new enemy:
//
// EnemyDef {
//     kind: EK_HUSK, // 1. new EK_* constant in consts.rs
//     name: "Husk",
//     role: "grunt",
//     hp: 28,
//     radius: 0.28,  // ~player scale; brutes 0.38, boss 0.62
//     zoff: 0.0,     // 0 = feet on floor; floats use negative lift
//     scale: 0.95,   // world-height multiplier, see renderer
//     texture: T_HUSK, // legacy fallback slot for non-BLACKSITE entities
//     sheet4: true,   // legacy 2x2 animation fallback
//     hostile: true,
//     cleared_on_wave: true,
// },

pub(crate) const ENEMY_DEFS: &[EnemyDef] = &[
    EnemyDef { kind: EK_HUSK, name: "Husk", role: "grunt", hp: 28, radius: 0.28, zoff: 0.0, scale: 0.95, texture: T_HUSK, sheet4: true, hostile: true, cleared_on_wave: true },
    EnemyDef { kind: EK_BRUTE, name: "Brute", role: "tank", hp: 78, radius: 0.38, zoff: 0.0, scale: 1.25, texture: T_BRUTE, sheet4: true, hostile: true, cleared_on_wave: true },
    EnemyDef { kind: EK_WRAITH, name: "Wraith", role: "skirmisher", hp: 20, radius: 0.26, zoff: -70.0, scale: 0.7, texture: T_WRAITH, sheet4: true, hostile: true, cleared_on_wave: true },
    EnemyDef { kind: EK_BOSS, name: "Vault Master", role: "boss", hp: 520, radius: 0.62, zoff: 8.0, scale: 2.45, texture: T_BRUTE, sheet4: true, hostile: true, cleared_on_wave: true },
    EnemyDef { kind: EK_PROJ, name: "Bolt", role: "fx", hp: 1, radius: 0.12, zoff: -10.0, scale: 0.28, texture: T_BALL, sheet4: false, hostile: false, cleared_on_wave: true },
    EnemyDef { kind: EK_RAY, name: "Ray", role: "fx", hp: 1, radius: 0.1, zoff: -6.0, scale: 0.10, texture: T_BALL, sheet4: false, hostile: false, cleared_on_wave: true },
    EnemyDef { kind: EK_BOLT, name: "Seeker", role: "fx", hp: 1, radius: 0.14, zoff: 12.0, scale: 0.36, texture: T_FLAME, sheet4: true, hostile: false, cleared_on_wave: true },
    EnemyDef { kind: EK_MED, name: "Medkit", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.38, texture: T_MED, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_AMMO, name: "Ammo", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.40, texture: T_AMMO, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_ARMOR, name: "Armor", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.48, texture: T_ARMOR, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_GUN2, name: "BR-12 case", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.48, texture: T_GUN2, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_GUN3, name: "KX-9 case", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.46, texture: T_GUN3, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_GUN4, name: "MR-4 case", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.54, texture: T_GUN4, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_GUN5, name: "VLK-6 case", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.50, texture: T_GUN5, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_GUN6, name: "AX-12 case", role: "pickup", hp: 1, radius: 0.22, zoff: 34.0, scale: 0.52, texture: T_GUN6, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_GUN7, name: "M91 case", role: "pickup", hp: 1, radius: 0.24, zoff: 34.0, scale: 0.60, texture: T_GUN7, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_BARREL, name: "Barrel", role: "prop", hp: 14, radius: 0.3, zoff: 78.0, scale: 0.72, texture: T_BARREL, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_GIB, name: "Gib", role: "fx", hp: 1, radius: 0.08, zoff: 0.0, scale: 0.18, texture: T_SPLAT, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_LAMP, name: "Lamp", role: "prop", hp: 1, radius: 0.16, zoff: -118.0, scale: 0.52, texture: T_LAMP, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_CRATE, name: "Crate", role: "prop", hp: 1, radius: 0.32, zoff: 86.0, scale: 0.62, texture: T_CRATE, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_CHAIN, name: "Chain", role: "prop", hp: 1, radius: 0.12, zoff: -104.0, scale: 1.05, texture: T_CHAIN, sheet4: false, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_IMPACT, name: "Impact", role: "fx", hp: 1, radius: 0.1, zoff: -6.0, scale: 0.48, texture: T_IMPACT, sheet4: true, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_SPARK, name: "Spark", role: "fx", hp: 1, radius: 0.06, zoff: 0.0, scale: 0.05, texture: T_MUZZLEFX, sheet4: true, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_SMOKE, name: "Smoke", role: "fx", hp: 1, radius: 0.1, zoff: -4.0, scale: 0.28, texture: T_FLAME, sheet4: true, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_FLAME, name: "Flame", role: "fx", hp: 1, radius: 0.14, zoff: 42.0, scale: 0.82, texture: T_FLAME, sheet4: true, hostile: false, cleared_on_wave: false },
    EnemyDef { kind: EK_FIREPATCH, name: "Fire patch", role: "fx", hp: 1, radius: 0.2, zoff: 0.0, scale: 0.60, texture: T_FLAME, sheet4: true, hostile: false, cleared_on_wave: true },
    EnemyDef { kind: EK_MARTYR, name: "Martyr", role: "skirmisher", hp: 18, radius: 0.26, zoff: -48.0, scale: 0.70, texture: T_BALL, sheet4: false, hostile: true, cleared_on_wave: true },
    EnemyDef { kind: EK_OVERRIDE_CONSOLE, name: "Override console", role: "objective", hp: 1, radius: 0.22, zoff: 18.0, scale: 0.95, texture: T_CONSOLE_UPPER, sheet4: false, hostile: false, cleared_on_wave: false },
];

/// Look up a kind's row. Returns `None` for kind 0 (empty slot).
pub(crate) fn enemy_def(kind: u8) -> Option<&'static EnemyDef> {
    if kind == EK_NONE {
        return None;
    }
    ENEMY_DEFS.iter().find(|d| d.kind == kind)
}

/// True for the four chasers counted as living and targeted by AI.
pub(crate) fn is_hostile_kind(kind: u8) -> bool {
    matches!(kind, v if enemy_def(v).is_some_and(|d| d.hostile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn roster_has_unique_kinds_and_names() {
        let mut kinds = HashSet::new();
        let mut names = HashSet::new();
        for d in ENEMY_DEFS {
            assert!(kinds.insert(d.kind), "duplicate kind {}", d.kind);
            assert!(names.insert(d.name), "duplicate name {}", d.name);
            assert_ne!(d.kind, EK_NONE, "row for empty slot");
        }
    }

    #[test]
    fn hostile_set_is_the_chasers() {
        let hostiles: Vec<u8> = ENEMY_DEFS.iter().filter(|d| d.hostile).map(|d| d.kind).collect();
        assert_eq!(hostiles, vec![EK_HUSK, EK_BRUTE, EK_WRAITH, EK_BOSS, EK_MARTYR]);
        for k in [EK_HUSK, EK_BRUTE, EK_WRAITH, EK_BOSS, EK_MARTYR] {
            assert!(is_hostile_kind(k));
            assert!(enemy_def(k).unwrap().cleared_on_wave);
        }
        assert!(!is_hostile_kind(EK_MED));
        assert!(!is_hostile_kind(0));
    }

    #[test]
    fn combat_stats_match_legacy_tuning() {
        // Guard against accidental rebalancing during refactors.
        let get = |k| enemy_def(k).unwrap();
        assert_eq!((get(EK_HUSK).hp, get(EK_BRUTE).hp, get(EK_WRAITH).hp, get(EK_BOSS).hp), (28, 78, 20, 520));
        assert_eq!((get(EK_HUSK).radius, get(EK_BRUTE).radius, get(EK_BOSS).radius), (0.28, 0.38, 0.62));
        assert_eq!((get(EK_HUSK).scale, get(EK_BRUTE).scale, get(EK_WRAITH).scale, get(EK_BOSS).scale), (0.95, 1.25, 0.7, 2.45));
    }

    #[test]
    fn every_row_is_sane() {
        for d in ENEMY_DEFS {
            assert!(d.hp > 0, "{} has no hp", d.name);
            assert!(d.radius > 0.0, "{} has no radius", d.name);
            assert!(d.scale > 0.0, "{} has no scale", d.name);
            assert!(!d.name.is_empty() && !d.role.is_empty());
        }
    }
}
