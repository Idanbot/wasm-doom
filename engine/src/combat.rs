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
    match skin {
        SKIN_BREACHER => Combat { speed: 2.0, range: 2.8, windup: 0.55, cooldown: 1.6, damage: 7, pellets: 3, spread: 0.14, ..rifle },
        SKIN_SUBJECT => Combat { speed: 2.5, range: 1.0, windup: 0.42, cooldown: 0.9, damage: 12, melee: true, ..rifle },
        SKIN_HAZMAT => Combat { speed: 1.25, range: 4.5, windup: 0.65, cooldown: 1.7, damage: 12, ..rifle },
        SKIN_GUNNER => Combat { speed: 1.0, range: 6.0, windup: 0.8, cooldown: 1.4, damage: 8, pellets: 3, spread: 0.07, ..rifle },
        SKIN_LOADER | SKIN_VATBRUTE => Combat { speed: 1.85, range: 1.1, windup: 0.65, cooldown: 1.0, damage: 22, melee: true, ..rifle },
        SKIN_MARKSMAN => Combat { speed: 1.5, range: 8.0, windup: 0.9, cooldown: 2.0, damage: 24, ..rifle },
        SKIN_HORNET => Combat { speed: 2.7, range: 4.0, windup: 0.4, cooldown: 1.0, damage: 7, ..rifle },
        SKIN_HOUND => Combat { speed: 3.0, range: 0.95, windup: 0.3, cooldown: 0.8, damage: 10, melee: true, ..rifle },
        SKIN_SPITTER => Combat { speed: 1.7, range: 4.0, windup: 0.65, cooldown: 1.5, damage: 14, ..rifle },
        SKIN_VEYRAN => Combat { speed: 1.28, range: 5.0, windup: 1.0, cooldown: 1.4, damage: 18, pellets: 5, spread: 0.16, ..rifle },
        _ if kind == EK_BRUTE => Combat { speed: 2.15, range: 1.1, windup: 0.6, cooldown: 0.9, damage: 14, melee: true, ..rifle },
        _ => rifle,
    }
}
