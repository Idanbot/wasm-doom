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

/// One on-screen damage bar. `layer` is 0 health, 1 armor, 2 shield.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct BarCue {
    pub screen_x: f32,
    pub screen_y: f32,
    pub frac: f32,
    pub layer: f32,
    pub fade: f32,
}

pub(crate) fn snapshot_bars(e: &Engine, out: &mut [BarCue]) -> usize {
    let (dx, dy) = (e.pa.cos(), e.pa.sin());
    let plane = 0.72 * (e.w as f32 / e.h as f32 / 1.6);
    let horizon = e.h as f32 * (0.5 + e.pitch * 0.9);
    let mut n = 0;
    for enemy in e.ents.iter() {
        if n >= out.len() { break; }
        if !crate::enemies::is_hostile_kind(enemy.kind) || enemy.hp <= 0 || enemy.bar_t <= 0.0 {
            continue;
        }
        let (sx, sy) = (enemy.x - e.px, enemy.y - e.py);
        let depth = sx * dx + sy * dy;
        if depth < 0.2 { continue; }
        let screen_x = 0.5 * (1.0 + (-dy * sx + dx * sy) / (plane * depth));
        if !(0.02..0.98).contains(&screen_x) { continue; }
        let scale = skin_def(crate::field::visual_skin(enemy.skin)).map(|s| s.scale).unwrap_or(0.9);
        let head = (horizon + enemy.zoff / depth) / e.h as f32 - scale / (2.0 * depth) - 0.02;
        let (frac, layer) = if enemy.shield_hp > 0 {
            (enemy.shield_hp as f32 / crate::enemies::SHIELD_CAP as f32, 2.0)
        } else if enemy.armor_hp > 0 {
            let cap = crate::enemies::armor_cap(enemy.kind, enemy.skin).max(1) as f32;
            (enemy.armor_hp as f32 / cap, 1.0)
        } else {
            let cap = crate::enemies::health_cap(enemy.kind, enemy.shield != 0).max(1) as f32;
            (enemy.hp as f32 / cap, 0.0)
        };
        let fade = if enemy.bar_t > 0.55 { 1.0 } else { enemy.bar_t / 0.55 };
        out[n] = BarCue {
            screen_x,
            screen_y: head,
            frac: frac.clamp(0.0, 1.0),
            layer,
            fade,
        };
        n += 1;
    }
    n
}
