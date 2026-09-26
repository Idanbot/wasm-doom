//! Sector objectives, shield facing, lockdown doors, and radio line ids.
//!
//! Map geometry stays in `map.rs`. This module is the contract for the
//! systems layered on top of those three sectors: a required node before the
//! override, readable terminals, secret-cache interiors, and the doors a
//! boss phase seals.

use crate::consts::*;
use crate::map::level_index;

pub(crate) const SHIELD_BIT: u8 = 0x80;
pub(crate) const RADIO_HANDLER: i32 = 1;
pub(crate) const RADIO_LOCKDOWN: i32 = 2;
pub(crate) const RADIO_EXPOSED: i32 = 3;
pub(crate) const RADIO_NODE: i32 = 4;
pub(crate) const RADIO_SECRET: i32 = 5;
pub(crate) const RADIO_BOSS_DROP: i32 = 6;
pub(crate) const RADIO_POWER: i32 = 7;
pub(crate) const RADIO_BOSS_KILL: i32 = 8;
pub(crate) const POWER_OVERDRIVE: u8 = 1;
pub(crate) const POWER_FEED: u8 = 2;
pub(crate) const POWER_AEGIS: u8 = 3;

pub(crate) fn visual_skin(skin: u8) -> u8 {
    skin & !SHIELD_BIT
}

pub(crate) fn is_shielded_spawn(skin: u8) -> bool {
    skin & SHIELD_BIT != 0
}

/// True when a hit origin sits in the shield's front arc.
pub(crate) fn shield_blocks(face: f32, ex: f32, ey: f32, hx: f32, hy: f32) -> bool {
    let dx = hx - ex;
    let dy = hy - ey;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.08 {
        return true;
    }
    let nx = dx / len;
    let ny = dy / len;
    nx * face.cos() + ny * face.sin() > 0.78
}

pub(crate) fn node_point(wave: i32) -> (f32, f32) {
    [(19.5, 4.5), (16.5, 10.5), (13.5, 15.5)][level_index(wave)]
}

pub(crate) fn terminals(wave: i32) -> &'static [(f32, f32)] {
    match level_index(wave) {
        1 => &[(6.5, 6.5), (38.5, 19.5)],
        2 => &[(6.5, 15.5), (36.5, 16.5)],
        _ => &[(8.5, 15.5), (33.5, 15.5)],
    }
}

pub(crate) fn radio_terminal(sector: usize, index: usize) -> i32 {
    10 + sector as i32 * 4 + index as i32
}

pub(crate) fn lockdown_doors(wave: i32) -> &'static [(i32, i32)] {
    match level_index(wave) {
        1 => &[(28, 23), (28, 24), (37, 14), (38, 14)],
        2 => &[(30, 10), (30, 11), (30, 15), (30, 16), (30, 21), (30, 22)],
        _ => &[(36, 18), (37, 18), (36, 19), (37, 19)],
    }
}

/// Interior cell behind a secret door. Opening either leaf of a pair pays once.
pub(crate) fn secret_interior(cx: i32, cy: i32) -> Option<(f32, f32)> {
    const PAIRS: &[(i32, i32, f32, f32)] = &[
        (4, 19, 4.5, 21.5),
        (5, 19, 5.5, 21.5),
        (29, 12, 30.5, 13.5),
        (29, 13, 30.5, 13.5),
        (33, 24, 32.5, 25.5),
        (33, 25, 32.5, 25.5),
        (4, 3, 4.5, 2.5),
        (5, 3, 5.5, 2.5),
        (31, 28, 31.5, 29.5),
        (32, 28, 32.5, 29.5),
        (14, 13, 13.5, 13.5),
        (14, 14, 13.5, 14.5),
        (45, 15, 46.5, 15.5),
        (45, 16, 46.5, 16.5),
    ];
    PAIRS.iter().find(|p| p.0 == cx && p.1 == cy).map(|p| (p.2, p.3))
}

/// Extra med, armor, ammo, and the map gun case. Each sector drops the same
/// gun style; later sectors can swap the kind without a new pickup path.
pub(crate) fn resupply(wave: i32) -> &'static [(u8, f32, f32)] {
    match level_index(wave) {
        1 => &[
            (EK_MED, 7.5, 4.5),
            (EK_ARMOR, 21.5, 7.5),
            (EK_AMMO, 36.5, 22.5),
            (EK_GUN8, 42.5, 8.5),
        ],
        2 => &[
            (EK_MED, 6.5, 16.5),
            (EK_ARMOR, 18.5, 15.5),
            (EK_AMMO, 38.5, 15.5),
            (EK_GUN8, 40.5, 16.5),
        ],
        _ => &[
            (EK_MED, 7.5, 17.5),
            (EK_ARMOR, 10.5, 14.5),
            (EK_AMMO, 18.5, 14.5),
            (EK_GUN8, 6.5, 14.5),
        ],
    }
}

pub(crate) fn boss_case(wave: i32) -> u8 {
    [EK_GUN9, EK_GUN10, EK_GUN11][level_index(wave)]
}

pub(crate) fn is_boss_case(kind: u8) -> bool {
    matches!(kind, EK_GUN9 | EK_GUN10 | EK_GUN11)
}

pub(crate) fn boss_slot(kind: u8) -> usize {
    match kind {
        EK_GUN10 => 9,
        EK_GUN11 => 10,
        _ => 8,
    }
}

/// One secret powerup per sector, behind a secret door.
pub(crate) fn powerup_point(wave: i32) -> (f32, f32) {
    [(5.5, 21.5), (31.5, 29.5), (46.5, 15.5)][level_index(wave)]
}

pub(crate) fn power_kind(wave: i32) -> u8 {
    [POWER_OVERDRIVE, POWER_FEED, POWER_AEGIS][level_index(wave)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shield_blocks_the_front_and_not_the_back() {
        let face = 0.0;
        assert!(shield_blocks(face, 5.0, 5.0, 7.0, 5.0));
        assert!(!shield_blocks(face, 5.0, 5.0, 3.0, 5.0));
        assert!(!shield_blocks(face, 5.0, 5.0, 5.0, 7.0));
    }
}
