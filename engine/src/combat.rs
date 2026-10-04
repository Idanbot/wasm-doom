//! Skin-specific combat roles, independent of sprite size and base HP.
use crate::consts::*;

#[derive(Clone, Copy)]
pub(crate) struct Combat {
    pub speed: f32,
    pub range: f32,
    pub windup: f32,
    pub cooldown: f32,
    pub damage: i32,
    pub pellets: usize,
    pub spread: f32,
    pub melee: bool,
}

pub(crate) fn profile(skin: u8, kind: u8) -> Combat {
    let rifle = Combat { speed: 1.7, range: 5.0, windup: 0.45, cooldown: 1.2, damage: 9, pellets: 1, spread: 0.0, melee: false };
    match crate::enemies::combat_skin(skin) {
        SKIN_BREACHER => Combat { speed: 2.0, range: 2.8, windup: 0.55, cooldown: 1.6, damage: 7, pellets: 3, spread: 0.14, ..rifle },
        SKIN_SUBJECT => Combat { speed: 2.5, range: 1.0, windup: 0.42, cooldown: 0.9, damage: 12, melee: true, ..rifle },
        SKIN_HAZMAT => Combat { speed: 1.25, range: 4.5, windup: 0.65, cooldown: 1.7, damage: 12, ..rifle },
        SKIN_GUNNER => Combat { speed: 1.0, range: 6.0, windup: 0.8, cooldown: 1.4, damage: 8, pellets: 3, spread: 0.07, ..rifle },
        SKIN_LOADER | SKIN_VATBRUTE => Combat { speed: 1.85, range: 1.1, windup: 0.65, cooldown: 1.0, damage: 22, melee: true, ..rifle },
        SKIN_MARKSMAN => Combat { speed: 1.5, range: 8.0, windup: 0.9, cooldown: 2.0, damage: 24, ..rifle },
        SKIN_HORNET => Combat { speed: 2.7, range: 4.0, windup: 0.4, cooldown: 1.0, damage: 7, ..rifle },
        SKIN_HOUND => Combat { speed: 3.0, range: 0.95, windup: 0.3, cooldown: 0.8, damage: 10, melee: true, ..rifle },
        SKIN_MARTYR => Combat { speed: 3.2, range: 1.1, windup: 0.5, cooldown: 1.0, damage: 0, melee: true, ..rifle },
        SKIN_SPITTER => Combat { speed: 1.7, range: 4.0, windup: 0.65, cooldown: 1.5, damage: 14, ..rifle },
        SKIN_VEYRAN => Combat { speed: 1.28, range: 5.0, windup: 1.0, cooldown: 1.4, damage: 18, pellets: 5, spread: 0.16, ..rifle },
        SKIN_HECATE => Combat { speed: 0.82, range: 7.0, windup: 0.72, cooldown: 1.05, damage: 10, pellets: 4, spread: 0.11, ..rifle },
        SKIN_CHIMERA => Combat { speed: 1.45, range: 4.2, windup: 0.5, cooldown: 0.85, damage: 16, pellets: 2, spread: 0.08, ..rifle },
        SKIN_ORACLE => Combat { speed: 1.8, range: 8.5, windup: 0.58, cooldown: 1.18, damage: 16, pellets: 2, spread: 0.035, ..rifle },
        SKIN_GRAVEMIND => Combat { speed: 1.05, range: 5.6, windup: 0.88, cooldown: 1.35, damage: 13, pellets: 5, spread: 0.17, ..rifle },
        SKIN_ARCHIVIST => Combat { speed: 1.62, range: 7.4, windup: 0.76, cooldown: 1.24, damage: 18, pellets: 3, spread: 0.06, ..rifle },
        SKIN_HALCYON => Combat { speed: 0.9, range: 6.2, windup: 0.7, cooldown: 1.35, damage: 14, pellets: 5, spread: 0.13, ..rifle },
        SKIN_RELAY => Combat { speed: 2.1, range: 8.2, windup: 0.42, cooldown: 0.9, damage: 12, pellets: 3, spread: 0.045, ..rifle },
        SKIN_TITAN => Combat { speed: 0.72, range: 7.0, windup: 1.15, cooldown: 1.6, damage: 25, pellets: 5, spread: 0.14, ..rifle },
        SKIN_MNEMOSYNE => Combat { speed: 1.35, range: 7.8, windup: 1.05, cooldown: 1.7, damage: 20, pellets: 6, spread: 0.09, ..rifle },
        SKIN_KEST => Combat { speed: 2.35, range: 7.5, windup: 0.38, cooldown: 0.78, damage: 11, pellets: 4, spread: 0.055, ..rifle },
        23 => Combat { speed: 0.80, range: 4.80, windup: 0.70, cooldown: 1.30, damage: 10, pellets: 1, spread: 0.035, ..rifle },
        24 => Combat { speed: 1.02, range: 5.50, windup: 0.86, cooldown: 1.47, damage: 11, pellets: 2, spread: 0.075, ..rifle },
        25 => Combat { speed: 1.24, range: 6.20, windup: 1.02, cooldown: 1.64, damage: 12, pellets: 3, spread: 0.115, ..rifle },
        26 => Combat { speed: 1.46, range: 6.90, windup: 1.18, cooldown: 1.81, damage: 13, pellets: 4, spread: 0.035, ..rifle },
        27 => Combat { speed: 1.68, range: 7.60, windup: 0.70, cooldown: 1.98, damage: 14, pellets: 1, spread: 0.075, ..rifle },
        28 => Combat { speed: 1.90, range: 4.80, windup: 0.86, cooldown: 1.30, damage: 15, pellets: 2, spread: 0.115, ..rifle },
        29 => Combat { speed: 0.80, range: 5.50, windup: 1.02, cooldown: 1.47, damage: 10, pellets: 3, spread: 0.035, ..rifle },
        30 => Combat { speed: 1.02, range: 6.20, windup: 1.18, cooldown: 1.64, damage: 11, pellets: 4, spread: 0.075, ..rifle },
        31 => Combat { speed: 1.24, range: 6.90, windup: 0.70, cooldown: 1.81, damage: 12, pellets: 1, spread: 0.115, ..rifle },
        32 => Combat { speed: 1.46, range: 7.60, windup: 0.86, cooldown: 1.98, damage: 13, pellets: 2, spread: 0.035, ..rifle },
        33 => Combat { speed: 1.68, range: 4.80, windup: 1.02, cooldown: 1.30, damage: 14, pellets: 3, spread: 0.075, ..rifle },
        34 => Combat { speed: 1.90, range: 5.50, windup: 1.18, cooldown: 1.47, damage: 15, pellets: 4, spread: 0.115, ..rifle },
        35 => Combat { speed: 0.80, range: 6.20, windup: 0.70, cooldown: 1.64, damage: 10, pellets: 1, spread: 0.035, ..rifle },
        36 => Combat { speed: 1.02, range: 6.90, windup: 0.86, cooldown: 1.81, damage: 11, pellets: 2, spread: 0.075, ..rifle },
        _ if kind == EK_BRUTE => Combat { speed: 2.15, range: 1.1, windup: 0.6, cooldown: 0.9, damage: 14, melee: true, ..rifle },
        _ => rifle,
    }
}
