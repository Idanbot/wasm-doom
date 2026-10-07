//! Small free helpers shared across the engine.
//!
//! These are the pure functions: entity-kind classification, animation timing
//! and colour helpers. Nothing here touches `Engine` state, which is why they
//! live apart from the simulation modules.

use super::*;

pub(crate) fn clamp_i(v: i32, a: i32, b: i32) -> i32 {
    if v < a {
        a
    } else if v > b {
        b
    } else {
        v
    }
}
pub(crate) fn solid_kind(k: u8) -> bool {
    k == EK_BARREL || is_hostile_kind(k)
}

/// Hitscan and blasts can break crates and shut lamps. Movement still uses
/// `solid_kind`, so these props stay walk-through.
pub(crate) fn target_kind(k: u8) -> bool {
    solid_kind(k)
        || k == EK_CRATE
        || k == EK_LAMP
        || matches!(
            k,
            EK_PROP_REACTOR
                | EK_PROP_SERVER
                | EK_PROP_AC
                | EK_PROP_VENT
                | EK_PROP_WLIGHT_C
                | EK_PROP_WLIGHT_W
                | EK_PROP_BEACON
        )
}

pub(crate) fn floor_prop(k: u8) -> bool {
    is_pickup(k)
        || matches!(
            k,
            EK_CRATE
                | EK_BARREL
                | EK_OVERRIDE_CONSOLE
                | EK_NODE
                | EK_TERMINAL
                | EK_FLAME
                | EK_FIREPATCH
                | EK_PROP_REACTOR
                | EK_PROP_SERVER
                | EK_PROP_AC
                | EK_PROP_VENT
                | EK_PROP_WLIGHT_C
                | EK_PROP_WLIGHT_W
                | EK_PROP_BEACON
        )
}

/// Seat a sprite's feet on the floor line. Camera sits at mid-wall, so a
/// scale-1 sprite is already planted; shorter art needs a downward bias,
/// plus a pad for the transparent margin under prop masters.
pub(crate) fn floor_seat(h: f32, scale: f32, bob: f32) -> f32 {
    (h * 0.5) * (1.0 - scale) + h * 0.14 + bob
}

pub(crate) fn segment_distance_sq(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let t = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy).max(1e-8)).clamp(0.0, 1.0);
    (x - ax - dx * t).powi(2) + (y - ay - dy * t).powi(2)
}

pub(crate) fn animation_fps(state: u8) -> f32 {
    match state { ANIM_IDLE => 2.2, ANIM_MOVE => 8.0, ANIM_PAIN => 7.0,
        ANIM_FIRE => 9.0, ANIM_RELOAD => 5.5, ANIM_DEAD => 3.2,
        ANIM_SPECIAL => 5.0, _ => 4.0 }
}

pub(crate) fn set_anim(e: &mut Ent, state: u8, lock: f32) {
    if e.anim != state {
        e.anim = state;
        e.anim_time = 0.0;
    }
    // One-shot states must actually display all authored poses before expiry.
    let duration = if !is_hostile_kind(e.kind) || matches!(state, ANIM_IDLE | ANIM_MOVE) { lock }
        else { lock.max(4.0 / animation_fps(state)) };
    e.anim_lock = e.anim_lock.max(duration);
}

pub(crate) fn advance_anim(e: &mut Ent, dt: f32) {
    e.anim_time += dt;
    if e.anim_lock > 0.0 {
        e.anim_lock = (e.anim_lock - dt).max(0.0);
    }
    if e.hp <= 0 {
        if e.anim_lock <= 0.0 {
            e.kind = EK_NONE;
        }
    } else if e.anim_lock <= 0.0 && matches!(e.anim, ANIM_PAIN | ANIM_FIRE | ANIM_RELOAD | ANIM_SPECIAL) {
        e.anim = ANIM_IDLE;
        e.anim_time = 0.0;
    }
}

pub(crate) fn anim_frame(e: &Ent) -> i32 {
    let state = (e.anim as usize).min(ANIM_FRAME_COUNTS.len() - 1);
    let count = ANIM_FRAME_COUNTS[state].max(1) as i32;
    let frame = (e.anim_time * animation_fps(e.anim)) as i32;
    if matches!(e.anim, ANIM_IDLE | ANIM_MOVE) { frame.rem_euclid(count) }
    else { frame.min(count - 1) }
}

