//! Read-only player-visible landmarks/pickups for simulation QA. No debug actions.
use crate::*;
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ItemCue {
    pub id: f32,
    pub category: f32,
    pub x: f32,
    pub y: f32,
    pub screen_x: f32,
    pub distance: f32,
    pub slot: f32,
}
pub(crate) fn snapshot(e: &Engine, out: &mut [ItemCue]) -> usize {
    let (dx, dy) = (e.pa.cos(), e.pa.sin());
    let plane = 0.72 * (e.w as f32 / e.h as f32 / 1.6);
    let mut n = 0;
    for (id, en) in e.ents.iter().enumerate() {
        if n >= out.len() {
            break;
        }
        let (category, slot) = match en.kind {
            EK_MED => (3, 0),
            EK_AMMO => (2, 0),
            EK_ARMOR => (4, 0),
            EK_NODE => (5, 0),
            EK_OVERRIDE_CONSOLE => (6, 0),
            EK_GUN2 => (1, 1),
            EK_GUN3 => (1, 2),
            EK_GUN4 => (1, 3),
            EK_GUN5 => (1, 4),
            EK_GUN6 => (1, 5),
            EK_GUN7 => (1, 6),
            EK_GUN8 => (1, 7),
            k if field::is_boss_case(k) => (7, field::boss_slot(k)),
            _ => continue,
        };
        let (x, y) = (en.x - e.px, en.y - e.py);
        let depth = x * dx + y * dy;
        let distance = x.hypot(y);
        if depth <= 0.12 || distance > 14.0 || !e.los(e.px, e.py, en.x, en.y) {
            continue;
        }
        let screen_x = 0.5 * (1.0 + (-dy * x + dx * y) / (plane * depth));
        if !(0.0..=1.0).contains(&screen_x) {
            continue;
        }
        out[n] = ItemCue {
            id: id as f32,
            category: category as f32,
            x: en.x,
            y: en.y,
            screen_x,
            distance,
            slot: slot as f32,
        };
        n += 1;
    }
    n
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn view_is_bounded_excludes_walls_behind_and_hostiles_without_mutation() {
        let mut e = crate::testutil::arena();
        e.px = 4.5;
        e.py = 4.5;
        e.pa = 0.0;
        let _ = e.spawn(EK_AMMO, 7.5, 4.5);
        let _ = e.spawn(EK_MED, 2.5, 4.5);
        let _ = e.spawn(EK_ARMOR, 9.5, 4.5);
        let _ = e.spawn(EK_HUSK, 6.5, 4.5);
        e.map[4 * MAP_W + 8] = 1;
        let before = (e.health, e.ammo, e.rng, e.px);
        let mut cues = [ItemCue::default(); ENT_N];
        let count = snapshot(&e, &mut cues);
        assert_eq!(count, 1);
        assert_eq!(cues[0].category, 2.0);
        assert_eq!(before, (e.health, e.ammo, e.rng, e.px));
        assert_eq!(snapshot(&e, &mut []), 0);
        assert_eq!(core::mem::size_of::<ItemCue>(), 28);
    }
}
