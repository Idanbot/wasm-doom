//! Read-only presentation snapshot. No gameplay decisions or audio in Rust.
use crate::{consts::*, enemies::skin_def, Engine};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct EnemyCue {
    pub id: f32, pub skin: f32, pub anim: f32, pub hp: f32,
    pub x: f32, pub y: f32, pub screen_x: f32, pub screen_y: f32,
    pub sight: f32, pub distance: f32,
}

pub(crate) fn snapshot(e: &Engine, out: &mut [EnemyCue; ENT_N]) -> usize {
    let (dx, dy) = (e.pa.cos(), e.pa.sin());
    let plane = 0.72 * (e.w as f32 / e.h as f32 / 1.6);
    let horizon = e.h as f32 * (0.5 + e.pitch * 0.9);
    let mut n = 0;
    for (id, enemy) in e.ents.iter().enumerate() {
        if enemy.kind == 0 { continue; }
        let Some(skin) = skin_def(enemy.skin) else { continue; };
        let (sx, sy) = (enemy.x - e.px, enemy.y - e.py);
        let depth = sx * dx + sy * dy;
        let distance = (sx * sx + sy * sy).sqrt();
        if distance > 20.0 { continue; }
        let screen_x = if depth > 0.12 { 0.5 * (1.0 + (-dy * sx + dx * sy) / (plane * depth)) } else { -10.0 };
        let screen_y = if depth > 0.12 { (horizon + enemy.zoff / depth) / e.h as f32 - skin.scale / (2.0 * depth) } else { -10.0 };
        out[n] = EnemyCue { id: id as f32, skin: enemy.skin as f32, anim: enemy.anim as f32, hp: enemy.hp as f32,
            x: enemy.x, y: enemy.y, screen_x, screen_y,
            sight: if e.los(e.px, e.py, enemy.x, enemy.y) { 1.0 } else { 0.0 }, distance };
        n += 1;
    }
    n
}