pub(crate) fn sigil_rgba(level: usize, u: f32, v: f32) -> [u32; 4] {
    let d = (u * u + v * v).sqrt();
    if d > 1.02 {
        return [0, 0, 0, 0];
    }
    let ring: f32 = if (d - 0.72).abs() < 0.04 { 1.0 } else { 0.0 };
    let (r, g, b, cover) = match level {
        1 => {
            let chev = if (u.abs() * 0.7 + v * 0.45 - 0.15).abs() < 0.035 { 1.0 } else { 0.0 };
            let bar = if (v + 0.35).abs() < 0.03 && u.abs() < 0.55 { 1.0 } else { 0.0 };
            (255, 107, 31, ring.max(chev).max(bar))
        }
        2 => {
            let hex = ((v.atan2(u) / 1.047).fract() - 0.5).abs();
            let edge = if (d - 0.62).abs() < 0.035 && hex < 0.2 { 1.0 } else { 0.0 };
            let mote = if ((u - 0.28).powi(2) + (v - 0.1).powi(2)).sqrt() < 0.07 { 1.0 } else { 0.0 };
            (89, 242, 107, ring.max(edge).max(mote))
        }
        3 => {
            let bus = if (u.abs() - 0.33).abs() < 0.025 || (v.abs() - 0.33).abs() < 0.025 { 1.0 } else { 0.0 };
            let core = if u.abs() < 0.1 && v.abs() < 0.1 { 0.8 } else { 0.0 };
            (70, 209, 250, ring.max(bus).max(core))
        }
        4 => {
            let core = if d < 0.18 { 0.85 } else { 0.0 };
            let spoke = if (u.abs() < 0.025 || v.abs() < 0.025) && d < 0.55 { 1.0 } else { 0.0 };
            (255, 169, 55, ring.max(core).max(spoke))
        }
        _ => {
            let slit = if u.abs() < 0.035 && v.abs() < 0.36 { 1.0 } else { 0.0 };
            let iris = if ((u * 1.4).powi(2) + v * v).sqrt() < 0.15 { 0.85 } else { 0.0 };
            (242, 158, 56, ring.max(slit).max(iris))
        }
    };
    if cover <= 0.0 || d > 0.98 {
        return [0, 0, 0, 0];
    }
    [r, g, b, (cover * 210.0) as u32]
}

pub(crate) fn sprite_style(e: &Ent) -> (usize, f32, bool, i32) {
    if matches!(e.kind, EK_PROJ|EK_BOLT|EK_RAY|EK_FLAME) && (1..=25).contains(&e.projectile_visual) {
        let index=(e.projectile_visual-1) as usize;
        let scale=if index==15 {0.38+(e.frame*24.0).sin()*0.018} else if matches!(index,8|17|21) {0.40} else if e.kind==EK_RAY {0.10} else {0.25};
        return (T_BOSS_PROJECTILE+index,scale,false,0);
    }
    if e.kind == EK_CRATE && (150..=224).contains(&e.skin) {
        return (T_SECTOR_PROP + (e.skin - 150) as usize, 0.85, false, 0);
    }
    if e.kind == EK_PROJ && (100..=136).contains(&e.skin) {
        let skin = (e.skin - 100) as usize;
        return (T_ENEMY_PROJECTILE + skin, if matches!(skin, 20 | 29 | 33) { 0.36 } else { 0.22 }, false, 0);
    }
    if (51..=64).contains(&e.kind) { return (T_EXPANSION_CASE + (e.kind - 51) as usize, 0.5, false, 0); }
    if e.kind == EK_IMPACT && (200..=203).contains(&e.skin) { return (T_IMPACT_NEW + (e.skin - 200) as usize, 0.45, true, ((e.frame * 12.0) as i32).min(3)); }
    let fx = matches!(
        e.kind,
        EK_PROJ | EK_RAY | EK_BOLT | EK_SMOKE | EK_FLAME | EK_FIREPATCH | EK_SPARK | EK_IMPACT | EK_GIB
    );
    if !fx {
        if matches!(e.kind, EK_OVERRIDE_CONSOLE | EK_NODE | EK_TERMINAL) {
            let texture = match e.skin {
                SKIN_CONSOLE_FOUNDRY => T_CONSOLE_FOUNDRY,
                SKIN_CONSOLE_BIOFORGE => T_CONSOLE_BIOFORGE,
                _ => T_CONSOLE_UPPER,
            };
            let scale = if e.kind == EK_TERMINAL { 0.62 } else if e.kind == EK_NODE { 0.82 } else { 0.95 };
            return (texture, scale, false, 0);
        }
        // Skin numbers are shared by props, effects and enemies. Only hostile
        // kinds may select enemy animation sheets (ordinary placed props use 0).
        if let Some(skin) = skin_def(e.skin).filter(|_| is_hostile_kind(e.kind)) {
            return (skin.texture + (e.anim as usize).min(ENEMY_ANIM_COUNT - 1), if e.kind == EK_BOSS { skin.scale.max(1.9) } else { skin.scale }, true, anim_frame(e));
        }
    }
    // VFX cells are distinct effects, not animation frames. Animate scale
    // over lifetime without cycling flames into smoke or muzzle flashes.
    match e.kind {
        EK_RAY => return (T_BALL, if e.zoff >= 11.0 { 0.07 } else { 0.065 }, false, 0),
        EK_PROJ => {
            if (230..=234).contains(&e.skin) {
                let visual = (e.skin - 230) as usize;
                return (if visual == 0 { T_PLAYER_MISSILE + if e.face as i32 == 11 { 2 } else { 0 } } else { T_PROJECTILE_NEW + visual }, if visual == 0 { 0.40 } else { 0.22 }, false, 0);
            }
            // 3 and 4 are the acid seeker. 4 stays allied so it never hits the shooter.
            let acid = e.effect_tick == 3.0 || e.effect_tick == 4.0;
            return (if acid { T_PROJECTILE_NEW + 3 } else { T_ENEMY_PROJECTILE }, 0.22, false, 0);
        }
        // The generated rear-view missile has a narrower silhouette than the
        // old fireball, so give it enough projected size to read in motion.
        EK_BOLT => return (T_PLAYER_MISSILE + usize::from(e.hp >= 85), 0.42, false, 0),
        EK_SMOKE => {
            if e.effect_tick >= 9.0 {
                return (T_FLAME, 1.85 + e.frame.min(2.4) * 0.45, false, -1);
            }
            return (T_FLAME, 0.22 + e.frame.min(0.8) * 0.4, true, 1);
        },
        EK_FLAME | EK_FIREPATCH => return (T_FLAME, 0.58 + (e.frame * 7.0).sin() * 0.035, true, 0),
        EK_SPARK if (16.0..24.0).contains(&e.effect_tick) =>
            return (T_BOSS_ENTRANCE+(e.effect_tick as usize-16),0.28+e.frame*0.35,false,0),
        EK_SPARK if e.effect_tick == 5.0 => return (if e.skin == 1 { T_CASING_SHELL } else { T_CASING }, 0.12, true,
            if e.shield >= 3 { 0 } else { ((e.frame * 18.0) as i32).rem_euclid(4) }),
        EK_SPARK => return if e.effect_tick < 4.0 {
            (T_MUZZLEFX, 0.10, true, e.effect_tick as i32)
        } else {
            (T_FLAME, if e.effect_tick == 5.0 { 0.08 } else { 0.12 }, true, if e.effect_tick == 5.0 { 3 } else { 2 })
        },
        EK_IMPACT => return (T_IMPACT, (0.24 + e.frame * 1.8).min(0.85), true, e.effect_tick as i32),
        _ => {}
    }
    match enemy_def(e.kind) {
        Some(d) => {
            let frame = if d.sheet4 { ((e.frame * 4.0) as i32).rem_euclid(4) } else { 0 };
            (d.texture, d.scale, d.sheet4, frame)
        }
        None => (T_SPLAT, 0.3, false, 0),
    }
}

pub(crate) fn is_pickup(k: u8) -> bool {
    matches!(
        k,
        EK_MED | EK_AMMO | EK_ARMOR | EK_POWER
    ) || is_weapon_item(k)
}

pub(crate) fn is_weapon_item(k: u8) -> bool {
    matches!(k, EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 | EK_GUN6 | EK_GUN7 | EK_GUN8)
        || field::is_boss_case(k)
}
