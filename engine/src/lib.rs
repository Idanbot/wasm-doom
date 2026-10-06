// Fixed-size parallel tables (MAG_SZ/RESERVE_CAP/AMMO_PICKUP/IN_W_SLOT and the
// light/sprite scratch grids) are indexed by slot throughout; the `zip` rewrite
// Clippy suggests reads worse and hides the slot invariant.
#![allow(clippy::needless_range_loop)]

#[cfg(test)]
mod spatial_checks;
mod boss_arena;
mod tactical;
mod tactical_roles;
mod combat;
mod campaign;
mod field;
mod voices;
mod sound;
mod consts;
mod enemies;
mod events;
mod hud;
mod map;
mod layouts;
mod types;

use consts::*;
use enemies::{default_skin, enemy_def, is_hostile_kind, skin_def};
use events::*;
pub use hud::{Hud, HUD_OFFSETS, HUD_SIZE};
use types::{AmbushTrigger, Ent, FxCmd};
#[derive(Clone, Copy)]
struct Smoke {
    x: f32,
    y: f32,
    age: f32,
    vx: f32,
    vy: f32,
}

impl Smoke {
    const DEAD: Self = Self { x: 0.0, y: 0.0, age: -1.0, vx: 0.0, vy: 0.0 };
}

struct Engine {
    w: usize,
    h: usize,
    fb: Vec<u32>,
    zbuf: Vec<f32>,
    tex: Vec<u32>,
    mipmaps: Vec<Vec<u32>>,
    static_light: Vec<[f32; 3]>,
    light_grid: Vec<[f32; 3]>,
    light_dirty: bool,
    map: Vec<u8>,
    floor: Vec<u8>,
    door: Vec<f32>,
    decal: Vec<u8>,
    ents: [Ent; ENT_N],
    px: f32,
    py: f32,
    pa: f32,
    pitch: f32,
    pr: f32,
    health: i32,
    armor: i32,
    ammo: [i32; WEP_N],
    mag: [i32; WEP_N],
    weapon: i32,
    /// Ownership flags for slots 1..=18. Slot 0 is the starting sidearm;
    /// slots 19.. live in `extra_weapons`.
    owned: [bool; OWNED_GUNS],
    /// Yaw-relative bearing of the last damage source, for the hurt vignette.
    hurt_dir: f32,
    /// Cached low-health strain level, refreshed every tick.
    strain: f32,
    extra_weapons: u32,
    pending_hostiles: usize,
    reinforcement_t: f32,
    reinforcement_cursor: usize,
    power: u8,
    power_t: f32,
    shield_pool: i32,
    cooldown: f32,
    reload_t: f32,
    reload_dur: f32,
    pickup_t: f32,
    pickup_dur: f32,
    iframes: f32,
    walk: f32,
    time: f32,
    elapsed: f32,
    kills: i32,
    secrets: i32,
    state: i32,
    bits: u32,
    mx: f32,
    my: f32,
    qa: bool,
    qa_bits: u32,
    hud: Hud,
    rng: u32,
    shake: f32,
    muzzle: f32,
    hurt: f32,
    kick: f32,
    hitmarker: f32,
    spread: f32,
    use_latched: bool,
    reload_latched: bool,
    wpn_latched: u32,
    sound_cues: Vec<sound::SoundCue>,
    sound_loops: Vec<sound::SoundCue>,
    events: u32,
    ev_weapon: i32,
    foot_acc: f32,
    flow_dir: Vec<u8>,
    flow_dist: Vec<u16>,
    flow_q: Vec<u16>,
    flow_age: f32,
    fx_q: [FxCmd; FX_CAP],
    fx_n: usize,
    wave: i32,
    hell: bool,
    boss_spawned: bool,
    boss_intro: f32,
    boss_phase: u8,
    boss_arena: boss_arena::ArenaState,
    tactical: tactical::Tactical,
    node_done: bool,
    lockdown: bool,
    boss_vuln: f32,
    radio_seq: i32,
    radio_line: i32,
    ambush: Vec<AmbushTrigger>,
    smokes: [Smoke; 8],
    smoke_grid: Vec<f32>,
    smoke_next: Vec<f32>,
    light_src: Vec<[f32; 3]>,
    /// Per-theme wall/door variants: 8 walls then 8 doors, 256x256 each.
    /// Slot order mirrors THEME_FILES in src/game/runtime.ts.
    theme_tex: Vec<u32>,
}

/// Single engine instance. WASM is single-threaded; raw-ref access keeps
/// `static_mut_refs` clippy-clean. All entry points run on the same JS
/// thread, so `eng()` exclusivity holds.
static mut E: Option<Engine> = None;

fn eng() -> &'static mut Engine {
    #[allow(static_mut_refs)]
    unsafe {
        // `&raw mut` avoids a `static_mut_refs` reference; clippy's suggested
        // bare `E` would take a real borrow of the static for the returned
        // lifetime, which is what this accessor is written to avoid.
        #[allow(clippy::deref_addrof)]
        (*(&raw mut E)).as_mut().expect("engine")
    }
}

fn eng_init(w: usize, h: usize) {
    #[allow(static_mut_refs)]
    unsafe {
        E = Some(Engine::new(w, h));
    }
}

/// Sanitises a float crossing the WASM ABI. `f32::clamp` propagates NaN and
/// leaves infinities alone, so a single bad value from JavaScript could
/// otherwise latch into permanent simulation state.
#[inline]
fn finite(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

/// Step clamp for the ABI: finite, non-negative, and capped. NaN becomes 0.
#[inline]
fn safe_dt(v: f32) -> f32 {
    finite(v).clamp(0.0, 0.08)
}

fn clamp_i(v: i32, a: i32, b: i32) -> i32 {
    if v < a {
        a
    } else if v > b {
        b
    } else {
        v
    }
}
fn solid_kind(k: u8) -> bool {
    k == EK_BARREL || is_hostile_kind(k)
}

/// Hitscan and blasts can break crates and shut lamps. Movement still uses
/// `solid_kind`, so these props stay walk-through.
fn target_kind(k: u8) -> bool {
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

fn floor_prop(k: u8) -> bool {
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
fn floor_seat(h: f32, scale: f32, bob: f32) -> f32 {
    (h * 0.5) * (1.0 - scale) + h * 0.14 + bob
}

fn segment_distance_sq(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let t = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy).max(1e-8)).clamp(0.0, 1.0);
    (x - ax - dx * t).powi(2) + (y - ay - dy * t).powi(2)
}

fn animation_fps(state: u8) -> f32 {
    match state { ANIM_IDLE => 2.2, ANIM_MOVE => 8.0, ANIM_PAIN => 7.0,
        ANIM_FIRE => 9.0, ANIM_RELOAD => 5.5, ANIM_DEAD => 3.2,
        ANIM_SPECIAL => 5.0, _ => 4.0 }
}

fn set_anim(e: &mut Ent, state: u8, lock: f32) {
    if e.anim != state {
        e.anim = state;
        e.anim_time = 0.0;
    }
    // One-shot states must actually display all authored poses before expiry.
    let duration = if !is_hostile_kind(e.kind) || matches!(state, ANIM_IDLE | ANIM_MOVE) { lock }
        else { lock.max(4.0 / animation_fps(state)) };
    e.anim_lock = e.anim_lock.max(duration);
}

fn advance_anim(e: &mut Ent, dt: f32) {
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

fn anim_frame(e: &Ent) -> i32 {
    let state = (e.anim as usize).min(ANIM_FRAME_COUNTS.len() - 1);
    let count = ANIM_FRAME_COUNTS[state].max(1) as i32;
    let frame = (e.anim_time * animation_fps(e.anim)) as i32;
    if matches!(e.anim, ANIM_IDLE | ANIM_MOVE) { frame.rem_euclid(count) }
    else { frame.min(count - 1) }
}

fn sigil_rgba(level: usize, u: f32, v: f32) -> [u32; 4] {
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

fn sprite_style(e: &Ent) -> (usize, f32, bool, i32) {
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
        if let Some(skin) = skin_def(e.skin) {
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

fn is_pickup(k: u8) -> bool {
    matches!(
        k,
        EK_MED | EK_AMMO | EK_ARMOR | EK_POWER
    ) || is_weapon_item(k)
}

fn is_weapon_item(k: u8) -> bool {
    matches!(k, EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 | EK_GUN6 | EK_GUN7 | EK_GUN8)
        || field::is_boss_case(k)
}

impl Engine {
    fn new(w: usize, h: usize) -> Self {
        Self::with_textures(w, h, None)
    }

    fn with_textures(w: usize, h: usize, textures: Option<Vec<u32>>) -> Self {
        let generate = textures.is_none();
        let w = w.clamp(160, MAX_W);
        let h = h.clamp(100, MAX_H);
        let mut e = Self {
            w,
            h,
            fb: vec![0; w * h],
            zbuf: vec![0.0; w],
            tex: textures.unwrap_or_else(|| vec![0; TEX_N * TEX * TEX]),
            mipmaps: Vec::new(),
            static_light: vec![[0.0; 3]; MAP_CELLS],
            light_grid: vec![[0.0; 3]; MAP_CELLS],
            light_dirty: true,
            map: vec![1; MAP_W * MAP_H],
            floor: vec![0; MAP_W * MAP_H],
            door: vec![0.0; MAP_W * MAP_H],
            decal: vec![0; MAP_W * MAP_H],
            ents: [Ent {
                aim: 0.0,
                kind: 0,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                hp: 0,
                timer: 0.0,
                frame: 0.0,
                anim: ANIM_IDLE,
                anim_time: 0.0,
                anim_lock: 0.0,
                skin: SKIN_NONE,
                projectile_visual: 0,
                radius: 0.25,
                flash: 0.0,
                stun: 0.0,
                effect_tick: 0.0,
                shield: 0,
                face: 0.0,
                zoff: 0.0,
                bar_t: 0.0,
                armor_hp: 0,
                shield_hp: 0,
            }; ENT_N],
            px: map::PLAYER_START.0,
            py: map::PLAYER_START.1,
            pa: map::PLAYER_START.2,
            pitch: 0.0,
            pr: 0.22,
            health: 100,
            armor: 0,
            ammo: { let mut a = [0; WEP_N]; a[0] = 36; a },
            mag: { let mut a = [0; WEP_N]; a[0] = 12; a },
            weapon: 0,
            owned: [false; OWNED_GUNS],
            hurt_dir: 0.0,
            strain: 0.0,
            extra_weapons: 0,
            pending_hostiles: 0,
            reinforcement_t: 0.0,
            reinforcement_cursor: 0,
            power: 0,
            power_t: 0.0,
            shield_pool: 0,
            cooldown: 0.0,
            reload_t: 0.0,
            reload_dur: 1.0,
            pickup_t: 0.0,
            pickup_dur: 0.6,
            iframes: 0.0,
            walk: 0.0,
            time: 0.0,
            elapsed: 0.0,
            kills: 0,
            secrets: 0,
            state: 0,
            bits: 0,
            mx: 0.0,
            my: 0.0,
            qa: false,
            qa_bits: 0,
            hud: Hud {
                hurt_dir: 0.0,
                strain: 0.0,
                health: 100,
                armor: 0,
                ammo: 12,
                weapon: 0,
                kills: 0,
                living: 0,
                state: 0,
                prompt: 0,
                has_w2: 0,
                has_w3: 0,
                secrets: 0,
                elapsed_ms: 0,
                shake: 0.0,
                muzzle: 0.0,
                hurt: 0.0,
                bob: 0.0,
                kick: 0.0,
                hitmarker: 0.0,
                yaw: 0.0,
                speed: 0.0,
                x: map::PLAYER_START.0,
                y: map::PLAYER_START.1,
                reserve: 36,
                reloading: 0.0,
                weap_frame: 0,
                has_w4: 0,
                has_w5: 0,
                events: 0,
                ev_weapon: 0,
                wave: 1,
                boss_health: 0,
                boss_max_health: 0,
                boss_phase: 0,
                has_w6: 0,
                has_w7: 0,
                has_w8: 0,
                objective: 0,
                radio_seq: 0,
                radio_line: 0,
                vuln: 0.0,
                node_x: 0.0,
                node_y: 0.0,
                has_w9: 0,
                has_w10: 0,
                has_w11: 0,
                has_w12: 0,
                has_w13: 0,
                has_w14: 0,
                has_w15: 0,
                has_w16: 0,
                has_w17: 0,
                has_w18: 0,
                has_w19: 0,
                extra_weapons: 0,
                power: 0,
                power_t: 0.0,
                splash: 0.0,
            },
            rng: 0xC0FFEE,
            shake: 0.0,
            muzzle: 0.0,
            hurt: 0.0,
            kick: 0.0,
            hitmarker: 0.0,
            spread: 0.0,
            use_latched: false,
            reload_latched: false,
            wpn_latched: 0,
            sound_cues: Vec::with_capacity(sound::CAP),
            sound_loops: Vec::with_capacity(sound::CAP),
            events: 0,
            ev_weapon: 0,
            foot_acc: 0.0,
            flow_dir: vec![0; MAP_CELLS],
            flow_dist: vec![0xFFFF; MAP_CELLS],
            flow_q: vec![0; MAP_CELLS],
            flow_age: 1.0,
            fx_q: [FxCmd {
                projectile_visual: 0,
                variant: 0,
                kind: 0,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                timer: 0.0,
                zoff: 0.0,
            }; FX_CAP],
            fx_n: 0,
            wave: 1,
            hell: false,
            boss_spawned: false,
            boss_intro: 0.0,
            boss_phase: 0,
            boss_arena: boss_arena::ArenaState::new(),
            tactical: tactical::Tactical::new(),
            node_done: false,
            lockdown: false,
            boss_vuln: 0.0,
            radio_seq: 0,
            radio_line: 0,
            ambush: Vec::new(),
            smokes: [Smoke::DEAD; 8],
            smoke_grid: vec![0.0; MAP_CELLS],
            smoke_next: vec![0.0; MAP_CELLS],
            light_src: vec![[0.0; 3]; MAP_CELLS],
            theme_tex: vec![0; 16 * TEX * TEX],
        };
        e.build_map();
        if generate { e.gen_textures(); }
        e.rebuild_mipmaps();
        e.place_ents();
        e
    }

    fn rnd(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.rng >> 8) as f32 / 16777216.0
    }

    fn spawn_smoke_cloud(&mut self, x: f32, y: f32) {
        let mut slot = 0usize;
        let mut oldest = -1.0f32;
        for (i, s) in self.smokes.iter().enumerate() {
            if s.age < 0.0 {
                slot = i;
                break;
            }
            if s.age > oldest {
                oldest = s.age;
                slot = i;
            }
        }
        // Grid-only smoke: the volumetric alpha effect in the smoke grid is
        // the whole effect now. No EK_SMOKE sprite companions are spawned.
        let drift = self.rnd() * core::f32::consts::TAU;
        self.smokes[slot] = Smoke {
            x,
            y,
            age: 0.0,
            vx: drift.cos() * 0.22,
            vy: drift.sin() * 0.22,
        };
    }

    fn age_smoke(&mut self, dt: f32) {
        for i in 0..self.smokes.len() {
            if self.smokes[i].age < 0.0 { continue; }
            self.smokes[i].age += dt;
            if self.smokes[i].age >= 4.0 {
                self.smokes[i] = Smoke::DEAD;
                continue;
            }
            let s = self.smokes[i];
            let mut vx = s.vx + (s.y * 1.7 + s.age * 3.0).sin() * dt * 1.1;
            let mut vy = s.vy + (s.x * 1.3 - s.age * 2.4).cos() * dt * 1.1;
            let nx = s.x + vx * dt;
            let ny = s.y + vy * dt;
            let mut x = s.x;
            let mut y = s.y;
            if self.blocked(nx.floor() as i32, s.y.floor() as i32) { vx = -vx * 0.45; } else { x = nx; }
            if self.blocked(x.floor() as i32, ny.floor() as i32) { vy = -vy * 0.45; } else { y = ny; }
            self.smokes[i].x = x;
            self.smokes[i].y = y;
            self.smokes[i].vx = vx * 0.985;
            self.smokes[i].vy = vy * 0.985;
        }
    }

    fn smoke_radius(age: f32) -> f32 {
        0.85 + (age / 2.0).clamp(0.0, 1.0) * 1.25
    }

    fn smoke_strength(age: f32) -> f32 {
        if age < 0.0 { return 0.0; }
        let bloom = 0.28 + 0.20 * (age / 0.4).clamp(0.0, 1.0);
        let fade = if age > 2.4 { ((4.0 - age) / 1.6).clamp(0.0, 1.0) } else { 1.0 };
        bloom * fade
    }

    fn rebuild_smoke_grid(&mut self) {
        // Scratch buffer: zeroed in place every frame, never reallocated.
        self.smoke_next.fill(0.0);
        let mut next = core::mem::take(&mut self.smoke_next);
        for y in 1..MAP_H - 1 {
            for x in 1..MAP_W - 1 {
                if self.blocked(x as i32, y as i32) { continue; }
                let i = y * MAP_W + x;
                let here = self.smoke_grid[i];
                if here < 0.015 { continue; }
                let gx = self.smoke_grid[i - 1] - self.smoke_grid[i + 1];
                let gy = self.smoke_grid[i - MAP_W] - self.smoke_grid[i + MAP_W];
                let sx = (x as f32 + 0.5 - gx * 0.55).clamp(0.0, (MAP_W - 1) as f32);
                let sy = (y as f32 + 0.5 - gy * 0.55).clamp(0.0, (MAP_H - 1) as f32);
                if self.blocked(sx.floor() as i32, sy.floor() as i32) { continue; }
                let dest = sy as usize * MAP_W + sx as usize;
                next[dest] = (next[dest] + here * 0.80).min(0.55);
            }
        }
        self.smoke_grid.copy_from_slice(&next);
        self.smoke_next = next;
        for s in self.smokes {
            if s.age < 0.0 { continue; }
            let radius = Self::smoke_radius(s.age);
            let strength = Self::smoke_strength(s.age);
            let x0 = (s.x - radius).floor().max(0.0) as usize;
            let y0 = (s.y - radius).floor().max(0.0) as usize;
            let x1 = ((s.x + radius).ceil() as usize).min(MAP_W - 1);
            let y1 = ((s.y + radius).ceil() as usize).min(MAP_H - 1);
            for cy in y0..=y1 {
                for cx in x0..=x1 {
                    if self.blocked(cx as i32, cy as i32) { continue; }
                    let d = ((cx as f32 + 0.5 - s.x).powi(2) + (cy as f32 + 0.5 - s.y).powi(2)).sqrt();
                    if d >= radius { continue; }
                    let f = 1.0 - d / radius;
                    let cover = f * f * (3.0 - 2.0 * f) * strength;
                    let i = cy * MAP_W + cx;
                    self.smoke_grid[i] = (self.smoke_grid[i] + cover).min(0.55);
                }
            }
        }
    }

    fn cell(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return 1;
        }
        self.map[y as usize * MAP_W + x as usize]
    }

    fn set_cell(&mut self, x: i32, y: i32, v: u8) {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return;
        }
        self.map[y as usize * MAP_W + x as usize] = v;
    }

    fn blocked(&self, x: i32, y: i32) -> bool {
        let c = self.cell(x, y);
        if c == 0 || c == 10 {
            return false;
        }
        if c == 8 || c == 9 {
            let o = self.door[y as usize * MAP_W + x as usize];
            return o < 0.98;
        }
        true
    }

    fn near_override(&self) -> bool {
        let (x, y) = map::override_point(self.wave);
        let distance = (self.px - x).powi(2) + (self.py - y).powi(2);
        distance < 1.7 * 1.7 && self.los(self.px, self.py, x, y)
    }

    fn boss_intro_duration(&self) -> f32 {
        boss_arena::PROFILES[map::level_index(self.wave)].intro
    }

    fn boss_intro_effect(&mut self, stage: u8) {
        let sector=map::level_index(self.wave);
        let (x,y)=map::boss_spots(self.wave)[0];
        let progress=stage as f32/4.0;
        // Each boss has its own ray count, rotation, radius and particle family.
        // Ray marches stop at solid geometry; entrances never shine through walls.
        let rays=3+sector%6;
        let rotation=sector as f32*0.43+progress*(1.1+sector as f32*0.08);
        let radius=1.0+(1.0-progress)*(2.0+sector as f32*0.07);
        for ray in 0..rays {
            let angle=rotation+ray as f32*core::f32::consts::TAU/rays as f32;
            for step in 1..=6 {
                let r=radius*step as f32/6.0;
                let theta=angle+if sector%5==2 {r*0.75+progress*2.0} else {0.0};
                let (dx,dy)=match sector%5 {
                    1=>((if ray%2==0 {-1.0}else{1.0})*(radius-progress), (step as f32-3.5)*0.65),
                    3=>((ray as f32-(rays-1) as f32*0.5)*0.6,(step as f32-3.5)*radius/3.0),
                    4=>(angle.cos()*radius,angle.sin()*radius),
                    _=>(theta.cos()*r,theta.sin()*r),
                };
                let distance=(dx*dx+dy*dy).sqrt().max(0.01);
                let reach=(self.wall_distance(x,y,dx/distance,dy/distance,distance)-0.16).max(0.0);
                if reach<0.1 {continue;}
                let (sx,sy)=(x+dx*reach/distance,y+dy*reach/distance);
                if self.blocked(sx.floor() as i32,sy.floor() as i32) || !self.los(x,y,sx,sy) {continue;}
                let family=[6,1,2,3,4,5,1,0,6,7,5,6,7,3,4,3,2,0,7,5,4,7,5,1,0][sector];
                self.effect(EK_SPARK,16+family,sx,sy,0.3+progress*0.35,
                    self.h as f32*(-0.28+step as f32*0.07));
            }
        }
        self.shake=(self.shake+0.06+progress*0.16).min(0.65);
        self.sound(24,(sector*5+stage as usize) as u8,x,y);
        if stage==0 {self.events|=EV_BOSS_HUSH;}
        if stage==4 {self.hell=true;self.events|=EV_DOOR;}
    }

    fn room(&mut self, x: i32, y: i32, w: i32, h: i32, wall: u8, fl: u8) {
        for j in y..y + h {
            for i in x..x + w {
                let edge = i == x || j == y || i == x + w - 1 || j == y + h - 1;
                self.set_cell(i, j, if edge { wall } else { 0 });
                if i >= 0 && j >= 0 && (i as usize) < MAP_W && (j as usize) < MAP_H {
                    self.floor[j as usize * MAP_W + i as usize] = fl;
                }
            }
        }
    }

    fn hall_h(&mut self, x0: i32, x1: i32, y: i32, door_x: i32) {
        let (a, b) = if x0 < x1 { (x0, x1) } else { (x1, x0) };
        for x in a..=b {
            self.set_cell(x, y, 0);
            self.set_cell(x, y + 1, 0);
        }
        self.set_cell(door_x, y, 8);
        self.set_cell(door_x, y + 1, 8);
    }

    fn hall_v(&mut self, x: i32, y0: i32, y1: i32, door_y: i32) {
        let (a, b) = if y0 < y1 { (y0, y1) } else { (y1, y0) };
        for y in a..=b {
            self.set_cell(x, y, 0);
            self.set_cell(x + 1, y, 0);
        }
        self.set_cell(x, door_y, 8);
        self.set_cell(x + 1, door_y, 8);
    }

    fn pillar(&mut self, x: i32, y: i32, t: u8) {
        self.set_cell(x, y, t);
        self.set_cell(x + 1, y, t);
        self.set_cell(x, y + 1, t);
        self.set_cell(x + 1, y + 1, t);
    }

    fn mix_edge(&mut self, x: i32, y: i32, w: i32, h: i32, kinds: &[u8]) {
        if kinds.is_empty() {
            return;
        }
        let mut n = 0usize;
        for j in y..y + h {
            for i in x..x + w {
                let edge = i == x || j == y || i == x + w - 1 || j == y + h - 1;
                if !edge {
                    continue;
                }
                let cur = self.cell(i, j);
                if cur == 0 || cur == 8 || cur == 9 || cur == 10 {
                    continue;
                }
                self.set_cell(i, j, kinds[n % kinds.len()]);
                n += 1;
            }
        }
    }

    fn build_map(&mut self) {
        map::build_level(self);
        self.install_secret_cache();
    }

    fn pack(r: u32, g: u32, b: u32, a: u32) -> u32 {
        (r.min(255)) | (g.min(255) << 8) | (b.min(255) << 16) | (a.min(255) << 24)
    }

    fn nbit(n: u32, x: i32, y: i32) -> f32 {
        let mut h = n.wrapping_add((x as u32).wrapping_mul(374761393));
        h = h.wrapping_add((y as u32).wrapping_mul(668265263));
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        (h >> 8) as f32 / 16777216.0
    }

    fn gen_textures(&mut self) {
        for id in 0..TEX_N {
            // Authored extension layers have a flat fallback until preload completes.
            // Filling directly avoids hashing millions of pixels for unused noise.
            if id >= T_ENEMY_PROJECTILE {
                self.tex[id * TEX * TEX..(id + 1) * TEX * TEX].fill(Self::pack(22, 14, 12, 255));
                continue;
            }
            for y in 0..TEX {
                for x in 0..TEX {
                    let n = Self::nbit(id as u32 + 3, x as i32, y as i32);
                    let n2 = Self::nbit(id as u32 + 9, x as i32 / 2, y as i32 / 2);
                    let c = match id {
                        T_BRICK => {
                            let bx = (x.wrapping_add(if (y / 8) % 2 == 1 { 8 } else { 0 })) % 16;
                            let by = y % 8;
                            let mortar = bx == 0 || by == 0;
                            if mortar {
                                Self::pack(28, 18, 16, 255)
                            } else {
                                let r = 90.0 + n * 40.0 + n2 * 20.0;
                                Self::pack(r as u32, (r * 0.32) as u32, (r * 0.22) as u32, 255)
                            }
                        }
                        T_METAL => {
                            let rivet = (x % 32 == 4 || x % 32 == 27) && (y % 32 == 4 || y % 32 == 27);
                            let panel = (x % 32 < 2) || (y % 32 < 2);
                            if rivet {
                                Self::pack(160, 150, 140, 255)
                            } else if panel {
                                Self::pack(18, 18, 20, 255)
                            } else {
                                let g = 36.0 + n * 22.0;
                                Self::pack(g as u32, (g * 0.95) as u32, (g * 0.9) as u32, 255)
                            }
                        }
                        T_FLESH => {
                            let v = (n * 80.0 + n2 * 50.0) as u32;
                            let vein = ((x as i32 * 3 + y as i32 * 5) % 17) == 0;
                            if vein {
                                Self::pack(40, 8, 10, 255)
                            } else {
                                Self::pack(90 + v / 3, 18 + v / 8, 22 + v / 10, 255)
                            }
                        }
                        T_SKULL => {
                            let bone = n > 0.55;
                            if bone {
                                Self::pack(180, 165, 140, 255)
                            } else {
                                Self::pack(50, 28, 22, 255)
                            }
                        }
                        T_DOOR => {
                            let seam = x > 60 && x < 68;
                            let stripe = y % 24 < 5;
                            if seam {
                                Self::pack(12, 12, 12, 255)
                            } else if stripe {
                                Self::pack(90, 70, 18, 255)
                            } else {
                                Self::pack(40 + (n * 20.0) as u32, 38, 36, 255)
                            }
                        }
                        T_GRATE => {
                            let g = (x % 16 < 3) || (y % 16 < 3);
                            if g {
                                Self::pack(70, 68, 62, 255)
                            } else {
                                Self::pack(16, 12, 10, 255)
                            }
                        }
                        T_CONC => {
                            let g = 70.0 + n * 30.0;
                            Self::pack(g as u32, (g * 0.92) as u32, (g * 0.82) as u32, 255)
                        }
                        T_CEIL => {
                            let pipe = y % 32 < 6;
                            if pipe {
                                Self::pack(40, 38, 36, 255)
                            } else {
                                Self::pack(28, 24, 22, 255)
                            }
                        }
                        T_HUSK => Self::silhouette(x, y, 0xFF2040C0, n),
                        T_BRUTE => Self::silhouette(x, y, 0xFF103090, n),
                        T_WRAITH => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 28.0 {
                                Self::pack(220, 210, 190, 255)
                            } else if d < 48.0 {
                                Self::pack(220, 80, 20, 220)
                            } else if d < 58.0 {
                                Self::pack(180, 40, 10, 120)
                            } else {
                                0
                            }
                        }
                        T_MED => {
                            let boxy = x > 30 && x < 98 && y > 36 && y < 100;
                            let cross = (x > 56 && x < 72 && y > 44 && y < 92)
                                || (x > 40 && x < 88 && y > 58 && y < 74);
                            if cross {
                                Self::pack(200, 30, 30, 255)
                            } else if boxy {
                                Self::pack(50, 70, 40, 255)
                            } else {
                                0
                            }
                        }
                        T_AMMO => {
                            let boxy = x > 28 && x < 100 && y > 40 && y < 104;
                            if boxy {
                                Self::pack(150, 110, 40, 255)
                            } else {
                                0
                            }
                        }
                        T_ARMOR => {
                            let vest = x > 34 && x < 94 && y > 28 && y < 108;
                            if vest {
                                Self::pack(50, 55, 60, 255)
                            } else {
                                0
                            }
                        }
                        T_BARREL => {
                            let cx = x as i32 - 64;
                            let body = cx.abs() < 28 && y > 18 && y < 118;
                            let band = y > 54 && y < 70;
                            if band && body {
                                Self::pack(180, 150, 20, 255)
                            } else if body {
                                Self::pack(140, 40, 22, 255)
                            } else {
                                0
                            }
                        }
                        T_ORDNANCE => {
                            let dx = (x % 128) as f32 - 64.0;
                            let dy = (y % 128) as f32 - 64.0;
                            let alpha = (1.0 - (dx * dx + dy * dy).sqrt() / 44.0).clamp(0.0, 1.0);
                            let frame = (x / 128) + (y / 128) * 2;
                            let rgb = match frame { 1 => [255, 140, 30], 3 => [100, 230, 30], _ => [70, 180, 255] };
                            Self::pack(rgb[0], rgb[1], rgb[2], (alpha * 255.0) as u32)
                        }
                        T_BALL => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 18.0 {
                                Self::pack(255, 230, 160, 255)
                            } else if d < 32.0 {
                                Self::pack(255, 90, 20, 230)
                            } else if d < 42.0 {
                                Self::pack(180, 20, 10, 80)
                            } else {
                                0
                            }
                        }
                        T_SPLAT => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 20.0 + n * 16.0 {
                                Self::pack(120, 10, 10, 200)
                            } else {
                                0
                            }
                        }
                        T_TECH => {
                            let panel = (x % 32 < 3) || (y % 32 < 3);
                            let screen = x % 32 > 6 && x % 32 < 26 && y % 32 > 6 && y % 32 < 22;
                            let scan = (y / 2) % 3 == 0;
                            if panel {
                                Self::pack(12, 14, 16, 255)
                            } else if screen {
                                let glow = 40.0 + n * 80.0;
                                if scan {
                                    Self::pack(20, (glow * 0.3) as u32, (glow * 0.2) as u32, 255)
                                } else {
                                    Self::pack((glow * 0.9) as u32, 18, 22, 255)
                                }
                            } else {
                                Self::pack(28 + (n * 10.0) as u32, 24, 22, 255)
                            }
                        }
                        T_HAZARD => {
                            let stripe = ((x as i32 + y as i32) / 12) % 2 == 0;
                            if stripe {
                                Self::pack(170, 140, 28, 255)
                            } else {
                                Self::pack(18, 16, 12, 255)
                            }
                        }
                        T_LAMP => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 50;
                            let cage = cx.abs() < 22 && y > 18 && y < 100;
                            let bulb = (cx * cx + cy * cy) < 14 * 14;
                            if bulb {
                                Self::pack(255, 200, 90, 255)
                            } else if cage {
                                let wire = (x % 6 < 2) || (y % 8 < 2);
                                if wire {
                                    Self::pack(90, 70, 40, 255)
                                } else {
                                    Self::pack(40, 32, 22, 180)
                                }
                            } else {
                                0
                            }
                        }
                        T_CRATE => {
                            let boxy = x > 24 && x < 104 && y > 28 && y < 118;
                            let band = y > 68 && y < 78;
                            if band && boxy {
                                Self::pack(160, 90, 30, 255)
                            } else if boxy {
                                Self::pack(110, 48, 28, 255)
                            } else {
                                0
                            }
                        }
                        T_IMPACT => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 10.0 {
                                Self::pack(255, 230, 160, 255)
                            } else if d < 22.0 + n * 8.0 {
                                Self::pack(255, 120, 30, 200)
                            } else if d < 36.0 {
                                Self::pack(180, 40, 16, 90)
                            } else {
                                0
                            }
                        }
                        T_MUZZLEFX => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            let star = (cx.abs() < 4 && cy.abs() < 40) || (cy.abs() < 4 && cx.abs() < 40);
                            if d < 12.0 || star {
                                Self::pack(255, 220, 140, 255)
                            } else if d < 28.0 {
                                Self::pack(255, 90, 20, 160)
                            } else {
                                0
                            }
                        }
                        T_FLAME => {
                            let cx = x as i32 - 64;
                            let taper = 18.0 - (127.0 - y as f32) * 0.12 + n * 6.0;
                            let body = cx.abs() < taper as i32 && y > 20;
                            if body {
                                let hot = y < 70;
                                if hot {
                                    Self::pack(255, 230, 140, 255)
                                } else {
                                    Self::pack(255, 90 + (n * 40.0) as u32, 20, 230)
                                }
                            } else {
                                0
                            }
                        }
                        T_CHAIN => {
                            let cx = x as i32 - 64;
                            let link = cx.abs() < 8 && (y % 14 < 9);
                            let hook = y > 96 && cx.abs() < 18 && y < 122;
                            if hook {
                                Self::pack(140, 40, 28, 255)
                            } else if link {
                                Self::pack(90, 80, 70, 255)
                            } else {
                                0
                            }
                        }
                        T_PIPES => {
                            let v = x % 22 < 7;
                            let hpipe = y % 28 < 6;
                            let valve = (x % 22 == 3) && (y % 28 == 3);
                            if valve {
                                Self::pack(160, 70, 30, 255)
                            } else if v {
                                Self::pack(70 + (n * 30.0) as u32, 48, 40, 255)
                            } else if hpipe {
                                Self::pack(50, 46, 42, 255)
                            } else {
                                Self::pack(22, 18, 16, 255)
                            }
                        }
                        T_GUN2 => {
                            let body = x > 36 && x < 92 && y > 22 && y < 118;
                            let barrel = x > 88 && x < 118 && y > 52 && y < 70;
                            if barrel {
                                Self::pack(220, 180, 70, 255)
                            } else if body {
                                Self::pack(180 + (n * 40.0) as u32, 150, 40, 255)
                            } else {
                                0
                            }
                        }
                        _ => Self::pack(22, 14, 12, 255),
                    };
                    self.tex[id * TEX * TEX + y * TEX + x] = c;
                }
            }
        }
    }

    fn silhouette(x: usize, y: usize, color: u32, n: f32) -> u32 {
        let cx = x as i32 - 64;
        let cy = y as i32 - 20;
        let head = {
            let dx = cx;
            let dy = cy - 8;
            (dx * dx + dy * dy) < 16 * 16
        };
        let body = cx.abs() < (18.0 + n * 4.0) as i32 && y > 40 && y < 100;
        let legs = (cx.abs() - 8).abs() < 7 && (96..122).contains(&y);
        if head || body || legs {
            color
        } else {
            0
        }
    }

    fn spawn(&mut self, kind: u8, x: f32, y: f32) -> Option<usize> {
        self.spawn_with_skin(kind, default_skin(kind), x, y)
    }

    fn spawn_with_skin(&mut self, kind: u8, skin: u8, x: f32, y: f32) -> Option<usize> {
        // Stats come from the roster table so new enemies need no code here.
        // Health rises gradually across the campaign and without a gameplay cap after 25.
        let (mut hp, radius, zoff) = match enemy_def(kind) {
            Some(d) => (d.hp, d.radius, d.zoff),
            None => (1, 0.2, 0.0),
        };
        let (x, y) = if is_hostile_kind(kind) {
            let position = self.nearest_open(x, y, radius);
            if self.circle_blocked(position.0, position.1, radius) { return None; }
            position
        } else { (x, y) };
        if is_hostile_kind(kind) { hp = ((hp as f32) * campaign::health_scale(self.wave)).round() as i32; }
        let zoff = skin_def(skin).map(|d| d.zoff).unwrap_or(zoff);
        for (i, e) in self.ents.iter_mut().enumerate() {
            if e.kind == 0 {
                *e = Ent {
                    aim: 0.0,
                    kind,
                    x,
                    y,
                    vx: 0.0,
                    vy: 0.0,
                    hp,
                    timer: if is_hostile_kind(kind) { 1.0 + (i % 5) as f32 * 0.12 } else { 0.4 },
                    frame: 0.0,
                    anim: ANIM_IDLE,
                    anim_time: 0.0,
                    anim_lock: 0.0,
                    skin,
                    projectile_visual: 0,
                    radius,
                    flash: 0.0,
                    stun: 0.0,
                    effect_tick: 0.0,
                    shield: 0,
                    face: 0.0,
                    zoff,
                    bar_t: 0.0,
                    armor_hp: enemies::armor_cap(kind, skin),
                    shield_hp: 0,
                };
                self.tactical.reset_entity(i);
                return Some(i);
            }
        }
        None
    }

    fn arm_shield(&mut self, i: usize) {
        if self.outage_at(self.ents[i].x,self.ents[i].y,1) {return;}
        let face = (self.py - self.ents[i].y).atan2(self.px - self.ents[i].x);
        let e = &mut self.ents[i];
        e.shield = 1;
        e.hp = ((e.hp as i64 * 14 / 10).max(e.hp as i64 + 8)).min(i32::MAX as i64) as i32;
        e.shield_hp = enemies::SHIELD_CAP;
        e.face = face;
    }

    fn say(&mut self, line: i32) {
        self.radio_seq = self.radio_seq.wrapping_add(1);
        self.radio_line = line;
        self.events |= EV_RADIO;
    }

    pub(crate) fn announce_sector(&mut self) {
        for slot in 19..WEP_N {
            if self.owns_slot(slot) {
                self.mag[slot] = MAG_SZ[slot];
                self.ammo[slot] = RESERVE_CAP[slot];
            }
        }
        self.node_done = false;
        self.lockdown = false;
        self.boss_vuln = 0.0;
        self.say(field::RADIO_HANDLER);
    }

    fn near_point(&self, x: f32, y: f32) -> bool {
        let d = (self.px - x).powi(2) + (self.py - y).powi(2);
        d < 1.7 * 1.7 && self.los(self.px, self.py, x, y)
    }

    /// Nearest prop of `kind` in the facing cone, within `reach`.
    fn facing_prop(&self, kind: u8, reach: f32) -> Option<usize> {
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let mut best: Option<(usize, f32)> = None;
        for (i, e) in self.ents.iter().enumerate() {
            if e.kind != kind || e.hp <= 0 || !self.los(self.px,self.py,e.x,e.y) {
                continue;
            }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            if t < 0.25 || t > reach {
                continue;
            }
            if (ex * dy - ey * dx).abs() > 0.75 {
                continue;
            }
            if best.is_none_or(|(_, d)| t < d) {
                best = Some((i, t));
            }
        }
        best.map(|(i, _)| i)
    }

    fn use_field(&mut self) {
        if !self.node_done {
            let (x, y) = field::node_point(self.wave);
            if self.near_point(x, y) {
                self.node_done = true;
                self.say(field::RADIO_NODE);
                self.events |= EV_PICK_GOLD;
                return;
            }
        }
        let sector = map::level_index(self.wave);
        for (index, &(x, y)) in field::terminals(self.wave).iter().enumerate() {
            if !self.near_point(x, y) {
                continue;
            }
            let mut unread = false;
            for e in self.ents.iter_mut() {
                if e.kind == EK_TERMINAL && (e.x - x).abs() < 0.45 && (e.y - y).abs() < 0.45 {
                    if e.timer < 0.0 {
                        return;
                    }
                    e.timer = -1.0;
                    unread = true;
                }
            }
            if unread {
                self.say(field::radio_terminal(sector, index));
            }
            return;
        }
        if let Some(i) = self.facing_prop(EK_LAMP, 2.6) {
            let on = self.ents[i].timer >= 0.0;
            self.ents[i].timer = if on { -1.0 } else { 0.0 };
            self.ents[i].flash = 0.15;
            self.light_dirty = true;
            self.events |= EV_HIT;
            self.sound(3, 0, self.ents[i].x, self.ents[i].y);
            return;
        }
        if let Some(i) = self.facing_prop(EK_CRATE, 1.8) {
            let (x, y) = (self.ents[i].x, self.ents[i].y);
            self.hurt_ent(i, 40, x, y);
        }
    }

    fn ensure_drop(&mut self, kind: u8, x: f32, y: f32) {
        if self.blocked(x.floor() as i32, y.floor() as i32) {
            return;
        }
        let taken = self.ents.iter().any(|e| {
            e.kind != 0 && (e.x - x).abs() < 0.32 && (e.y - y).abs() < 0.32
        });
        if !taken {
            let _ = self.spawn(kind, x, y);
        }
    }

    fn pay_secret(&mut self, cx: i32, cy: i32) {
        let Some((x, y)) = self.secret_room(cx, cy) else { return };
        if self.tactical.secrets_paid.iter().any(|&(a,b)| (x-a).hypot(y-b)<1.5) {return;}
        self.tactical.secrets_paid.push((x,y));
        self.ensure_drop(EK_MED, x, y);
        self.ensure_drop(EK_ARMOR, x + 0.35, y);
        self.ensure_drop(EK_AMMO, x - 0.35, y);
        self.say(210+(map::level_index(self.wave)%3) as i32);
    }

    fn boss_pos(&self) -> Option<(f32, f32)> {
        self.ents.iter().find(|e| e.kind == EK_BOSS && e.hp > 0).map(|e| (e.x, e.y))
    }

    fn seal_lockdown(&mut self) {
        self.lockdown = true;
        // Slam the leaves shut, but leave them as doors. A wall would trap
        // anyone who stepped out before the seal.
        for &(x, y) in field::lockdown_doors(self.wave) {
            if self.cell(x, y) != 8 || x < 0 || y < 0 {
                continue;
            }
            let idx = y as usize * MAP_W + x as usize;
            if idx < self.door.len() {
                if self.door[idx] > 0.05 {self.sound(1, 1, x as f32 + 0.5, y as f32 + 0.5);}
                self.door[idx] = 0.0;
            }
        }
        // Eject anyone whose center is caught inside a slamming leaf: a
        // shut door cell is solid, so that wedges the player with no legal
        // move. Mere overlap from outside still allows backing out.
        for &(x, y) in field::lockdown_doors(self.wave) {
            if self.cell(x, y) != 8 {
                continue;
            }
            if self.px.floor() as i32 == x && self.py.floor() as i32 == y {
                let (nx, ny) = self.nearest_open(self.px, self.py, self.pr);
                self.px = nx;
                self.py = ny;
                break;
            }
        }
        self.light_dirty = true;
        match map::level_index(self.wave) {
            1 => {
                if let Some((bx, by)) = self.boss_pos() {
                    self.ignite(bx + 1.2, by);
                    self.ignite(bx - 1.1, by + 0.8);
                    self.ignite(bx, by - 1.3);
                }
            }
            2 => {
                if let Some((bx, by)) = self.boss_pos() {
                    self.ignite(bx + 1.4, by);
                    self.ignite(bx - 1.2, by + 0.9);
                    self.ignite(bx, by - 1.5);
                }
            }
            _ => {}
        }
        self.say(field::RADIO_LOCKDOWN);
    }

    fn expose_boss(&mut self) {
        self.boss_vuln = 3.4;
        self.say(field::RADIO_EXPOSED);
        for e in self.ents.iter_mut() {
            if e.kind == EK_BOSS && e.hp > 0 {
                e.stun = e.stun.max(0.85);
                e.flash = 0.45;
            }
        }
    }

    /// Ownership flag for slot `i + 1` of the `owned` array.
    #[inline]
    fn has_w(&self, i: usize) -> bool {
        self.owned.get(i).copied().unwrap_or(false)
    }

    /// Same flag in the 0/1 form the `Hud` wire struct carries.
    #[inline]
    fn owned_flag(&self, i: usize) -> i32 {
        self.has_w(i) as i32
    }

    /// Grants ownership of slot `i + 1`.
    #[inline]
    fn set_w(&mut self, i: usize) {
        if let Some(slot) = self.owned.get_mut(i) {
            *slot = true;
        }
    }

    fn owns_slot(&self, slot: usize) -> bool {
        if slot >= 19 { return slot < WEP_N && self.extra_weapons & (1 << (slot - 19)) != 0; }
        if slot == 0 { return true; }
        self.has_w(slot - 1)
    }

    fn select_weapon(&mut self, slot: usize) {
        if slot < WEP_N && self.owns_slot(slot) && slot != self.weapon as usize {
            self.weapon = slot as i32;
            self.reload_t = 0.0;
            self.pickup_t = 0.28;self.pickup_dur=0.28;
            self.cooldown=self.cooldown.min(0.12);
        }
    }

    fn campaign_impact(&mut self, x: f32, y: f32, variant: usize) {
        self.sound(13, (variant % 4) as u8, x, y);
        if let Some(i) = self.spawn(EK_IMPACT, x, y) {
            self.ents[i].skin = 200 + (variant % 4) as u8;
            self.ents[i].timer = 0.34;
            self.ents[i].zoff = -8.0;
        }
    }

    fn projectile_impact(&mut self, x: f32, y: f32, visual: u8, palette: i32) {
        if visual == 110 || visual == 114 || visual == 128 || visual == 233 { self.effect(EK_IMPACT, 3, x, y, 0.28, -8.0); return; }
        let effect = if palette == 10 || palette == 12 { 3 }
            else if matches!(palette, 4 | 9 | 14) { 2 }
            else if visual == 230 { 0 }
            else if visual == 234 { 2 } else { 1 };
        self.campaign_impact(x, y, effect);
    }

    fn campaign_projectile(&mut self, x: f32, y: f32, a: f32, weapon: campaign::Weapon, friendly: bool, visual: usize) {
        if let Some(i) = self.spawn(EK_PROJ, x, y) {
            let e = &mut self.ents[i];
            e.vx = a.cos() * weapon.speed;
            e.vy = a.sin() * weapon.speed;
            e.hp = weapon.damage;
            e.timer = 3.0;
            e.effect_tick = if friendly { 6.0 } else { 0.0 };
            e.skin = if friendly { 230 + visual.min(4) as u8 } else { 123 + weapon.mode };
            e.face = weapon.mode as f32 + 1.0;
            e.zoff = 10.0;
            e.aim = if friendly && weapon.mode == 4 { 1.0 } else { 0.0 };
            if e.aim == 1.0 { e.timer=4.0; e.radius=0.12; e.zoff=8.0; }
        }
    }

    fn fire_campaign_weapon(&mut self) {
        let slot = self.weapon as usize;
        let spec = campaign::WEAPONS[slot - 19];
        self.cooldown = spec.cadence;
        self.muzzle = 1.0;
        self.kick = if matches!(spec.mode, 6 | 10) { 1.5 } else { 0.85 };
        let visual = match spec.mode { 0 | 6 | 10 => 0, 1 | 12 => 2, 5 => 3, 8 | 13 => 4, _ => 1 };
        let count = match spec.mode { 2 => 3, 4 => 5, 5 => 4, 7 => 5, 12 => 3, _ => 1 };
        if matches!(spec.mode, 3 | 7 | 11) {
            for n in 0..count {
                let a = self.pa + (n as f32 - (count - 1) as f32 * 0.5) * 0.035;
                self.hitscan(a, spec.damage, 25.0);
            }
            let t = self.wall_distance(self.px, self.py, self.pa.cos(), self.pa.sin(), 25.0);
            self.campaign_impact(self.px + self.pa.cos() * t * 0.96, self.py + self.pa.sin() * t * 0.96, if spec.mode == 3 { 2 } else if spec.mode == 11 { 3 } else { 0 });
            for n in 1..=12 {
                let d = t.min(12.0) * n as f32 / 13.0;
                self.spawn_timed(EK_RAY, self.px + self.pa.cos() * d, self.py + self.pa.sin() * d, 0.10, 14.0);
            }
        } else {
            for n in 0..count {
                let a = self.pa + (n as f32 - (count - 1) as f32 * 0.5) * if spec.mode==4 {0.12} else {0.045};
                let mut shot=spec;
                if spec.mode==4 {shot.damage=(spec.damage*3+n)/count;}
                self.campaign_projectile(self.px + a.cos() * 1.15, self.py + a.sin() * 1.15, a, shot, true, visual);
            }
        }
    }

    fn fire_campaign_boss(&mut self, i: usize) {
        let shooter = self.ents[i];
        let index = (shooter.skin - 23) as usize;
        let mut spec = campaign::WEAPONS[index];
        spec.damage = combat::profile(shooter.skin, EK_BOSS).damage;
        spec.speed = (spec.speed * 0.48).max(3.5);
        let visual = match spec.mode { 0 | 6 | 10 => 0, 1 | 12 => 2, 5 => 3, 8 | 13 => 4, _ => 1 };
        set_anim(&mut self.ents[i], ANIM_FIRE, 0.3);
        if spec.mode == 4 {
            for n in 0..3 {
                if map::living_hostiles(self) >= campaign::ACTIVE_HOSTILES { break; }
                let a = shooter.aim + (n as f32 - 1.0) * 0.7;
                let (x, y) = self.nearest_open(shooter.x + a.cos() * 1.6, shooter.y + a.sin() * 1.6, 0.28);
                let _ = self.spawn_with_skin(EK_MARTYR, SKIN_MARTYR, x, y);
                self.campaign_impact(x, y, 2);
            }
            return;
        }
        if spec.mode == 8 {
            let a = shooter.aim + 1.3;
            let (x, y) = self.nearest_open(shooter.x + a.cos() * 3.0, shooter.y + a.sin() * 3.0, shooter.radius);
            self.campaign_impact(shooter.x, shooter.y, 3);
            self.ents[i].x = x; self.ents[i].y = y;
        }
        if spec.mode == 12 && self.boss_phase > 0 {
            self.ents[i].shield = 1;
            self.ents[i].shield_hp = self.ents[i].shield_hp.max(35);
            self.ents[i].face = shooter.aim;
        }
        if spec.mode == 3 {
            let dx = shooter.x - self.px; let dy = shooter.y - self.py;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 4.0 && self.los(shooter.x, shooter.y, self.px, self.py) {
                self.try_move(self.px + dx / distance * 0.35, self.py + dy / distance * 0.35);
                self.shake = self.shake.max(0.22);
            }
        }
        if spec.mode == 13 && self.boss_phase > 0 {
            let guards = self.ents.iter().filter(|ent| ent.kind == EK_BRUTE && ent.skin == SKIN_GUNNER && ent.shield != 0 && ent.hp > 0).count();
            if guards < 4 && map::living_hostiles(self) < campaign::ACTIVE_HOSTILES {
                let (x,y) = self.nearest_open(shooter.x + 2.0, shooter.y + 1.5, 0.34);
                if let Some(guard) = self.spawn_with_skin(EK_BRUTE, SKIN_GUNNER, x, y) { self.arm_shield(guard); }
                self.campaign_impact(x,y,2);
            }
        }
        let count = [3, 1, 3, 5, 3, 4, 3, 7, 2, 8, 2, 3, 5, 6][index];
        for n in 0..count {
            let a = if spec.mode == 9 { n as f32 * core::f32::consts::TAU / count as f32 }
                else { shooter.aim + (n as f32 - (count - 1) as f32 * 0.5) * (0.04 + (index % 4) as f32 * 0.025) };
            let mut shot = spec;
            if spec.mode == 11 { shot.speed *= 0.7 + n as f32 * 0.25; }
            self.campaign_projectile(shooter.x, shooter.y, a, shot, false, visual);
        }
        self.campaign_impact(shooter.x, shooter.y, match spec.mode { 0 | 5 | 7 => 0, 1 | 2 | 12 => 1, 9 | 11 => 3, _ => 2 });
        if self.boss_phase >= 2 { self.boss_vuln = self.boss_vuln.max(1.2); }
    }

    fn grant_slot(&mut self, slot: usize) {
        match slot {
            1..=18 => self.owned[slot - 1] = true,
            19..=32 => self.extra_weapons |= 1 << (slot - 19),
            _ => return,
        }
        let grant = MAG_SZ[slot] * 2;
        let cap = RESERVE_CAP[slot];
        self.ammo[slot] = (self.ammo[slot] + grant).min(cap);
        if self.mag[slot] <= 0 {
            self.mag[slot] = MAG_SZ[slot];
        }
        self.weapon = slot as i32;
        self.reload_t = 0.0;
        self.events |= EV_PICK_GOLD;
        self.sound(11, 0, self.px, self.py);
    }

    /// QA-only full heal so long single-page smokes don't die mid-run.
    /// Keeps weapons, armor and position; revives and clears hostiles,
    /// in-flight projectiles and lingering fire so a converged crowd
    /// can't wedge the rest of the script.
    fn qa_heal(&mut self) {
        if !self.qa { return; }
        self.health = 100;
        self.pending_hostiles = 0;
        self.iframes = 1.5;
        self.state = 0;
        for ent in &mut self.ents {
            if is_hostile_kind(ent.kind)
                || matches!(ent.kind, EK_PROJ | EK_FIREPATCH | EK_FLAME)
            {
                ent.kind = EK_NONE;
                ent.hp = 0;
            }
        }
    }

    fn drop_boss_case(&mut self, x: f32, y: f32) {
        let kind = field::boss_case(self.wave);
        // A one-tick event lets the client play the death voice at the kill,
        // without replaying the persistent radio line on the next sector.
        self.events |= EV_BOSS_VOICE;
        if self.spawn(kind, x, y).is_some() {
            self.say(field::RADIO_BOSS_KILL);
            self.shake = (self.shake + 0.4).min(1.0);
        } else {
            self.grant_slot(field::boss_slot(kind));
            self.state = 2;
        }
    }

    fn spawn_timed(&mut self, kind: u8, x: f32, y: f32, life: f32, zoff: f32) {
        self.queue_fx(kind, x, y, 0.0, 0.0, life, zoff);
    }

    fn burst_fx(&mut self, x: f32, y: f32, kind: u8, n: i32, life: f32) {
        for _ in 0..n {
            let a = self.rnd() * core::f32::consts::TAU;
            let sp = 0.8 + self.rnd() * 2.2;
            let life_j = life * (0.6 + self.rnd() * 0.6);
            let z = -4.0 + self.rnd() * 18.0;
            self.queue_fx(kind, x, y, a.cos() * sp, a.sin() * sp, life_j, z);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn queue_fx(&mut self, kind: u8, x: f32, y: f32, vx: f32, vy: f32, timer: f32, zoff: f32) {
        if self.fx_n >= FX_CAP {
            return;
        }
        self.fx_q[self.fx_n] = FxCmd {
            projectile_visual: 0,
            variant: if kind == EK_SPARK { 4 } else { 0 },
            kind,
            x,
            y,
            vx,
            vy,
            timer,
            zoff,
        };
        self.fx_n += 1;
    }

    fn flush_fx(&mut self) {
        if self.fx_n == 0 {
            return;
        }
        let mut q = 0usize;
        for e in self.ents.iter_mut() {
            if q >= self.fx_n {
                break;
            }
            if e.kind != 0 {
                continue;
            }
            let f = self.fx_q[q];
            q += 1;
            let (hp, radius, zdef) = match f.kind {
                EK_GIB => (1, 0.08, 0.0),
                EK_IMPACT => (1, 0.1, -6.0),
                EK_SPARK => (1, 0.06, 0.0),
                EK_SMOKE => (1, 0.1, -4.0),
                EK_FLAME => (1, 0.14, 10.0),
                _ => (1, 0.1, 0.0),
            };
            *e = Ent {
                aim: if f.kind == EK_SPARK && f.variant == 5 { 1.6 } else { 0.0 },
                kind: f.kind,
                x: f.x,
                y: f.y,
                vx: f.vx,
                vy: f.vy,
                hp,
                timer: f.timer,
                frame: 0.0,
                anim: ANIM_IDLE,
                anim_time: 0.0,
                anim_lock: 0.0,
                skin: if f.kind == EK_SPARK && f.variant == 5 && self.weapon == 1 { 1 } else { SKIN_NONE },
                projectile_visual: f.projectile_visual,
                radius,
                flash: 0.0,
                stun: if f.kind == EK_SPARK && f.variant == 5 { 0.74 } else { 0.0 },
                effect_tick: f.variant as f32,
                shield: 0,
                face: 0.0,
                zoff: if f.zoff != 0.0 { f.zoff } else { zdef },
                bar_t: 0.0,
                armor_hp: 0,
                shield_hp: 0,
            };
        }
        self.fx_n = 0;
    }

    fn sound(&mut self, kind: u8, variant: u8, x: f32, y: f32) {
        if self.sound_cues.len() < sound::CAP {
            self.sound_cues.push(sound::SoundCue {kind: kind as f32, variant: variant as f32, x, y});
        }
    }

    fn effect(&mut self, kind: u8, variant: u8, x: f32, y: f32, life: f32, zoff: f32) {
        if kind == EK_IMPACT && variant != 2 { self.sound(13, variant, x, y); }
        let slot = self.fx_n;
        self.spawn_timed(kind, x, y, life, zoff);
        if self.fx_n > slot { self.fx_q[slot].variant = variant; }
    }

    fn walkable(&self, x: i32, y: i32) -> bool {
        !self.blocked(x, y)
    }

    fn rebuild_flow(&mut self) {
        for i in 0..MAP_CELLS {
            self.flow_dir[i] = 0;
            self.flow_dist[i] = 0xFFFF;
        }
        let sx = clamp_i(self.px.floor() as i32, 0, MAP_W as i32 - 1) as usize;
        let sy = clamp_i(self.py.floor() as i32, 0, MAP_H as i32 - 1) as usize;
        let start = sy * MAP_W + sx;
        self.flow_dist[start] = 0;
        self.flow_q[0] = start as u16;
        let mut head = 0usize;
        let mut tail = 1usize;
        let dirs: [(i32, i32, u8); 4] = [(1, 0, 2), (-1, 0, 1), (0, 1, 4), (0, -1, 3)];
        while head < tail {
            let i = self.flow_q[head] as usize;
            head += 1;
            let x = (i % MAP_W) as i32;
            let y = (i / MAP_W) as i32;
            let d0 = self.flow_dist[i];
            for (dx, dy, back) in dirs {
                let nx = x + dx;
                let ny = y + dy;
                if !self.walkable(nx, ny) {
                    continue;
                }
                let ni = ny as usize * MAP_W + nx as usize;
                if self.flow_dist[ni] != 0xFFFF {
                    continue;
                }
                self.flow_dist[ni] = d0 + 1;
                self.flow_dir[ni] = back;
                if tail < MAP_CELLS {
                    self.flow_q[tail] = ni as u16;
                    tail += 1;
                }
            }
        }
    }

    fn stamp_decal(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return;
        }
        let i = y as usize * MAP_W + x as usize;
        self.decal[i] = self.decal[i].saturating_add(1).min(3);
    }

    fn blend(a: u32, b: u32, t: f32) -> u32 {
        let t = t.clamp(0.0, 1.0);
        let ar = (a & 255) as f32;
        let ag = ((a >> 8) & 255) as f32;
        let ab = ((a >> 16) & 255) as f32;
        let br = (b & 255) as f32;
        let bg = ((b >> 8) & 255) as f32;
        let bb = ((b >> 16) & 255) as f32;
        let r = ar * (1.0 - t) + br * t;
        let g = ag * (1.0 - t) + bg * t;
        let bl = ab * (1.0 - t) + bb * t;
        let alpha = (a >> 24) & 255;
        (r as u32) | ((g as u32) << 8) | ((bl as u32) << 16) | (alpha << 24)
    }

    fn place_ents(&mut self) {
        map::place_level(self);
        self.spawn_hostiles(1);
    }

    /// Map guns the player already owns become a supply drop instead.
    pub(crate) fn replace_owned_weapon_drops(&mut self) {
        let owned = self.owned;
        let slot = |kind: u8| match kind {
            EK_GUN2 => Some(0),
            EK_GUN3 => Some(1),
            EK_GUN4 => Some(2),
            EK_GUN5 => Some(3),
            EK_GUN6 => Some(4),
            EK_GUN7 => Some(5),
            EK_GUN8 => Some(6),
            EK_GUN9 => Some(7),
            EK_GUN10 => Some(8),
            EK_GUN11 => Some(9),
            EK_GUN12 => Some(10),
            EK_GUN13 => Some(11),
            EK_GUN14 => Some(12),
            EK_GUN15 => Some(13),
            EK_GUN16 => Some(14),
            EK_GUN17 => Some(15),
            EK_GUN18 => Some(16),
            EK_GUN19 => Some(17),
            _ => None,
        };
        let mut n = 0i32;
        for e in self.ents.iter_mut() {
            let Some(i) = slot(e.kind) else { continue; };
            if !owned[i] { continue; }
            let kind = if n % 2 == 0 { EK_AMMO } else { EK_MED };
            n += 1;
            e.kind = kind;
            e.skin = SKIN_NONE;
            if let Some(d) = enemy_def(kind) {
                e.hp = d.hp;
                e.radius = d.radius;
            }
        }
    }

    /// Relocate authored threats outside the arrival room and doorway buffer.
    fn safe_arrival_point(&self,x:f32,y:f32,radius:f32)->(f32,f32) {
        let mut position = self.nearest_open(x, y, radius);
        if map::in_spawn_room(self.wave, position.0, position.1) {
            let mut best=None;
            for cy in 1..MAP_H-1 { for cx in 1..MAP_W-1 {
                let (sx,sy)=(cx as f32+0.5,cy as f32+0.5);
                if map::in_spawn_room(self.wave,sx,sy) || self.circle_blocked(sx,sy,radius) {continue;}
                let distance=(sx-x).powi(2)+(sy-y).powi(2);
                if best.is_none_or(|(d,_,_)|distance<d) {best=Some((distance,sx,sy));}
            }}
            if let Some((_,sx,sy))=best {position=(sx,sy);}
            // Fully synthetic test arenas may have no outside floor.
        }
        position
    }


    /// Missing starting weapons remain recoverable in later sectors.
    fn recover_basic_weapons(&mut self) {
        if self.wave<=1 {return;}
        let kinds=[EK_GUN2,EK_GUN3,EK_GUN4,EK_GUN5,EK_GUN6,EK_GUN7];
        let owned=self.owned;
        for e in &mut self.ents {if kinds.contains(&e.kind) {e.kind=EK_NONE;}}
        let mut cells=Vec::new();
        for y in 1..MAP_H-1 {for x in 1..MAP_W-1 {
            let (x,y)=(x as f32+0.5,y as f32+0.5);
            if !map::in_spawn_room(self.wave,x,y) && !self.circle_blocked(x,y,0.35)
                && self.ents.iter().all(|e| e.kind==EK_NONE || (e.x-x).powi(2)+(e.y-y).powi(2)>2.25) {
                cells.push((x,y));
            }
        }}
        for (i,&kind) in kinds.iter().enumerate() {
            if owned[i] || cells.is_empty() {continue;}
            let choice=(self.rnd()*cells.len() as f32) as usize % cells.len();
            let (x,y)=cells.swap_remove(choice);self.spawn(kind,x,y);
            cells.retain(|&(sx,sy)|(sx-x).powi(2)+(sy-y).powi(2)>16.0);
        }
    }

    fn spawn_hostiles(&mut self, mult: i32) {
        let roster = map::hostiles(self.wave).len();
        self.pending_hostiles = if self.wave > 25 { campaign::hostile_total(self.wave, roster) }
            else { roster * mult.max(1) as usize };
        self.reinforcement_cursor = 0;
        self.reinforcement_t = 1.25;
        while self.pending_hostiles > 0 && map::living_hostiles(self) < campaign::ACTIVE_HOSTILES {
            if !self.spawn_reinforcement(false) { break; }
        }
    }

    fn spawn_reinforcement(&mut self, telegraph: bool) -> bool {
        if self.pending_hostiles == 0 || map::living_hostiles(self) >= campaign::ACTIVE_HOSTILES { return false; }
        let roster = map::hostiles(self.wave);
        let mut point = roster[self.reinforcement_cursor % roster.len()];
        if telegraph {
            let mut safe = None;
            for n in 0..roster.len() {
                let candidate = roster[(self.reinforcement_cursor + n) % roster.len()];
                if (candidate.2 - self.px).powi(2) + (candidate.3 - self.py).powi(2) > 64.0 {
                    safe = Some(candidate); break;
                }
            }
            let Some(candidate) = safe else { return false; };
            point = candidate;
        }
        let (mut kind, mut packed, x, y) = point;
        if telegraph && self.outage_at(x,y,2) {
            self.pending_hostiles-=1;self.reinforcement_cursor=self.reinforcement_cursor.saturating_add(1);return true;
        }
        if (self.reinforcement_cursor % roster.len()).is_multiple_of(2) {
            (kind, packed) = enemies::sector_spawn(self.wave, packed);
        }
        let copy = self.reinforcement_cursor / roster.len();
        let jx = if copy == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
        let jy = if copy == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
        let radius = enemy_def(kind).map(|d| d.radius).unwrap_or(0.28);
        let (sx, sy) = self.safe_arrival_point(x + jx, y + jy, radius);
        if telegraph && (sx - self.px).powi(2) + (sy - self.py).powi(2) < 49.0 { return false; }
        let Some(i) = self.spawn_with_skin(kind, field::visual_skin(packed), sx, sy) else { return false; };
        if field::is_shielded_spawn(packed) { self.arm_shield(i); }
        if telegraph {
            self.ents[i].stun = 0.9;
            self.ents[i].timer = 1.2;
            self.campaign_impact(sx, sy, 2);
        }
        self.pending_hostiles -= 1;
        self.reinforcement_cursor = self.reinforcement_cursor.saturating_add(1);
        true
    }

    fn next_wave(&mut self) {
        // Twenty-five authored sectors repeat in endless mode. Health and total
        // enemies grow continuously; queued arrivals preserve effect slots.
        self.wave = self.wave.saturating_add(1);
        // Capped at 4x (56 hostiles + ambushes): the uncapped 1<<8 shift
        // filled all 192 entity slots with hostiles and starved FX.
        let shift = (self.wave - 1).clamp(0, 2);
        let mult = 1i32 << shift;
        // Winning keeps every unlocked weapon, restores full health and full
        // ammo, and preserves armor clamped to [0, 100].
        self.health = 100;
        self.armor = self.armor.clamp(0, 100);
        self.iframes = 1.4;
        let start = map::player_start(self.wave);
        self.px = start.0;
        self.py = start.1;
        self.pa = start.2;
        self.pitch = 0.0;
        self.state = 0;
        self.cooldown = 0.0;
        self.reload_t = 0.0;
        self.hurt = 0.0;
        self.muzzle = 0.0;
        self.kick = 0.0;
        self.shake = 0.0;
        // Slot 0 is always carried; owned slots refill to their authored cap.
        for slot in 0..WEP_N {
            if slot != 0 && !self.has_w(slot - 1) { continue; }
            self.mag[slot] = MAG_SZ[slot];
            self.ammo[slot] = RESERVE_CAP[slot];
        }
        self.node_done = false;
        self.lockdown = false;
        self.boss_vuln = 0.0;
        self.apply_theme(self.wave);
        self.clear_boss_arena();
        self.tactical=tactical::Tactical::new();
        self.build_map();
        self.door.fill(0.0);
        self.hell = false;
        self.light_dirty = true;
        self.boss_spawned = false;
        self.boss_intro = 0.0;
        self.boss_phase = 0;
        map::place_level(self);
        self.spawn_hostiles(mult);
    }

    fn maybe_spawn_boss(&mut self) {
        if self.boss_spawned || self.state != 0 {
            return;
        }
        self.boss_spawned = true;
        self.hell = true;
        self.shake = 1.0;
        self.events |= EV_EXPLODE | EV_BOSS_DROP;
        // Endless scaling: 480 x1.5 per wave, capped at 6000 so deep
        // runs stay killable (480 / 720 / 1080 / 1620 / ... / 6000).
        let hp = self.boss_max_health();
        let fx = self.pa.cos();
        let fy = self.pa.sin();
        let boss_spots = map::boss_spots(self.wave);
        let spots = [
            boss_spots[0],
            boss_spots[1],
            (self.px + fx * 4.6, self.py + fy * 4.6),
            (self.px + fx * 3.2 - fy * 2.4, self.py + fy * 3.2 + fx * 2.4),
        ];
        for (x, y) in spots {
            if self.blocked(x.floor() as i32, y.floor() as i32) {
                continue;
            }
            if let Some(i) = self.spawn_with_skin(EK_BOSS, map::boss_skin(self.wave), x, y) {
                self.ents[i].hp = hp;
                self.sound(2, 0, x, y);
                if matches!(map::level_index(self.wave), 5 | 6) {
                    // The Archivist opens behind a breakable directional
                    // memory shield; flanking or sustained fire strips it.
                    self.ents[i].shield = 1;
                    self.ents[i].shield_hp = if map::level_index(self.wave) == 6 { 100 } else { 80 };
                    self.ents[i].face = (self.py - y).atan2(self.px - x);
                }
                break;
            }
        }
        let sector = map::level_index(self.wave);
        let entry = map::boss_spots(self.wave)[0];
        let burst = [14, 22, 10, 18, 20, 24, 18, 16, 28, 12, 24].get(sector).copied().unwrap_or(18);
        if sector == 1 {
            self.spawn_smoke_cloud(entry.0, entry.1);
            self.spawn_smoke_cloud(entry.0 + 1.8, entry.1 - 1.2);
        }
        for _ in 0..burst {
            let a = self.rnd() * core::f32::consts::TAU;
            let r = 0.5 + self.rnd() * 1.6;
            self.spawn_timed(
                EK_SPARK,
                entry.0 + a.cos() * r,
                entry.1 + a.sin() * r,
                0.7,
                if sector == 1 { -32.0 } else { -16.0 },
            );
        }
        let escorts = match map::level_index(self.wave) {
            1 => [(EK_WRAITH, SKIN_HORNET, -2.4, -1.6), (EK_MARTYR, SKIN_MARTYR, 2.4, -1.6), (EK_HUSK, SKIN_GUNNER, -2.8, 1.8), (EK_BRUTE, SKIN_HAZMAT, 2.8, 1.8)],
            2 => [(EK_HUSK, SKIN_HOUND, -2.4, -1.6), (EK_WRAITH, SKIN_SPITTER, 2.4, -1.6), (EK_HUSK, SKIN_SUBJECT, -2.8, 1.8), (EK_BRUTE, SKIN_VATBRUTE, 2.8, 1.8)],
            3 => [(EK_WRAITH, SKIN_MARKSMAN, -2.4, -1.6), (EK_WRAITH, SKIN_HORNET, 2.4, -1.6), (EK_HUSK, SKIN_RIFLEMAN, -2.8, 1.8), (EK_BRUTE, SKIN_GUNNER, 2.8, 1.8)],
            4 => [(EK_BRUTE, SKIN_HAZMAT, -2.4, -1.6), (EK_WRAITH, SKIN_HORNET, 2.4, -1.6), (EK_HUSK, SKIN_GUNNER, -2.8, 1.8), (EK_BRUTE, SKIN_LOADER, 2.8, 1.8)],
            5 => [(EK_WRAITH, SKIN_MARKSMAN, -2.4, -1.6), (EK_HUSK, SKIN_RIFLEMAN, 2.4, -1.6), (EK_WRAITH, SKIN_HORNET, -2.8, 1.8), (EK_BRUTE, SKIN_GUNNER, 2.8, 1.8)],
            _ => [(EK_WRAITH, SKIN_HORNET, -2.4, -1.6), (EK_WRAITH, SKIN_MARKSMAN, 2.4, -1.6), (EK_HUSK, SKIN_RIFLEMAN, -2.8, 1.8), (EK_BRUTE, SKIN_LOADER, 2.8, 1.8)],
        };
        for (n,&(kind, skin, ox, oy)) in escorts[..(2 + self.wave.min(2) as usize)].iter().enumerate() {
            let (kind,skin)=if n%2==0 {enemies::sector_spawn(self.wave,skin)} else {(kind,skin)};
            let skin=field::visual_skin(skin);
            let x = entry.0 + ox;
            let y = entry.1 + oy;
            if !self.blocked(x.floor() as i32, y.floor() as i32) {
                let _ = self.spawn_with_skin(kind, skin, x, y);
            }
        }
    }

    fn boss_max_health(&self) -> i32 {
        campaign::boss_health(self.wave)
    }

    fn sample(&self, id: usize, u: i32, v: i32) -> u32 {
        let u = (u as usize) & (TEX - 1);
        let v = (v as usize) & (TEX - 1);
        self.tex[id * TEX * TEX + v * TEX + u]
    }

    fn rebuild_mipmaps(&mut self) {
        self.mipmaps.clear();
        for level in 1..=8 {
            let size = TEX >> level;
            let source = if level == 1 { &self.tex } else { &self.mipmaps[level - 2] };
            let mut pixels = vec![0u32; TEX_N * size * size];
            for id in 0..TEX_N {
                Self::mipmap_layer_into(source, &mut pixels, id, level);
            }
            self.mipmaps.push(pixels);
        }
    }

    fn mipmap_layer_into(source: &[u32], pixels: &mut [u32], id: usize, level: usize) {
        let size = TEX >> level;
        let prev_size = size * 2;
        for y in 0..size {
            for x in 0..size {
                let p = id * prev_size * prev_size + y * 2 * prev_size + x * 2;
                let samples = [source[p], source[p + 1], source[p + prev_size], source[p + prev_size + 1]];
                let mut color = 0;
                for shift in [0, 8, 16, 24] {
                    let channel: u32 = samples.iter().map(|c| (c >> shift) & 255).sum();
                    color |= (channel / 4) << shift;
                }
                pixels[id * size * size + y * size + x] = color;
            }
        }
    }

    /// Rebuild one layer's mipmap chain (used after a theme swap so a
    /// wave change doesn't pay for all 140 layers).
    fn rebuild_mipmap_layer(&mut self, id: usize) {
        for level in 1..=8 {
            if level > self.mipmaps.len() {
                return;
            }
            // Borrow the previous level's pixels without aliasing self.
            let prev: Vec<u32> = if level == 1 {
                self.tex.clone()
            } else {
                self.mipmaps[level - 2].clone()
            };
            Self::mipmap_layer_into(&prev, &mut self.mipmaps[level - 1], id, level);
        }
    }

    /// Theme index shared with THEME order in src/game/runtime.ts.
    fn theme_index(wave: i32) -> usize {
        [0, 4, 5, 3, 6, 7, 4, 3, 6, 0, 7].get(map::level_index(wave)).copied().unwrap_or(3 + map::level_index(wave) % 4)
    }

    /// Copy the wave's wall/door variants into the live T_TECH/T_DOOR
    /// atlas slots and refresh their mipmap chains.
    fn apply_theme(&mut self, wave: i32) {
        let theme = Self::theme_index(wave);
        let wall = &self.theme_tex[theme * TEX * TEX..(theme + 1) * TEX * TEX].to_vec();
        let door = &self.theme_tex[(8 + theme) * TEX * TEX..(9 + theme) * TEX * TEX].to_vec();
        self.tex[T_TECH * TEX * TEX..(T_TECH + 1) * TEX * TEX].copy_from_slice(wall);
        self.tex[T_DOOR * TEX * TEX..(T_DOOR + 1) * TEX * TEX].copy_from_slice(door);
        self.rebuild_mipmap_layer(T_TECH);
        self.rebuild_mipmap_layer(T_DOOR);
        self.light_dirty = true;
    }

    // Test-only probe into the mipmap chain (see shade_and_mip_blend).
    #[cfg(test)]
    fn sample_lod(&self, id: usize, u: i32, v: i32, footprint: f32) -> u32 {
        let lod = footprint.max(1.0).log2().clamp(0.0, 8.0);
        let level = lod as usize;
        self.sample_mip(id, u, v, level, (lod.fract() * 256.0) as u32)
    }

    fn mip_slot(id: usize, u: i32, v: i32, level: usize) -> (u8, usize) {
        if level == 0 {
            let u = (u as usize) & (TEX - 1);
            let v = (v as usize) & (TEX - 1);
            return (0, id * TEX * TEX + v * TEX + u);
        }
        let size = TEX >> level;
        let x = ((u & TEXM) as usize) >> level;
        let y = ((v & TEXM) as usize) >> level;
        (level as u8, id * size * size + y * size + x)
    }

    fn sample_mip_at(&self, id: usize, u: i32, v: i32, level: usize) -> u32 {
        let (which, idx) = Self::mip_slot(id, u, v, level);
        if which == 0 { self.tex[idx] } else { self.mipmaps[which as usize - 1][idx] }
    }

    fn blend_mips(a: u32, b: u32, mix: u32) -> u32 {
        let inv = 256 - mix;
        let rb = (((a & 0x00ff00ff) * inv + (b & 0x00ff00ff) * mix) >> 8) & 0x00ff00ff;
        let g = (((a & 0x0000ff00) * inv + (b & 0x0000ff00) * mix) >> 8) & 0x0000ff00;
        rb | g | (a & 0xff000000)
    }

    /// Four-wide mip blend. Wasm builds this with simd128; native tests use the
    /// same integer formula lane by lane so a refactor cannot drift the picture.
    #[inline(always)]
    fn blend_mips4(a: [u32; 4], b: [u32; 4], mix: u32) -> [u32; 4] {
        #[cfg(target_feature = "simd128")]
        {
            use core::arch::wasm32::*;
            let av = u32x4(a[0], a[1], a[2], a[3]);
            let bv = u32x4(b[0], b[1], b[2], b[3]);
            let mix_v = u32x4_splat(mix);
            let inv = u32x4_splat(256 - mix);
            let rb_mask = u32x4_splat(0x00ff_00ff);
            let g_mask = u32x4_splat(0x0000_ff00);
            let rb = v128_and(
                u32x4_shr(
                    u32x4_add(
                        u32x4_mul(v128_and(av, rb_mask), inv),
                        u32x4_mul(v128_and(bv, rb_mask), mix_v),
                    ),
                    8,
                ),
                rb_mask,
            );
            let g = v128_and(
                u32x4_shr(
                    u32x4_add(
                        u32x4_mul(v128_and(av, g_mask), inv),
                        u32x4_mul(v128_and(bv, g_mask), mix_v),
                    ),
                    8,
                ),
                g_mask,
            );
            let out = v128_or(v128_or(rb, g), v128_and(av, u32x4_splat(0xff00_0000)));
            return [
                u32x4_extract_lane::<0>(out),
                u32x4_extract_lane::<1>(out),
                u32x4_extract_lane::<2>(out),
                u32x4_extract_lane::<3>(out),
            ];
        }
        #[cfg(not(target_feature = "simd128"))]
        [
            Self::blend_mips(a[0], b[0], mix),
            Self::blend_mips(a[1], b[1], mix),
            Self::blend_mips(a[2], b[2], mix),
            Self::blend_mips(a[3], b[3], mix),
        ]
    }

    fn sample_mip(&self, id: usize, u: i32, v: i32, level: usize, mix: u32) -> u32 {
        let a = self.sample_mip_at(id, u, v, level);
        let b = self.sample_mip_at(id, u, v, (level + 1).min(8));
        Self::blend_mips(a, b, mix)
    }

    #[inline(always)]
    fn sample_mip4(&self, id: usize, uv: [(i32, i32); 4], level: usize, mix: u32) -> [u32; 4] {
        let next = (level + 1).min(8);
        // Close rows rarely share a texel, and the slot compare loses to four gathers.
        if level >= 3 {
            let slot = Self::mip_slot(id, uv[0].0, uv[0].1, level);
            let slot_b = Self::mip_slot(id, uv[0].0, uv[0].1, next);
            let shared = (1..4).all(|i| {
                Self::mip_slot(id, uv[i].0, uv[i].1, level) == slot
                    && Self::mip_slot(id, uv[i].0, uv[i].1, next) == slot_b
            });
            if shared {
                let a = self.sample_mip_at(id, uv[0].0, uv[0].1, level);
                let b = self.sample_mip_at(id, uv[0].0, uv[0].1, next);
                return Self::blend_mips4([a, a, a, a], [b, b, b, b], mix);
            }
        }
        let mut a = [0u32; 4];
        let mut b = [0u32; 4];
        for i in 0..4 {
            a[i] = self.sample_mip_at(id, uv[i].0, uv[i].1, level);
            b[i] = self.sample_mip_at(id, uv[i].0, uv[i].1, next);
        }
        Self::blend_mips4(a, b, mix)
    }

    fn shade(c: u32, f: f32) -> u32 {
        let f = f.clamp(0.0, 1.4);
        let r = ((c & 255) as f32 * f) as u32;
        let g = (((c >> 8) & 255) as f32 * f) as u32;
        let b = (((c >> 16) & 255) as f32 * f) as u32;
        let a = (c >> 24) & 255;
        (r.min(255)) | (g.min(255) << 8) | (b.min(255) << 16) | (a << 24)
    }

    fn fog(c: u32, dist: f32) -> u32 {
        let a = ((dist * (1.0 / 28.0)).clamp(0.0, 1.0) * 255.0) as u32;
        (c & 0x00FFFFFF) | (a << 24)
    }

    fn try_move(&mut self, nx: f32, ny: f32) {
        let r = self.pr;
        if !self.circle_blocked(nx, ny, r) {
            self.px = nx;
            self.py = ny;
            return;
        }
        // Resolve both intended components independently so walls preserve tangential motion.
        if !self.circle_blocked(nx, self.py, r) {
            self.px = nx;
        }
        if !self.circle_blocked(self.px, ny, r) {
            self.py = ny;
        }
    }

    fn circle_blocked(&self, x: f32, y: f32, r: f32) -> bool {
        let x0 = (x - r).floor() as i32;
        let y0 = (y - r).floor() as i32;
        let x1 = (x + r).floor() as i32;
        let y1 = (y + r).floor() as i32;
        for j in y0..=y1 {
            for i in x0..=x1 {
                if !self.blocked(i, j) {
                    continue;
                }
                let cx = i as f32 + 0.5;
                let cy = j as f32 + 0.5;
                let dx = (x - cx).abs() - 0.5;
                let dy = (y - cy).abs() - 0.5;
                let ox = dx.max(0.0);
                let oy = dy.max(0.0);
                if ox * ox + oy * oy < r * r {
                    return true;
                }
            }
        }
        false
    }

    /// Nearest point whose radius does not intersect a wall. Spawn and
    /// unstick both use this so a listed coordinate inside geometry cannot
    /// leave a hostile embedded.
    fn nearest_open(&self, x: f32, y: f32, radius: f32) -> (f32, f32) {
        if !self.circle_blocked(x, y, radius) {
            return (x, y);
        }
        for ring in 1..10 {
            let step = ring as f32 * 0.45;
            for n in 0..8 {
                let a = n as f32 * core::f32::consts::TAU / 8.0;
                let nx = x + a.cos() * step;
                let ny = y + a.sin() * step;
                if nx < 1.2 || ny < 1.2 || nx > MAP_W as f32 - 1.2 || ny > MAP_H as f32 - 1.2 {
                    continue;
                }
                if !self.circle_blocked(nx, ny, radius) {
                    return (nx, ny);
                }
            }
        }
        // Rare fallback for a spawn deep inside solid geometry: search the
        // authored layout rather than returning an embedded hostile.
        let mut best = None;
        let mut distance = f32::INFINITY;
        for cy in 1..MAP_H-1 { for cx in 1..MAP_W-1 {
            let (nx, ny) = (cx as f32 + 0.5, cy as f32 + 0.5);
            let d = (nx-x).powi(2) + (ny-y).powi(2);
            if d < distance && !self.circle_blocked(nx, ny, radius) {
                best = Some((nx, ny)); distance = d;
            }
        }}
        best.unwrap_or((x, y))
    }

    pub(crate) fn spawn_clear(&mut self, kind: u8, x: f32, y: f32) -> Option<usize> {
        let radius = enemy_def(kind).map(|d| d.radius).unwrap_or(0.28);
        let (nx, ny) = self.nearest_open(x, y, radius);
        self.spawn(kind, nx, ny)
    }

    fn los(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        let steps = (dist * 8.0) as i32 + 1;
        let sx = dx / steps as f32;
        let sy = dy / steps as f32;
        let mut x = x0;
        let mut y = y0;
        for _ in 0..steps {
            x += sx;
            y += sy;
            if self.blocked(x.floor() as i32, y.floor() as i32) {
                return false;
            }
        }
        true
    }

    fn wall_tex(&self, c: u8, _x: i32, _y: i32) -> usize {
        let base = T_SECTOR_SURFACE + map::level_index(self.wave) * 5;
        if c==9 {return base+1;}
        if c==8 {return base+2;}
        if self.hell { return T_BOSS_ARENA + map::level_index(self.wave); }
        base + if matches!(c, 4 | 6) { 1 } else { 0 }
    }

    /// True when this leaf is a sealed boss door and the player is on the
    /// arena side. The outside face still opens so they can walk back in.
    fn lockdown_refuses(&self, cx: i32, cy: i32) -> bool {
        if !self.lockdown || !field::lockdown_doors(self.wave).contains(&(cx, cy)) {
            return false;
        }
        !self.outside_lock(cx, cy)
    }

    fn outside_lock(&self, dx: i32, dy: i32) -> bool {
        let (bx, by) = self.boss_pos().unwrap_or_else(|| {
            let spots = map::boss_spots(self.wave);
            spots[0]
        });
        let ix = bx - (dx as f32 + 0.5);
        let iy = by - (dy as f32 + 0.5);
        let px = self.px - (dx as f32 + 0.5);
        let py = self.py - (dy as f32 + 0.5);
        // Opposite side of the boss, with slack so the threshold still opens.
        px * ix + py * iy < 0.4
    }

    fn open_door_at(&mut self, cx: i32, cy: i32, force: bool) -> bool {
        if self.lockdown_refuses(cx, cy) {
            return false;
        }
        let c = self.cell(cx, cy);
        if c != 8 && !(c == 9 && force) {
            return false;
        }
        if cx < 0 || cy < 0 {
            return false;
        }
        let idx = cy as usize * MAP_W + cx as usize;
        if idx >= self.door.len() || self.door[idx] >= 0.05 {
            return false;
        }
        self.door[idx] = 0.06;
        self.sound(1, 0, cx as f32 + 0.5, cy as f32 + 0.5);
        self.events |= EV_DOOR;
        if c == 9 {
            self.secrets += 1;
            self.pay_secret(cx, cy);
        }
        true
    }

    fn open_door_pair(&mut self, cx: i32, cy: i32, force: bool) {
        if !self.open_door_at(cx, cy, force) {
            return;
        }
        const N: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in N {
            self.open_door_at(cx + dx, cy + dy, force);
        }
    }

    fn open_nearby_doors(&mut self, force: bool) {
        let fx = self.px + self.pa.cos() * 0.9;
        let fy = self.py + self.pa.sin() * 0.9;
        let cells = [
            (self.px.floor() as i32, self.py.floor() as i32),
            (fx.floor() as i32, fy.floor() as i32),
            (fx.floor() as i32 + 1, fy.floor() as i32),
            (fx.floor() as i32, fy.floor() as i32 + 1),
        ];
        for (cx, cy) in cells {
            self.open_door_pair(cx, cy, force);
        }
        let x0 = (self.px - 1.3).floor() as i32;
        let y0 = (self.py - 1.3).floor() as i32;
        let x1 = (self.px + 1.3).floor() as i32;
        let y1 = (self.py + 1.3).floor() as i32;
        for j in y0..=y1 {
            for i in x0..=x1 {
                if self.cell(i, j) == 8 || (force && self.cell(i, j) == 9) {
                    self.open_door_pair(i, j, force);
                }
            }
        }
    }

    /// Damage with no known source: used by tests and scripted hazards. The
    /// vignette stays centred because there is nothing to point at.
    fn damage_player(&mut self, dmg: i32) {
        self.damage_player_from(dmg, self.px, self.py);
    }

    /// Damage the player and record where it came from, so the hurt vignette
    /// can point at the shooter instead of ringing the whole screen.
    fn damage_player_from(&mut self, dmg: i32, sx: f32, sy: f32) {
        if self.iframes > 0.0 || self.state != 0 {
            return;
        }
        if self.power == field::POWER_AEGIS && self.power_t > 10.0 {
            self.events |= EV_HIT;
            return;
        }
        let mut d = dmg;
        if self.shield_pool > 0 {
            let take = d.min(self.shield_pool);
            self.shield_pool -= take;
            d -= take;
        }
        if d <= 0 {
            self.events |= EV_HIT;
            return;
        }
        if self.armor > 0 {
            let soak = (d * 2) / 3;
            let take = soak.min(self.armor);
            self.armor -= take;
            d -= take;
        }
        // Bearing of the source relative to facing: 0 dead ahead, +pi behind.
        // `pa` follows `atan2(dy, dx)` (see `pa.cos()/pa.sin()`), and Rust's
        // `y.atan2(x)` is the angle of the point `(x, y)`, so the y delta is the
        // receiver. Stored wrapped to [-pi, pi] so the shader's sin/cos can
        // consume it directly.
        let bearing = (sy - self.py).atan2(sx - self.px);
        self.hurt_dir = (bearing - self.pa + core::f32::consts::PI)
            .rem_euclid(core::f32::consts::TAU)
            - core::f32::consts::PI;
        self.hurt = 1.0;
        self.health -= d.max(1);
        self.iframes = 0.35;
        self.shake = (self.shake + 0.55).min(1.0);
        self.events |= EV_HURT;
        if self.health <= 0 {
            self.health = 0;
            self.state = 1;
            self.events |= EV_DIE;
        }
    }

    fn explode(&mut self, x: f32, y: f32, radius: f32, dmg: f32) {
        self.sound(2, 0, x, y);
        self.effect(EK_IMPACT, 2, x, y, 0.38, 10.0);
        self.spawn_smoke_cloud(x, y);
        self.shake = (self.shake + 0.8).min(1.0);
        self.events |= EV_EXPLODE;
        let pd = ((self.px - x).powi(2) + (self.py - y).powi(2)).sqrt();
        if pd < radius && self.los(x, y, self.px, self.py) {
            let fall = 1.0 - pd / radius;
            self.damage_player_from((dmg * fall) as i32, x, y);
        }
        let mut hits: Vec<(usize, i32)> = Vec::new();
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !target_kind(e.kind) {
                continue;
            }
            let d = ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt();
            if d < radius && self.los(x, y, e.x, e.y) {
                let fall = 1.0 - d / radius;
                hits.push((i, (dmg * fall) as i32));
            }
        }
        for (i, d) in hits {
            self.hurt_ent(i, d.max(1), x, y);
        }
        for _ in 0..7 {
            let a = self.rnd() * core::f32::consts::TAU;
            let sp = 1.5 + self.rnd() * 2.5;
            let life = 0.4 + self.rnd() * 0.4;
            self.queue_fx(EK_GIB, x, y, a.cos() * sp, a.sin() * sp, life, 0.0);
        }
    }

    fn hurt_ent(&mut self, i: usize, dmg: i32, hx: f32, hy: f32) {
        self.hurt_ent_role(i,dmg,hx,hy,None);
    }

    fn hurt_ent_role(&mut self, i: usize, mut dmg: i32, hx: f32, hy: f32, role: Option<tactical::Role>) {
        if i >= ENT_N || dmg <= 0 {
            return;
        }
        if self.ents[i].kind == 0 || self.ents[i].hp <= 0 {
            return;
        }
        if is_hostile_kind(self.ents[i].kind) {
            use tactical::Role;
            if let Some(r)=role {
                if r==Role::Precision && self.tactical.acid[i]>0.0 {
                    dmg=(dmg as f32*1.25).round() as i32;
                    self.tactical.acid[i]=0.0;
                    self.effect(EK_IMPACT,3,self.ents[i].x,self.ents[i].y,0.25,4.0);
                }
                match r {
                    Role::Acid=>{if self.tactical.acid[i]<=0.0 {self.effect(EK_IMPACT,3,self.ents[i].x,self.ents[i].y,0.22,4.0);}self.tactical.acid[i]=3.0;},
                    Role::Freeze=>{if self.tactical.slow[i]<=0.0 {self.effect(EK_IMPACT,2,self.ents[i].x,self.ents[i].y,0.22,4.0);}self.tactical.slow[i]=3.0;},
                    Role::Shock if self.tactical.shock_lock[i]<=0.0=>{
                        let boss=self.ents[i].kind==EK_BOSS;
                        self.ents[i].stun=self.ents[i].stun.max(if boss {0.18}else{0.65});
                        self.effect(EK_IMPACT,1,self.ents[i].x,self.ents[i].y,0.22,4.0);
                        self.ents[i].shield_hp=(self.ents[i].shield_hp-20).max(0);
                        self.tactical.shock_lock[i]=if boss {3.0}else{1.2};
                    },
                    Role::Stagger=>self.ents[i].stun=self.ents[i].stun.max(if self.ents[i].kind==EK_BOSS {0.10}else{0.34}),
                    Role::Suppress=>self.ents[i].timer=self.ents[i].timer.max(if self.ents[i].kind==EK_BOSS {0.45}else{0.85}),
                    Role::Control=>self.tactical.slow[i]=self.tactical.slow[i].max(0.6),
                    _=>{}
                }
            }
        }
        let target = self.ents[i];
        let metal = target.armor_hp > 0 || target.shield_hp > 0 ||
            !is_hostile_kind(target.kind) || matches!(enemies::combat_skin(target.skin), 5 | 8 | 11 | 13 | 15..=20 | 22..=27 | 29..=36);
        self.sound(if metal {3} else {4}, 0, target.x, target.y);
        // A shot or a USE shuts the lamp. It stays in the world, dark.
        if self.ents[i].kind == EK_LAMP {
            self.ents[i].timer = -1.0;
            self.ents[i].flash = 0.2;
            self.light_dirty = true;
            self.events |= EV_HIT;
            return;
        }
        if self.ents[i].shield_hp > 0
            && field::shield_blocks(self.ents[i].face, self.ents[i].x, self.ents[i].y, hx, hy)
        {
            let (x, y) = (self.ents[i].x, self.ents[i].y);
            let soak = dmg.min(self.ents[i].shield_hp);
            self.ents[i].shield_hp -= soak;
            dmg -= soak;
            self.ents[i].bar_t = 2.0;
            self.ents[i].flash = 0.2;
            self.events |= EV_HIT;
            self.burst_fx(x, y, EK_SPARK, 4, 0.14);
            if dmg <= 0 {
                return;
            }
        }
        if self.ents[i].armor_hp > 0 {
            let eligible = if role==Some(tactical::Role::Precision) {dmg*3/5}else{dmg};
            let soak = eligible.min(self.ents[i].armor_hp);
            self.ents[i].armor_hp -= soak;
            dmg -= soak;
            self.ents[i].bar_t = 2.0;
            self.ents[i].flash = 0.16;
            if dmg <= 0 {
                // Absorbed by armor: a weaker, shorter confirm than a flesh hit.
                self.hitmarker = 0.7;
                self.events |= EV_HIT;
                return;
            }
        }
        if self.ents[i].kind == EK_BOSS && self.boss_vuln > 0.0 {
            dmg = (dmg * 2).max(2);
        }
        let kind;
        let skin;
        let x;
        let y;
        {
            let e = &mut self.ents[i];
            e.hp -= dmg;
            e.bar_t = 2.0;
            e.flash = 0.12;
            let dx = e.x - hx;
            let dy = e.y - hy;
            let l = (dx * dx + dy * dy).sqrt().max(0.01);
            e.vx += dx / l * 1.6;
            e.vy += dy / l * 1.6;
            kind = e.kind;
            skin = e.skin;
            x = e.x;
            y = e.y;
            if e.hp > 0 {
                if is_hostile_kind(kind) {
                    e.effect_tick = 0.0;
                    e.timer = e.timer.max(0.45);
                }
                set_anim(e, ANIM_PAIN, 0.24);
                self.hitmarker = 1.0;
                self.events |= EV_HIT;
                return;
            }
            e.hp = 0;
            e.vx = 0.0;
            e.vy = 0.0;
            set_anim(e, ANIM_DEAD, 0.62);
        }
        self.hitmarker = 1.7;
        self.events |= EV_KILL;
        if solid_kind(kind) && kind != EK_BARREL {
            self.kills += 1;
            for _ in 0..5 {
                let a = self.rnd() * core::f32::consts::TAU;
                let sp = 1.2 + self.rnd() * 2.0;
                let life = 0.45 + self.rnd() * 0.25;
                self.queue_fx(EK_GIB, x, y, a.cos() * sp, a.sin() * sp, life, 0.0);
            }
        }
        if kind == EK_BARREL {
            self.light_dirty = true;
            if skin == 1 {
                // Fuel drum: the blast is the fire it leaves, not a grenade.
                self.ignite(x, y);
                self.ignite(x + 0.45, y - 0.2);
                self.explode(x, y, 1.15, 12.0);
            } else {
                self.explode(x, y, 2.6, 55.0);
            }
        } else if kind == EK_MARTYR {
            self.light_dirty = true;
            self.explode(x, y, 2.6, 55.0);
        }
        if !is_hostile_kind(kind) && kind != EK_BARREL {
            self.sound(if kind == EK_CRATE && !(150..=224).contains(&skin) {7} else if kind == EK_LAMP || kind == EK_PROP_SERVER {5} else {6}, 0, x, y);
        }
        if matches!(kind,EK_PROP_SERVER|EK_PROP_AC|EK_PROP_REACTOR|EK_PROP_VENT) {
            let role=match kind {EK_PROP_SERVER=>2,EK_PROP_AC=>1,EK_PROP_REACTOR=>0,_=>3};
            let sector=map::level_index(self.wave);
            if let Some(index)=tactical_roles::MACHINERY[sector].iter().position(|&r| r==role) {
                self.machinery_destroyed(150+(sector*3+index) as u8,x,y);
            }
        }
        if kind == EK_CRATE {
            if (150..=224).contains(&skin) {
                self.machinery_destroyed(skin,x,y);
                self.campaign_impact(x, y, if (skin - 150) % 3 == 1 { 1 } else { 2 });
                self.burst_fx(x, y, EK_SPARK, 5, 0.22);
                self.light_dirty = true;
            }
            let roll = (self.rnd() * 3.0) as i32;
            let drop = [EK_AMMO, EK_MED, EK_ARMOR][roll.clamp(0, 2) as usize];
            self.ensure_drop(drop, x, y + 0.35);
        }
        if kind == EK_BOSS {
            self.drop_boss_case(x, y);
        }
    }

    fn hitscan(&mut self, ang: f32, mut dmg: i32, maxd: f32) -> bool {
        if self.power == field::POWER_OVERDRIVE {
            dmg = ((dmg as f32) * 1.65).round() as i32;
        }
        let dx = ang.cos();
        let dy = ang.sin();
        let mut best_t = maxd;
        let mut best_e: Option<usize> = None;
        for (i, e) in self.ents.iter().enumerate() {
            if e.kind == 0 || e.hp <= 0 || !target_kind(e.kind) {
                continue;
            }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            if t < 0.12 || t > best_t {
                continue;
            }
            let px = self.px + dx * t;
            let py = self.py + dy * t;
            let rad = e.radius * 1.35;
            if (px - e.x).powi(2) + (py - e.y).powi(2) < rad * rad {
                best_t = t;
                best_e = Some(i);
            }
        }
        // wall
        let mut t = 0.05;
        let mut wall_t = maxd;
        while t < maxd {
            let x = self.px + dx * t;
            let y = self.py + dy * t;
            if self.blocked(x.floor() as i32, y.floor() as i32) {
                wall_t = t;
                break;
            }
            t += 0.08;
        }
        if let Some(i) = best_e {
            if best_t < wall_t {
                let (ex, ey) = (self.ents[i].x, self.ents[i].y);
                let damage = if self.weapon == 1 {
                    let falloff = (1.0 - (best_t - 2.0).max(0.0) * 0.085).clamp(0.25, 1.0);
                    if best_t < 4.0 {
                        self.ents[i].stun = if self.ents[i].kind == EK_BOSS { 0.10 } else { 0.34 };
                    }
                    (dmg as f32 * falloff).round().max(1.0) as i32
                } else { dmg };
                self.player_hit(i, damage, self.px, self.py,self.weapon as usize);
                self.burst_fx(ex, ey, EK_SPARK, 3, 0.28);
                return true;
            }
        }
        if wall_t < maxd {
            let hx = self.px + dx * wall_t * 0.96;
            let hy = self.py + dy * wall_t * 0.96;
            self.stamp_decal(hx.floor() as i32, hy.floor() as i32);
            self.spawn_timed(EK_IMPACT, hx, hy, 0.28, -8.0);
            self.burst_fx(hx, hy, EK_SPARK, 2, 0.18);
        }
        false
    }

    fn wall_distance(&self, x: f32, y: f32, dx: f32, dy: f32, max_distance: f32) -> f32 {
        let mut cx = x.floor() as i32;
        let mut cy = y.floor() as i32;
        if self.blocked(cx, cy) { return 0.0; }
        let sx = if dx < 0.0 { -1 } else { 1 };
        let sy = if dy < 0.0 { -1 } else { 1 };
        let delta_x = if dx.abs() < 1e-6 { f32::INFINITY } else { dx.recip().abs() };
        let delta_y = if dy.abs() < 1e-6 { f32::INFINITY } else { dy.recip().abs() };
        let mut tx = (if dx < 0.0 { x - cx as f32 } else { cx as f32 + 1.0 - x }) * delta_x;
        let mut ty = (if dy < 0.0 { y - cy as f32 } else { cy as f32 + 1.0 - y }) * delta_y;
        for _ in 0..(MAP_W + MAP_H) {
            let distance;
            if tx < ty { distance = tx; tx += delta_x; cx += sx; }
            else { distance = ty; ty += delta_y; cy += sy; }
            if distance >= max_distance { return max_distance; }
            if self.blocked(cx, cy) { return distance; }
        }
        max_distance
    }

    fn fire_lance(&mut self) {
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let mut end = self.wall_distance(self.px, self.py, dx, dy, 28.0);
        let mut hits = [(0.0f32, 0usize); ENT_N];
        let mut count = 0;
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !target_kind(e.kind) { continue; }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            let side = ex * dy - ey * dx;
            let radius = e.radius + 0.08;
            if t <= 0.0 || side.abs() > radius { continue; }
            let entry = (t - (radius * radius - side * side).sqrt()).max(0.0);
            if entry >= end { continue; }
            hits[count] = (entry, i);
            count += 1;
        }
        // Nearest first, with the entity index as a deterministic tiebreak: two
        // entities at an identical f32 depth must always resolve the same way,
        // because the rank below decides who eats the falloff damage.
        hits[..count].sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (n, &(distance, i)) in hits[..count].iter().take(3).enumerate() {
            if self.ents[i].kind == EK_NONE { continue; }
            let barrel = self.ents[i].kind == EK_BARREL;
            self.player_hit(i, [160, 112, 78][n], self.px, self.py,self.weapon as usize);
            if barrel || n == 2 { end = distance; break; }
        }
        // Short-lived tracer sprites have no gameplay collision; the beam resolves once.
        let segments = ((end / 0.5).ceil() as usize).clamp(1, 24);
        for n in 0..segments {
            let t = 0.25 + (end - 0.25).max(0.0) * n as f32 / segments as f32;
            if t >= end { break; }
            self.spawn_timed(EK_RAY, self.px + dx * t, self.py + dy * t, 0.09, 6.0);
        }
        let t = (end - 0.04).max(0.0);
        self.spawn_timed(EK_IMPACT, self.px + dx * t, self.py + dy * t, 0.22, 0.0);
    }

    fn ignite(&mut self, x: f32, y: f32) {
        if self.blocked(x.floor() as i32, y.floor() as i32) { return; }
        let mut count = 0;
        for e in &mut self.ents {
            if e.kind != EK_FIREPATCH { continue; }
            count += 1;
            if (e.x - x).powi(2) + (e.y - y).powi(2) < 0.36 {
                e.timer = 10.0;
                return;
            }
        }
        if count >= 12 { return; }
        if let Some(i) = self.spawn(EK_FIREPATCH, x, y) {
            self.ents[i].timer = 10.0;
            self.ents[i].radius = 0.7;
            self.ents[i].zoff = 64.0;
        }
    }

    fn begin_reload(&mut self) {
        if self.reload_t > 0.0 || self.state != 0 {
            return;
        }
        let w = self.weapon as usize;
        if self.mag[w] >= MAG_SZ[w] || self.ammo[w] <= 0 {
            return;
        }
        // BR-12 feeds its tube one shell at a time. Other weapons swap a magazine.
        self.reload_dur = if w == 1 { 0.42 } else { RELOAD_T[w] };
        self.reload_t = self.reload_dur;
        self.events |= EV_RELOAD;
    }

    fn finish_reload(&mut self) {
        let w = self.weapon as usize;
        let need = MAG_SZ[w] - self.mag[w];
        let take = need.min(self.ammo[w]).max(0).min(if w == 1 { 1 } else { i32::MAX });
        self.mag[w] += take;
        self.ammo[w] -= take;
        self.reload_t = if w == 1 && self.mag[w] < MAG_SZ[w] && self.ammo[w] > 0 {
            self.events |= EV_RELOAD;
            self.reload_dur
        } else {
            0.0
        };
    }

    fn fire(&mut self) {
        if self.state != 0 || self.cooldown > 0.0 || self.tactical.cooldowns[self.weapon as usize]>0.0 {
            return;
        }
        let w = self.weapon as usize;
        // A tube-fed BR-12 can fire the shells already loaded. Pulling the
        // trigger interrupts the remaining shell-feed sequence.
        if self.reload_t > 0.0 {
            if w != 1 || self.mag[w] <= 0 { return; }
            self.reload_t = 0.0;
        }
        if self.mag[w] <= 0 {
            self.sound(21, w as u8, self.px, self.py);
            self.cooldown = 0.22;
            return;
        }
        self.mag[w] -= 1;
        if self.power == field::POWER_FEED {
            self.mag[w] += 1;
        }
        self.events |= EV_FIRE;
        self.ev_weapon = self.weapon;
        let fx_start=self.fx_n;
        let free_slots: [bool; ENT_N] = core::array::from_fn(|i| self.ents[i].kind == EK_NONE);
        match self.weapon {
            0 => {
                self.cooldown = 0.26;
                self.muzzle = 1.0;
                self.kick = 1.0;
                self.shake = (self.shake + 0.12).min(1.0);
                let a = self.pa + (self.rnd() - 0.5) * 0.02;
                self.hitscan(a, 15, 22.0);
                self.eject_casing();
            }
            1 => {
                self.cooldown = 0.56;
                self.muzzle = 1.0;
                self.kick = 1.4;
                self.shake = (self.shake + 0.38).min(1.0);
                for _ in 0..8 {
                    let a = self.pa + (self.rnd() - 0.5) * 0.19;
                    self.hitscan(a, 7, 11.0);
                }
                self.eject_casing();
            }
            2 => {
                self.cooldown = 0.075;
                self.muzzle = 1.0;
                self.kick = 0.7;
                self.shake = (self.shake + 0.08).min(1.0);
                self.spread = (self.spread + 0.020).min(0.18);
                let a = self.pa + (self.rnd() - 0.5) * (0.03 + self.spread);
                self.hitscan(a, 8, 20.0);
                self.eject_casing();
            }
            3 => {
                self.cooldown = 0.78;
                self.muzzle = 1.0;
                self.kick = 1.1;
                self.shake = (self.shake + 0.22).min(1.0);
                self.fire_lance();
            }
            4 => {
                self.cooldown = 0.78;
                self.muzzle = 1.0;
                self.kick = 1.5;
                self.shake = (self.shake + 0.28).min(1.0);
                let a = self.pa;
                if let Some(i) = self.spawn(EK_BOLT, self.px, self.py) {
                    self.ents[i].vx = a.cos() * 11.0;
                    self.ents[i].vy = a.sin() * 11.0;
                    self.ents[i].timer = 1.25;
                    self.ents[i].zoff = 10.0;
                }
            }
            5 => {
                self.cooldown = 0.42;
                self.muzzle = 1.0;
                self.kick = 0.9;
                self.shake = (self.shake + 0.18).min(1.0);
                let a = self.pa + (self.rnd() - 0.5) * 0.018;
                let hit = self.hitscan(a, 34, 18.0);
                let length = if hit { 9 } else { 15 };
                for n in 1..length {
                    let t = n as f32 * 0.55;
                    self.spawn_timed(EK_RAY, self.px + a.cos() * t, self.py + a.sin() * t, 0.1, 4.0);
                }
            }
            6 => {
                self.cooldown = 0.058;
                self.muzzle = 1.0;
                self.kick = 0.52;
                self.shake = (self.shake + 0.055).min(1.0);
                self.spread = (self.spread + 0.014).min(0.17);
                let a = self.pa + (self.rnd() - 0.5) * (0.035 + self.spread);
                self.hitscan(a, 7, 24.0);
                self.eject_casing();
            }
            7 => {
                self.cooldown = 0.62;
                self.muzzle = 1.0;
                self.kick = 1.15;
                self.shake = (self.shake + 0.24).min(1.0);
                let a = self.pa + (self.rnd() - 0.5) * 0.04;
                let _hit = self.hitscan(a, 22, 12.0);
                let dist = self.wall_distance(self.px, self.py, a.cos(), a.sin(), 11.0);
                let lead = 1.7;
                let reach = (dist * 0.92).clamp(lead, (dist - 0.12).max(lead));
                self.ignite(self.px + a.cos() * reach, self.py + a.sin() * reach);
                for n in 0..6 {
                    let t = lead + n as f32 * 0.65;
                    if t >= dist - 0.05 { break; }
                    self.spawn_timed(EK_FLAME, self.px + a.cos() * t, self.py + a.sin() * t, 2.5, 8.0);
                }
            }
            19..=32 => self.fire_campaign_weapon(),
            8 => self.fire_override(),
            9 => self.fire_forge(),
            10 => self.fire_chimera(),
            11 => {
                // Archivist's carbine is a precise amber pulse: a single
                // instant hit with a brief, narrow afterimage along its path.
                self.cooldown = 0.38;
                self.muzzle = 1.0;
                self.kick = 0.8;
                self.shake = (self.shake + 0.16).min(1.0);
                let a = self.pa;
                self.hitscan(a, 42, 22.0);
                let reach = self.wall_distance(self.px, self.py, a.cos(), a.sin(), 22.0).min(15.0);
                for n in 1..=20 {
                    let t = reach * n as f32 / 21.0;
                    self.spawn_timed(EK_RAY, self.px + a.cos() * t, self.py + a.sin() * t, 0.09, 12.0);
                }
            }
            12 => {
                // Oracle's predictor lands a delayed-looking, accurate double pulse.
                self.cooldown = 0.68; self.muzzle = 1.0; self.kick = 1.1;
                self.hitscan(self.pa, 58, 28.0);
                self.hitscan(self.pa + 0.012, 24, 28.0);
                for n in 1..=18 {
                    let t = n as f32 * 0.7;
                    self.spawn_timed(EK_RAY, self.px + self.pa.cos()*t, self.py + self.pa.sin()*t, 0.12, 13.0);
                }
            }
            13 => {
                // Reactor sink: short, dense copper scatter with a pressure kick.
                self.cooldown = 0.9; self.muzzle = 1.0; self.kick = 1.7;
                self.shake = (self.shake + 0.38).min(1.0);
                for _ in 0..11 {
                    let a = self.pa + (self.rnd()-0.5)*0.22;
                    self.hitscan(a, 11, 10.0);
                }
                self.eject_casing();
            }
            14 => {
                // Rime's three slow coolant slugs force movement through lanes.
                self.cooldown = 0.75; self.muzzle = 1.0; self.kick = 1.2;
                for n in -1..=1 {
                    let a = self.pa + n as f32 * 0.055;
                    if let Some(i) = self.spawn(EK_PROJ, self.px, self.py) {
                        self.ents[i].vx = a.cos()*6.1; self.ents[i].vy = a.sin()*6.1;
                        self.ents[i].timer = 2.4; self.ents[i].hp = 30; self.ents[i].zoff = 12.0;
                        self.ents[i].effect_tick = 5.0;
                        self.ents[i].skin = 232;
                    }
                }
            }
            15 => {
                // Relay discharges a fast, narrow signal burst.
                self.cooldown = 0.11; self.muzzle = 1.0; self.kick = 0.55;
                let a = self.pa + (self.rnd()-0.5)*0.045;
                self.hitscan(a, 14, 23.0);
                for n in 1..=8 { let t = n as f32 * 1.1; self.spawn_timed(EK_RAY, self.px+a.cos()*t, self.py+a.sin()*t, 0.06, 14.0); }
            }
            16 => {
                // Titan's siege shell is a slow, visible heavy explosive.
                self.cooldown = 1.25; self.muzzle = 1.0; self.kick = 1.9;
                self.shake = (self.shake + 0.48).min(1.0);
                let a = self.pa;
                if let Some(i) = self.spawn(EK_BOLT, self.px, self.py) {
                    self.ents[i].vx = a.cos()*8.2; self.ents[i].vy = a.sin()*8.2;
                    self.ents[i].timer = 1.65; self.ents[i].zoff = 10.0; self.ents[i].hp = 85;
                }
            }
            17 => {
                // Kest's assault rifle fires an accurate three-shot burst.
                self.cooldown = 0.32; self.muzzle = 1.0; self.kick = 0.85;
                for n in -1..=1 { self.hitscan(self.pa + n as f32 * 0.025, 14, 22.0); }
                self.eject_casing();
            }
            18 => {
                self.muzzle = 1.0;
                self.fire_lance();
                self.cooldown = 0.52;
                self.kick = 0.95;
                for n in 1..=12 {
                    let t = n as f32 * 0.9;
                    self.spawn_timed(EK_RAY, self.px+self.pa.cos()*t, self.py+self.pa.sin()*t, 0.16, 15.0);
                }
            }
            _ => {}
        }
        if self.weapon >= 8 {
            if !self.fx_q[fx_start..self.fx_n].iter().any(|fx|matches!(fx.kind,EK_RAY|EK_FLAME)) && !self.ents.iter().enumerate().any(|(i,en)| free_slots[i] && matches!(en.kind,EK_PROJ|EK_BOLT|EK_RAY|EK_FLAME)) {
                let reach=self.wall_distance(self.px,self.py,self.pa.cos(),self.pa.sin(),20.0).min(12.0);
                for n in 1..=8 { let d=reach*n as f32/9.0; self.spawn_timed(EK_RAY,self.px+self.pa.cos()*d,self.py+self.pa.sin()*d,0.10,12.0); }
            }
            for fx in &mut self.fx_q[fx_start..self.fx_n] {if matches!(fx.kind,EK_RAY|EK_FLAME) {fx.projectile_visual=1+(self.weapon-8) as u8;}}
            for (i, ent) in self.ents.iter_mut().enumerate() {
                if free_slots[i] && matches!(ent.kind,EK_PROJ|EK_BOLT|EK_RAY|EK_FLAME) { ent.projectile_visual=1+(self.weapon-8) as u8; }
            }
        }
        if self.power == field::POWER_FEED {
            self.cooldown *= 0.45;
        }
        self.tactical.cooldowns[w]=self.cooldown;
    }

    /// Veyran's rail. Pierces every target in the lane, then bursts.
    fn fire_override(&mut self) {
        self.cooldown = 0.8;
        self.muzzle = 1.0;
        self.kick = 1.5;
        self.shake = (self.shake + 0.4).min(1.0);
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let end = self.wall_distance(self.px, self.py, dx, dy, 32.0);
        let mut hits = [(0.0f32, 0usize); ENT_N];
        let mut count = 0;
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !target_kind(e.kind) { continue; }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            let side = ex * dy - ey * dx;
            let radius = e.radius + 0.12;
            if t <= 0.2 || side.abs() > radius || t >= end { continue; }
            hits[count] = (t, i);
            count += 1;
        }
        // Index tiebreak keeps equally-deep targets in a stable damage order.
        hits[..count].sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for &(_, i) in &hits[..count] {
            if self.ents[i].hp <= 0 { continue; }
            self.player_hit(i, 120, self.px, self.py,self.weapon as usize);
        }
        let tip = (end - 0.2).max(0.4);
        let (x, y) = (self.px + dx * tip, self.py + dy * tip);
        self.explode(x, y, 2.8, 90.0);
        for n in 0..12 {
            let t = tip * n as f32 / 12.0;
            self.spawn_timed(EK_RAY, self.px + dx * t, self.py + dy * t, 0.14, 4.0);
        }
    }

    /// HECATE's cutter. A wide beam that splashes at the strike.
    fn fire_forge(&mut self) {
        self.cooldown = 0.48;
        self.muzzle = 1.0;
        self.kick = 0.7;
        self.shake = (self.shake + 0.16).min(1.0);
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let mut strike = self.wall_distance(self.px, self.py, dx, dy, 18.0);
        for (offset, dmg) in [(0.0, 78), (-0.08, 42), (0.08, 42)] {
            let a = self.pa + offset;
            if self.hitscan(a, dmg, 18.0) {
                strike = strike.min(self.wall_distance(self.px, self.py, a.cos(), a.sin(), 18.0));
            }
            for n in 1..8 {
                let t = n as f32 * 0.4;
                if t >= strike { break; }
                self.spawn_timed(EK_RAY, self.px + a.cos() * t, self.py + a.sin() * t, 0.08, -4.0);
            }
        }
        let reach = (strike * 0.92).max(0.6);
        self.explode(self.px + dx * reach, self.py + dy * reach, 1.8, 48.0);
    }

    /// CHIMERA's specimen fan. Acid bolts burst and leave a short pool.
    fn fire_chimera(&mut self) {
        self.cooldown = 0.62;
        self.muzzle = 1.0;
        self.kick = 1.05;
        self.shake = (self.shake + 0.22).min(1.0);
        for n in 0..5 {
            let a = self.pa + (n as f32 - 2.0) * 0.07;
            // Clear of the player. Spawning on the muzzle made the first
            // step count as a self-hit and the burst killed the shooter.
            let x = self.px + a.cos() * 1.15;
            let y = self.py + a.sin() * 1.15;
            if let Some(i) = self.spawn(EK_PROJ, x, y) {
                let e = &mut self.ents[i];
                e.vx = a.cos() * 9.5;
                e.vy = a.sin() * 9.5;
                e.timer = 0.7;
                e.hp = 28; // 42 × 0.67, rounded to integer damage.
                e.effect_tick = 4.0;
                e.skin = 233;
                e.zoff = -6.0;
            }
        }
    }

    fn chimera_burst(&mut self, x: f32, y: f32) {
        self.sound(2, 3, x, y);
        // Allied blast: enemies only. The pool is the same.
        self.effect(EK_IMPACT, 3, x, y, 0.38, 10.0);
        self.spawn_smoke_cloud(x, y);
        self.events |= EV_EXPLODE;
        let mut hits = Vec::new();
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !solid_kind(e.kind) { continue; }
            let d = ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt();
            if d < 2.6 && self.los(x, y, e.x, e.y) {
                let fall = 1.0 - d / 2.6;
                hits.push((i, (80.0 * 0.67 * fall) as i32));
            }
        }
        for (i, dmg) in hits {
            self.player_hit(i, dmg.max(1), x, y,10);
        }
        self.scorch(x, y, 4.0);
    }

    fn scorch(&mut self, x: f32, y: f32, life: f32) {
        if self.blocked(x.floor() as i32, y.floor() as i32) { return; }
        for e in &mut self.ents {
            if e.kind != EK_FIREPATCH { continue; }
            if (e.x - x).powi(2) + (e.y - y).powi(2) < 0.36 {
                e.timer = e.timer.max(life);
                return;
            }
        }
        if let Some(i) = self.spawn(EK_FIREPATCH, x, y) {
            self.ents[i].timer = life;
            self.ents[i].radius = 0.7;
            self.ents[i].skin = 2;
            self.ents[i].hp = 100;
        }
    }

    /// Damage the player and hostiles standing in a live flame.
    fn burn_at(&mut self, x: f32, y: f32, dmg: i32, hurt_player: bool) {
        if (self.px-x).powi(2) + (self.py-y).powi(2) < 64.0 { self.sound(12, 0, x, y); }
        let pd = (self.px - x).powi(2) + (self.py - y).powi(2);
        if hurt_player && pd < 0.9 * 0.9 && self.los(x, y, self.px, self.py) {
            self.damage_player_from(dmg, x, y);
        }
        let mut hits = Vec::new();
        for (j, target) in self.ents.iter().enumerate() {
            if target.hp <= 0 || !solid_kind(target.kind) { continue; }
            let range = target.radius + 0.7;
            if (target.x - x).powi(2) + (target.y - y).powi(2) < range * range
                && self.los(x, y, target.x, target.y)
            {
                hits.push(j);
            }
        }
        for j in hits {
            if hurt_player {self.hurt_ent(j,dmg,x,y);}else{self.player_hit(j,dmg,x,y,10);}
        }
    }

    fn eject_casing(&mut self) {
        // Cosmetic shells never crowd out hostiles or projectiles.
        let shells: Vec<usize> = self.ents.iter().enumerate()
            .filter(|(_, e)| e.kind == EK_SPARK && e.effect_tick == 5.0).map(|(i, _)| i).collect();
        if shells.len() >= 24 {
            if let Some(i) = shells.into_iter().min_by(|a, b| self.ents[*a].timer.total_cmp(&self.ents[*b].timer)) {
                self.ents[i].kind = EK_NONE;
            }
        }
        let rx = -self.pa.sin();
        let ry = self.pa.cos();
        let (x, y) = (self.px + self.pa.cos() * 0.55 + rx * 0.18,
                      self.py + self.pa.sin() * 0.55 + ry * 0.18);
        let (x, y) = if self.circle_blocked(x, y, 0.06) { (self.px, self.py) } else { (x, y) };
        let jx = 1.2 + self.rnd();
        let jy = 1.2 + self.rnd();
        let pax = self.pa.cos() * 0.2;
        let pay = self.pa.sin() * 0.2;
        let slot = self.fx_n;
        self.queue_fx(EK_SPARK, x, y, rx * jx + pax, ry * jy + pay, 6.0, self.h as f32 * 0.08);
        if self.fx_n > slot { self.fx_q[slot].variant = 5; }
        self.spawn_timed(EK_SMOKE, x, y, 0.22, self.h as f32 * 0.08);
    }

    fn enemy_shoot(&mut self, i: usize) {
        if (23..=36).contains(&self.ents[i].skin) {
            self.fire_campaign_boss(i);
            return;
        }
        let shooter = self.ents[i];
        let (x, y) = (shooter.x, shooter.y);
        let role = combat::profile(shooter.skin, shooter.kind);
        set_anim(&mut self.ents[i], ANIM_FIRE, 0.24);
        let sp = if enemies::combat_skin(shooter.skin) == SKIN_MARKSMAN { 8.0 } else { 5.4 };
        let zoff = if enemies::combat_skin(shooter.skin) == SKIN_HORNET { -35.0 } else { -8.0 };
        for n in 0..role.pellets {
            let a = shooter.aim + (n as f32 - (role.pellets - 1) as f32 * 0.5) * role.spread;
            if let Some(index) = self.spawn(EK_PROJ, x, y) {
                let e = &mut self.ents[index];
                e.vx = a.cos() * sp;
                e.vy = a.sin() * sp;
                e.timer = 2.8;
                e.hp = role.damage;
                e.effect_tick = if enemies::combat_skin(shooter.skin) == SKIN_SPITTER { 3.0 } else { 0.0 };
                e.skin = 100 + enemies::combat_skin(shooter.skin).min(ENEMY_PROJECTILE_COUNT as u8 - 1);
                e.zoff = zoff;
            }
        }
        self.effect(EK_SPARK, if role.pellets > 1 { 1 } else { 0 }, x, y, 0.12, zoff);
        if shooter.kind == EK_BOSS && self.boss_phase >= 2 {
            self.boss_vuln = self.boss_vuln.max(1.15);
        }
    }

    fn pickup(&mut self, kind: u8) {
        self.sound(match kind { EK_MED => 8, EK_ARMOR => 9, EK_AMMO => 10, _ => 11 }, 0, self.px, self.py);
        if kind == EK_AMMO {
            for slot in 19..WEP_N {
                if self.owns_slot(slot) { self.ammo[slot] = (self.ammo[slot] + MAG_SZ[slot] * 2).min(MAG_SZ[slot] * 6); }
            }
        }
        match kind {
            EK_MED => {
                self.health = (self.health + 35).min(100);
                self.events |= EV_PICK_SILVER;
            }
            EK_AMMO => {
                for slot in 0..WEP_N {
                    if slot != 0 && !self.has_w(slot - 1) { continue; }
                    self.ammo[slot] = (self.ammo[slot] + AMMO_PICKUP[slot]).min(RESERVE_CAP[slot]);
                }
                self.events |= EV_PICK_SILVER;
            }
            EK_ARMOR => {
                self.armor = (self.armor + 50).min(100);
                self.events |= EV_PICK_SILVER;
            }
            // Kinds are not contiguous (EK_OVERRIDE_CONSOLE, EK_NODE and
            // EK_TERMINAL sit between them), so the seven guns are named.
            EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 | EK_GUN6 | EK_GUN7 | EK_GUN8 => {
                let slot = match kind {
                    EK_GUN2 => 1,
    EK_GUN3 => 2,
    EK_GUN4 => 3,
    EK_GUN5 => 4,
                    EK_GUN6 => 5,
    EK_GUN7 => 6,
    _ => 7,
                };
                self.set_w(slot - 1);
                self.ammo[slot] = (self.ammo[slot] + GROUND_GUN_PICKUP[slot]).min(RESERVE_CAP[slot]);
                if self.mag[slot] <= 0 { self.mag[slot] = MAG_SZ[slot]; }
                self.weapon = slot as i32;
                self.reload_t = 0.0;
                self.pickup_t = 0.6;self.pickup_dur=0.6;
                self.events |= EV_PICK_GOLD;
            }
            k if field::is_boss_case(k) => {
                self.grant_slot(field::boss_slot(kind));
                self.state = 2;
            }
            EK_POWER => {
                let boost = field::power_kind(self.wave);
                self.power = boost;
                self.power_t = if boost == field::POWER_AEGIS { 14.0 } else if boost == field::POWER_FEED { 10.0 } else { 12.0 };
                self.shield_pool = if boost == field::POWER_AEGIS { 96 } else { 0 };
                self.say(field::RADIO_POWER);
                self.events |= EV_PICK_GOLD;
            }
            _ => {}
        }
    }

    fn needs_pickup(&self, kind: u8) -> bool {
        match kind {
            EK_MED => self.health < 100,
            EK_ARMOR => self.armor < 100,
            EK_AMMO => (0..WEP_N).any(|slot| {
                self.owns_slot(slot) && self.ammo[slot] < RESERVE_CAP[slot]
            }),
            _ => true,
        }
    }

    fn tick(&mut self, dt: f32) {
        let dt = safe_dt(dt);
        self.time += dt;
        self.age_smoke(dt);
        self.boss_vuln = (self.boss_vuln - dt).max(0.0);
        if self.power_t > 0.0 {
            self.power_t = (self.power_t - dt).max(0.0);
            if self.power == field::POWER_AEGIS {
                self.shield_pool = ((self.power_t / 14.0) * 96.0) as i32;
            }
            if self.power_t <= 0.0 {
                self.power = 0;
                self.shield_pool = 0;
            }
        }
        self.events = 0;
        self.sound_cues.clear();
        self.ev_weapon = 0;
        if self.state == 0 {
            self.elapsed += dt;
        }
        self.tactical.tick(dt);
        let previous_weapon=self.weapon;
        self.cooldown = (self.cooldown - dt).max(0.0);
        if self.reload_t > 0.0 {
            self.reload_t = (self.reload_t - dt).max(0.0);
            if self.reload_t <= 0.0 {
                self.finish_reload();
            }
        }
        self.iframes = (self.iframes - dt).max(0.0);
        self.pickup_t = (self.pickup_t - dt).max(0.0);
        self.shake = (self.shake - dt * 2.2).max(0.0);
        self.muzzle = (self.muzzle - dt * 8.0).max(0.0);
        self.hurt = (self.hurt - dt * 2.6).max(0.0);
        self.kick = (self.kick - dt * 6.0).max(0.0);
        self.hitmarker = (self.hitmarker - dt * if self.hitmarker > 1.0 { 2.4 } else { 5.0 }).max(0.0);
        // Low-health strain: ramps in under 35 HP and breathes at ~1 Hz, so the
        // player feels danger without having to read the health plate.
        self.strain = if self.state != 0 || self.health >= 35 {
            (self.strain - dt * 2.2).max(0.0)
        } else {
            let base = 1.0 - (self.health.max(0) as f32 / 35.0);
            (base * 0.72 + (base * (self.time * 2.2).sin() * 0.5 + 0.5) * 0.28).min(1.0)
        };
        if self.cooldown <= 0.0 || self.weapon != 2 {
            self.spread = (self.spread - dt * 0.28).max(0.0);
        }

        for d in self.door.iter_mut() {
            if *d > 0.0 && *d < 1.0 {
                let previous = *d;
                *d = (*d + dt * 1.6).min(1.0);
                if previous < 0.98 && *d >= 0.98 { self.light_dirty = true; }
            }
        }

        let bits = if self.qa { self.qa_bits } else { self.bits };

        if self.state == 0 {
            self.pa += self.mx * 0.0062;
            self.pitch -= self.my * 0.0054;
            self.pitch = self.pitch.clamp(-0.42, 0.42);
            self.mx = 0.0;
            self.my = 0.0;

            if bits & IN_TURNL != 0 {
                self.pa -= 2.4 * dt;
            }
            if bits & IN_TURNR != 0 {
                self.pa += 2.4 * dt;
            }

            for (slot, &bit) in IN_W_SLOT.iter().enumerate() {
                if bits & bit != 0 && self.wpn_latched & bit == 0 && self.owns_slot(slot) {
                    self.weapon = slot as i32;
                    self.reload_t = 0.0;
                }
            }
            self.wpn_latched = bits & IN_W_ALL;

            if bits & IN_RELOAD != 0 {
                if !self.reload_latched {
                    self.begin_reload();
                }
                self.reload_latched = true;
            } else {
                self.reload_latched = false;
            }

            let dir_x = self.pa.cos();
            let dir_y = self.pa.sin();
            let right_x = -dir_y;
            let right_y = dir_x;
            let mut mx = 0.0f32;
            let mut my = 0.0f32;
            if bits & IN_W != 0 {
                mx += dir_x;
                my += dir_y;
            }
            if bits & IN_S != 0 {
                mx -= dir_x;
                my -= dir_y;
            }
            if bits & IN_D != 0 {
                mx += right_x;
                my += right_y;
            }
            if bits & IN_A != 0 {
                mx -= right_x;
                my -= right_y;
            }
            let mag = (mx * mx + my * my).sqrt();
            let sprint = if bits & IN_SPRINT != 0 { 1.32 } else { 1.0 };
            let boosted: f32 = 3.35 * sprint * if self.power == field::POWER_OVERDRIVE { 1.85 } else { 1.0 };
            // Absolute cap: nothing moves faster than 1.32x walk speed.
            let speed = boosted.min(3.35 * 1.32);
            if mag > 0.001 {
                mx /= mag;
                my /= mag;
                self.try_move(self.px + mx * speed * dt, self.py + my * speed * dt);
                self.walk += dt * speed * 2.2;
                self.hud.speed = speed;
                self.foot_acc += dt * speed;
                if self.foot_acc > 1.15 {
                    self.foot_acc = 0.0;
                    self.events |= EV_FOOT;
                }
            } else {
                self.hud.speed = 0.0;
                self.walk *= 1.0 - (dt * 6.0).min(1.0);
                self.foot_acc = 0.0;
            }
            // Backstop: no sequence (slamming seals, loads, spawns) may
            // leave the player's center embedded in solid rock with no
            // legal move. Wall-hugging (circle overlap only) is normal
            // play and must not relocate.
            if self.blocked(self.px.floor() as i32, self.py.floor() as i32) {
                let (nx, ny) = self.nearest_open(self.px, self.py, self.pr);
                self.px = nx;
                self.py = ny;
            }

            if bits & IN_USE != 0 {
                if !self.use_latched {
                    self.open_nearby_doors(true);
                    self.use_field();
                    let clear = self.pending_hostiles == 0 && !self.ents.iter().any(|e| e.hp > 0 && is_hostile_kind(e.kind));
                    if clear && self.node_done && self.near_override() && !self.boss_spawned && self.boss_intro <= 0.0 {
                        self.boss_intro = self.boss_intro_duration();
                        self.events |= EV_BOSS;
                    }
                }
                self.use_latched = true;
            } else {
                self.use_latched = false;
                self.open_nearby_doors(false);
            }

            if bits & IN_FIRE != 0 {
                self.fire();
            }

            let mut pick: Vec<usize> = Vec::new();
            for (i, e) in self.ents.iter().enumerate() {
                if is_pickup(e.kind) {
                    let d = (e.x - self.px).powi(2) + (e.y - self.py).powi(2);
                    if d < 0.45 && self.needs_pickup(e.kind) && self.los(self.px, self.py, e.x, e.y) {
                        pick.push(i);
                    }
                }
            }
            for i in pick {
                let k = self.ents[i].kind;
                self.ents[i].kind = 0;
                self.pickup(k);
            }
            map::check_ambushes(self);
        } else {
            self.hud.speed = 0.0;
        }

        if previous_weapon!=self.weapon {self.cooldown=self.cooldown.min(0.12);self.pickup_t=0.28;self.pickup_dur=0.28;}

        // AI
        self.flow_age += dt;
        if self.flow_age > 0.18 && self.state == 0 {
            self.rebuild_flow();
            self.flow_age = 0.0;
        }
        let px = self.px;
        let py = self.py;
        let pstate = self.state;
        let mut shots: Vec<usize> = Vec::new();
        let mut melee: Vec<(usize, i32)> = Vec::new();
        for i in 0..ENT_N {
            let e = self.ents[i];
            if e.kind == 0 {
                continue;
            }
            if e.kind == EK_PROJ && e.effect_tick == 6.0 && e.aim == 1.0 {
                let target = self.ents.iter().filter(|other| other.hp > 0 && is_hostile_kind(other.kind))
                    .filter(|other| { let dx = other.x - e.x; let dy = other.y - e.y; dx * e.vx + dy * e.vy > 0.0 && dx * dx + dy * dy < 144.0 })
                    .min_by(|a,b| ((a.x-e.x).powi(2)+(a.y-e.y).powi(2)).total_cmp(&((b.x-e.x).powi(2)+(b.y-e.y).powi(2))))
                    .map(|other| (other.x,other.y));
                if let Some((x,y)) = target {
                    let a = (y-e.y).atan2(x-e.x); let speed = (e.vx*e.vx+e.vy*e.vy).sqrt();
                    let blend = (dt*3.5).min(1.0);
                    self.ents[i].vx += (a.cos()*speed-e.vx)*blend;
                    self.ents[i].vy += (a.sin()*speed-e.vy)*blend;
                    self.ents[i].zoff = 8.0 + (e.frame*18.0+i as f32*1.7).sin()*2.2;
                }
            }
            let e = &mut self.ents[i];
            e.flash = (e.flash - dt).max(0.0);
            e.bar_t = (e.bar_t - dt).max(0.0);
            advance_anim(e, dt);
            if e.kind == EK_NONE || e.hp <= 0 {
                continue;
            }
            if !is_hostile_kind(e.kind) {
                e.frame += dt;
            }
            match e.kind {
                EK_HUSK | EK_BRUTE | EK_WRAITH | EK_BOSS | EK_MARTYR => {
                    if pstate != 0 {
                        continue;
                    }
                    if e.stun > 0.0 {
                        e.effect_tick = 0.0;
                        e.stun = (e.stun - dt).max(0.0);
                        e.timer = e.timer.max(0.1);
                        e.vx *= 0.85;
                        e.vy *= 0.85;
                        let (y, nx, ny, radius) = (e.y, e.x + e.vx * dt, e.y + e.vy * dt, e.radius);
                        let _ = e;
                        if !self.circle_blocked(nx, y, radius) { self.ents[i].x = nx; }
                        if !self.circle_blocked(self.ents[i].x, ny, radius) { self.ents[i].y = ny; }
                        continue;
                    }
                    let dx = px - e.x;
                    let dy = py - e.y;
                    let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                    e.timer -= dt;
                    let role = combat::profile(e.skin, e.kind);
                    let spd = role.speed * if e.shield != 0 { 0.82 } else { 1.0 } * if self.tactical.slow[i]>0.0 {0.45}else{1.0};
                    let hold = role.range;
                    let (ex, ey, kind) = (e.x, e.y, e.kind);
                    let _ = e;
                    let clear = dist < 16.0 && self.los(ex, ey, px, py);
                    let e = &mut self.ents[i];
                    // Freeze position and aim during anticipation. Dodging or
                    // breaking sight during this window defeats the attack.
                    if e.effect_tick > 0.0 {
                        e.effect_tick = (e.effect_tick - dt).max(0.0);
                        if e.effect_tick <= 0.0 {
                            e.timer = role.cooldown;
                            set_anim(e, ANIM_FIRE, 0.24);
                            if clear {
                                if role.melee {
                                    if dist < role.range + 0.3 { melee.push((i, role.damage)); }
                                } else { shots.push(i); }
                            }
                        }
                        continue;
                    }
                    if clear && e.timer <= 0.0 && dist < if role.melee { role.range + 0.15 } else { 12.0 } {
                        e.aim = dy.atan2(dx);
                        e.effect_tick = role.windup;
                        set_anim(e, ANIM_SPECIAL, role.windup);
                        continue;
                    }
                    let (ex0, ey0) = (e.x, e.y);
                    if dist > hold {
                        if clear {
                            e.x += dx / dist * spd * dt;
                            e.y += dy / dist * spd * dt;
                        } else if dist < 22.0 {
                            let cx = clamp_i(ex.floor() as i32, 0, MAP_W as i32 - 1) as usize;
                            let cy = clamp_i(ey.floor() as i32, 0, MAP_H as i32 - 1) as usize;
                            let dir = self.flow_dir[cy * MAP_W + cx];
                            if dir != 0 {
                                let (wx, wy) = match dir {
                                    1 => (1.0, 0.0),
                                    2 => (-1.0, 0.0),
                                    3 => (0.0, 1.0),
                                    _ => (0.0, -1.0),
                                };
                                let tx = cx as f32 + 0.5 + wx * 0.9;
                                let ty = cy as f32 + 0.5 + wy * 0.9;
                                let ddx = tx - e.x;
                                let ddy = ty - e.y;
                                let l = (ddx * ddx + ddy * ddy).sqrt().max(0.05);
                                e.x += ddx / l * spd * dt;
                                e.y += ddy / l * spd * dt;
                            }
                        }
                    } else if clear && !role.melee && dist < hold * 0.65 {
                        e.x -= dx / dist * spd * dt * 0.65;
                        e.y -= dy / dist * spd * dt * 0.65;
                    } else if kind == EK_WRAITH && !role.melee {
                        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
                        e.x += -dy / dist * spd * dt * side * 0.6;
                        e.y += dx / dist * spd * dt * side * 0.6;
                    }
                    let moved = ((e.x - ex0).powi(2) + (e.y - ey0).powi(2)).sqrt();
                    if e.shield != 0 {
                        let target = if clear {
                            dy.atan2(dx)
                        } else if moved > 0.001 {
                            (e.y - ey0).atan2(e.x - ex0)
                        } else {
                            e.face
                        };
                        let mut delta = target - e.face;
                        if delta > core::f32::consts::PI { delta -= core::f32::consts::TAU; }
                        if delta < -core::f32::consts::PI { delta += core::f32::consts::TAU; }
                        e.face += delta.clamp(-1.5 * dt, 1.5 * dt);
                        if field::shield_blocks(e.face, e.x, e.y, px, py) {
                            e.flash = e.flash.max(0.08);
                        }
                    }
                    e.frame += dt * 1.4 + moved * 6.5;
                    // Locomotion bob so chasers read as moving, not sliding:
                    // floaters hover, ground units step. Assigned, never
                    // accumulated, from the skin's base height.
                    let base_z = skin_def(e.skin).map(|s| s.zoff).unwrap_or(0.0);
                    if enemies::combat_skin(e.skin) == SKIN_HORNET || e.kind == EK_MARTYR {
                        e.zoff = base_z + (self.time * 5.0 + i as f32 * 1.7).sin() * 7.0;
                    } else if moved > 0.001 {
                        e.zoff = base_z + (e.frame * 9.0).sin() * 2.0;
                    } else {
                        e.zoff = base_z;
                    }
                    if e.anim_lock <= 0.0 {
                        if moved > 0.001 {
                            set_anim(e, ANIM_MOVE, 0.0);
                        } else if e.anim == ANIM_MOVE {
                            set_anim(e, ANIM_IDLE, 0.0);
                        }
                        // The charge/reload beat happens in the quiet tail of
                        // a ranged cooldown, giving every shooter a readable
                        // two-frame preparation before its next attack.
                        if e.timer > 0.0 && e.timer < 0.34 && kind != EK_BRUTE && kind != EK_BOSS {
                            set_anim(e, ANIM_RELOAD, 0.28);
                        }
                    }
                    e.vx *= 0.85;
                    e.vy *= 0.85;
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    let (ex, ey, radius) = (e.x, e.y, e.radius);
                    let _ = e;
                    if self.circle_blocked(ex, ey, radius) {
                        let slide_x = !self.circle_blocked(ex, ey0, radius);
                        let resolved_x = if slide_x { ex } else { ex0 };
                        let slide_y = !self.circle_blocked(resolved_x, ey, radius);
                        let e = &mut self.ents[i];
                        e.x = resolved_x;
                        e.y = if slide_y { ey } else { ey0 };
                        e.vx = 0.0;
                        e.vy = 0.0;
                    }
                    if self.circle_blocked(self.ents[i].x, self.ents[i].y, radius) {
                        let (ux, uy) = self.nearest_open(self.ents[i].x, self.ents[i].y, radius);
                        self.ents[i].x = ux;
                        self.ents[i].y = uy;
                        self.ents[i].vx = 0.0;
                        self.ents[i].vy = 0.0;
                    }
                }
                EK_PROJ => {
                    let (old_x, old_y) = (e.x, e.y);
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.timer -= dt;
                    let (ex, ey, dead, damage, variant, visual, palette, trail) = (e.x, e.y, e.timer <= 0.0, e.hp, e.effect_tick, e.skin, e.face as i32, matches!(e.skin, 120 | 129 | 133 | 230) && (e.frame * 12.0).floor() != ((e.frame - dt) * 12.0).floor());
                    let _ = e;
                    if trail && !dead { self.spawn_timed(EK_SMOKE, old_x, old_y, 0.75, 10.0); }
                    if dead || !self.los(old_x, old_y, ex, ey) {
                        self.ents[i].kind = 0;
                        if (100..=136).contains(&visual) || (230..=234).contains(&visual) { self.projectile_impact(old_x, old_y, visual, palette); }
                        if visual == 230 && !dead && variant == 6.0 { self.explode(old_x, old_y, 1.6, damage as f32 * 0.4); }
                        if variant == 4.0 {
                            self.chimera_burst(old_x, old_y);
                        } else if !dead {
                            self.effect(EK_IMPACT, if variant == 3.0 { 3 } else { 1 }, old_x, old_y, 0.22, -6.0);
                        }
                    } else if pstate == 0 {
                        let d = segment_distance_sq(px, py, old_x, old_y, ex, ey);
                        if d < 0.22 {
                            if variant != 4.0 && variant != 5.0 && variant != 6.0 {
                                self.ents[i].kind = 0;
                                melee.push((i, damage));
                            }
                        } else if variant == 4.0 || variant == 5.0 || variant == 6.0 {
                            let mut hit = None;
                            for (j, o) in self.ents.iter().enumerate() {
                                if o.hp <= 0 || !target_kind(o.kind) { continue; }
                                let d2 = segment_distance_sq(o.x, o.y, old_x, old_y, ex, ey);
                                if d2 < (o.radius + 0.25).powi(2) { hit = Some(j); break; }
                            }
                            if let Some(j) = hit {
                                self.ents[i].kind = 0;
                                let slot=if self.ents[i].projectile_visual>0 {7+self.ents[i].projectile_visual as usize}else if variant==4.0 {10}else{14};
                                self.player_hit(j, damage, ex, ey,slot);
                                if (230..=234).contains(&visual) {
                                    self.projectile_impact(ex, ey, visual, palette);
                                    if visual == 230 { self.explode(ex, ey, 1.6, damage as f32 * 0.4); }
                                }
                                if variant == 4.0 { self.chimera_burst(ex, ey); }
                                else {
                                    self.ents[j].timer += 0.45;
                                    self.effect(EK_IMPACT, 1, ex, ey, 0.22, -8.0);
                                }
                            }
                        }
                    }
                }
                EK_RAY => {
                    e.timer -= dt;
                    if e.timer <= 0.0 { e.kind = EK_NONE; }
                }
                EK_FIREPATCH => {
                    e.timer -= dt;
                    e.effect_tick -= dt;
                    e.frame += dt * 3.0;
                    let (x, y, pulse, acid) = (e.x, e.y, e.effect_tick <= 0.0, e.skin == 2);
                    if e.timer <= 0.0 { e.kind = EK_NONE; continue; }
                    let damage = if acid {
                        // Fractional accumulator keeps pool damage at exactly 67% over time.
                        if pulse { e.hp += 536; }
                        let damage = (e.hp - 100) / 100;
                        if pulse { e.hp = 100 + (e.hp - 100) % 100; }
                        damage
                    } else { 8 };
                    if pulse { e.effect_tick = 0.25; }
                    let _ = e;
                    if pulse && pstate == 0 {
                        let _ = e;
                        self.burn_at(x, y, damage, !acid);
                    }
                }
                EK_GIB => {
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.vx *= 0.9;
                    e.vy *= 0.9;
                    e.timer -= dt;
                    if e.timer <= 0.0 {
                        e.kind = 0;
                    }
                }
                EK_IMPACT | EK_SPARK | EK_SMOKE => {
                    if e.kind == EK_SPARK && e.effect_tick == 5.0 {
                        // World-space metres: stun stores elevation and aim vertical velocity.
                        // Gravity is independent of framebuffer resolution and frame rate.
                        let h = self.h as f32;
                        let old_x=e.x;let old_y=e.y;
                        e.x+=e.vx*dt;e.y+=e.vy*dt;
                        let mut landed=false;
                        if e.shield<3 {
                            e.aim-=9.8*dt;e.stun+=e.aim*dt;
                            if e.stun<=0.0 {
                                e.stun=0.0;landed=e.aim < -0.2;
                                e.shield+=1;e.aim=-e.aim*0.32;
                                e.vx*=0.5;e.vy*=0.5;
                                if e.shield>=3 {e.aim=0.0;e.vx=0.0;e.vy=0.0;}
                            }
                        }
                        e.zoff=h*(0.45-e.stun*0.5);
                        e.timer -= dt;
                        if e.timer <= 0.0 { e.kind = EK_NONE; }
                        let (x, y, shell) = (e.x, e.y, e.skin == 1);
                        let _ = e;
                        if self.circle_blocked(x, y, 0.06) {
                            self.ents[i].x = old_x; self.ents[i].y = old_y;
                            self.ents[i].vx *= -0.4; self.ents[i].vy *= -0.4;
                        }
                        if landed { self.sound(23, u8::from(shell), old_x, old_y); }
                        continue;
                    }
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    if e.kind == EK_SMOKE && e.effect_tick >= 9.0 {
                        e.vx *= 0.985;
                        e.vy *= 0.985;
                        e.frame += dt;
                    } else {
                        e.vx *= 0.88;
                        e.vy *= 0.88;
                        e.zoff += if e.kind == EK_SPARK { -40.0 * dt } else { 8.0 * dt };
                    }
                    e.timer -= dt;
                    if e.timer <= 0.0 { e.kind = 0; }
                }
                EK_BOLT => {
                    let (old_x, old_y) = (e.x, e.y);
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.timer -= dt;
                    e.frame += dt * 6.0;
                    e.effect_tick += dt;
                    let trail = e.effect_tick >= 0.08;
                    if trail { e.effect_tick -= 0.08; }
                    let (ex, ey, dead, heavy) = (e.x, e.y, e.timer <= 0.0, e.hp > 1);
                    let _ = e;
                    if trail && !dead {
                        // Small puffs trace the missile's flight and expire within one second.
                        self.spawn_timed(EK_SMOKE, old_x, old_y, 0.9, 10.0);
                    }
                    if dead || !self.los(old_x, old_y, ex, ey) {
                        self.ents[i].kind = 0;
                        self.explode(old_x, old_y, if heavy { 5.6 } else { 4.8 }, if heavy { 95.0 } else { 65.0 });
                    } else {
                        let mut hit = None;
                        for (j, o) in self.ents.iter().enumerate() {
                            if j == i || o.hp <= 0 || !target_kind(o.kind) {
                                continue;
                            }
                            let d2 = segment_distance_sq(o.x, o.y, old_x, old_y, ex, ey);
                            if d2 < (o.radius + 0.2).powi(2) {
                                hit = Some(j);
                                break;
                            }
                        }
                        if let Some(j) = hit {
                            self.ents[i].kind = 0;
                            self.hurt_ent(j, if heavy { 45 } else { 24 }, ex, ey);
                            self.explode(ex, ey, if heavy { 5.6 } else { 4.8 }, if heavy { 95.0 } else { 65.0 });
                        }
                    }
                }
                EK_LAMP => {
                    e.zoff = -118.0 + (self.time * 2.6 + e.x).sin() * 5.0;
                }
                EK_FLAME => {
                    e.timer -= dt;
                    e.frame += dt * 2.0;
                    let (x, y, live) = (e.x, e.y, e.timer > 0.0);
                    if !live {
                        e.kind = EK_NONE;
                        continue;
                    }
                    e.effect_tick -= dt;
                    if e.effect_tick > 0.0 { continue; }
                    e.effect_tick = 0.25;
                    let _ = e;
                    self.burn_at(x, y, 6, true);
                }
                EK_CHAIN => {
                    e.zoff = -104.0 + (self.time * 2.2 + e.x).sin() * 6.0;
                }
                EK_BARREL => {
                    e.vx *= 0.8;
                    e.vy *= 0.8;
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                }
                _ => {}
            }
            let seated = self.ents[i].kind;
            if floor_prop(seated) {
                let scale = sprite_style(&self.ents[i]).1;
                let bob = if is_pickup(seated) { (self.time * 2.2 + self.ents[i].x).sin() * 3.0 } else { 0.0 };
                self.ents[i].zoff = floor_seat(self.h as f32, scale, bob);
            }
        }
        for i in shots {
            if self.ents[i].kind != 0 && self.ents[i].hp > 0 {
                self.enemy_shoot(i);
            }
        }
        for (i, dmg) in melee {
            // Martyrs detonate instead of dealing contact damage. The blast
            // can hurt the player and other hostiles; the drone is consumed.
            if self.ents.get(i).is_some_and(|e| e.kind == EK_MARTYR && e.hp > 0) {
                let (x, y) = (self.ents[i].x, self.ents[i].y);
                self.ents[i].hp = 0;
                self.ents[i].kind = EK_NONE;
                self.light_dirty = true;
                self.explode(x, y, 2.4, 55.0);
                continue;
            }
            self.damage_player_from(dmg, self.ents[i].x, self.ents[i].y);
        }

        self.flush_fx();

        if self.state == 0 && self.pending_hostiles > 0 {
            self.reinforcement_t -= dt;
            if self.reinforcement_t <= 0.0 {
                self.reinforcement_t = 1.25;
                self.spawn_reinforcement(true);
            }
        }

        if !self.boss_spawned && self.state == 0 && self.boss_intro > 0.0 {
            {
                let prev = self.boss_intro;
                self.boss_intro -= dt;
                let duration=self.boss_intro_duration();
                for stage in 0..5 {
                    let threshold=duration*(1.0-stage as f32*0.2)-0.01;
                    if prev>threshold && self.boss_intro<=threshold {self.boss_intro_effect(stage);}
                }
                if self.boss_intro <= 0.0 {
                    self.boss_intro = 0.0;
                    self.maybe_spawn_boss();
                }
            }
        }

        if self.boss_spawned && self.state == 0 {
            let max_hp = self.boss_max_health();
            let boss_hp = self.ents.iter().find(|e| e.kind == EK_BOSS && e.hp > 0).map(|e| e.hp).unwrap_or(0);
            let phase = if boss_hp > 0 && boss_hp as i64 * 3 <= max_hp as i64 { 2 } else if boss_hp > 0 && boss_hp as i64 * 3 <= max_hp as i64 * 2 { 1 } else { 0 };
            if phase > self.boss_phase {
                let entered = phase;
                self.boss_phase = phase;
                self.shake = (self.shake + 0.5).min(1.0);
                self.events |= EV_EXPLODE;
                if entered == 1 { self.seal_lockdown(); }
                if entered >= 2 { self.expose_boss(); }
                match map::level_index(self.wave) {
                    6 if entered == 1 => {
                        if let Some(boss) = self.ents.iter_mut().find(|e| e.kind == EK_BOSS && e.hp > 0) {
                            boss.shield = 1; boss.shield_hp = 55;
                        }
                    }
                    7 => {
                        let (x, y) = map::boss_spots(self.wave)[entered as usize % 2];
                        if !self.blocked(x as i32, y as i32) {
                            if let Some(boss) = self.ents.iter_mut().find(|e| e.kind == EK_BOSS && e.hp > 0) {
                                boss.x = x; boss.y = y;
                            }
                            self.spawn_smoke_cloud(x, y);
                        }
                    }
                    10 => {
                        if let Some((x, y)) = self.boss_pos() {
                            for n in 0..8 {
                                let a = n as f32 * core::f32::consts::TAU / 8.0;
                                let _ = self.spawn_with_skin(EK_MARTYR, SKIN_MARTYR, x+a.cos()*2.2, y+a.sin()*2.2);
                            }
                        }
                    }
                    8 => {
                        self.shake = 1.0;
                        if let Some((x, y)) = self.boss_pos() { self.spawn_smoke_cloud(x, y); }
                    }
                    _ => {}
                }
                let support = match (map::level_index(self.wave), phase) {
                    (1, 1) => [(EK_WRAITH, SKIN_HORNET), (EK_HUSK, SKIN_GUNNER)],
                    (1, _) => [(EK_MARTYR, SKIN_MARTYR), (EK_WRAITH, SKIN_HORNET)],
                    (2, 1) => [(EK_HUSK, SKIN_HOUND), (EK_WRAITH, SKIN_SPITTER)],
                    (2, _) => [(EK_BRUTE, SKIN_VATBRUTE), (EK_HUSK, SKIN_HOUND)],
                    (3, 1) => [(EK_WRAITH, SKIN_MARKSMAN), (EK_WRAITH, SKIN_HORNET)],
                    (3, _) => [(EK_HUSK, SKIN_GUNNER), (EK_WRAITH, SKIN_MARKSMAN)],
                    (4, 1) => [(EK_BRUTE, SKIN_HAZMAT), (EK_HUSK, SKIN_GUNNER)],
                    (4, _) => [(EK_BRUTE, SKIN_LOADER), (EK_MARTYR, SKIN_MARTYR)],
                    (5, 1) => [(EK_WRAITH, SKIN_MARKSMAN), (EK_HUSK, SKIN_RIFLEMAN)],
                    (5, _) => [(EK_WRAITH, SKIN_HORNET), (EK_BRUTE, SKIN_GUNNER)],
                    (6, 1) => [(EK_BRUTE, SKIN_HAZMAT), (EK_WRAITH, SKIN_HORNET)],
                    (6, _) => [(EK_BRUTE, SKIN_LOADER), (EK_MARTYR, SKIN_MARTYR)],
                    (7, 1) => [(EK_WRAITH, SKIN_MARKSMAN), (EK_MARTYR, SKIN_MARTYR)],
                    (7, _) => [(EK_WRAITH, SKIN_HORNET), (EK_HUSK, SKIN_GUNNER)],
                    (8, 1) => [(EK_BRUTE, SKIN_LOADER), (EK_BRUTE, SKIN_GUNNER)],
                    (8, _) => [(EK_MARTYR, SKIN_MARTYR), (EK_BRUTE, SKIN_HAZMAT)],
                    (9, 1) => [(EK_HUSK, SKIN_BREACHER), (EK_WRAITH, SKIN_MARKSMAN)],
                    (9, _) => [(EK_HUSK, SKIN_GUNNER), (EK_WRAITH, SKIN_HORNET)],
                    (_, 1) => [(EK_WRAITH, SKIN_HORNET), (EK_HUSK, SKIN_RIFLEMAN)],
                    _ => [(EK_MARTYR, SKIN_MARTYR), (EK_BRUTE, SKIN_LOADER)],
                };
                let boss_spots = map::boss_spots(self.wave);
                for (index, &(kind, skin)) in support.iter().enumerate() {
                    let (x, y) = boss_spots[(phase as usize + index) % boss_spots.len()];
                    if !self.blocked(x.floor() as i32, y.floor() as i32) {
                        let _ = self.spawn_with_skin(kind, skin, x, y);
                    }
                    for n in 0..6 {
                        let a = n as f32 * core::f32::consts::TAU / 6.0;
                        self.spawn_timed(EK_SPARK, x + a.cos() * 0.35, y + a.sin() * 0.35, 0.55, -12.0);
                    }
                }
                // HECATE-9 discharges a fast electrical ring; CHIMERA-9
                // launches a slower corrosive nova before its support closes.
                if map::level_index(self.wave) > 0 {
                    if let Some((bx, by)) = self.ents.iter().find(|e| e.kind == EK_BOSS && e.hp > 0).map(|e| (e.x, e.y)) {
                        let sector = map::level_index(self.wave);
                        let count = [0, 8, 6, 10, 12, 14, 12, 16, 10, 18, 20].get(sector).copied().unwrap_or(6 + (sector % 7) * 2);
                        for n in 0..count {
                            let a = n as f32 * core::f32::consts::TAU / count as f32 + phase as f32 * 0.18;
                            if let Some(i) = self.spawn(EK_PROJ, bx, by) {
                                let speed = if sector == 5 { if n % 2 == 0 { 6.2 } else { 4.3 } } else { [0.0, 6.7, 4.1, 7.5, 3.6, 0.0, 4.2, 7.4, 3.5, 8.0, 5.4].get(sector).copied().unwrap_or(4.2 + (sector % 5) as f32 * 0.4) };
                                self.ents[i].vx = a.cos() * speed;
                                self.ents[i].vy = a.sin() * speed;
                                self.ents[i].timer = 3.2;
                                self.ents[i].hp = if phase == 2 { 18 } else { 12 };
                                self.ents[i].effect_tick = if sector == 2 { 3.0 } else { 0.0 };
                            }
                        }
                    }
                }
            }
        }


        // Door closures, teleports and phase support can alter geometry after AI.
        // Resolve overlap even for stunned enemies before publishing the frame.
        for i in 0..ENT_N {
            let en = self.ents[i];
            if en.hp > 0 && is_hostile_kind(en.kind) && self.circle_blocked(en.x, en.y, en.radius) {
                let (x, y) = self.nearest_open(en.x, en.y, en.radius);
                self.ents[i].x = x; self.ents[i].y = y;
                self.ents[i].vx = 0.0; self.ents[i].vy = 0.0;
            }
        }

        self.tick_boss_arena(dt);

        // Count after boss spawning and phase support, including queued endless arrivals.
        let living = (map::living_hostiles(self) + self.pending_hostiles)
            .saturating_add(usize::from(!self.boss_spawned && self.boss_intro > 0.0))
            .min(i32::MAX as usize) as i32;
        let mut prompt = 0;
        let fx = self.px + self.pa.cos();
        let fy = self.py + self.pa.sin();
        let fc = self.cell(fx.floor() as i32, fy.floor() as i32);
        let secret_closed=fc==9 && self.door[fy.floor() as usize*MAP_W+fx.floor() as usize]<0.05;
        if fc==8 {prompt=1;}
        if prompt == 0 && (self.facing_prop(EK_LAMP, 2.6).is_some() || self.facing_prop(EK_CRATE, 1.8).is_some()) {
            prompt = 1;
        }
        let wpn = self.weapon as usize;
        if self.mag[wpn] <= 0 && self.ammo[wpn] > 0 {
            prompt = 2;
        }
        for e in self.ents.iter() {
            if is_pickup(e.kind) {
                let d = (e.x - self.px).powi(2) + (e.y - self.py).powi(2);
                if d < 2.2 {
                    prompt = if is_weapon_item(e.kind) { 4 } else { 5 };
                    break;
                }
            }
        }
        if !self.node_done && self.near_point(field::node_point(self.wave).0, field::node_point(self.wave).1) {
            prompt = 13;
        }
        for &(x, y) in field::terminals(self.wave) {
            let near = self.near_point(x, y);
            let unread = self.ents.iter().any(|e| {
                e.kind == EK_TERMINAL && e.timer >= 0.0 && (e.x - x).abs() < 0.45 && (e.y - y).abs() < 0.45
            });
            if near && unread {
                prompt = 14;
            }
        }
        if living == 0 && !self.boss_spawned && self.boss_intro <= 0.0 && self.state == 0 {
            prompt = if self.node_done { 6 } else { 15 };
        }
        if living == 0 && self.node_done && self.near_override() && !self.boss_spawned && self.boss_intro <= 0.0 {
            prompt = 3;
        }
        if self.boss_intro > 0.0 {
            let sector = map::level_index(self.wave) as i32;
            prompt = (if sector < 3 { 7 + sector * 2 } else { 17 + (sector - 3) * 2 })
                + if self.boss_intro <= self.boss_intro_duration() * 0.5 { 1 } else { 0 };
        }
        if self.boss_intro <= 0.0 && self.ents.iter().any(|e| field::is_boss_case(e.kind)) {
            prompt = 16;
        }

        if self.boss_intro<=0.0 && prompt!=16 && prompt!=2 {
            if secret_closed {prompt=123;}
            else if let Some(i)=self.facing_prop(EK_CRATE,4.5) {
                if (150..=224).contains(&self.ents[i].skin) {prompt=120+((self.ents[i].skin-150)%3) as i32;}
            }
        }

        // v2 5x5 sheet cells: 0 full, 1 half, 2 low, 3 empty, 4 no
        // magazine, 5 unused, 6-9 pickup, 10-14 reload, 15-19 fire.
        // Cells 20-24 (alt-fire) have no mechanic and stay unused.
        let mag_now = self.mag[wpn];
        let mag_max = MAG_SZ[wpn];
        let weap_frame = if self.reload_t > 0.0 {
            let p = (1.0 - self.reload_t / self.reload_dur.max(0.05)).clamp(0.0, 0.999);
            10 + (p * 5.0) as i32
        } else if self.muzzle > 0.08 {
            if self.weapon == 2 {
                15 + ((self.time * 18.0) as i32).rem_euclid(5)
            } else if self.muzzle > 0.84 {
                15
            } else if self.muzzle > 0.64 {
                16
            } else if self.muzzle > 0.44 {
                17
            } else if self.muzzle > 0.24 {
                18
            } else {
                19
            }
        } else if self.pickup_t > 0.0 {
            6 + (((self.pickup_dur - self.pickup_t) / self.pickup_dur.max(0.01)).clamp(0.0, 0.999) * 4.0) as i32
        } else if mag_now >= mag_max {
            0
        } else if mag_now == 0 && self.ammo[wpn] == 0 {
            4
        } else if mag_now == 0 {
            3
        } else if mag_now * 2 >= mag_max {
            1
        } else {
            2
        };

        let boss_health = self.ents.iter()
            .find(|e| e.kind == EK_BOSS && e.hp > 0)
            .map(|e| e.hp)
            .unwrap_or(0);
        // Self-splash telegraph: these guns burst at the aimed wall and
        // the blast reaches back to the shooter inside these radii.
        let splash = if self.state == 0 {
            let radius = match self.weapon {
                4 => 4.8,
                8 => 3.0,
                9 => 2.6,
                10 => 1.6,
                16 => 5.6,
                _ => 0.0,
            };
            if radius > 0.0
                && self.wall_distance(self.px, self.py, self.pa.cos(), self.pa.sin(), 8.0) < radius
            {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        self.hud = Hud {
            health: self.health,
            armor: self.armor,
            ammo: self.mag[wpn],
            weapon: self.weapon,
            kills: self.kills,
            living,
            state: self.state,
            prompt,
            has_w2: self.owned_flag(0),
            has_w3: self.owned_flag(1),
            secrets: self.secrets,
            elapsed_ms: (self.elapsed * 1000.0) as i32,
            shake: self.shake * self.shake,
            muzzle: self.muzzle,
            hurt: self.hurt,
            bob: self.walk.sin() * (if self.hud.speed > 0.2 { 1.0 } else { 0.12 }),
            kick: self.kick,
            hitmarker: self.hitmarker,
            yaw: self.pa,
            speed: self.hud.speed,
            x: self.px,
            y: self.py,
            reserve: self.ammo[wpn],
            reloading: if self.reload_t > 0.0 {
                1.0 - self.reload_t / self.reload_dur.max(0.05)
            } else {
                0.0
            },
            weap_frame,
            has_w4: self.owned_flag(2),
            has_w5: self.owned_flag(3),
            events: self.events,
            ev_weapon: self.ev_weapon,
            wave: self.wave,
            boss_health,
            boss_max_health: if boss_health > 0 { self.boss_max_health() } else { 0 },
            boss_phase: if boss_health > 0 { self.boss_phase as i32 } else { 0 },
            has_w6: self.owned_flag(4),
            has_w7: self.owned_flag(5),
            has_w8: self.owned_flag(6),
            objective: if self.node_done { 1 } else { 0 },
            radio_seq: self.radio_seq,
            radio_line: self.radio_line,
            vuln: self.boss_vuln,
            node_x: field::node_point(self.wave).0,
            node_y: field::node_point(self.wave).1,
            has_w9: self.owned_flag(7),
            has_w10: self.owned_flag(8),
            has_w11: self.owned_flag(9),
            has_w12: self.owned_flag(10),
            has_w13: self.owned_flag(11),
            has_w14: self.owned_flag(12),
            has_w15: self.owned_flag(13),
            has_w16: self.owned_flag(14),
            has_w17: self.owned_flag(15),
            has_w18: self.owned_flag(16),
            has_w19: self.owned_flag(17),
            extra_weapons: self.extra_weapons,
            hurt_dir: self.hurt_dir,
            strain: self.strain,
            power: self.power as i32,
            power_t: self.power_t,
            splash,
        };
    }

    fn add_light(&self, grid: &mut [[f32; 3]], x: f32, y: f32, radius: f32, rgb: [f32; 3]) {
        let x0 = (x - radius).floor().max(0.0) as usize;
        let y0 = (y - radius).floor().max(0.0) as usize;
        let x1 = ((x + radius).ceil() as usize).min(MAP_W - 1);
        let y1 = ((y + radius).ceil() as usize).min(MAP_H - 1);
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let wx = cx as f32 + 0.5;
                let wy = cy as f32 + 0.5;
                let d2 = (wx - x).powi(2) + (wy - y).powi(2);
                if d2 >= radius * radius || self.blocked(cx as i32, cy as i32)
                    || !self.los(x, y, wx, wy) { continue; }
                let falloff = (1.0 - d2 / (radius * radius)).powi(2) / (1.0 + d2 * 0.08);
                for channel in 0..3 { grid[cy * MAP_W + cx][channel] += rgb[channel] * falloff; }
            }
        }
    }

    fn update_lighting(&mut self) {
        if self.light_dirty {
            let mut grid = vec![[0.0; 3]; MAP_CELLS];
            for y in 1..MAP_H - 1 {
                for x in 1..MAP_W - 1 {
                    let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter()
                        .filter(|&&(dx, dy)| self.blocked(x as i32 + dx, y as i32 + dy)).count();
                    grid[y * MAP_W + x] = [-0.045 * neighbors as f32; 3];
                }
            }
            for e in &self.ents {
                if e.kind == EK_LAMP && e.timer >= 0.0 {
                    self.add_light(&mut grid, e.x, e.y, 8.2, [0.78, 0.40, 0.14]);
                } else if e.kind == EK_FLAME {
                    self.add_light(&mut grid, e.x, e.y, 5.4, [0.62, 0.18, 0.04]);
                } else if matches!(e.kind, EK_TERMINAL | EK_NODE | EK_OVERRIDE_CONSOLE) {
                    self.add_light(&mut grid, e.x, e.y, 6.4, [0.16, 0.42, 0.62]);
                } else if e.kind == EK_PROP_WLIGHT_C {
                    self.add_light(&mut grid, e.x, e.y, 6.0, [0.20, 0.70, 0.80]);
                } else if e.kind == EK_PROP_WLIGHT_W {
                    self.add_light(&mut grid, e.x, e.y, 6.0, [0.70, 0.70, 0.62]);
                } else if e.kind == EK_PROP_BEACON {
                    self.add_light(&mut grid, e.x, e.y, 5.0, [0.80, 0.36, 0.10]);
                } else if matches!(e.kind, EK_CRATE | EK_BARREL) {
                    let i = (e.y.floor() as usize).min(MAP_H - 1) * MAP_W
                        + (e.x.floor() as usize).min(MAP_W - 1);
                    for c in &mut grid[i] { *c -= 0.10; }
                }
            }
            for y in 1..MAP_H - 1 {
                for x in 1..MAP_W - 1 {
                    if self.cell(x as i32, y as i32) != 6 || (x + y) % 3 != 0 { continue; }
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        if !self.blocked(x as i32 + dx, y as i32 + dy) {
                            self.add_light(&mut grid, x as f32 + 0.5 + dx as f32 * 0.6,
                                y as f32 + 0.5 + dy as f32 * 0.6, 6.4, [0.10, 0.42, 0.62]);
                        }
                    }
                }
            }
            self.static_light = grid;
            self.light_dirty = false;
        }
        let mut grid = std::mem::take(&mut self.light_grid);
        grid.copy_from_slice(&self.static_light);
        // Emitters are ranked by distance to the player before the budget is
        // applied. Walking the entity array in index order made which
        // projectiles lit the room an allocation accident.
        let mut emitters: Vec<(f32, usize)> = self.ents.iter().enumerate()
            .filter(|(_, e)| matches!(e.kind, EK_PROJ | EK_RAY | EK_BOLT | EK_FIREPATCH | EK_IMPACT | EK_BOSS))
            .map(|(i, e)| ((e.x - self.px).powi(2) + (e.y - self.py).powi(2), i))
            .filter(|&(d2, _)| d2 <= DYNAMIC_LIGHT_RANGE * DYNAMIC_LIGHT_RANGE)
            .collect();
        emitters.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        emitters.truncate(DYNAMIC_LIGHT_BUDGET);
        let count = emitters.len();
        for &(_, i) in &emitters {
            let e = &self.ents[i];
            let rgb = if e.kind == EK_BOSS && (23..=36).contains(&e.skin) {
                campaign::LIGHTS[(e.skin - 23) as usize]
            } else if e.kind == EK_BOSS {
                match map::level_index(self.wave) {
                    1 => [0.55, 0.22, 0.05],
                    2 => [0.12, 0.48, 0.16],
                    3 => [0.05, 0.44, 0.62],
                    4 => [0.62, 0.28, 0.04],
                    _ => [0.55, 0.28, 0.08],
                }
            } else if e.kind == EK_PROJ && (100..=136).contains(&e.skin) {
                let source = e.skin - 100;
                if source >= 23 { campaign::LIGHTS[(source - 23) as usize] }
                else { match source { 10 | 14 => [0.14,0.62,0.08], 18 => [0.12,0.48,0.72], 22 => [0.40,0.10,0.65], 12 | 17 => [0.72,0.32,0.04], 20 => [0.72,0.22,0.04], _ => [0.08,0.55,0.72] } }
            } else if e.kind == EK_PROJ && (230..=234).contains(&e.skin) {
                [[0.72,0.22,0.04],[0.08,0.55,0.72],[0.12,0.48,0.72],[0.14,0.62,0.08],[0.40,0.10,0.65]][(e.skin - 230) as usize]
            } else if e.kind == EK_PROJ && (e.effect_tick == 3.0 || e.effect_tick == 4.0) {
                [0.12, 0.55, 0.08]
            } else if matches!(e.kind, EK_RAY | EK_PROJ) {
                [0.08, 0.42, 0.72]
            } else {
                [0.42, 0.12, 0.03]
            };
            let radius = if e.kind == EK_BOSS { 9.5 } else { 5.2 };
            self.add_light(&mut grid, e.x, e.y, radius, rgb);
        }
        debug_assert!(count <= DYNAMIC_LIGHT_BUDGET);
        if self.boss_intro>0.0 && !self.boss_spawned {
            let (x,y)=map::boss_spots(self.wave)[0];
            let profile=boss_arena::PROFILES[map::level_index(self.wave)];
            let progress=1.0-self.boss_intro/profile.intro;
            let intensity=0.2+progress*0.9+(progress*40.0).sin().abs()*0.18;
            self.add_light(&mut grid,x,y,7.0+progress*4.0,profile.color.map(|v|v*intensity));
        }
        if self.muzzle > 0.05 {
            let power = self.muzzle * if self.weapon == 0 { 0.28 } else { 0.72 };
            self.add_light(&mut grid, self.px, self.py, 5.2, [power, power * 0.55, power * 0.18]);
        }
        if self.boss_arena.armed && self.boss_arena.stage != 0 {
            let color=boss_arena::PROFILES[map::level_index(self.wave)].color;
            let intensity=if self.boss_arena.stage==4 {0.44}else{0.10};
            for (i,cell) in grid.iter_mut().enumerate() {
                if self.boss_arena.mask[i]!=0 {for channel in 0..3 {cell[channel]+=color[channel]*intensity;}}
            }
        }
        self.light_grid = grid;
        self.bounce_light();
    }


    /// Two passes at decreasing weight. A single pass stopped light dead at
    /// wall corners; the second lets it wrap one cell further, which is what
    /// makes a lit doorway read as connected to the room beyond.
    /// Two diffusion passes at decreasing weight, ping-ponging between the two
    /// scratch buffers so no pass can cascade within itself. One pass left
    /// light dead at wall corners; the second lets it wrap a cell further,
    /// which is what makes a lit doorway read as connected to the room beyond.
    /// Diffuses light from `src` into `dst` by a fraction of each open cell's
    /// mean neighbour brightness. `src` and `dst` must be distinct buffers so a
    /// pass cannot cascade within itself.
    fn bounce_pass(&self, src: &[[f32; 3]], dst: &mut [[f32; 3]], weight: f32) {
        for y in 1..MAP_H - 1 {
            for x in 1..MAP_W - 1 {
                if self.blocked(x as i32, y as i32) { continue; }
                let i = y * MAP_W + x;
                let mut acc = [0.0f32; 3];
                let mut n = 0.0f32;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if self.blocked(nx, ny) { continue; }
                    let j = ny as usize * MAP_W + nx as usize;
                    for c in 0..3 { acc[c] += src[j][c].max(0.0); }
                    n += 1.0;
                }
                if n <= 0.0 { continue; }
                for c in 0..3 { dst[i][c] += acc[c] / n * weight; }
            }
        }
    }

    /// Two passes at decreasing weight. A single pass left light dead at wall
    /// corners; the second lets it wrap a cell further, which is what makes a
    /// lit doorway read as connected to the room beyond.
    fn bounce_light(&mut self) {
        let mut grid = core::mem::take(&mut self.light_grid);
        let mut scratch = core::mem::take(&mut self.light_src);
        scratch.copy_from_slice(&grid);
        self.bounce_pass(&scratch, &mut grid, 0.22);
        self.bounce_pass(&grid, &mut scratch, 0.10);
        for (cell, extra) in grid.iter_mut().zip(scratch.iter()) {
            for c in 0..3 { cell[c] += extra[c]; }
        }
        self.light_grid = grid;
        self.light_src = scratch;
    }

    fn light_at(&self, x: f32, y: f32) -> [f32; 3] {
        let x = (x - 0.5).clamp(0.0, (MAP_W - 2) as f32);
        let y = (y - 0.5).clamp(0.0, (MAP_H - 2) as f32);
        let ix = x as usize;
        let iy = y as usize;
        let fx = x.fract();
        let fy = y.fract();
        let i = iy * MAP_W + ix;
        let mut out = [0.0; 3];
        for c in 0..3 {
            let a = self.light_grid[i][c] * (1.0 - fx) + self.light_grid[i + 1][c] * fx;
            let b = self.light_grid[i + MAP_W][c] * (1.0 - fx) + self.light_grid[i + MAP_W + 1][c] * fx;
            out[c] = a * (1.0 - fy) + b * fy;
        }
        out
    }

    fn shade_rgb(color: u32, ambient: f32, light: [f32; 3]) -> u32 {
        let mut out = color & 0xff000000;
        for channel in 0..3 {
            let value = ((color >> (channel * 8)) & 255) as f32;
            let lit = (value * (ambient + light[channel]).clamp(0.18, 1.8)).min(255.0) as u32;
            out |= lit << (channel * 8);
        }
        out
    }

    /// Four-wide copy of [`shade_rgb`]. Scale is constant across a wall column.
    #[inline(always)]
    fn shade_rgb4(colors: [u32; 4], ambient: f32, light: [f32; 3]) -> [u32; 4] {
        #[cfg(target_feature = "simd128")]
        {
            use core::arch::wasm32::*;
            let scale = [
                (ambient + light[0]).clamp(0.18, 1.8),
                (ambient + light[1]).clamp(0.18, 1.8),
                (ambient + light[2]).clamp(0.18, 1.8),
            ];
            let color = u32x4(colors[0], colors[1], colors[2], colors[3]);
            let mask = u32x4_splat(255);
            let cap = f32x4_splat(255.0);
            let mut out = v128_and(color, u32x4_splat(0xff00_0000));
            for channel in 0..3 {
                let chan = v128_and(u32x4_shr(color, channel * 8), mask);
                let lit = f32x4_min(f32x4_mul(f32x4_convert_i32x4(chan), f32x4_splat(scale[channel as usize])), cap);
                let bits = i32x4_trunc_sat_f32x4(lit);
                out = v128_or(out, u32x4_shl(bits, channel * 8));
            }
            return [
                u32x4_extract_lane::<0>(out),
                u32x4_extract_lane::<1>(out),
                u32x4_extract_lane::<2>(out),
                u32x4_extract_lane::<3>(out),
            ];
        }
        #[cfg(not(target_feature = "simd128"))]
        [
            Self::shade_rgb(colors[0], ambient, light),
            Self::shade_rgb(colors[1], ambient, light),
            Self::shade_rgb(colors[2], ambient, light),
            Self::shade_rgb(colors[3], ambient, light),
        ]
    }

    #[cfg_attr(target_feature = "simd128", allow(dead_code))]
    fn shade_texel(color: u32, gain: [i32; 3], alpha: u32) -> u32 {
        let r = (((color & 255) * gain[0] as u32) >> 16).min(255);
        let g = ((((color >> 8) & 255) * gain[1] as u32) >> 16).min(255);
        let b = ((((color >> 16) & 255) * gain[2] as u32) >> 16).min(255);
        r | (g << 8) | (b << 16) | alpha
    }

    /// Floor/ceiling shade of four texels. Pixel-identical to [`shade_texel`];
    /// the wasm build widens the multiplies with simd128.
    #[inline(always)]
    fn shade_texels4(colors: [u32; 4], gain: [[i32; 3]; 4], alpha: u32) -> [u32; 4] {
        #[cfg(target_feature = "simd128")]
        {
            use core::arch::wasm32::*;
            let color = u32x4(colors[0], colors[1], colors[2], colors[3]);
            let mask = u32x4_splat(255);
            let shade_ch = |shift: u32, g0: i32, g1: i32, g2: i32, g3: i32| {
                let chan = v128_and(u32x4_shr(color, shift), mask);
                let g = i32x4(g0, g1, g2, g3);
                u32x4_min(u32x4_shr(u32x4_mul(chan, g), 16), mask)
            };
            let r = shade_ch(0, gain[0][0], gain[1][0], gain[2][0], gain[3][0]);
            let g = u32x4_shl(shade_ch(8, gain[0][1], gain[1][1], gain[2][1], gain[3][1]), 8);
            let b = u32x4_shl(shade_ch(16, gain[0][2], gain[1][2], gain[2][2], gain[3][2]), 16);
            let out = v128_or(v128_or(v128_or(r, g), b), u32x4_splat(alpha));
            return [
                u32x4_extract_lane::<0>(out),
                u32x4_extract_lane::<1>(out),
                u32x4_extract_lane::<2>(out),
                u32x4_extract_lane::<3>(out),
            ];
        }
        #[cfg(not(target_feature = "simd128"))]
        [
            Self::shade_texel(colors[0], gain[0], alpha),
            Self::shade_texel(colors[1], gain[1], alpha),
            Self::shade_texel(colors[2], gain[2], alpha),
            Self::shade_texel(colors[3], gain[3], alpha),
        ]
    }

    /// Texel footprint of one wall column. Near columns stay at LOD 0, so they
    /// match an unfiltered sample; distant columns use the floor mip chain.
    fn column_lod(perp: f32, line_h: i32, plane_len: f32, w: usize) -> (usize, u32) {
        let vertical = TEX as f32 / line_h.max(1) as f32;
        let horizontal = perp * 2.0 * plane_len / w.max(1) as f32 * TEX as f32;
        let lod = horizontal.max(vertical).max(1.0).log2().clamp(0.0, 8.0);
        (lod as usize, (lod.fract() * 256.0) as u32)
    }

    fn plane_sample(&self, floor: bool, fx: f32, fy: f32, level: usize) -> (usize, i32, i32, usize) {
        let tx = (fx * TEX as f32).floor() as i32;
        let ty = (fy * TEX as f32).floor() as i32;
        let id=if self.hell { T_BOSS_ARENA + map::level_index(self.wave) }
            else { T_SECTOR_SURFACE + map::level_index(self.wave) * 5 + if floor {3} else {4} };
        (id,tx,ty,level)
    }

    fn sigil_uv(&self, fx: f32, fy: f32) -> Option<(f32, f32, usize)> {
        let mx = fx.floor() as i32;
        let my = fy.floor() as i32;
        if mx < 0 || my < 0 || mx >= MAP_W as i32 || my >= MAP_H as i32 { return None; }
        if self.floor[my as usize * MAP_W + mx as usize] != 2 { return None; }
        let level = map::level_index(self.wave);
        let (cx, cy) = map::override_point(self.wave);
        let u = (fx - cx) / 1.55;
        let v = (fy - cy) / 1.55;
        if u.abs() > 1.15 || v.abs() > 1.15 { return None; }
        Some((u, v, level))
    }

    fn blend_sigil(&self, base: u32, fx: f32, fy: f32) -> u32 {
        let (x,y)=(fx.floor() as i32,fy.floor() as i32);
        if x>=0 && y>=0 && x<MAP_W as i32 && y<MAP_H as i32 {
            let style=self.floor[y as usize*MAP_W+x as usize];
            if (5..=7).contains(&style) {
                let (u,v)=(fx.fract(),fy.fract());
                let mark=match style {5=>((u-v*0.25-0.25).abs()<0.018)||((u-v*0.25-0.61).abs()<0.018),6=>(u-0.35).abs()<0.025||(u-0.65).abs()<0.025,_=>(v*9.0).fract()<0.14&&u>0.2&&u<0.8};
                if mark {return Self::blend(base,Self::pack(118,142,145,255),0.42);}
                return base;
            }
        }
        let Some((u, v, level)) = self.sigil_uv(fx, fy) else { return base; };
        let mark = sigil_rgba(level, u, v);
        if mark[3] == 0 { return base; }
        let a = mark[3];
        let inv = 255 - a;
        let ch = |shift: u32, src: u32| {
            (((base >> shift) & 255) * inv + src * a) / 255
        };
        ch(0, mark[0]) | (ch(8, mark[1]) << 8) | (ch(16, mark[2]) << 16) | (base & 0xFF00_0000)
    }

    fn plane_texel(&self, floor: bool, fx: f32, fy: f32, level: usize, mix: u32) -> u32 {
        let (id, u, v, lvl) = self.plane_sample(floor, fx, fy, level);
        let base = self.sample_mip(id, u, v, lvl, mix);
        if floor && self.boss_arena.armed {
            let x=fx.floor() as i32; let y=fy.floor() as i32;
            if x>=0 && y>=0 && x<MAP_W as i32 && y<MAP_H as i32 {
                let style=self.floor[y as usize*MAP_W+x as usize];
                if style==3 || style==4 {
                    let p=boss_arena::PROFILES[map::level_index(self.wave)];
                    let color=Self::pack((p.color[0]*255.0) as u32,(p.color[1]*255.0) as u32,(p.color[2]*255.0) as u32,255);
                    let line=((fx.fract()-0.5).abs()>0.39 || (fy.fract()-0.5).abs()>0.39) as u8 as f32;
                    let amount=if style==4 {0.32+line*0.38} else {line*0.5};
                    return Self::blend(base,color,amount);
                }
            }
            self.blend_sigil(base,fx,fy)
        } else if floor { self.blend_sigil(base,fx,fy) } else {base}
    }


    fn render_planes(&mut self, dx: f32, dy: f32, plane_x: f32, plane_y: f32,
        horizon: f32, walls: &[[i32; 2]]) {
        let w = self.w;
        let h = self.h;
        let plane_len = (plane_x * plane_x + plane_y * plane_y).sqrt();
        for y in 0..h {
            let p = y as f32 - horizon;
            if p.abs() < 0.5 {
                for x in 0..w {
                    if (y as i32) < walls[x][0] || (y as i32) > walls[x][1] {
                        self.fb[y * w + x] = Self::fog(Self::pack(18, 11, 10, 255), 28.0);
                    }
                }
                continue;
            }
            let floor = p > 0.0;
            let distance = (0.5 * h as f32) / p.abs();
            let step_x = 2.0 * plane_x * distance / w as f32;
            let step_y = 2.0 * plane_y * distance / w as f32;
            let start_x = self.px + (dx - plane_x) * distance + step_x * 0.5;
            let start_y = self.py + (dy - plane_y) * distance + step_y * 0.5;
            let footprint = (distance * 2.0 * plane_len / w as f32)
                .max(distance / p.abs()) * TEX as f32;
            let lod = footprint.max(1.0).log2().clamp(0.0, 8.0);
            let level = lod as usize;
            let mix = (lod.fract() * 256.0) as u32;
            let shade = if floor { 0.72 + 0.2 / (1.0 + distance * 0.2)
                + self.muzzle * 0.45 / (1.0 + distance * distance) } else { 0.78 };
            let alpha = Self::fog(0, distance);
            let mut gain = [0i32; 3];
            let mut gain_step = [0i32; 3];
            let mut x = 0;
            while x < w {
                let n = (w - x).min(4);
                let mut colors = [0u32; 4];
                let mut gains = [[0i32; 3]; 4];
                let mut write = [false; 4];
                let mut uv = [(0i32, 0i32); 4];
                let mut tid = 0usize;
                let mut sample_level = level;
                let mut uniform = true;
                let mut seen = false;
                for i in 0..n {
                    let px = x + i;
                    if px % 8 == 0 {
                        let light = self.light_at(start_x + step_x * px as f32, start_y + step_y * px as f32);
                        let next = self.light_at(start_x + step_x * (px + 8) as f32, start_y + step_y * (px + 8) as f32);
                        for c in 0..3 {
                            gain[c] = ((shade + light[c]).clamp(0.18, 1.8) * 65536.0) as i32;
                            let target = ((shade + next[c]).clamp(0.18, 1.8) * 65536.0) as i32;
                            gain_step[c] = (target - gain[c]) / 8;
                        }
                    } else {
                        for c in 0..3 { gain[c] += gain_step[c]; }
                    }
                    gains[i] = gain;
                    if (y as i32) >= walls[px][0] && (y as i32) <= walls[px][1] {
                        continue;
                    }
                    write[i] = true;
                    let fx = start_x + step_x * px as f32;
                    let fy = start_y + step_y * px as f32;
                    let (id, u, v, lvl) = self.plane_sample(floor, fx, fy, level);
                    uv[i] = (u, v);
                    if !seen {
                        tid = id;
                        sample_level = lvl;
                        seen = true;
                    } else if id != tid || lvl != sample_level {
                        uniform = false;
                    }
                }
                if seen {
                    if uniform {
                        colors = self.sample_mip4(tid, uv, sample_level, mix);
                    } else {
                        for i in 0..n {
                            if !write[i] { continue; }
                            let fx = start_x + step_x * (x + i) as f32;
                            let fy = start_y + step_y * (x + i) as f32;
                            colors[i] = self.plane_texel(floor, fx, fy, level, mix);
                        }
                    }
                    let shaded = Self::shade_texels4(colors, gains, alpha);
                    let row = y * w;
                    for i in 0..n {
                        if write[i] { self.fb[row + x + i] = shaded[i]; }
                    }
                }
                x += n;
            }
        }
    }

    fn prepare_gpu(&mut self) {
        self.update_lighting();
        self.rebuild_smoke_grid();
        let w = self.w.min(MAX_W);
        let h = self.h;
        let dir_x = self.pa.cos();
        let dir_y = self.pa.sin();
        let aspect = w as f32 / h.max(1) as f32;
        let plane_len = 0.72 * (aspect / 1.6);
        let plane_x = -dir_y * plane_len;
        let plane_y = dir_x * plane_len;
        let horizon = h as f32 * 0.5 + self.pitch * h as f32 * 0.9;
        let tnow = self.time;
        let scratch = gpu_scratch();
        scratch.view = GpuView {
            px: self.px,
            py: self.py,
            dir_x,
            dir_y,
            plane_x,
            plane_y,
            horizon,
            time: tnow,
            hell: if self.hell { 1.0 } else { 0.0 },
            muzzle: self.muzzle,
            w: w as f32,
            h: h as f32,
            plane_len,
            sprite_n: 0.0,
            _p1: map::level_index(self.wave) as f32,
            _p2: if self.ents.iter().any(|e| e.kind == EK_BOSS && e.hp > 0) { 1.0 } else { 0.0 },
        };
        for x in 0..w {
            let cam = 2.0 * (x as f32 + 0.5) / w as f32 - 1.0;
            let mut rdx = dir_x + plane_x * cam;
            let mut rdy = dir_y + plane_y * cam;
            if rdx.abs() < 1e-6 { rdx = 1e-6; }
            if rdy.abs() < 1e-6 { rdy = 1e-6; }
            let mut map_x = self.px.floor() as i32;
            let mut map_y = self.py.floor() as i32;
            let ddx = (1.0 / rdx).abs();
            let ddy = (1.0 / rdy).abs();
            let step_x = if rdx < 0.0 { -1 } else { 1 };
            let step_y = if rdy < 0.0 { -1 } else { 1 };
            let mut sdx = if rdx < 0.0 { (self.px - map_x as f32) * ddx } else { (map_x as f32 + 1.0 - self.px) * ddx };
            let mut sdy = if rdy < 0.0 { (self.py - map_y as f32) * ddy } else { (map_y as f32 + 1.0 - self.py) * ddy };
            let mut side = 0;
            let mut hit = 0u8;
            for _ in 0..(MAP_W + MAP_H) {
                if sdx < sdy {
                    sdx += ddx;
                    map_x += step_x;
                    side = 0;
                } else {
                    sdy += ddy;
                    map_y += step_y;
                    side = 1;
                }
                let c = self.cell(map_x, map_y);
                if c != 0 && c != 10 {
                    let open = if (c == 8 || c == 9) && map_x >= 0 && map_y >= 0 && (map_x as usize) < MAP_W && (map_y as usize) < MAP_H {
                        self.door[map_y as usize * MAP_W + map_x as usize]
                    } else { 0.0 };
                    if open >= 0.98 { continue; }
                    hit = c;
                    break;
                }
            }
            let perp = if side == 0 {
                (map_x as f32 - self.px + (1 - step_x) as f32 / 2.0) / rdx
            } else {
                (map_y as f32 - self.py + (1 - step_y) as f32 / 2.0) / rdy
            }.abs().max(0.05);
            let z = if hit == 0 { 40.0 } else { perp };
            let line_h = (h as f32 / perp) as i32;
            let ds_full = -line_h / 2 + horizon as i32;
            let mut draw0 = ds_full;
            let mut draw1 = line_h / 2 + horizon as i32;
            if draw0 < 0 { draw0 = 0; }
            if draw1 >= h as i32 { draw1 = h as i32 - 1; }
            if hit == 0 { draw0 = h as i32; draw1 = -1; }
            let mut wall_x = if side == 0 { self.py + perp * rdy } else { self.px + perp * rdx };
            wall_x -= wall_x.floor();
            let mut tex_x = (wall_x * TEX as f32) as i32;
            if side == 0 && rdx > 0.0 { tex_x = TEX as i32 - tex_x - 1; }
            if side == 1 && rdy < 0.0 { tex_x = TEX as i32 - tex_x - 1; }
            let open = if (hit == 8 || hit == 9) && map_x >= 0 && map_y >= 0 && (map_x as usize) < MAP_W && (map_y as usize) < MAP_H {
                self.door[map_y as usize * MAP_W + map_x as usize]
            } else { 0.0 };
            tex_x = (tex_x + (open * TEX as f32) as i32) & TEXM;
            let mut tid = if hit == 0 { 0 } else { self.wall_tex(hit, map_x, map_y) };
            let hash = (map_x.wrapping_mul(19) + map_y.wrapping_mul(7)) as u32;
            if tid == T_METAL && hash.is_multiple_of(7) { tid = T_HAZARD; }
            if tid == T_BRICK && hash.is_multiple_of(5) { tid = T_SKULL; }
            let light = self.light_at(self.px + (perp - 0.03) * rdx, self.py + (perp - 0.03) * rdy);
            let dec = if map_x >= 0 && map_y >= 0 && (map_x as usize) < MAP_W && (map_y as usize) < MAP_H {
                self.decal[map_y as usize * MAP_W + map_x as usize]
            } else { 0 };
            scratch.cols[x] = GpuCol {
                perp,
                tex_x: tex_x as f32,
                light_r: light[0],
                light_g: light[1],
                light_b: light[2],
                draw0: draw0 as f32,
                draw1: draw1 as f32,
                line_h: line_h.max(1) as f32,
                ds_full: ds_full as f32,
                tex: tid as f32,
                side: side as f32,
                dec: dec as f32,
                hash: hash as f32,
                hit: hit as f32,
                z,
                _pad: 0.0,
            };
        }
        let mut n = 0usize;
        for e in &self.ents {
            if e.kind == 0 || n >= ENT_N { continue; }
            let (tex, scale, sheet4, frame_i) = sprite_style(e);
            let frame = if sheet4 { frame_i as f32 } else { -1.0 };
            scratch.sprites[n] = GpuSprite {
                x: e.x,
                y: e.y,
                zoff: e.zoff,
                scale,
                tex: tex as f32,
                frame,
                flash: if e.kind == EK_LAMP && e.timer < 0.0 { -1.0 } else if e.flash > 0.0 { 1.0 } else if is_hostile_kind(e.kind) && e.effect_tick > 0.0 { 2.0 } else { 0.0 },
                kind: e.kind as f32,
            };
            n += 1;
        }
        // Sprites write RGB and packed depth into the shared world target.
        // Paint far to near, matching the CPU path; transparent holes keep
        // the already drawn scenery instead of overwriting closer entities.
        scratch.sprites[..n].sort_by(|a, b| {
            let depth = |s: &GpuSprite| (s.x - self.px) * dir_x + (s.y - self.py) * dir_y;
            depth(b).total_cmp(&depth(a))
        });
        scratch.sprite_n = n;
        scratch.view.sprite_n = n as f32;
    }

    fn render(&mut self) {
        self.update_lighting();
        self.rebuild_smoke_grid();
        let w = self.w;
        let h = self.h;
        let dir_x = self.pa.cos();
        let dir_y = self.pa.sin();
        let aspect = w as f32 / h as f32;
        let plane_len = 0.72 * (aspect / 1.6);
        let plane_x = -dir_y * plane_len;
        let plane_y = dir_x * plane_len;
        let horizon = h as f32 * 0.5 + self.pitch * h as f32 * 0.9;
        let shx = 0i32;
        let shy = 0i32;
        let tnow = self.time;

        let mut wall_spans = [[0i32; 2]; MAX_W];

        for x in 0..w {
            let cam = 2.0 * (x as f32 + 0.5) / w as f32 - 1.0;
            let mut rdx = dir_x + plane_x * cam;
            let mut rdy = dir_y + plane_y * cam;
            if rdx.abs() < 1e-6 {
                rdx = 1e-6;
            }
            if rdy.abs() < 1e-6 {
                rdy = 1e-6;
            }
            let mut map_x = self.px.floor() as i32;
            let mut map_y = self.py.floor() as i32;
            let ddx = (1.0 / rdx).abs();
            let ddy = (1.0 / rdy).abs();
            let step_x = if rdx < 0.0 { -1 } else { 1 };
            let step_y = if rdy < 0.0 { -1 } else { 1 };
            let mut sdx = if rdx < 0.0 {
                (self.px - map_x as f32) * ddx
            } else {
                (map_x as f32 + 1.0 - self.px) * ddx
            };
            let mut sdy = if rdy < 0.0 {
                (self.py - map_y as f32) * ddy
            } else {
                (map_y as f32 + 1.0 - self.py) * ddy
            };
            let mut side = 0;
            let mut hit = 0u8;
            for _ in 0..(MAP_W + MAP_H) {
                if sdx < sdy {
                    sdx += ddx;
                    map_x += step_x;
                    side = 0;
                } else {
                    sdy += ddy;
                    map_y += step_y;
                    side = 1;
                }
                let c = self.cell(map_x, map_y);
                if c != 0 && c != 10 {
                    let open = if c == 8 || c == 9 {
                        self.door[map_y as usize * MAP_W + map_x as usize]
                    } else {
                        0.0
                    };
                    if open >= 0.98 {
                        continue;
                    }
                    hit = c;
                    break;
                }
            }
            let perp = if side == 0 {
                (map_x as f32 - self.px + (1 - step_x) as f32 / 2.0) / rdx
            } else {
                (map_y as f32 - self.py + (1 - step_y) as f32 / 2.0) / rdy
            }
            .abs()
            .max(0.05);
            self.zbuf[x] = if hit == 0 { 40.0 } else { perp };
            let line_h = (h as f32 / perp) as i32;
            let mut draw0 = -line_h / 2 + horizon as i32;
            let mut draw1 = line_h / 2 + horizon as i32;
            let ds_full = draw0;
            if draw0 < 0 {
                draw0 = 0;
            }
            if draw1 >= h as i32 {
                draw1 = h as i32 - 1;
            }
            wall_spans[x] = if hit == 0 { [h as i32, -1] } else { [draw0, draw1] };

            let mut wall_x = if side == 0 {
                self.py + perp * rdy
            } else {
                self.px + perp * rdx
            };
            wall_x -= wall_x.floor();
            let mut tex_x = (wall_x * TEX as f32) as i32;
            if side == 0 && rdx > 0.0 {
                tex_x = TEX as i32 - tex_x - 1;
            }
            if side == 1 && rdy < 0.0 {
                tex_x = TEX as i32 - tex_x - 1;
            }
            let open = if hit == 8 || hit == 9 {
                self.door[map_y as usize * MAP_W + map_x as usize]
            } else {
                0.0
            };
            tex_x = (tex_x + (open * TEX as f32) as i32) & TEXM;
            let mut tid = self.wall_tex(hit, map_x, map_y);
            let hash = (map_x.wrapping_mul(19) + map_y.wrapping_mul(7)) as u32;
            if tid == T_METAL && hash.is_multiple_of(7) {
                tid = T_HAZARD;
            }
            if tid == T_BRICK && hash.is_multiple_of(5) {
                tid = T_SKULL;
            }
            let step = TEX as f32 / line_h.max(1) as f32;
            let (level, mix) = Self::column_lod(perp, line_h, plane_len, w);
            let scroll = if tid == T_FLESH {
                ((tnow * 7.0).sin() * 4.0) as i32
            } else if tid == T_TECH {
                (tnow * 26.0) as i32
            } else if tid == T_PIPES {
                (tnow * 10.0) as i32
            } else if tid == T_SKULL {
                ((tnow * 3.5).sin() * 3.0) as i32
            } else if tid == T_BRICK {
                let drip = Self::nbit(3, tex_x, 0);
                if drip > 0.84 { (tnow * (10.0 + drip * 18.0)) as i32 } else { 0 }
            } else {
                0
            };
            let mut tex_pos = (draw0 - ds_full) as f32 * step;
            let side_mul = if side == 1 { 0.65 } else { 1.0 };
            let mut dist_mul = (1.0 / (1.0 + perp * 0.12)) * side_mul;
            if tid == T_FLESH {
                dist_mul *= 0.82 + 0.22 * (tnow * 3.4 + map_x as f32).sin().abs();
            }
            if tid == T_TECH {
                dist_mul *= 0.9 + 0.35 * ((tnow * 11.0 + map_y as f32).sin().abs());
            }
            if tid == T_HAZARD || tid == T_METAL {
                let flick = (tnow * 17.0 + map_x as f32 * 2.1).sin().abs();
                if (hash & 1) == 0 {
                    dist_mul *= 0.85 + 0.4 * flick;
                }
            }
            let light = self.light_at(self.px + (perp - 0.03) * rdx, self.py + (perp - 0.03) * rdy);
            dist_mul += self.muzzle * 0.55 / (1.0 + perp * perp * 0.3);
            let dec = if map_x >= 0 && map_y >= 0 && map_x < MAP_W as i32 && map_y < MAP_H as i32 {
                self.decal[map_y as usize * MAP_W + map_x as usize]
            } else {
                0
            };

            if hit != 0 && tex_x >= 0 && tex_x < TEX as i32 {
                let mut y = draw0;
                let end = draw1 + 1;
                while y < end {
                    let n = (end - y).min(4) as usize;
                    let mut tex_y = [0i32; 4];
                    let mut uv = [(tex_x, 0); 4];
                    for i in 0..n {
                        let ty = (tex_pos as i32).wrapping_add(scroll) & TEXM;
                        tex_pos += step;
                        tex_y[i] = ty;
                        uv[i] = (tex_x, ty);
                    }
                    if n < 4 {
                        let pad = uv[0];
                        uv[n..4].fill(pad);
                    }
                    let mut colors = self.sample_mip4(tid, uv, level, mix);
                    for i in 0..n {
                        let ty = tex_y[i];
                        if tid == T_TECH && (ty & 7) == 0 {
                            colors[i] = Self::shade(colors[i], 1.35);
                        }
                        if (tid == T_METAL || tid == T_HAZARD) && (ty & 31) < 3 && (hash & 1) == 0 {
                            colors[i] = Self::blend(colors[i], Self::pack(255, 140, 40, 255), 0.35);
                        }
                        if dec > 0 {
                            let splat = self.sample_mip(
                                T_SPLAT, tex_x, ty.wrapping_add(dec as i32 * 17), level, mix,
                            );
                            if ((splat >> 24) & 255) > 24 {
                                colors[i] = Self::blend(colors[i], splat, 0.28 + dec as f32 * 0.18);
                            }
                        }
                    }
                    let shaded = Self::shade_rgb4(colors, dist_mul, light);
                    for i in 0..n {
                        let ty = tex_y[i];
                        let mut col = shaded[i];
                        if tid == T_TECH && (ty & 31) < 2 {
                            col = Self::blend(col, Self::pack(64, 190, 224, 255), 0.62);
                        }
                        col = Self::fog(col, perp);
                        let yy = y + i as i32 + shy;
                        let xx = x as i32 + shx;
                        if yy >= 0 && yy < h as i32 && xx >= 0 && xx < w as i32 {
                            self.fb[yy as usize * w + xx as usize] = col;
                        }
                    }
                    y += n as i32;
                }
            }

        }
        self.render_planes(dir_x, dir_y, plane_x, plane_y, horizon, &wall_spans[..w]);

        // sprites
        let mut order = [(0.0f32, 0usize); ENT_N];
        let mut count = 0;
        for (i, e) in self.ents.iter().enumerate() {
            if e.kind == 0 {
                continue;
            }
            let depth = (e.x - self.px) * dir_x + (e.y - self.py) * dir_y;
            if depth <= 0.12 { continue; }
            order[count] = (depth, i);
            count += 1;
        }
        // Painter's order, far to near. Index tiebreak so equal depths resolve
        // identically every frame instead of flickering.
        order[..count].sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        for &(_d, i) in &order[..count] {
            let e = self.ents[i];
            let sx = e.x - self.px;
            let sy = e.y - self.py;
            let tx = inv_det * (dir_y * sx - dir_x * sy);
            let ty = inv_det * (-plane_y * sx + plane_x * sy);
            if ty <= 0.12 {
                continue;
            }
            // Presentation comes from the roster table: texture slot,
            // world scale and sheet layout per kind (see enemies.rs).
            let (tid, scale, sheet4, fr) = sprite_style(&e);
            let sprite_h = (h as f32 / ty * scale).abs();
            let voff = e.zoff / ty;
            let ds_y = (-sprite_h * 0.5 + horizon + voff) as i32;
            let de_y = (sprite_h * 0.5 + horizon + voff) as i32;
            let sprite_w = sprite_h;
            let screen_x = (w as f32 / 2.0) * (1.0 + tx / ty);
            let ds_x = (-sprite_w / 2.0 + screen_x) as i32;
            let de_x = (sprite_w / 2.0 + screen_x) as i32;
            let flash = e.flash > 0.0;
            let half = TEX as i32 / 2;
            let ou = if sheet4 { (fr & 1) * half } else { 0 };
            let ov = if sheet4 { ((fr >> 1) & 1) * half } else { 0 };
            let cell = if sheet4 { half } else { TEX as i32 };
            let glow = if e.kind == EK_LAMP && e.timer < 0.0 {
                0.28
            } else if e.kind == EK_LAMP {
                1.1 + 0.45 * (tnow * 13.0 + e.x).sin().abs()
            } else if matches!(e.kind, EK_FLAME | EK_BOLT | EK_FIREPATCH) {
                1.3 + 0.45 * (tnow * 18.0 + e.x).sin().abs()
            } else if e.kind == EK_SPARK && e.effect_tick == 5.0 {
                1.0
            } else if e.kind == EK_IMPACT || e.kind == EK_SPARK || e.kind == EK_RAY {
                1.45
            } else {
                1.05
            };
            let sprite_light = self.light_at(e.x, e.y);
            let _distp = _d.sqrt();
            let item = is_pickup(e.kind);
            let gold = is_weapon_item(e.kind);
            let pulse = 0.62 + 0.38 * (tnow * 3.8 + e.x).sin().abs();
            if item {
                let hs_w = sprite_w * 1.12;
                let hs_h = sprite_h * 1.12;
                let hx0 = (-hs_w / 2.0 + screen_x) as i32;
                let hx1 = (hs_w / 2.0 + screen_x) as i32;
                let hy0 = (-hs_h * 0.5 + horizon + voff) as i32;
                let hy1 = (hs_h * 0.5 + horizon + voff) as i32;
                let (hr, hg, hb) = if gold {
                    (255.0, 196.0, 72.0)
                } else {
                    (210.0, 220.0, 232.0)
                };
                for stripe in hx0.max(0)..hx1.min(w as i32) {
                    if ty >= self.zbuf[stripe as usize] * 1.02 {
                        continue;
                    }
                    let nx = (stripe as f32 - screen_x) / (hs_w * 0.5);
                    for y in hy0.max(0)..hy1.min(h as i32) {
                        let ny = (y as f32 - (horizon + voff)) / (hs_h * 0.5);
                        let rad = (nx * nx + ny * ny).sqrt();
                        if !(0.9..=1.0).contains(&rad) {
                            continue;
                        }
                        let band = 1.0 - ((rad - 0.95).abs() / 0.05);
                        let f = band.max(0.0) * (0.18 + 0.14 * pulse);
                        let yy = y + shy;
                        let xx = stripe + shx;
                        if yy < 0 || yy >= h as i32 || xx < 0 || xx >= w as i32 {
                            continue;
                        }
                        let p = self.fb[yy as usize * w + xx as usize];
                        let r = ((p & 255) as f32 + hr * f).min(255.0) as u32;
                        let g = (((p >> 8) & 255) as f32 + hg * f).min(255.0) as u32;
                        let b = (((p >> 16) & 255) as f32 + hb * f).min(255.0) as u32;
                        self.fb[yy as usize * w + xx as usize] =
                            r | (g << 8) | (b << 16) | (p & 0xFF000000);
                    }
                }
            }
            for stripe in ds_x.max(0)..de_x.min(w as i32) {
                if ty >= self.zbuf[stripe as usize] {
                    continue;
                }
                let mut tex_x = ((stripe as f32 - (-sprite_w / 2.0 + screen_x)) * cell as f32 / sprite_w) as i32;
                if tex_x < 0 || tex_x >= cell {
                    continue;
                }
                tex_x += ou;
                for y in ds_y.max(0)..de_y.min(h as i32) {
                    let d = y as f32 - ds_y as f32;
                    let mut tex_y = (d * cell as f32 / (de_y - ds_y).max(1) as f32) as i32;
                    if tex_y < 0 || tex_y >= cell {
                        continue;
                    }
                    tex_y += ov;
                    let mut col = self.sample(tid, tex_x, tex_y);
                    if e.kind == EK_RAY {
                        let nx = (stripe as f32 - screen_x) / (sprite_w * 0.5).max(0.001);
                        let ny = (y as f32 - (horizon + voff)) / (sprite_h * 0.5).max(0.001);
                        let rad = nx * nx + ny * ny;
                        if rad >= 1.0 {
                            continue;
                        }
                        let core = (1.0 - rad).powf(2.2);
                        col = if e.zoff >= 11.0 {
                            Self::pack(255, (148.0 + core * 90.0) as u32, (34.0 + core * 150.0) as u32, (core * 235.0) as u32)
                        } else {
                            Self::pack((115.0 + core * 140.0) as u32, (210.0 + core * 45.0) as u32, 255, (core * 230.0) as u32)
                        };
                    } else {
                        let a = (col >> 24) & 255;
                        if a < 16 {
                            continue;
                        }
                        let lum = (col & 255) + ((col >> 8) & 255) + ((col >> 16) & 255);
                        if item && lum < 48 {
                            continue;
                        }
                        if e.kind == EK_BOSS {
                            col = Self::blend(col, Self::pack(255, 36, 24, 255), 0.42);
                        }
                        if flash {
                            col = Self::pack(255, 220, 220, a);
                        } else if is_hostile_kind(e.kind) && e.effect_tick > 0.0 {
                            col = Self::blend(col, Self::pack(255, 184, 60, a), 0.32);
                        }
                    }
                    col = Self::fog(Self::shade_rgb(col, glow, sprite_light), ty);
                    let yy = y + shy;
                    let xx = stripe + shx;
                    if yy >= 0 && yy < h as i32 && xx >= 0 && xx < w as i32 {
                        let index = yy as usize * w + xx as usize;
                        let alpha = ((col >> 24) & 255) as f32 / 255.0;
                        self.fb[index] = Self::blend(self.fb[index], col, alpha);
                    }
                }
            }
        }

    }
}


/// repr(C) frame shared with TypeScript through `hs_gpu_view`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuView {
    px: f32, py: f32, dir_x: f32, dir_y: f32,
    plane_x: f32, plane_y: f32, horizon: f32, time: f32,
    hell: f32, muzzle: f32, w: f32, h: f32,
    plane_len: f32, sprite_n: f32, _p1: f32, _p2: f32,
}

/// repr(C) frame shared with TypeScript through `hs_gpu_cols`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuCol {
    perp: f32, tex_x: f32, light_r: f32, light_g: f32,
    light_b: f32, draw0: f32, draw1: f32, line_h: f32,
    ds_full: f32, tex: f32, side: f32, dec: f32,
    hash: f32, hit: f32, z: f32, _pad: f32,
}

/// repr(C) frame shared with TypeScript through `hs_gpu_sprites`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuSprite {
    x: f32, y: f32, zoff: f32, scale: f32,
    tex: f32, frame: f32, flash: f32, kind: f32,
}

struct GpuScratch {
    enemy_cues: [voices::EnemyCue; ENT_N],
    bar_cues: [voices::BarCue; 48],
    cols: Vec<GpuCol>,
    sprites: Vec<GpuSprite>,
    sprite_n: usize,
    view: GpuView,
}

// Single-threaded WASM: every entry point runs on the same JS thread, so
// exclusive access holds (same contract as eng() above).
#[allow(static_mut_refs)]
fn gpu_scratch() -> &'static mut GpuScratch {
    static mut G: Option<GpuScratch> = None;
    unsafe {
        if G.is_none() {
            G = Some(GpuScratch {
                enemy_cues: [voices::EnemyCue::default(); ENT_N],
                bar_cues: [voices::BarCue::default(); 48],
                cols: vec![GpuCol {
                    perp: 0.0, tex_x: 0.0, light_r: 0.0, light_g: 0.0,
                    light_b: 0.0, draw0: 0.0, draw1: -1.0, line_h: 1.0,
                    ds_full: 0.0, tex: 0.0, side: 0.0, dec: 0.0,
                    hash: 0.0, hit: 0.0, z: 40.0, _pad: 0.0,
                }; MAX_W],
                sprites: vec![GpuSprite {
                    x: 0.0, y: 0.0, zoff: 0.0, scale: 0.0,
                    tex: 0.0, frame: -1.0, flash: 0.0, kind: 0.0,
                }; ENT_N],
                sprite_n: 0,
                view: GpuView {
                    px: 0.0, py: 0.0, dir_x: 1.0, dir_y: 0.0,
                    plane_x: 0.0, plane_y: 0.0, horizon: 0.0, time: 0.0,
                    hell: 0.0, muzzle: 0.0, w: 0.0, h: 0.0,
                    plane_len: 0.0, sprite_n: 0.0, _p1: 0.0, _p2: 0.0,
                },
            });
        }
        G.as_mut().unwrap_unchecked()
    }
}

#[no_mangle]
pub extern "C" fn hs_sound_count() -> i32 { eng().sound_cues.len() as i32 }
#[no_mangle]
pub extern "C" fn hs_sound_cues() -> *const sound::SoundCue { eng().sound_cues.as_ptr() }

/// Read-only spatial loop snapshot; disappearing entities stop their own sound.
#[no_mangle]
pub extern "C" fn hs_prepare_sound_loops() -> i32 {
    let e = eng(); e.sound_loops.clear();
    for (id, ent) in e.ents.iter().enumerate() {
        if (ent.x-e.px).powi(2) + (ent.y-e.py).powi(2) > 100.0 { continue; }
        let kind = match ent.kind {
            EK_BOLT => 14,
            EK_PROJ if matches!(ent.skin, 120 | 129 | 208 | 216 | 225 | 229) => 14,
            EK_PROJ if ent.effect_tick == 3.0 || matches!(ent.skin, 110 | 114 | 128 | 233) => 16,
            EK_PROJ => 15,
            EK_FIREPATCH if ent.timer > 0.0 => 17,
            _ => continue,
        };
        if e.sound_loops.len() < sound::CAP {
            e.sound_loops.push(sound::SoundCue {kind: kind as f32, variant:id as f32, x:ent.x, y:ent.y});
        }
    }
    e.sound_loops.len() as i32
}
#[no_mangle]
pub extern "C" fn hs_sound_loops() -> *const sound::SoundCue { eng().sound_loops.as_ptr() }

#[no_mangle]
pub extern "C" fn hs_prepare_enemies() -> i32 {
    voices::snapshot(eng(), &mut gpu_scratch().enemy_cues) as i32
}

#[no_mangle]
pub extern "C" fn hs_enemy_cues() -> *const voices::EnemyCue {
    gpu_scratch().enemy_cues.as_ptr()
}

#[no_mangle]
pub extern "C" fn hs_prepare_bars() -> i32 {
    let scratch = gpu_scratch();
    voices::snapshot_bars(eng(), &mut scratch.bar_cues) as i32
}

#[no_mangle]
pub extern "C" fn hs_bars() -> *const voices::BarCue {
    gpu_scratch().bar_cues.as_ptr()
}

#[no_mangle]
pub extern "C" fn hs_prepare_gpu() {
    eng().prepare_gpu();
}

#[no_mangle]
pub extern "C" fn hs_gpu_view() -> *const GpuView {
    &gpu_scratch().view
}

#[no_mangle]
pub extern "C" fn hs_gpu_cols() -> *const GpuCol {
    gpu_scratch().cols.as_ptr()
}

#[no_mangle]
pub extern "C" fn hs_gpu_sprites() -> *const GpuSprite {
    gpu_scratch().sprites.as_ptr()
}

#[no_mangle]
pub extern "C" fn hs_gpu_sprite_count() -> i32 {
    gpu_scratch().sprite_n as i32
}

#[no_mangle]
pub extern "C" fn hs_floor_ptr() -> *const u8 {
    eng().floor.as_ptr()
}

#[no_mangle]
pub extern "C" fn hs_light_ptr() -> *const f32 {
    eng().light_grid.as_ptr() as *const f32
}

#[no_mangle]
pub extern "C" fn hs_smoke_ptr() -> *const f32 {
    eng().smoke_grid.as_ptr()
}

#[no_mangle]
pub extern "C" fn hs_init(w: i32, h: i32) -> i32 {
    eng_init(w as usize, h as usize);
    0
}

#[no_mangle]
pub extern "C" fn hs_restart() {
    let e = eng();
    let textures = std::mem::take(&mut e.tex);
    let theme = std::mem::take(&mut e.theme_tex);
    let (w, h) = (e.w, e.h);
    *e = Engine::with_textures(w, h, Some(textures));
    e.theme_tex = theme;
    e.apply_theme(1);
}

#[no_mangle]
pub extern "C" fn hs_next_wave() {
    eng().next_wave();
}

#[no_mangle]
pub extern "C" fn hs_select_weapon(slot: i32) {
    if slot >= 0 { eng().select_weapon(slot as usize); }
}

#[no_mangle]
pub extern "C" fn hs_resize(w: i32, h: i32) {
    let e = eng();
    e.w = (w as usize).clamp(160, MAX_W);
    e.h = (h as usize).clamp(100, MAX_H);
    let need = e.w * e.h;
    if e.fb.len() < need {
        e.fb.resize(need, 0);
    }
    if e.zbuf.len() < e.w {
        e.zbuf.resize(e.w, 0.0);
    }
}

#[no_mangle]
pub extern "C" fn hs_fb_ptr() -> *mut u8 {
    eng().fb.as_mut_ptr() as *mut u8
}

#[no_mangle]
pub extern "C" fn hs_fb_w() -> i32 {
    eng().w as i32
}

#[no_mangle]
pub extern "C" fn hs_fb_h() -> i32 {
    eng().h as i32
}

#[no_mangle]
pub extern "C" fn hs_hud_ptr() -> *mut Hud {
    &mut eng().hud
}

/// Byte size of the Hud struct. TS asserts this on boot so a Rust-side
/// field reorder/addition fails loudly instead of misreading memory.
#[no_mangle]
pub extern "C" fn hs_hud_size() -> i32 {
    HUD_SIZE as i32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RunSave {
    pub wave: i32,
    pub health: i32,
    pub armor: i32,
    pub weapon: i32,
    pub flags: i32,
    pub kills: i32,
    pub secrets: i32,
    pub elapsed_ms: i32,
    pub ammo: [i32; WEP_N],
    pub mag: [i32; WEP_N],
    pub extra_weapons: u32,
}

fn capture_save(e: &Engine) -> RunSave {
    let mut flags = 0;
    for (i, owned) in e.owned.iter().enumerate() {
        if *owned { flags |= 1 << i; }
    }
    RunSave {
        wave: e.wave,
        health: e.health,
        armor: e.armor,
        weapon: e.weapon,
        flags,
        kills: e.kills,
        secrets: e.secrets,
        elapsed_ms: (e.elapsed * 1000.0) as i32,
        ammo: e.ammo,
        mag: e.mag,
        extra_weapons: e.extra_weapons,
    }
}

fn apply_save(e: &mut Engine, s: &RunSave) {
    e.clear_boss_arena();
    e.tactical=tactical::Tactical::new();
    e.wave = s.wave.max(1);
    e.health = s.health.clamp(1, 100);
    e.armor = s.armor.clamp(0, 100);
    e.kills = s.kills.max(0);
    e.secrets = s.secrets.max(0);
    e.elapsed = (s.elapsed_ms.max(0) as f32) / 1000.0;
    for (i, slot) in e.owned.iter_mut().enumerate() {
        *slot = s.flags & (1 << i) != 0;
    }
    e.ammo = s.ammo;
    e.mag = s.mag;
    e.extra_weapons = s.extra_weapons & 16383;
    e.weapon = if (0..WEP_N as i32).contains(&s.weapon) && e.owns_slot(s.weapon as usize) { s.weapon } else { 0 };
    e.state = 0;
    e.boss_spawned = false;
    e.boss_intro = 0.0;
    e.boss_phase = 0;
    e.hell = false;
    e.lockdown = false;
    e.iframes = 1.2;
    let start = map::player_start(e.wave);
    e.px = start.0;
    e.py = start.1;
    e.pa = start.2;
    e.pitch = 0.0;
    e.apply_theme(e.wave);
    e.build_map();
    e.door.fill(0.0);
    e.light_dirty = true;
    map::place_level(e);
    let shift = (e.wave - 1).clamp(0, 2);
    e.spawn_hostiles(1i32 << shift);
}

fn save_slot() -> &'static mut RunSave {
    static mut SAVE: RunSave = RunSave {
        wave: 1, health: 100, armor: 0, weapon: 0, flags: 0, kills: 0, secrets: 0, elapsed_ms: 0,
        ammo: [0; WEP_N], mag: [0; WEP_N], extra_weapons: 0,
    };
    #[allow(static_mut_refs)]
    unsafe { &mut SAVE }
}

fn load_slot() -> &'static mut RunSave {
    static mut LOAD: RunSave = RunSave {
        wave: 1, health: 100, armor: 0, weapon: 0, flags: 0, kills: 0, secrets: 0, elapsed_ms: 0,
        ammo: [0; WEP_N], mag: [0; WEP_N], extra_weapons: 0,
    };
    #[allow(static_mut_refs)]
    unsafe { &mut LOAD }
}

#[no_mangle]
pub extern "C" fn hs_save_ptr() -> *const RunSave {
    let e = eng();
    let slot = save_slot();
    *slot = capture_save(e);
    slot
}

#[no_mangle]
pub extern "C" fn hs_save_size() -> i32 {
    core::mem::size_of::<RunSave>() as i32
}

#[no_mangle]
pub extern "C" fn hs_load_ptr() -> *mut RunSave {
    load_slot()
}

#[no_mangle]
pub extern "C" fn hs_load_run() {
    let save = *load_slot();
    apply_save(eng(), &save);
}

#[no_mangle]
pub extern "C" fn hs_map_ptr() -> *const u8 {
    eng().map.as_ptr()
}

#[no_mangle]
pub extern "C" fn hs_door_ptr() -> *const f32 { eng().door.as_ptr() }

#[no_mangle]
pub extern "C" fn hs_map_w() -> i32 { MAP_W as i32 }

#[no_mangle]
pub extern "C" fn hs_map_h() -> i32 { MAP_H as i32 }
/// Lightweight event accessors so the frame loop can drain SFX events per
/// fixed substep without decoding the full HUD struct each time.
#[no_mangle]
pub extern "C" fn hs_events() -> u32 {
    eng().hud.events
}

#[no_mangle]
pub extern "C" fn hs_ev_weapon() -> i32 {
    eng().hud.ev_weapon
}

#[no_mangle]
pub extern "C" fn hs_tex_ptr(id: i32) -> *mut u8 {
    // Reject out-of-range slots instead of silently clamping: a clamped id would
    // hand TypeScript a valid pointer to the wrong texture. Mirrors hs_theme_ptr.
    if id < 0 || id >= TEX_N as i32 {
        return core::ptr::null_mut();
    }
    let o = id as usize * TEX * TEX;
    debug_assert!(o + TEX * TEX <= eng().tex.len(), "atlas layer out of bounds");
    unsafe { eng().tex.as_mut_ptr().add(o) as *mut u8 }
}

#[no_mangle]
pub extern "C" fn hs_tex_size() -> i32 {
    TEX as i32
}

#[no_mangle]
pub extern "C" fn hs_textures_ready() {
    eng().rebuild_mipmaps();
}

/// Staging pointer for one theme variant layer (0-7 walls, 8-15 doors),
/// 256x256 RGBA. Slots mirror THEME_FILES in src/game/runtime.ts.
#[no_mangle]
pub extern "C" fn hs_theme_ptr(slot: i32) -> *mut u8 {
    if !(0..16).contains(&slot) {
        return core::ptr::null_mut();
    }
    let o = slot as usize * TEX * TEX;
    debug_assert!(o + TEX * TEX <= eng().theme_tex.len(), "theme layer out of bounds");
    unsafe { eng().theme_tex.as_mut_ptr().add(o) as *mut u8 }
}

/// Copy the wave's wall/door variants into the live atlas slots.
#[no_mangle]
pub extern "C" fn hs_apply_theme(wave: i32) {
    eng().apply_theme(wave);
}

#[no_mangle]
pub extern "C" fn hs_input(bits: u32, mx: f32, my: f32) {
    let e = eng();
    e.bits = bits;
    // A NaN or inf arriving from JavaScript would otherwise propagate straight
    // into `pa` and poison every downstream transform.
    e.mx = finite(mx);
    e.my = finite(my);
}

#[no_mangle]
pub extern "C" fn hs_qa(bits: u32, enabled: i32) {
    let e = eng();
    e.qa = enabled != 0;
    e.qa_bits = bits;
}

#[no_mangle]
pub extern "C" fn hs_qa_armory() {
    let e = eng();
    if !e.qa { return; }
    e.owned = [true; OWNED_GUNS];
    e.mag = MAG_SZ;
    for slot in 0..WEP_N { e.ammo[slot] = RESERVE_CAP[slot]; }
    e.extra_weapons = 16383;
}

/// QA-only full heal so long single-page smokes don't die mid-run.
#[no_mangle]
pub extern "C" fn hs_qa_heal() {
    eng().qa_heal();
}

/// QA-only replay of the real boss reward spawn and world-overlap pickup path.
#[no_mangle]
pub extern "C" fn hs_qa_reward() {
    let e = eng();
    if !e.qa { return; }
    e.qa_heal();
    e.boss_intro = 0.0;
    e.drop_boss_case(e.px + e.pa.cos(), e.py + e.pa.sin());
}

#[no_mangle]
pub extern "C" fn hs_qa_end(state: i32) {
    let e = eng();
    if matches!(state, 1 | 2) { e.state = state; }
}

#[no_mangle]
pub extern "C" fn hs_qa_boss(phase: i32) {
    let e = eng();
    if !e.qa { return; }
    if phase<0 {
        e.boss_spawned=false;e.boss_phase=0;e.boss_intro=e.boss_intro_duration();
        let (x,y)=map::boss_spots(e.wave)[0];e.pa=(y-e.py).atan2(x-e.px);
        return;
    }
    e.boss_spawned = false;
    e.boss_phase = 0;
    e.maybe_spawn_boss();
    let max_hp = e.boss_max_health();
    if let Some(boss) = e.ents.iter_mut().find(|enemy| enemy.kind == EK_BOSS && enemy.hp > 0) {
        boss.hp = match phase.clamp(0, 2) {
            1 => (max_hp as i64 * 2 / 3) as i32,
            2 => max_hp / 3,
            _ => max_hp,
        };
    }
}

/// Local-only fixtures exercise the real USE / firing paths, never grant their rewards.
#[no_mangle]
pub extern "C" fn hs_qa_tactical(case:i32,index:i32) {
    let e=eng();if !e.qa {return;}e.qa_heal();
    let target=if case==0 {
        e.map.iter().position(|&c|c==9).map(|i|((i%MAP_W) as f32+0.5,(i/MAP_W) as f32+0.5))
    } else {
        e.ents.iter().filter(|en|en.kind==EK_CRATE&&(150..=224).contains(&en.skin)&&en.hp>0).nth(index.max(0) as usize).map(|en|(en.x,en.y))
    };
    if let Some((x,y))=target {
        for (dx,dy) in [(-1.0,0.0),(1.0,0.0),(0.0,-1.0),(0.0,1.0)] {
            let (a,b)=(x+dx,y+dy);
            if e.circle_blocked(a,b,e.pr) || (case!=0 && !e.los(a,b,x,y)) {continue;}
            e.px=a;e.py=b;e.pa=(-dy).atan2(-dx);e.pitch=0.0;
            e.weapon=0;e.mag[0]=12;e.cooldown=0.0;e.tactical.cooldowns.fill(0.0);break;
        }
    }
}

#[no_mangle]
pub extern "C" fn hs_qa_objective() {
    let e = eng();
    if !e.qa { return; }
    let (x, y) = map::override_point(e.wave);
    e.px = x - 1.2;
    e.py = y;
    e.pa = 0.0;
    for ent in &mut e.ents {
        if is_hostile_kind(ent.kind) { ent.kind = EK_NONE; }
    }
}

#[no_mangle]
pub extern "C" fn hs_fire_patches() -> i32 {
    eng().ents.iter().filter(|e| e.kind == EK_FIREPATCH).count() as i32
}

#[no_mangle]
pub extern "C" fn hs_tick(dt: f32) {
    eng().tick(dt);
}

#[no_mangle]
pub extern "C" fn hs_render() {
    eng().render();
}

/// Debug-only check that the simd128 floor path matches the scalar formula.
/// Absent from release builds. Returns the mismatch count.
#[cfg(all(debug_assertions, target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn hs_simd_probe() -> i32 {
    let mut mismatches = 0i32;
    let colors = [0xff11_2233, 0x00ff_ffff, 0x0100_0000, 0x80c0_e0ff];
    let gains = [
        [117_964, 32_768, 11_796],
        [65_536, 65_536, 65_536],
        [0, 1_000, 200_000],
        [117_964, 117_964, 117_964],
    ];
    let shaded = Engine::shade_texels4(colors, gains, 0x3c00_0000);
    for i in 0..4 {
        if shaded[i] != Engine::shade_texel(colors[i], gains[i], 0x3c00_0000) {
            mismatches += 1;
        }
    }
    let a = [0x1122_3344, 0xff00_ff00, 0x0102_0304, 0x00ff_00ff];
    let b = [0xaabb_ccdd, 0x00ff_00ff, 0xffff_ffff, 0x1234_5678];
    for mix in [0u32, 1, 127, 128, 255] {
        let blended = Engine::blend_mips4(a, b, mix);
        for i in 0..4 {
            if blended[i] != Engine::blend_mips(a[i], b[i], mix) {
                mismatches += 1;
            }
        }
    }
    let light = [0.4, -0.2, 1.2];
    let rgb = Engine::shade_rgb4(colors, 0.7, light);
    for i in 0..4 {
        if rgb[i] != Engine::shade_rgb(colors[i], 0.7, light) {
            mismatches += 1;
        }
    }
    mismatches
}

#[no_mangle]
pub extern "C" fn hs_yaw() -> f32 {
    eng().pa
}

#[no_mangle]
pub extern "C" fn hs_speed() -> f32 {
    eng().hud.speed
}

#[no_mangle]
pub extern "C" fn hs_spread() -> f32 { eng().spread }

#[no_mangle]
pub extern "C" fn hs_x() -> f32 {
    eng().px
}

#[no_mangle]
pub extern "C" fn hs_y() -> f32 {
    eng().py
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arena() -> Engine {
        let mut e = Engine::new(160, 100);
        e.map.fill(0);
        for ent in &mut e.ents { ent.kind = 0; }
        e.px = 4.5;
        e.py = 4.5;
        e
    }

    #[test]
    fn one_shot_enemy_states_reach_all_four_frames_without_looping() {
        let mut e = Engine::new(320, 200);
        let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
        for state in [ANIM_PAIN, ANIM_FIRE, ANIM_RELOAD, ANIM_DEAD, ANIM_SPECIAL] {
            e.ents[i].anim = ANIM_IDLE;
            e.ents[i].anim_lock = 0.0;
            set_anim(&mut e.ents[i], state, 0.24);
            let duration = e.ents[i].anim_lock;
            for frame in 0..4 {
                e.ents[i].anim_time = (frame as f32 + 0.1) / animation_fps(state);
                assert_eq!(anim_frame(&e.ents[i]), frame);
                assert!(e.ents[i].anim_time < duration);
            }
            e.ents[i].anim_time = duration + 0.5;
            assert_eq!(anim_frame(&e.ents[i]), 3);
        }
    }

    #[test]
    fn each_sector_spawns_its_exclusive_enemy_with_the_correct_role() {
        let mut e = Engine::new(320, 200);
        for wave in 1..=50 {
            e.wave = wave;
            map::build_level(&mut e);
            for ent in &mut e.ents { ent.kind = EK_NONE; ent.hp = 0; }
            e.spawn_hostiles(1);
            let skin = 37 + map::level_index(wave) as u8;
            assert!(e.ents.iter().any(|ent| ent.skin == skin && ent.hp > 0));
            assert!(e.ents.iter().filter(|ent| ent.hp > 0 && (37..62).contains(&ent.skin)).all(|ent| ent.skin == skin));
            let base = enemies::combat_skin(skin);
            let ent = e.ents.iter().find(|ent| ent.skin == skin).unwrap();
            assert_eq!(combat::profile(skin, ent.kind).damage, combat::profile(base, ent.kind).damage);
            assert_eq!(enemies::armor_cap(ent.kind, skin), enemies::armor_cap(ent.kind, base));
        }
    }

    #[test]
    fn enemy_cues_project_above_sprites_and_respect_cover() {
        let mut e = arena();
        e.pa = 0.0;
        e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5);
        let mut cues = [voices::EnemyCue::default(); ENT_N];
        assert_eq!(voices::snapshot(&e, &mut cues), 1);
        assert_eq!(std::mem::size_of::<voices::EnemyCue>(), 40);
        assert!((cues[0].screen_x - 0.5).abs() < 0.001);
        assert!(cues[0].screen_y < 0.5);
        assert_eq!(cues[0].sight, 1.0);
        e.set_cell(6, 4, 1);
        voices::snapshot(&e, &mut cues);
        assert_eq!(cues[0].sight, 0.0);
        e.pa = std::f32::consts::PI;
        voices::snapshot(&e, &mut cues);
        assert!(cues[0].screen_x < 0.0);
    }

    #[test]
    fn ranged_windup_commits_aim_and_damage() {
        let mut e = arena();
        let i = e.spawn_with_skin(EK_HUSK, SKIN_MARKSMAN, 9.5, 4.5).unwrap();
        e.ents[i].timer = 0.0;
        e.tick(1.0 / 60.0);
        assert!(e.ents[i].effect_tick > 0.8);
        assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ));
        // Strafe after the warning: the marksman must shoot at the old aim.
        e.py = 7.5;
        for _ in 0..56 { e.tick(1.0 / 60.0); }
        let shot = e.ents.iter().find(|p| p.kind == EK_PROJ).expect("windup releases a shot");
        assert!(shot.vy.abs() < 0.01, "shots must not track dodges during windup");
        assert_eq!(shot.hp, 24);
    }

    #[test]
    fn damage_interrupts_windup_and_cover_cancels_shot() {
        let mut e = arena();
        let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
        e.ents[i].timer = 0.0;
        e.tick(1.0 / 60.0);
        e.hurt_ent(i, 1, e.px, e.py);
        assert_eq!(e.ents[i].effect_tick, 0.0);
        for _ in 0..20 { e.tick(1.0 / 60.0); }
        assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ));
        e.ents[i].timer = 0.0;
        e.tick(1.0 / 60.0);
        assert!(e.ents[i].effect_tick > 0.0);
        for y in 0..MAP_H { e.set_cell(6, y as i32, 1); }
        for _ in 0..30 { e.tick(1.0 / 60.0); }
        assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ), "cover cancels attack");
    }

    #[test]
    fn hound_melee_can_be_dodged_and_does_not_fire() {
        let mut e = arena();
        let i = e.spawn_with_skin(EK_WRAITH, SKIN_HOUND, 5.3, 4.5).unwrap();
        e.ents[i].timer = 0.0;
        e.tick(1.0 / 60.0);
        assert!(e.ents[i].effect_tick > 0.0);
        e.px = 2.5;
        for _ in 0..20 { e.tick(1.0 / 60.0); }
        assert_eq!(e.health, 100);
        assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ));
    }

    #[test]
    fn martyr_chases_with_move_anim_then_detonates() {
        let mut e = arena();
        let i = e.spawn(EK_MARTYR, 6.5, 4.5).unwrap();
        assert_eq!(e.ents[i].skin, SKIN_MARTYR);
        for _ in 0..10 { e.tick(1.0 / 60.0); }
        assert_eq!(e.ents[i].anim, ANIM_MOVE, "chaser must play its move sheet");
        assert!(e.ents[i].x < 6.5, "martyr must float toward the player");
        e.ents[i].timer = 0.0;
        for _ in 0..120 {
            e.tick(1.0 / 60.0);
            // Break on the detonation tick itself: the blast FX reuses the
            // drone's freed slot, so slot state alone cannot mark the moment.
            if e.health < 100 { break; }
        }
        assert!(e.health < 100, "point-blank detonation must hurt");
        assert!(e.ents.iter().any(|p| p.kind == EK_IMPACT), "detonation leaves a blast mark");
        assert_ne!(e.ents[i].kind, EK_MARTYR, "detonation consumes the drone");
    }

    #[test]
    fn martyr_gunfire_death_explodes_like_a_barrel() {
        let mut e = arena();
        let i = e.spawn(EK_MARTYR, 6.5, 4.5).unwrap();
        e.hurt_ent(i, 500, e.px, e.py);
        e.tick(1.0 / 60.0);
        assert!(e.ents.iter().any(|p| p.kind == EK_IMPACT));
    }

    #[test]
    fn barrels_render_as_props_not_martyr_sheets() {
        let mut e = arena();
        let i = e.spawn(EK_BARREL, 6.5, 4.5).unwrap();
        assert_eq!(e.ents[i].skin, SKIN_NONE);
        let (tex, _, sheet4, _) = sprite_style(&e.ents[i]);
        assert_eq!((tex, sheet4), (T_BARREL, false));
    }

    #[test]
    fn chimera_direct_splash_and_pool_damage_are_reduced_by_a_third() {
        let mut e=arena();
        e.fire_chimera();
        assert_eq!(e.ents.iter().filter(|v| v.kind==EK_PROJ).count(),5);
        assert!(e.ents.iter().filter(|v| v.kind==EK_PROJ).all(|v| v.hp==28));
        for ent in &mut e.ents {ent.kind=EK_NONE;}
        let target=e.spawn(EK_HUSK,8.5,4.5).unwrap();
        e.ents[target].hp=1000; e.ents[target].armor_hp=0;
        e.chimera_burst(8.5,4.5);
        assert_eq!(e.ents[target].hp,947);
        let pool=e.ents.iter().position(|v| v.kind==EK_FIREPATCH).unwrap();
        assert_eq!(e.ents[pool].hp,100);
        e.ents[pool].timer=8.0;
        for _ in 0..100 {
            e.ents[target].x=8.5; e.ents[target].y=4.5;
            e.ents[target].stun=100.0;
            e.tick(0.0625);
        }
        assert_eq!(e.ents[target].hp,813, "25 pool pulses deal exactly 134 damage");
    }

    #[test]
    fn chimera_bolts_are_acid_and_pools_are_not_enemies() {
        let mut e = arena();
        let bolt = e.spawn(EK_PROJ, 4.0, 4.0).unwrap();
        e.ents[bolt].effect_tick = 4.0;
        e.ents[bolt].skin = SKIN_SUBJECT;
        let (tex, _, _, frame) = sprite_style(&e.ents[bolt]);
        assert_eq!((tex, frame), (T_PROJECTILE_NEW + 3, 0));
        let pool = e.spawn(EK_FIREPATCH, 5.0, 4.0).unwrap();
        e.ents[pool].skin = 2;
        let (tex, _, sheet, _) = sprite_style(&e.ents[pool]);
        assert_eq!((tex, sheet), (T_FLAME, true));
        assert!(tex < ENEMY_TEX_BASE);
    }

    #[test]
    fn sprint_is_ten_percent_faster_and_caps_overdrive() {
        let mut e = arena();
        e.pa = 0.0;
        e.bits = IN_W | IN_SPRINT;
        e.power = field::POWER_OVERDRIVE;
        e.tick(0.08);
        let dist = ((e.px - 4.5).powi(2) + (e.py - 4.5).powi(2)).sqrt();
        assert!((dist - 3.35 * 1.32 * 0.08).abs() < 0.001);
    }

    #[test]
    fn qa_heal_restores_health_and_revives() {
        let mut e = arena();
        e.qa = true;
        e.health = 12;
        e.state = 1;
        e.spawn(EK_HUSK, 6.5, 4.5).unwrap();
        e.qa_heal();
        assert_eq!((e.health, e.state), (100, 0));
        assert!(
            e.ents.iter().all(|en| !is_hostile_kind(en.kind)),
            "heal clears the converged crowd"
        );
        e.qa = false;
        e.health = 5;
        e.qa_heal();
        assert_eq!(e.health, 5, "heal is QA-only");
    }

    #[test]
    fn splash_warns_when_wall_inside_blast_radius() {
        let mut e = arena();
        e.set_cell(6, 4, 1);
        e.px = 4.5;
        e.py = 4.5;
        e.pa = 0.0;
        e.weapon = 4;
        e.tick(1.0 / 60.0);
        assert_eq!(e.hud.splash, 1.0, "VLK-6 at a 2m wall must warn");
        e.pa = core::f32::consts::FRAC_PI_2;
        e.tick(1.0 / 60.0);
        assert_eq!(e.hud.splash, 0.0, "open lane must not warn");
        e.weapon = 0;
        e.pa = 0.0;
        e.tick(1.0 / 60.0);
        assert_eq!(e.hud.splash, 0.0, "hitscan guns never warn");
    }

    #[test]
    fn theme_swap_follows_sector_cycle() {
        let mut e = Engine::new(160, 100);
        for slot in 0..16 {
            for px in e.theme_tex[slot * TEX * TEX..(slot + 1) * TEX * TEX].iter_mut() {
                *px = 0xFF000000 | (slot as u32);
            }
        }
        e.apply_theme(1);
        let tech = e.tex[T_TECH * TEX * TEX];
        let door = e.tex[T_DOOR * TEX * TEX];
        assert_eq!((tech, door), (0xFF000000, 0xFF000008));
        e.apply_theme(26);
        assert_eq!(e.tex[T_TECH * TEX * TEX], tech, "wave 26 reuses wave 1 theme");
        e.apply_theme(6);
        assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000007);
        e.apply_theme(2);
        assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000004);
        assert_eq!(e.tex[T_DOOR * TEX * TEX], 0xFF00000C);
        e.apply_theme(4);
        assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000003);
        e.apply_theme(5);
        assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000006);
    }

    #[test]
    fn every_sector_boss_entry_and_phase_is_safe() {
        let mut e = Engine::new(160, 100);
        for wave in 1..=map::LEVEL_COUNT as i32 {
            e.wave = wave;
            map::build_level(&mut e);
            map::place_level(&mut e);
            e.boss_spawned = false;
            e.boss_phase = 0;
            e.state = 0;
            e.maybe_spawn_boss();
            let i = e.ents.iter().position(|x| x.kind == EK_BOSS).expect("boss spawns");
            assert_eq!(e.ents[i].skin, map::boss_skin(wave));
            e.ents[i].hp = e.boss_max_health() / 2;
            e.tick(0.016);
            assert_eq!(e.boss_phase, 1);
            if wave == 11 {
                assert!(e.ents.iter().any(|x| x.kind == EK_MARTYR && x.hp > 0));
            }
        }
    }

    #[test]
    fn every_boss_case_is_collected_by_world_overlap() {
        let mut e = arena();
        for wave in 1..=map::LEVEL_COUNT as i32 {
            e.state = 0;
            e.wave = wave;
            let kind = field::boss_case(wave);
            let slot = field::boss_slot(kind);
            assert!(is_weapon_item(kind), "wave {wave} reward isn't a weapon pickup");
            let index = e.spawn(kind, e.px, e.py).unwrap();
            e.tick(0.016);
            assert_eq!(e.ents[index].kind, EK_NONE, "wave {wave} case wasn't collected");
            assert_eq!(e.weapon as usize, slot, "wave {wave} equipped wrong reward");
            assert_eq!(e.state, 2, "wave {wave} didn't complete after collection");
        }
    }

    #[test]
    fn expansion_weapons_fire_reload_and_last_reward_survives_endless_transition() {
        let mut e = arena();
        e.select_weapon(32);
        assert_eq!(e.weapon, 0, "locked rewards cannot be selected");
        for slot in 19..WEP_N {
            e.ents.iter_mut().for_each(|ent| ent.kind = EK_NONE);
            e.grant_slot(slot);
            e.state = 0; e.pickup_t = 0.0; e.cooldown = 0.0;
            e.fire();
            assert_eq!(e.mag[slot], MAG_SZ[slot] - 1);
            assert!(e.muzzle > 0.0 && e.cooldown > 0.0);
            e.mag[slot] = 0; e.ammo[slot] = 2; e.begin_reload();
            assert!(e.reload_t > 0.0);
            for _ in 0..60 { e.tick(0.08); }
            assert_eq!(e.mag[slot], 2, "slot {slot} must reload from reserve");
            assert_eq!(e.ammo[slot], 0);
        }
        e.wave = 25;
        e.weapon = 32;
        let save = capture_save(&e);
        e.extra_weapons = 0;
        apply_save(&mut e, &save);
        assert_eq!(e.weapon, 32);
        assert_eq!(e.extra_weapons, 16383);
        e.next_wave();
        assert_eq!(e.wave, 26);
        assert_eq!(map::level_index(e.wave), 0);
        assert_eq!(e.weapon, 32);
        assert_eq!(e.mag[32], MAG_SZ[32]);
    }

    #[test]
    fn reinforcement_queue_grows_and_replenishes_without_spawning_on_player() {
        let mut e = arena();
        e.wave = 1000;
        e.spawn_hostiles(4);
        let remaining = e.pending_hostiles;
        assert!(remaining > 100);
        assert_eq!(map::living_hostiles(&e), campaign::ACTIVE_HOSTILES);
        assert!(!e.spawn_reinforcement(true));
        assert_eq!(e.pending_hostiles, remaining);
        for ent in &mut e.ents { if is_hostile_kind(ent.kind) { ent.kind = EK_NONE; break; } }
        assert!(e.spawn_reinforcement(true));
        assert_eq!(e.pending_hostiles, remaining - 1);
        let arrival = e.ents.iter().find(|ent| is_hostile_kind(ent.kind) && ent.stun > 0.0).unwrap();
        assert!((arrival.x - e.px).powi(2) + (arrival.y - e.py).powi(2) >= 49.0);
        assert!(arrival.timer >= 1.0);
    }

    #[test]
    fn rocket_art_keeps_outgoing_model_when_camera_turns() {
        let mut e = arena();
        let i = e.spawn(EK_BOLT, 7.5, 4.5).unwrap();
        e.ents[i].hp = 50; e.ents[i].timer = 2.0; e.ents[i].vx = 5.0;
        e.tick(0.016);
        assert_eq!(sprite_style(&e.ents[i]).0, T_PLAYER_MISSILE);
        assert!(!sprite_style(&e.ents[i]).2);
        e.ents[i].hp = 95;
        assert_eq!(sprite_style(&e.ents[i]).0, T_PLAYER_MISSILE + 1);
        e.ents[i].vx = -5.0;
        e.tick(0.016);
        assert_eq!(sprite_style(&e.ents[i]).0, T_PLAYER_MISSILE + 1);
    }

    #[test]
    fn every_sector_has_exclusive_surfaces_and_three_breakable_types() {
        let mut e = arena();
        for wave in 1..=25 {
            e.wave = wave;
            e.build_map();
            for ent in &mut e.ents { ent.kind = EK_NONE; }
            map::place_level(&mut e);
            let base = T_SECTOR_SURFACE + (wave as usize - 1) * 5;
            assert_eq!(e.wall_tex(1, 1, 1), base);
            assert_eq!(e.wall_tex(6, 1, 1), base + 1);
            assert_eq!(e.wall_tex(8, 1, 1), base + 2);
            assert_eq!(e.plane_sample(true, 4.5, 4.5, 0).0, base + 3);
            assert_eq!(e.plane_sample(false, 4.5, 4.5, 0).0, base + 4);
            for ty in 0..3 {
                let skin = 150 + ((wave - 1) * 3 + ty) as u8;
                let i = e.ents.iter().position(|ent| ent.kind == EK_CRATE && ent.skin == skin).expect("missing exclusive prop type");
                assert_eq!(sprite_style(&e.ents[i]).0, T_SECTOR_PROP + ((wave - 1) * 3 + ty) as usize);
                assert!(!e.blocked(e.ents[i].x as i32, e.ents[i].y as i32));
                let kills = e.kills;
                e.hurt_ent(i, 1000, e.px, e.py);
                assert_eq!(e.ents[i].hp, 0);
                assert_eq!(e.kills, kills, "breaking scenery must not count as an enemy kill");
            }
        }
        e.wave = 26;
        assert_eq!(e.wall_tex(1, 1, 1), T_SECTOR_SURFACE);
    }

    #[test]
    fn all_boss_reward_guns_emit_unique_player_visuals_without_changing_element_semantics() {
        let mut e=arena();
        for slot in 8..WEP_N {
            for en in &mut e.ents {en.kind=EK_NONE;}
            e.weapon=slot as i32; e.mag[slot]=MAG_SZ[slot]; e.cooldown=0.0; e.reload_t=0.0;
            e.fire(); e.flush_fx();
            let travel: Vec<_>=e.ents.iter().filter(|en|matches!(en.kind,EK_RAY|EK_PROJ|EK_BOLT|EK_FLAME)).collect();
            assert!(!travel.is_empty(),"slot {slot} needs its own travel presentation");
            for en in travel {assert_eq!(sprite_style(en).0,T_BOSS_PROJECTILE+slot-8);}
        }
    }

    #[test]
    fn swarm_launches_five_visible_homing_drones_and_keeps_allied_collision() {
        let mut e=arena();
        let target=e.spawn_with_skin(EK_HUSK,SKIN_RIFLEMAN,12.5,6.5).unwrap();
        e.ents[target].timer=10.0;
        e.weapon=23; e.mag[23]=12; e.fire();
        let drones:Vec<_>=e.ents.iter().enumerate().filter(|(_,en)|en.kind==EK_PROJ).map(|(i,_)|i).collect();
        assert_eq!(drones.len(),5);
        assert_eq!(drones.iter().map(|&i|e.ents[i].hp).sum::<i32>(),66,"more visible drones preserve volley damage");
        let before=e.ents[drones[0]].vy; let hp=e.health;
        for &i in &drones { assert_eq!(e.ents[i].effect_tick,6.0); assert_eq!(sprite_style(&e.ents[i]).0,T_BOSS_PROJECTILE+15); }
        e.tick(0.05);
        assert_ne!(e.ents[drones[0]].vy,before,"swarm must steer toward a hostile");
        assert_eq!(e.health,hp,"allied swarm cannot damage its shooter");
    }

    #[test]
    fn enemy_attacks_use_isolated_type_specific_projectiles() {
        let mut e = arena();
        for skin in [0, 1, 3, 4, 7, 8, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 28, 29, 30, 31, 32, 33, 34, 35, 36] {
            for ent in &mut e.ents { ent.kind = EK_NONE; }
            let shooter = e.spawn_with_skin(EK_BOSS, skin, 9.5, 4.5).unwrap();
            e.ents[shooter].aim = core::f32::consts::PI;
            e.enemy_shoot(shooter);
            let shots: Vec<_> = e.ents.iter().filter(|ent| ent.kind == EK_PROJ).collect();
            assert!(!shots.is_empty(), "skin {skin} did not shoot");
            for shot in shots {
                let (texture, _, sheet, _) = sprite_style(shot);
                assert_eq!(texture, T_ENEMY_PROJECTILE + skin as usize, "skin {skin} shares or mixes another projectile");
                assert!(!sheet, "enemy shots must sample one isolated complete image");
            }
        }
    }

    #[test]
    fn new_boss_attacks_have_distinct_summon_phase_and_shield_behaviors() {
        let mut e = arena();
        let brood = e.spawn_with_skin(EK_BOSS, 27, 9.5, 4.5).unwrap();
        e.enemy_shoot(brood);
        assert_eq!(e.ents.iter().filter(|ent| ent.kind == EK_MARTYR).count(), 3);
        let umbra = e.spawn_with_skin(EK_BOSS, 31, 12.5, 6.5).unwrap();
        let before = (e.ents[umbra].x, e.ents[umbra].y);
        e.enemy_shoot(umbra);
        assert_ne!((e.ents[umbra].x, e.ents[umbra].y), before);
        assert!(e.ents.iter().any(|ent| ent.kind == EK_PROJ && ent.skin == 131));
        let boreas = e.spawn_with_skin(EK_BOSS, 35, 15.5, 7.5).unwrap();
        e.boss_phase = 1;
        e.enemy_shoot(boreas);
        assert!(e.ents[boreas].shield_hp >= 35);
        assert!(e.ents.iter().any(|ent| ent.kind == EK_PROJ && ent.skin == 135));
    }

    #[test]
    fn echo_reward_unlocks_fires_and_survives_save() {
        let mut e = arena();
        assert!(!e.has_w(17));
        e.pickup(EK_GUN19);
        assert!(e.has_w(17));
        assert_eq!(e.weapon, 18);
        assert_eq!(e.mag[18], MAG_SZ[18]);
        e.pa = 0.0;
        e.state = 0;
        e.cooldown = 0.0;
        let target = e.spawn(EK_BRUTE, 6.5, 4.5).unwrap();
        let before = e.ents[target].hp;
        e.fire();
        assert!(e.ents[target].hp < before);
        assert_eq!(e.mag[18], MAG_SZ[18] - 1);
        let save = capture_save(&e);
        let mut restored = arena();
        apply_save(&mut restored, &save);
        assert!(restored.has_w(17));
        assert_eq!(restored.mag[18], e.mag[18]);
        assert_eq!(restored.weapon, 18);
    }

    #[test]
    fn run_save_layout_matches_ts_side() {
        // Pinned against SAVE_SIZE/SAVE_AMMO_BASE/SAVE_MAG_BASE in
        // src/game/save-abi.ts. Update both files together when WEP_N changes.
        assert_eq!(core::mem::size_of::<RunSave>(), 300);
        let s = RunSave {
            wave: 0, health: 0, armor: 0, weapon: 0, flags: 0, kills: 0,
            secrets: 0, elapsed_ms: 0, ammo: [0; WEP_N], mag: [0; WEP_N], extra_weapons: 0,
        };
        let base = &s as *const RunSave as usize;
        assert_eq!(&s.ammo as *const _ as usize - base, 32);
        assert_eq!(&s.mag as *const _ as usize - base, 164);
    }

    #[test]
    fn smoke_cloud_fills_then_clears_after_ten_seconds() {
        let mut e = arena();
        e.spawn_smoke_cloud(e.px, e.py);
        e.rebuild_smoke_grid();
        assert!(e.smoke_grid.iter().any(|d| *d > 0.2), "a fresh cloud must occupy the cell");
        assert!(e.smoke_grid.iter().all(|d| *d <= 0.56), "thin smoke must never block the view");
        for _ in 0..700 { e.age_smoke(1.0 / 60.0); }
        assert!(e.smokes.iter().all(|s| s.age < 0.0), "smoke must be gone after 10 seconds");
    }

    #[test]
    fn next_wave_stays_within_entity_budget() {
        let mut e = Engine::new(160, 100);
        for _ in 0..10 { e.next_wave(); }
        assert_eq!(e.wave, 11);
        assert!(map::living_hostiles(&e) <= 60, "wave spawn must leave FX slots free");
        e.maybe_spawn_boss();
        let boss = e.ents.iter().find(|x| x.kind == EK_BOSS).expect("boss spawns");
        assert!(boss.hp <= 6000, "late-wave boss must stay killable");
    }

    #[test]
    fn deep_waves_scale_hostile_health() {
        let mut e = arena();
        e.wave = 25;
        let base = e.spawn(EK_HUSK, 6.5, 4.5).unwrap();
        let base_hp = e.ents[base].hp;
        e.wave = 26;
        let next = e.spawn(EK_HUSK, 7.5, 4.5).unwrap();
        assert!(e.ents[next].hp > base_hp);
        assert!(e.ents[next].hp <= (base_hp as f32 * 1.06).ceil() as i32);
        e.wave = 1000;
        let deep = e.spawn(EK_HUSK, 8.5, 4.5).unwrap();
        assert!(e.ents[deep].hp > base_hp * 20);
    }

    #[test]
    fn waves_are_endless_and_keep_scaling() {
        let mut e = Engine::new(160, 100);
        e.wave = 25;
        e.next_wave();
        assert_eq!(e.wave, 26);
        assert_eq!(map::level_index(e.wave), 0);
        assert!(map::living_hostiles(&e) <= campaign::ACTIVE_HOSTILES);
        let hp = e.boss_max_health();
        e.wave = 999;
        e.next_wave();
        assert_eq!(e.wave, 1000);
        assert!(e.boss_max_health() > 6000 && e.boss_max_health() > hp);
        assert!(e.pending_hostiles > 0, "uncapped total is queued within the active budget");
        assert!(map::living_hostiles(&e) <= campaign::ACTIVE_HOSTILES);
    }

    #[test]
    fn warden_fires_one_round_and_blast_respects_wall() {
        let mut e = arena();
        e.weapon = 4;
        e.mag[4] = MAG_SZ[4];
        e.pa = 0.0;
        e.set_cell(7, 4, 1);
        let exposed = e.spawn(EK_BRUTE, 6.5, 5.5).unwrap();
        let covered = e.spawn(EK_BRUTE, 8.5, 4.5).unwrap();
        for i in [exposed, covered] { e.ents[i].stun = 5.0; }
        e.fire();
        e.fire();
        assert_eq!(e.mag[4], MAG_SZ[4] - 1, "cooldown must prevent a second shot");
        assert_eq!(e.ents.iter().filter(|p| p.kind == EK_BOLT).count(), 1);
        for _ in 0..24 { e.tick(1.0 / 60.0); }
        assert!(e.ents[exposed].hp < 78);
        assert_eq!(e.ents[covered].hp, 78);
        assert!(!e.ents.iter().any(|p| p.kind == EK_FIREPATCH), "missiles no longer leave floating flame patches");
        assert!(e.ents.iter().any(|p| p.kind == EK_IMPACT && p.effect_tick == 2.0));
    }

    #[test]
    fn empty_magazine_waits_for_manual_reload_without_dry_fire_animation() {
        let mut e = arena();
        e.mag[0] = 0;
        e.ammo[0] = 9;
        e.tick(1.0 / 60.0);
        assert_eq!(e.hud.weap_frame, 3, "reserve ammo keeps the empty-mag pose");
        e.fire();
        assert_eq!(e.reload_t, 0.0);
        e.tick(1.0 / 60.0);
        assert_eq!(e.hud.weap_frame, 3, "an empty trigger pull keeps the empty-mag pose");
        assert_eq!(e.events & EV_EMPTY, 0);
        e.begin_reload();
        assert!(e.reload_t > 0.0, "reload remains available on explicit input");

        e.reload_t = 0.0;
        e.ammo[0] = 0;
        e.tick(1.0 / 60.0);
        assert_eq!(e.hud.weap_frame, 4, "zero magazine and reserve use no-ammo pose");
    }

    #[test]
    fn empty_trigger_repeats_at_bounded_cadence_without_firing_animation() {
        let mut e = arena();
        e.mag[0] = 0;
        e.bits = IN_FIRE;
        let mut clicks = 0;
        for _ in 0..60 {
            e.tick(1.0 / 60.0);
            clicks += e.sound_cues.iter().filter(|c| c.kind == 21.0).count();
            assert_eq!(e.events & EV_FIRE, 0);
            assert_eq!(e.hud.weap_frame, 3);
        }
        assert!((4..=5).contains(&clicks), "empty trigger should click about 4.5 times/sec: {clicks}");
        e.bits = 0;
        for _ in 0..30 {
            e.tick(1.0 / 60.0);
            assert!(!e.sound_cues.iter().any(|c| c.kind == 21.0));
        }
        assert_eq!(e.mag[0], 0);
    }

    #[test]
    fn casings_tumble_bounce_sound_then_settle_and_expire() {
        let mut e = arena();
        e.eject_casing(); e.flush_fx();
        let i = e.ents.iter().position(|p| p.kind == EK_SPARK && p.effect_tick == 5.0).unwrap();
        assert_eq!(sprite_style(&e.ents[i]).0, T_CASING);
        let initial = e.ents[i].zoff;
        e.tick(1.0/60.0);
        assert!(e.ents[i].zoff < initial, "initial ejection rises");
        let mut clicks = 0;
        for _ in 0..180 {
            e.tick(1.0/60.0);
            clicks += e.sound_cues.iter().filter(|c| c.kind == 23.0).count();
            assert!(!e.circle_blocked(e.ents[i].x, e.ents[i].y, 0.04));
        }
        assert!((1..=3).contains(&clicks), "landing cues are physical and bounded: {clicks}");
        assert_eq!(e.ents[i].shield, 3);
        assert_eq!(e.ents[i].vx, 0.0); assert_eq!(e.ents[i].vy, 0.0);
        assert_eq!(sprite_style(&e.ents[i]).3, 0, "settled casing stops tumbling");
        for _ in 0..200 { e.tick(1.0/60.0); }
        assert_eq!(e.ents[i].kind, EK_NONE);
    }

    #[test]
    fn cosmetic_casings_are_capped_and_shotgun_uses_shell_landing_cue() {
        let mut e = arena(); e.weapon = 1;
        for _ in 0..40 { e.eject_casing(); e.flush_fx(); }
        assert_eq!(e.ents.iter().filter(|p| p.kind == EK_SPARK && p.effect_tick == 5.0).count(), 24);
        let mut shell_sound = false;
        for _ in 0..120 {
            e.tick(1.0/60.0);
            shell_sound |= e.sound_cues.iter().any(|c| c.kind == 23.0 && c.variant == 1.0);
        }
        assert!(shell_sound);
    }

    #[test]
    fn br12_reload_feeds_one_shell_per_step() {
        let mut e = arena();
        e.weapon = 1;
        e.set_w(0);
        e.mag[1] = 2;
        e.ammo[1] = 3;
        e.begin_reload();
        assert!((e.reload_t - 0.42).abs() < 0.001);
        for expected in 3..=5 {
            while e.mag[1] < expected { e.tick(1.0 / 60.0); }
            assert_eq!(e.mag[1], expected);
            assert_eq!(e.ammo[1], 5 - expected);
            assert_eq!(e.reload_t > 0.0, expected < 5);
        }
    }

    #[test]
    fn br12_trigger_interrupts_shell_feed_after_a_shell_is_loaded() {
        let mut e = arena();
        e.weapon = 1;
        e.set_w(0);
        e.mag[1] = 0;
        e.ammo[1] = 3;
        e.begin_reload();
        e.fire();
        assert_eq!(e.mag[1], 0, "empty tube cannot interrupt the first shell");
        assert!(e.reload_t > 0.0);
        while e.mag[1] == 0 { e.tick(1.0 / 60.0); }
        assert!(e.reload_t > 0.0, "the next shell is being loaded");
        e.fire();
        assert_eq!(e.mag[1], 0, "loaded shell fires");
        assert_eq!(e.reload_t, 0.0, "trigger cancels the shell-feed sequence");
        assert_ne!(e.events & EV_FIRE, 0);
    }

    #[test]
    fn player_missile_smoke_trail_expires_within_one_second() {
        let mut e = arena();
        e.weapon = 4;
        e.mag[4] = 1;
        e.fire();
        e.tick(0.1);
        assert!(e.ents.iter().any(|en| en.kind == EK_SMOKE), "missile leaves a small puff");
        for en in &mut e.ents { if en.kind == EK_BOLT { en.kind = EK_NONE; } }
        for _ in 0..70 { e.tick(1.0 / 60.0); }
        assert!(!e.ents.iter().any(|en| en.kind == EK_SMOKE), "trail disappears after flight");
    }

    #[test]
    fn projectile_sweep_hits_between_steps() {
        let mut e = arena();
        let i = e.spawn(EK_PROJ, 3.5, 4.5).unwrap();
        e.ents[i].vx = 30.0;
        e.ents[i].hp = 24;
        e.tick(0.08);
        assert_eq!(e.health, 76, "swept hit must use projectile damage");
        assert_eq!(e.ents[i].kind, EK_NONE);
    }

    #[test]
    fn effects_keep_their_identity_over_lifetime() {
        let mut e = arena();
        for (kind, texture, frame) in [(EK_SMOKE, T_FLAME, 1), (EK_FIREPATCH, T_FLAME, 0), (EK_BOLT, T_PLAYER_MISSILE, 0)] {
            let i = e.spawn(kind, 5.5, 5.5).unwrap();
            for age in [0.0, 0.3, 0.7, 1.1] {
                e.ents[i].frame = age;
                let style = sprite_style(&e.ents[i]);
                assert_eq!((style.0, style.3), (texture, frame));
            }
        }
    }

    #[test]
    fn diagonal_motion_slides_along_wall() {
        let mut e = arena();
        e.set_cell(5, 4, 1);
        e.px = 4.75;
        e.try_move(5.0, 4.6);
        assert!((e.py - 4.6).abs() < 0.001, "wall must not stop tangential motion");
        assert!(!e.circle_blocked(e.px, e.py, e.pr));
    }

    #[test]
    fn blast_respects_cover_and_preserves_pickups() {
        let mut e = arena();
        e.set_cell(5, 4, 1);
        e.px = 6.2;
        let enemy = e.spawn(EK_HUSK, 6.2, 4.5).unwrap();
        let hp = e.ents[enemy].hp;
        let med = e.spawn(EK_MED, 4.4, 4.5).unwrap();
        e.explode(4.5, 4.5, 3.0, 80.0);
        assert_eq!(e.health, 100, "cover must protect player");
        assert_eq!(e.ents[enemy].hp, hp, "cover must protect enemies");
        assert_eq!(e.ents[med].kind, EK_MED, "explosions must not erase supplies");
    }

    #[test]
    fn full_health_does_not_consume_medkit() {
        let mut e = arena();
        let med = e.spawn(EK_MED, e.px, e.py).unwrap();
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents[med].kind, EK_MED);
        e.health = 70;
        e.tick(1.0 / 60.0);
        assert_eq!(e.health, 100);
        assert_eq!(e.ents[med].kind, EK_NONE);
    }

    #[test]
    fn restart_keeps_uploaded_art() {
        hs_init(160, 100);
        eng().tex[123] = 0x12345678;
        eng().health = 12;
        hs_restart();
        assert_eq!(eng().tex[123], 0x12345678);
        assert_eq!(eng().health, 100);
    }

    #[test]
    fn strafing_and_diagonal_speed_match_camera() {
        let mut e = arena();
        e.bits = IN_A;
        e.tick(1.0 / 60.0);
        assert!(e.py < 4.5);
        e.bits = IN_D;
        e.tick(1.0 / 60.0);
        assert!((e.py - 4.5).abs() < 0.001);
        e.bits = IN_W | IN_D;
        e.tick(1.0 / 60.0);
        let distance = ((e.px - 4.5).powi(2) + (e.py - 4.5).powi(2)).sqrt();
        assert!((distance - 3.35 / 60.0).abs() < 0.001);
    }

    #[test]
    fn strafing_enemy_stays_out_of_walls() {
        let mut e = arena();
        e.px = 4.3;
        e.py = 6.5;
        e.set_cell(3, 4, 1);
        let i = e.spawn(EK_WRAITH, 4.3, 4.5).unwrap();
        for _ in 0..20 { e.tick(1.0 / 60.0); }
        let enemy = e.ents[i];
        assert!(!e.circle_blocked(enemy.x, enemy.y, enemy.radius));
    }

    #[test]
    fn mipmaps_average_texture_detail_and_render_at_extreme_pitch() {
        let mut e = arena();
        for y in 0..TEX {
            for x in 0..TEX {
                e.tex[y * TEX + x] = if (x + y) % 2 == 0 { 0xffffffff } else { 0xff000000 };
            }
        }
        e.rebuild_mipmaps();
        assert_eq!(e.sample_lod(0, 0, 0, 2.0) & 0xffffff, 0x7f7f7f);
        for pitch in [-0.42, 0.0, 0.42] {
            e.pitch = pitch;
            e.render();
            assert!(e.fb.iter().any(|c| c & 0xffffff != 0));
        }
    }

    #[test]
    fn shotgun_has_close_range_damage_and_stagger() {
        let mut e = arena();
        e.weapon = 1;
        let near = e.spawn(EK_BRUTE, 6.5, 4.5).unwrap();
        assert!(e.hitscan(0.0, 7, 11.0));
        let close_damage = 78 - e.ents[near].hp;
        assert!(e.ents[near].stun >= 0.3);
        e.ents[near].kind = EK_NONE;
        let far = e.spawn(EK_BRUTE, 12.5, 4.5).unwrap();
        assert!(e.hitscan(0.0, 7, 11.0));
        assert!(78 - e.ents[far].hp < close_damage);
        assert_eq!(e.ents[far].stun, 0.0);
    }

    #[test]
    fn ripper_spread_builds_and_recovers() {
        let mut e = arena();
        e.weapon = 2;
        e.mag[2] = 32;
        e.bits = IN_FIRE;
        for _ in 0..40 { e.tick(1.0 / 60.0); }
        assert!(e.spread > 0.12);
        e.bits = 0;
        for _ in 0..65 { e.tick(1.0 / 60.0); }
        assert_eq!(e.spread, 0.0);
    }

    #[test]
    fn lance_penetrates_enemies_but_not_cover() {
        let mut e = arena();
        let mut ids = Vec::new();
        for x in [6.5, 8.5, 10.5, 12.5] {
            let i = e.spawn(EK_BRUTE, x, 4.5).unwrap();
            e.ents[i].hp = 500;
            ids.push(i);
        }
        e.set_cell(9, 4, 1);
        e.fire_lance();
        assert_eq!(e.ents[ids[0]].hp, 340);
        assert_eq!(e.ents[ids[1]].hp, 388);
        assert_eq!(e.ents[ids[2]].hp, 500);
        e.set_cell(9, 4, 0);
        e.fire_lance();
        assert_eq!(e.ents[ids[2]].hp, 422);
        assert_eq!(e.ents[ids[3]].hp, 500, "three-enemy penetration cap");
    }

    #[test]
    fn pyre_patches_burn_expire_merge_and_stay_bounded() {
        let mut e = arena();
        let target = e.spawn(EK_HUSK, 5.0, 4.5).unwrap();
        e.ents[target].hp = 500;
        e.ignite(4.5, 4.5);
        e.ignite(4.6, 4.5);
        assert_eq!(e.ents.iter().filter(|e| e.kind == EK_FIREPATCH).count(), 1);
        for _ in 0..30 { e.tick(1.0 / 60.0); }
        assert!(e.ents[target].hp <= 490);
        for _ in 0..120 { e.tick(1.0 / 60.0); }
        assert_eq!(e.ents.iter().filter(|en| en.kind == EK_FIREPATCH).count(), 1, "the trail is still burning at two seconds");
        for _ in 0..500 { e.tick(1.0 / 60.0); }
        assert_eq!(e.ents.iter().filter(|en| en.kind == EK_FIREPATCH).count(), 0, "the trail is gone after ten seconds");
        for x in 1..30 { e.ignite(x as f32, 10.5); }
        assert_eq!(e.ents.iter().filter(|e| e.kind == EK_FIREPATCH).count(), 12);
    }

    #[test]
    fn lighting_is_colored_occluded_and_does_not_advance_simulation() {
        let mut e = arena();
        e.spawn(EK_LAMP, 4.5, 4.5);
        for y in 0..MAP_H { e.set_cell(5, y as i32, 1); }
        let rng = e.rng;
        e.update_lighting();
        let warm = e.light_at(4.5, 4.5);
        assert!(warm[0] > warm[1] && warm[1] > warm[2]);
        let covered = e.light_at(6.5, 4.5);
        assert!(covered[0] <= 0.0);
        e.render();
        assert_eq!(e.rng, rng);
        assert_eq!(e.elapsed, 0.0);
    }

    #[test]
    fn horizontal_planes_preserve_wall_pixels() {
        let mut e = arena();
        for y in 0..MAP_H { e.set_cell(8, y as i32, 2); }
        let wall = e.wall_tex(2, 8, 4);
        e.tex[wall * TEX * TEX..(wall + 1) * TEX * TEX].fill(Engine::pack(200, 0, 0, 255));
        e.tex[T_SKULL * TEX * TEX..(T_SKULL + 1) * TEX * TEX].fill(Engine::pack(200, 0, 0, 255));
        e.rebuild_mipmaps();
        e.render();
        let middle = e.fb[e.h / 2 * e.w + e.w / 2];
        assert!(middle & 255 > 40);
        assert_eq!((middle >> 16) & 255, 0);
        assert!(e.fb.iter().all(|c| *c != 0));
    }

    #[test]
    fn armor_soaks_two_thirds_then_iframes_and_death_stick() {
        let mut e = arena();
        e.armor = 30;
        e.damage_player(15);
        assert_eq!(e.armor, 20, "armor soaks two thirds, capped by the vest");
        assert_eq!(e.health, 95);
        assert!(e.events & EV_HURT != 0);
        e.damage_player(40);
        assert_eq!(e.health, 95, "iframes must swallow the follow-up");
        assert_eq!(e.armor, 20);
        e.iframes = 0.0;
        e.armor = 0;
        e.damage_player(200);
        assert_eq!(e.health, 0);
        assert_eq!(e.state, 1);
        assert!(e.events & EV_DIE != 0);
        e.damage_player(20);
        assert_eq!(e.health, 0, "a downed player takes no further damage");
        assert_eq!(e.state, 1);
    }

    #[test]
    fn reload_draws_reserve_and_blocks_the_trigger() {
        let mut e = arena();
        e.mag[0] = MAG_SZ[0];
        e.ammo[0] = 4;
        e.bits = IN_RELOAD;
        e.tick(1.0 / 60.0);
        assert_eq!(e.reload_t, 0.0, "a full magazine must not reload");
        assert_eq!(e.ammo[0], 4);

        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.mag[0] = 2;
        e.ammo[0] = 0;
        e.bits = IN_RELOAD;
        e.tick(1.0 / 60.0);
        assert_eq!(e.reload_t, 0.0, "an empty reserve must not start a reload");

        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.ammo[0] = 5;
        e.bits = IN_RELOAD;
        e.tick(1.0 / 60.0);
        assert!((e.reload_t - RELOAD_T[0]).abs() < 0.001);
        assert!(e.events & EV_RELOAD != 0);
        e.bits = IN_FIRE;
        e.tick(1.0 / 60.0);
        assert_eq!(e.mag[0], 2, "the trigger is dead while reloading");
        e.bits = 0;
        while e.reload_t > 0.0 {
            e.tick(1.0 / 60.0);
        }
        assert_eq!(e.mag[0], 7);
        assert_eq!(e.ammo[0], 0);

        e.mag[0] = 10;
        e.ammo[0] = 1;
        e.begin_reload();
        while e.reload_t > 0.0 {
            e.tick(1.0 / 60.0);
        }
        assert_eq!(e.mag[0], 11, "reload takes only what the reserve holds");
        assert_eq!(e.ammo[0], 0);

        e.mag[0] = 0;
        e.bits = IN_FIRE;
        e.tick(1.0 / 60.0);
        assert_eq!(e.events & EV_EMPTY, 0);
        assert_eq!(e.reload_t, 0.0);
        assert_eq!(e.mag[0], 0);
    }

    #[test]
    fn held_weapon_key_does_not_cancel_reload() {
        let mut e = arena();
        e.set_w(0);
        e.mag[1] = MAG_SZ[1];
        e.ammo[1] = 6;
        e.bits = IN_W2;
        e.tick(1.0 / 60.0);
        assert_eq!(e.weapon, 1);
        e.mag[1] = 1;
        e.begin_reload();
        e.bits = IN_W2;
        e.tick(1.0 / 60.0);
        assert_eq!(e.weapon, 1);
        assert!(e.reload_t > 0.0, "a held weapon key must not restart the swap");
        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.bits = IN_W1;
        e.tick(1.0 / 60.0);
        assert_eq!(e.weapon, 0);
        assert_eq!(e.reload_t, 0.0, "an actual swap cancels the reload");
        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.bits = IN_W3;
        e.tick(1.0 / 60.0);
        assert_eq!(e.weapon, 0, "a locked gun must not switch");
    }

    #[test]
    fn number_keys_select_every_available_weapon_once_per_press() {
        let mut e = arena();
        e.set_w(0);
        e.set_w(1);
        e.set_w(2);
        e.set_w(3);
        e.set_w(4);
        e.set_w(5);
        e.set_w(6);
        for (expected, bit) in [IN_W1, IN_W2, IN_W3, IN_W4, IN_W5, IN_W6, IN_W7, IN_W8]
            .into_iter()
            .enumerate()
        {
            e.bits = 0;
            e.tick(1.0 / 60.0);
            e.bits = bit;
            e.tick(1.0 / 60.0);
            assert_eq!(e.weapon, expected as i32, "key {} selects its weapon", expected + 1);
        }
    }

    #[test]
    fn arc_and_rotary_stay_locked_until_their_cases_are_picked_up() {
        let mut e = arena();
        for bit in [IN_W6, IN_W7, IN_W8] {
            e.bits = bit;
            e.tick(1.0 / 60.0);
            assert_eq!(e.weapon, 0, "locked late-game weapons must not switch");
            e.bits = 0;
            e.tick(1.0 / 60.0);
        }
        e.pickup(EK_GUN6);
        assert!(e.has_w(4));
        assert_eq!(e.weapon, 5);
        e.pickup(EK_GUN7);
        assert!(e.has_w(5));
        assert_eq!(e.weapon, 6);
        e.pickup(EK_GUN8);
        assert!(e.has_w(6));
        assert_eq!(e.weapon, 7);
    }

    #[test]
    fn approach_opens_doors_but_secrets_need_use() {
        let mut e = arena();
        e.pa = 0.0;
        e.set_cell(5, 4, 8);
        e.set_cell(5, 3, 9);
        e.tick(1.0 / 60.0);
        assert!(e.door[4 * MAP_W + 5] > 0.0, "a door opens on approach");
        assert_eq!(e.door[3 * MAP_W + 5], 0.0, "a secret stays shut");
        assert_eq!(e.secrets, 0);
        e.bits = IN_USE;
        e.tick(1.0 / 60.0);
        assert!(e.door[3 * MAP_W + 5] > 0.0);
        assert_eq!(e.secrets, 1);
        e.tick(1.0 / 60.0);
        assert_eq!(e.secrets, 1, "holding use must not recount the secret");
        assert_eq!(e.events & EV_BOSS_VOICE, 0, "doors never play a boss voice");
        assert_ne!(e.radio_line, 5, "secret doors never announce a voice line");
    }

    #[test]
    fn node_and_terminal_announcements_fire_once_per_interactable() {
        let mut e = arena();
        let (nx, ny) = field::node_point(e.wave);
        (e.px, e.py) = (nx, ny);
        e.use_field();
        let node_seq = e.radio_seq;
        assert_eq!(e.events & EV_BOSS_VOICE, 0, "the node cannot trigger a boss death voice");
        e.use_field();
        assert_eq!(e.radio_seq, node_seq, "the same node cannot speak twice");

        let (tx, ty) = field::terminals(e.wave)[0];
        let terminal = e.spawn(EK_TERMINAL, tx, ty).unwrap();
        (e.px, e.py) = (tx, ty);
        e.use_field();
        let terminal_seq = e.radio_seq;
        assert_eq!(e.events & EV_BOSS_VOICE, 0, "the terminal cannot trigger a boss death voice");
        assert!(terminal_seq > node_seq);
        assert!(e.ents[terminal].timer < 0.0);
        e.use_field();
        assert_eq!(e.radio_seq, terminal_seq, "the same terminal cannot speak twice");
    }

    #[test]
    fn hitscan_stops_at_cover() {
        let mut e = arena();
        e.pa = 0.0;
        let behind = e.spawn(EK_HUSK, 8.5, 4.5).unwrap();
        let hp = e.ents[behind].hp;
        e.set_cell(6, 4, 1);
        assert!(!e.hitscan(0.0, 15, 22.0));
        assert_eq!(e.ents[behind].hp, hp);
        e.set_cell(6, 4, 0);
        assert!(e.hitscan(0.0, 15, 22.0));
        assert_eq!(e.ents[behind].hp, hp - 15);
    }

    #[test]
    fn boss_death_wins_and_barrels_chain_without_kills() {
        let mut e = arena();
        let boss = e.spawn(EK_BOSS, 6.5, 4.5).unwrap();
        e.ents[boss].hp = 1;
        e.hurt_ent(boss, 5, e.px, e.py);
        assert_ne!(e.events & EV_BOSS_VOICE, 0, "boss death emits one voice event");
        assert_eq!(e.ents[boss].anim, ANIM_DEAD);
        assert_eq!(e.state, 0, "the boss drops a weapon instead of ending the sector");
        assert!(e.ents.iter().any(|en| field::is_boss_case(en.kind)));
        assert_eq!(e.kills, 1);
        e.tick(1.0 / 60.0);
        assert_eq!(e.events & EV_BOSS_VOICE, 0, "voice event clears on the next tick");
        for _ in 1..90 { e.tick(1.0 / 60.0); }
        assert_eq!(e.ents[boss].kind, EK_NONE);

        let mut e = arena();
        let a = e.spawn(EK_BARREL, 5.0, 4.5).unwrap();
        let b = e.spawn(EK_BARREL, 6.2, 4.5).unwrap();
        e.ents[a].hp = 1;
        e.ents[b].hp = 1;
        e.hurt_ent(a, 5, 4.0, 4.5);
        assert_eq!(e.ents[a].anim, ANIM_DEAD);
        assert_eq!(e.ents[b].anim, ANIM_DEAD, "a barrel blast chains");
        assert_eq!(e.kills, 0, "props are not kills");
        assert_eq!(e.state, 0);
        assert!(e.health > 0, "a single chain must not delete the player");
        for _ in 0..45 { e.tick(1.0 / 60.0); }
        assert_eq!(e.ents[a].kind, EK_NONE);
        assert_eq!(e.ents[b].kind, EK_NONE);
    }

    #[test]
    fn boss_kill_ends_the_level_and_next_wave_restarts_the_hunt() {
        let mut e = arena();
        let boss = e.spawn(EK_BOSS, 6.5, 4.5).unwrap();
        e.ents[boss].hp = 1;
        e.hurt_ent(boss, 5, e.px, e.py);
        assert_eq!(e.state, 0, "killing the boss drops the case");
        let case = e.ents.iter().position(|en| field::is_boss_case(en.kind)).unwrap();
        e.pickup(e.ents[case].kind);
        assert_eq!(e.state, 2, "taking the boss weapon ends the level");
        assert!(e.has_w(7));
        e.next_wave();
        assert_eq!((e.state, e.wave), (0, 2));
        assert!(!e.boss_spawned && !e.hell);
        assert!(map::living_hostiles(&e) > 0, "the next level spawns its cast");
    }

    #[test]
    fn lockdown_seal_ejects_player_from_doorway() {
        let mut e = Engine::new(160, 100);
        assert_eq!(e.wave, 1);
        let (dx, dy) = field::lockdown_doors(1)[0];
        assert_eq!(e.cell(dx, dy), 8, "the vault leaf must be a door cell");
        let idx = dy as usize * MAP_W + dx as usize;
        e.door[idx] = 1.0;
        e.px = dx as f32 + 0.5;
        e.py = dy as f32 + 0.5;
        assert!(!e.circle_blocked(e.px, e.py, e.pr), "open doorway starts walkable");
        e.seal_lockdown();
        assert!(!e.circle_blocked(e.px, e.py, e.pr), "the seal must eject, not trap");
    }

    #[test]
    fn embedded_player_is_relocated_by_tick() {
        let mut e = arena();
        e.set_cell(5, 4, 1);
        e.px = 5.5;
        e.py = 4.5;
        assert!(e.circle_blocked(e.px, e.py, e.pr));
        e.tick(1.0 / 60.0);
        assert!(!e.circle_blocked(e.px, e.py, e.pr), "a tick must unstick the player");
    }

    #[test]
    fn next_wave_restores_the_body_and_keeps_found_gear() {
        let mut e = arena();
        e.health = 11;
        e.state = 1;
        e.set_w(2);
        e.weapon = 3;
        e.ammo[3] = 1;
        e.mag[3] = 0;
        let _med = e.spawn(EK_MED, 8.5, 8.5).unwrap();
        e.spawn(EK_HUSK, 9.5, 8.5).unwrap();
        e.next_wave();
        assert_eq!(e.wave, 2);
        assert_eq!(e.health, 100);
        assert_eq!(e.state, 0);
        assert!(e.has_w(2));
        assert_eq!(e.weapon, 3, "a found gun stays in hand");
        assert_eq!(e.mag[3], MAG_SZ[3]);
        assert!(e.ammo[3] >= 8);
        assert!(e.ents.iter().any(|en| en.kind == EK_MED), "the new sector supplies are placed");
        assert_eq!(
            e.ents.iter().filter(|en| is_hostile_kind(en.kind)).count(),
            map::hostiles(2).len() * 2,
            "the previous cast is cleared, then two copies of the sector roster spawn",
        );
        e.wave = 12;
        e.next_wave();
        assert_eq!(e.wave, 13, "endless waves climb past 12");
        assert!(e.ents.iter().any(|en| en.kind == EK_MED));
    }

    /// Pulls every `reserve: <int>` out of the TypeScript arsenal table. Kept
    /// dependency-free (no regex crate) because this only runs in tests.
    fn ts_reserve_caps(ts: &str) -> Vec<i32> {
        let mut out = Vec::new();
        let mut rest = ts;
        while let Some(at) = rest.find("reserve: ") {
            rest = &rest[at + "reserve: ".len()..];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if digits.is_empty() { continue; }
            out.push(digits.parse().expect("numeric reserve cap"));
        }
        out
    }

    #[test]
    fn damage_records_the_bearing_of_its_source() {
        let mut e = arena();
        e.state = 0;
        e.pa = 0.0;
        e.iframes = 0.0;
        // Facing +x, so a source at +y sits 90 degrees to the left and one at
        // -y sits to the right.
        e.damage_player_from(10, e.px, e.py + 4.0);
        assert!((e.hurt_dir - core::f32::consts::FRAC_PI_2).abs() < 1e-4,
            "a source to the left should read +pi/2, got {}", e.hurt_dir);
        e.iframes = 0.0;
        e.health = 100;
        e.damage_player_from(10, e.px, e.py - 4.0);
        assert!((e.hurt_dir + core::f32::consts::FRAC_PI_2).abs() < 1e-4,
            "a source to the right should read -pi/2, got {}", e.hurt_dir);
        e.iframes = 0.0;
        e.health = 100;
        e.damage_player_from(10, e.px + 4.0, e.py);
        assert!(e.hurt_dir.abs() < 1e-4, "a source dead ahead should read 0, got {}", e.hurt_dir);
        e.iframes = 0.0;
        e.health = 100;
        e.damage_player_from(10, e.px - 4.0, e.py);
        assert!((e.hurt_dir.abs() - core::f32::consts::PI).abs() < 1e-4,
            "a source behind should read +-pi, got {}", e.hurt_dir);
        // The bearing stays wrapped into [-pi, pi] so the shader's sin/cos
        // can consume it directly.
        e.iframes = 0.0;
        e.health = 100;
        e.pa = 2.9;
        e.damage_player_from(10, e.px - 4.0, e.py + 4.0);
        assert!((-core::f32::consts::PI..=core::f32::consts::PI).contains(&e.hurt_dir),
            "hurt_dir escaped [-pi, pi]: {}", e.hurt_dir);
        assert!(e.hurt_dir.is_finite());
    }

    #[test]
    fn low_health_strain_only_appears_when_it_should() {
        let mut e = arena();
        e.state = 0;
        e.health = 100;
        for _ in 0..30 { e.tick(1.0 / 60.0); }
        assert_eq!(e.strain, 0.0, "a healthy player has no strain");

        // Cross the threshold: strain rises but stays bounded.
        e.health = 20;
        e.strain = 0.0;
        let mut peak = 0.0f32;
        for _ in 0..240 { e.tick(1.0 / 60.0); peak = peak.max(e.strain); }
        assert!(peak > 0.4, "low health must be felt, peak was {peak}");
        assert!(peak <= 1.0, "strain must stay in 0..1, peak was {peak}");
        // It breathes rather than sitting still.
        let a = e.strain;
        for _ in 0..12 { e.tick(1.0 / 60.0); }
        assert!((e.strain - a).abs() > 1e-4, "strain must pulse, it was static at {a}");

        // Healing clears it.
        e.health = 100;
        for _ in 0..180 { e.tick(1.0 / 60.0); }
        assert!(e.strain < 0.02, "strain should decay after healing, got {}", e.strain);
        // Dying clears it too, so the death card is not tinted.
        e.state = 1;
        e.health = 10;
        for _ in 0..180 { e.tick(1.0 / 60.0); }
        assert!(e.strain < 0.02, "strain must not persist into the death state");
    }

    #[test]
    fn reserve_caps_match_the_arsenal_table() {
        // `WEAPONS[].reserve` in src/components/game/data.ts documents itself as
        // mirroring the engine pickup caps. Parsed from the TS source so a
        // data-side retune cannot silently diverge from the engine.
        let ts = include_str!("../../src/components/game/data.ts");
        let expected = ts_reserve_caps(ts);
        assert_eq!(expected.len(), 19, "could not read the first 19 arsenal reserves");
        assert_eq!(&RESERVE_CAP[..19], &expected[..], "engine and TS reserve caps diverged");
        for (slot, cap) in RESERVE_CAP.iter().enumerate() {
            assert!(*cap > 0 && *cap <= MAG_SZ[slot] * 12, "slot {slot} reserve {cap} is implausible");
        }
    }

    #[test]
    fn weapon_ownership_survives_a_save_round_trip() {
        let fresh = arena();
        assert!(!fresh.has_w(0), "a fresh run owns only the starting sidearm");
        assert!(fresh.owns_slot(0), "slot 0 is always carried");
        assert!(!fresh.owns_slot(1));
        assert_eq!(capture_save(&fresh).flags, 0, "nothing outside slot 0 starts owned");
        // Every tracked flag must survive the flag word, including the last one.
        for i in 0..OWNED_GUNS {
            let mut r = arena();
            r.owned = [false; OWNED_GUNS];
            r.set_w(i);
            assert!(r.owns_slot(i + 1), "set_w({i}) did not grant slot {}", i + 1);
            let save = capture_save(&r);
            assert!(save.flags & (1 << i) != 0, "flag {i} lost its bit");
            assert_eq!(save.flags, 1 << i, "flag {i} leaked into other bits");
        }
    }

    #[test]
    fn a_hitch_cannot_outrun_the_step_cap() {
        let mut e = arena();
        e.pa = 0.0;
        e.bits = IN_W;
        e.tick(5.0);
        let dist = ((e.px - 4.5).powi(2) + (e.py - 4.5).powi(2)).sqrt();
        assert!((dist - 3.35 * 0.08).abs() < 0.001);
        assert!((e.time - 0.08).abs() < 1e-5);
        let x = e.px;
        e.tick(-3.0);
        assert!((e.time - 0.08).abs() < 1e-5, "a negative step must not rewind");
        assert!((e.px - x).abs() < 0.001);
    }

    #[test]
    fn non_finite_abi_input_cannot_poison_the_simulation() {
        // Feeding sanitised look deltas keeps every downstream transform finite.
        // `finite()` is exactly what `hs_input` applies before storing them.
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut e = arena();
            e.px = 4.5;
            e.py = 4.5;
            e.pa = 0.0;
            e.mx = finite(bad);
            e.my = finite(bad);
            e.tick(1.0 / 60.0);
            assert!(e.time.is_finite(), "time must stay finite after {bad}");
            assert!(e.px.is_finite() && e.py.is_finite(), "position must stay finite after {bad}");
            assert!(e.pa.is_finite() && e.pitch.is_finite(), "facing must stay finite after {bad}");
        }

        // A NaN step advances nothing rather than rewinding or exploding.
        let mut n = arena();
        n.pa = 0.0;
        n.tick(f32::NAN);
        assert!((n.time - 0.0).abs() < 1e-6, "a NaN step must not advance time");
        assert!(n.time.is_finite());

        // The sanitisers themselves: `hs_input` and `hs_tick` are the only doors
        // in and both route through these.
        assert_eq!(finite(f32::NAN), 0.0);
        assert_eq!(finite(f32::INFINITY), 0.0);
        assert_eq!(finite(f32::NEG_INFINITY), 0.0);
        assert_eq!(finite(0.5), 0.5);
        assert_eq!(finite(-2.0), -2.0);
        assert_eq!(safe_dt(f32::NAN), 0.0);
        // A non-finite step becomes a no-op rather than a full-length jump.
        assert_eq!(safe_dt(f32::INFINITY), 0.0);
        assert_eq!(safe_dt(f32::NEG_INFINITY), 0.0);
        assert_eq!(safe_dt(-1.0), 0.0);
        assert_eq!(safe_dt(0.5), 0.08);
        assert_eq!(safe_dt(1.0 / 60.0), 1.0 / 60.0);
    }

    #[test]
    fn full_armor_is_left_on_the_floor() {
        let mut e = arena();
        e.armor = 100;
        let item = e.spawn(EK_ARMOR, e.px, e.py).unwrap();
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents[item].kind, EK_ARMOR);
        assert_eq!(e.armor, 100);
        e.armor = 40;
        e.tick(1.0 / 60.0);
        assert_eq!(e.armor, 90);
        assert_eq!(e.ents[item].kind, EK_NONE);
    }

    #[test]
    fn ammo_crates_restock_the_complete_eight_weapon_arsenal() {
        let mut e = arena();
        e.set_w(0);
        e.set_w(1);
        e.set_w(2);
        e.set_w(3);
        e.set_w(4);
        e.set_w(5);
        e.set_w(6);
        e.ammo = [0; WEP_N];
        let item = e.spawn(EK_AMMO, e.px, e.py).unwrap();
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents[item].kind, EK_NONE, "a useful ammo crate is collected");
        assert_eq!(&e.ammo[..19], &[18, 10, 45, 5, 2, 15, 90, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(e.ammo[19..].iter().all(|ammo| *ammo == 0));

        e.ammo[..19].copy_from_slice(&[120, 48, 216, 20, 16, 80, 450, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let full = e.spawn(EK_AMMO, e.px, e.py).unwrap();
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents[full].kind, EK_AMMO, "a full arsenal leaves supplies available");
    }

    #[test]
    fn override_requires_use_then_spawns_the_sector_boss() {
        let mut e = arena();
        e.node_done = true;
        e.wave = 2;
        (e.px, e.py) = map::override_point(e.wave);
        e.tick(1.0 / 60.0);
        assert_eq!(e.boss_intro, 0.0, "standing on the beacon is not enough");
        e.bits = IN_USE;
        e.tick(1.0 / 60.0);
        assert!(e.boss_intro > 5.0, "USE begins the override countdown");
        e.bits = 0;
        for _ in 0..370 { e.tick(1.0 / 60.0); }
        let bosses: Vec<_> = e.ents.iter().filter(|en| en.kind == EK_BOSS).collect();
        assert_eq!(bosses.len(), 1);
        assert_eq!(bosses[0].hp, campaign::boss_health(2));
        assert_eq!(bosses[0].skin, SKIN_HECATE);
        assert!(e.boss_spawned);
        assert!(e.hell);
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents.iter().filter(|en| en.kind == EK_BOSS).count(), 1);
    }

    #[test]
    fn override_use_requires_a_clear_path_to_the_console() {
        let mut e = arena();
        e.node_done = true;
        e.wave = 1;
        e.px = 38.9;
        e.py = 21.5;
        e.set_cell(39, 21, 1);
        e.bits = IN_USE;
        e.tick(1.0 / 60.0);
        assert_eq!(e.boss_intro, 0.0);
        e.set_cell(39, 21, 0);
        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.bits = IN_USE;
        e.tick(1.0 / 60.0);
        assert!(e.boss_intro > 0.0);
    }

    #[test]
    fn boss_sequences_have_distinct_timing_and_entry_effects() {
        let mut e = arena();
        assert_eq!(e.boss_intro_duration(), 4.0);
        e.wave = 2;
        assert_eq!(e.boss_intro_duration(), 5.2);
        e.boss_intro_effect(0);
        assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.kind == EK_SPARK));
        e.fx_n = 0;
        e.wave = 3;
        assert_eq!(e.boss_intro_duration(), 4.6);
        e.boss_intro_effect(0);
        assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.kind == EK_SPARK));
        e.fx_n = 0;
        e.wave = 4;
        assert_eq!(e.boss_intro_duration(), 4.8);
        e.boss_intro_effect(0);
        assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.kind == EK_SPARK));
        e.fx_n = 0;
        e.wave = 5;
        assert_eq!(e.boss_intro_duration(), 5.0);
        e.boss_intro_effect(0);
        assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.variant == 20));
    }

    #[test]
    fn boss_health_phases_call_support_with_electrical_entry_fx() {
        let mut e = arena();
        e.boss_spawned = true;
        let boss = e.spawn_with_skin(EK_BOSS, SKIN_VEYRAN, 9.5, 4.5).unwrap();
        e.ents[boss].hp = 300;
        e.tick(1.0 / 60.0);
        assert_eq!(e.boss_phase, 1);
        e.tick(1.0 / 60.0);
        assert!(e.ents.iter().any(|x| x.kind == EK_SPARK));
        assert!(e.ents.iter().any(|x| x.skin == SKIN_HORNET && x.hp > 0));
        assert!(e.ents.iter().any(|x| x.skin == SKIN_RIFLEMAN && x.hp > 0));

        let phase_one_support = e.ents.iter().filter(|x| is_hostile_kind(x.kind) && x.kind != EK_BOSS).count();
        e.ents[boss].hp = 150;
        e.tick(1.0 / 60.0);
        assert_eq!(e.boss_phase, 2);
        assert!(e.ents.iter().any(|x| x.skin == SKIN_MARTYR && x.hp > 0));
        assert!(e.ents.iter().any(|x| x.skin == SKIN_LOADER && x.hp > 0));
        let phase_two_support = e.ents.iter().filter(|x| is_hostile_kind(x.kind) && x.kind != EK_BOSS).count();
        assert!(phase_two_support > phase_one_support);
        e.tick(1.0 / 60.0);
        assert_eq!(
            e.ents.iter().filter(|x| is_hostile_kind(x.kind) && x.kind != EK_BOSS).count(),
            phase_two_support,
            "a phase summons its support squad only once",
        );
    }

    #[test]
    fn sector_bosses_release_distinct_phase_barrages() {
        let mut foundry = arena();
        foundry.wave = 2;
        foundry.boss_spawned = true;
        let boss = foundry.spawn_with_skin(EK_BOSS, map::boss_skin(2), 9.5, 4.5).unwrap();
        foundry.ents[boss].hp = foundry.boss_max_health() / 2;
        foundry.tick(1.0 / 60.0);
        assert_eq!(foundry.ents.iter().filter(|e| e.kind == EK_PROJ).count(), 8);
        assert!(foundry.ents.iter().any(|e| e.skin == SKIN_HECATE && e.kind == EK_BOSS));

        let mut bioforge = arena();
        bioforge.wave = 3;
        bioforge.boss_spawned = true;
        let boss = bioforge.spawn_with_skin(EK_BOSS, map::boss_skin(3), 9.5, 4.5).unwrap();
        bioforge.ents[boss].hp = bioforge.boss_max_health() / 2;
        bioforge.tick(1.0 / 60.0);
        assert_eq!(bioforge.ents.iter().filter(|e| e.kind == EK_PROJ && e.effect_tick == 3.0).count(), 6);
        assert!(bioforge.ents.iter().any(|e| e.skin == SKIN_CHIMERA && e.kind == EK_BOSS));

        for (wave, skin, _hp, barrage) in [(4, SKIN_ORACLE, 1000, 10), (5, SKIN_GRAVEMIND, 1600, 12)] {
            let mut level = arena();
            level.wave = wave;
            level.boss_spawned = true;
            let boss = level.spawn_with_skin(EK_BOSS, map::boss_skin(wave), 9.5, 4.5).unwrap();
            level.ents[boss].hp = level.boss_max_health() / 2;
            level.tick(1.0 / 60.0);
            assert_eq!(level.ents.iter().filter(|e| e.kind == EK_PROJ).count(), barrage);
            assert!(level.ents.iter().any(|e| e.skin == skin && e.kind == EK_BOSS));
        }
    }

    #[test]
    fn boss_fight_reports_health_and_phase_to_the_hud() {
        let mut e = arena();
        e.boss_spawned = true;
        let boss = e.spawn_with_skin(EK_BOSS, SKIN_VEYRAN, 9.5, 4.5).unwrap();
        e.ents[boss].hp = 300;
        e.tick(1.0 / 60.0);
        assert_eq!(e.hud.boss_health, 300);
        assert_eq!(e.hud.boss_max_health, 480);
        assert_eq!(e.hud.boss_phase, 1);

        e.ents[boss].hp = 0;
        e.tick(1.0 / 60.0);
        assert_eq!((e.hud.boss_health, e.hud.boss_max_health, e.hud.boss_phase), (0, 0, 0));
    }

    #[test]
    fn hell_swaps_walls_and_keeps_override_doors_legible() {
        let mut e = arena();
        assert_eq!(e.wall_tex(2, 1, 1), T_SECTOR_SURFACE);
        assert_eq!(e.wall_tex(8, 36, 18), T_SECTOR_SURFACE + 2);
        e.hell = true;
        assert_eq!(e.wall_tex(2, 1, 1), T_BOSS_ARENA);
        assert_eq!(e.wall_tex(6, 1, 1), T_BOSS_ARENA);
        assert_eq!(e.wall_tex(7, 1, 1), T_BOSS_ARENA);
        assert_eq!(e.wall_tex(9, 1, 1), T_SECTOR_SURFACE + 1);
        assert_eq!(e.wall_tex(8, 36, 18), T_SECTOR_SURFACE + 2);
        assert_eq!(e.wall_tex(9, 36, 19), T_SECTOR_SURFACE + 1);
    }

    #[test]
    fn shade_and_mip_blend_match_the_scalar_formula() {
        let color = Engine::pack(200, 40, 10, 255);
        let gain = [(1.8 * 65536.0) as i32, (0.5 * 65536.0) as i32, (0.18 * 65536.0) as i32];
        let shaded = Engine::shade_texel(color, gain, 0x3c00_0000);
        assert_eq!(shaded & 255, 255, "channel clamp");
        assert_eq!((shaded >> 8) & 255, 20);
        assert_eq!((shaded >> 16) & 255, 1);
        assert_eq!(shaded >> 24, 0x3c);
        let batch = Engine::shade_texels4([color, 0, color, 0xff], [gain, [0; 3], gain, [65536; 3]], 0);
        assert_eq!(batch[0], Engine::shade_texel(color, gain, 0));
        assert_eq!(batch[2], batch[0]);
        assert_eq!(batch[3], Engine::shade_texel(0xff, [65536; 3], 0));

        let mut e = arena();
        e.tex[3 + 5 * TEX] = 0xff11_2233;
        e.rebuild_mipmaps();
        assert_eq!(e.sample_mip(0, 3, 5, 0, 0), e.sample(0, 3, 5));
        let a = e.sample_mip_at(0, 3, 5, 0);
        let b = e.sample_mip_at(0, 3, 5, 1);
        assert_eq!(Engine::blend_mips4([a, 0, a, 1], [b, 0, b, 2], 128)[0], Engine::blend_mips(a, b, 128));
        assert_eq!(Engine::blend_mips4([a, 0, a, 1], [b, 0, b, 2], 128)[2], Engine::blend_mips(a, b, 128));
    }

    #[test]
    fn wall_lod_stays_sharp_up_close() {
        let (level, mix) = Engine::column_lod(0.15, 500, 0.72, 640);
        assert_eq!(level, 0);
        assert_eq!(mix, 0, "a near column must match an unfiltered sample");
        let (far, _) = Engine::column_lod(16.0, 12, 0.72, 640);
        assert!(far >= 3, "a distant column must use the mip chain, got {far}");
        let (capped, _) = Engine::column_lod(80.0, 1, 0.72, 320);
        assert!(capped <= 8);
    }

    #[test]
    fn wall_shade_matches_one_pixel_at_a_time() {
        let colors = [Engine::pack(200, 10, 255, 128), 0x0102_0304, 0x00ff_ffff, 0];
        let light = [0.55, -0.4, 2.0];
        let batch = Engine::shade_rgb4(colors, 0.82, light);
        for i in 0..4 {
            assert_eq!(batch[i], Engine::shade_rgb(colors[i], 0.82, light));
        }
        let shared = [(3, 5), (259, 5), (-253, 261), (3, 5 + 512)];
        let mut e = arena();
        e.rebuild_mipmaps();
        assert_eq!(
            e.sample_mip4(0, shared, 0, 0),
            [e.sample(0, 3, 5); 4],
            "wrapped uvs that land on one texel must splat, not resample",
        );
    }

    #[test]
    fn gpu_cast_hits_a_wall_and_packs_sprites() {
        hs_init(160, 100);
        hs_prepare_gpu();
        let cols = unsafe { std::slice::from_raw_parts(hs_gpu_cols(), 160) };
        assert!(cols.iter().any(|c| c.hit > 0.5), "the enclosed map must hit a wall");
        assert!(cols.iter().all(|c| c.perp.is_finite() && c.z.is_finite()));
        assert!(hs_gpu_sprite_count() > 0);
        for angle in [0.0, 1.2, core::f32::consts::PI] {
            eng().pa = angle;
            hs_prepare_gpu();
            let sprites = unsafe { std::slice::from_raw_parts(hs_gpu_sprites(), hs_gpu_sprite_count() as usize) };
            let view = gpu_scratch().view;
            let depth = |s: &GpuSprite| (s.x - view.px) * view.dir_x + (s.y - view.py) * view.dir_y;
            assert!(sprites.windows(2).all(|pair| depth(&pair[0]) >= depth(&pair[1])),
                "GPU sprites must paint far to near regardless of camera yaw");
        }
        assert_eq!(std::mem::size_of::<GpuCol>() / 4, 16);
        assert_eq!(std::mem::size_of::<GpuView>() / 4, 16);
        assert_eq!(std::mem::size_of::<GpuSprite>() / 4, 8);
    }

    #[test]
    fn override_stays_dark_until_the_sector_node_is_used() {
        let mut e = arena();
        (e.px, e.py) = map::override_point(e.wave);
        e.bits = IN_USE;
        e.tick(1.0 / 60.0);
        assert_eq!(e.boss_intro, 0.0, "the console ignores USE before the node");
        let (x, y) = field::node_point(e.wave);
        e.px = x;
        e.py = y;
        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.bits = IN_USE;
        e.tick(1.0 / 60.0);
        assert!(e.node_done);
        assert_eq!(e.hud.objective, 1);
    }

    #[test]
    fn shield_blocks_the_front_and_takes_the_flank() {
        let mut e = arena();
        let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
        e.arm_shield(i);
        let hp = e.ents[i].hp;
        e.ents[i].face = 0.0;
        e.hurt_ent(i, 12, 10.0, 4.5);
        assert_eq!(e.ents[i].hp, hp, "the first front hit stays on the plate");
        assert!(e.ents[i].shield_hp < enemies::SHIELD_CAP, "the plate takes the hit");
        assert!(e.ents[i].bar_t > 1.0, "a clang shows the bar");
        e.hurt_ent(i, 12, 6.0, 4.5);
        assert!(e.ents[i].hp < hp, "a rear hit gets through");
    }

    #[test]
    fn pyre_leaves_a_fire_patch() {
        let mut e = arena();
        e.set_w(6);
        e.weapon = 7;
        e.mag[7] = 6;
        e.fire();
        assert!(e.ents.iter().any(|en| en.kind == EK_FIREPATCH));
        assert_eq!(e.mag[7], 5);
    }

    #[test]
    fn rifleman_and_gunner_die_from_the_front() {
        let mut e = arena();
        let rifle = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
        e.hurt_ent(rifle, 80, 6.0, 4.5);
        assert_eq!(e.ents[rifle].hp, 0, "an unshielded rifleman dies");
        let gunner = e.spawn_with_skin(EK_BRUTE, SKIN_GUNNER, 12.5, 4.5).unwrap();
        e.arm_shield(gunner);
        e.ents[gunner].face = 0.0;
        for _ in 0..40 {
            e.hurt_ent(gunner, 40, 14.0, 4.5);
        }
        assert_eq!(e.ents[gunner].hp, 0, "a shield gunner dies once the plate is gone");
    }

    #[test]
    fn floor_props_drop_onto_the_floor() {
        let mut e = arena();
        e.h = 480;
        let console = e.spawn(EK_OVERRIDE_CONSOLE, 6.5, 4.5).unwrap();
        let kit = e.spawn(EK_MED, 7.5, 4.5).unwrap();
        e.tick(1.0 / 60.0);
        assert!(e.ents[console].zoff > 50.0, "console zoff {}", e.ents[console].zoff);
        assert!(e.ents[kit].zoff > 100.0, "pickup zoff {}", e.ents[kit].zoff);
    }

    #[test]
    fn use_shuts_a_lamp_and_a_crate_drops_loot() {
        let mut e = arena();
        e.px = 4.5;
        e.py = 4.5;
        e.pa = 0.0;
        e.spawn(EK_LAMP, 6.0, 4.5);
        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.bits = IN_USE;
        e.tick(1.0 / 60.0);
        let lamp = e.ents.iter().find(|en| en.kind == EK_LAMP).unwrap();
        assert!(lamp.timer < 0.0, "USE facing a lamp shuts it");
        let crate_i = e.spawn(EK_CRATE, 6.2, 4.5).unwrap();
        e.hurt_ent(crate_i, 20, 4.5, 4.5);
        assert!(e.ents.iter().any(|en| matches!(en.kind, EK_MED | EK_AMMO | EK_ARMOR)));
    }

    #[test]
    fn a_fuel_barrel_leaves_fire() {
        let mut e = arena();
        let i = e.spawn_with_skin(EK_BARREL, 1, 8.0, 4.5).unwrap();
        e.hurt_ent(i, 40, 4.5, 4.5);
        assert!(e.ents.iter().any(|en| en.kind == EK_FIREPATCH || en.kind == EK_FLAME));
    }

    #[test]
    fn hostiles_spawn_in_open_space_on_every_sector() {
        let mut e = Engine::new(160, 100);
        for wave in 1..=map::LEVEL_COUNT as i32 {
            e.wave = wave;
            e.build_map();
            e.place_ents();
            for ent in e.ents.iter().filter(|en| is_hostile_kind(en.kind) && en.hp > 0) {
                assert!(
                    !e.circle_blocked(ent.x, ent.y, ent.radius),
                    "wave {wave} {} stuck at {}, {}",
                    ent.kind, ent.x, ent.y
                );
            }
        }
    }

    #[test]
    fn chimera_fan_does_not_leave_a_pyre() {
        let mut e = arena();
        e.set_w(9);
        e.weapon = 10;
        e.mag[10] = 5;
        e.fire();
        assert!(e.ents.iter().any(|en| en.kind == EK_PROJ && en.effect_tick == 4.0));
        assert!(!e.ents.iter().any(|en| en.kind == EK_FIREPATCH));
    }

    #[test]
    fn pyre_shot_flames_burn_then_expire() {
        let mut e = arena();
        e.set_w(6);
        e.weapon = 7;
        e.mag[7] = 6;
        let target = e.spawn(EK_HUSK, 6.2, 4.5).unwrap();
        e.ents[target].hp = 200;
        e.fire();
        e.tick(1.0 / 60.0);
        assert!(e.ents.iter().any(|en| en.kind == EK_FLAME));
        for _ in 0..40 { e.tick(1.0 / 60.0); }
        assert!(e.ents[target].hp < 200, "shot flames damage while they burn");
        assert!(e.ents.iter().any(|en| en.kind == EK_FLAME), "flames last more than a blink");
        for _ in 0..140 { e.tick(1.0 / 60.0); }
        assert!(!e.ents.iter().any(|en| en.kind == EK_FLAME), "shot flames are gone after a few seconds");
    }

    #[test]
    fn warden_blast_reaches_twice_as_far() {
        let mut e = arena();
        e.weapon = 4;
        e.mag[4] = 4;
        e.pa = 0.0;
        e.set_cell(8, 4, 1);
        let far = e.spawn(EK_BRUTE, 6.2, 8.2).unwrap();
        e.ents[far].hp = 78;
        e.ents[far].stun = 5.0;
        e.fire();
        for _ in 0..30 { e.tick(1.0 / 60.0); }
        assert!(e.ents[far].hp < 78, "a target about four cells off the impact still takes the blast");
    }

    #[test]
    fn override_pierces_the_lane_for_boss_damage() {
        let mut e = arena();
        e.set_w(7);
        e.weapon = 8;
        e.mag[8] = 4;
        let a = e.spawn(EK_HUSK, 6.5, 4.5).unwrap();
        let b = e.spawn(EK_HUSK, 8.5, 4.5).unwrap();
        e.fire();
        assert!(e.ents[a].hp <= 0 || e.ents[a].hp < 28 - 80);
        assert!(e.ents[b].hp <= 0 || e.ents[b].hp < 28 - 80, "the rail does not stop at the first body");
    }

    #[test]
    fn lockdown_opens_from_outside_and_stays_shut_from_inside() {
        let mut e = Engine::new(160, 100);
        let cases = [
            (1, 36.5, 17.5, 36.5, 20.5, 36i32, 18i32),
            (2, 27.5, 23.5, 29.5, 23.5, 28, 23),
            (3, 29.5, 15.5, 31.5, 15.5, 30, 15),
        ];
        for (wave, ox, oy, ix, iy, dx, dy) in cases {
            e.wave = wave;
            e.build_map();
            e.door.fill(0.0);
            let spot = map::boss_spots(wave)[0];
            let _ = e.spawn(EK_BOSS, spot.0, spot.1);
            e.seal_lockdown();
            assert_eq!(e.cell(dx, dy), 8, "wave {wave} door must stay a door");
            let idx = dy as usize * MAP_W + dx as usize;
            assert_eq!(e.door[idx], 0.0);
            e.px = ix;
            e.py = iy;
            e.bits = 0;
            e.state = 0;
            e.tick(1.0 / 60.0);
            assert_eq!(e.door[idx], 0.0, "wave {wave} stays shut from inside");
            e.door.fill(0.0);
            e.px = ox;
            e.py = oy;
            e.tick(1.0 / 60.0);
            assert!(e.door[idx] > 0.0, "wave {wave} opens from outside");
            for ent in e.ents.iter_mut() { ent.kind = 0; }
        }
    }

    #[test]
    fn owned_map_guns_become_supplies_on_the_next_sector() {
        let mut e = Engine::new(160, 100);
        e.set_w(0);
        e.set_w(6);
        e.wave = 1;
        e.build_map();
        map::place_level(&mut e);
        assert!(!e.ents.iter().any(|en| en.kind == EK_GUN2), "an owned breaker case must not return");
        assert!(!e.ents.iter().any(|en| en.kind == EK_GUN8), "an owned pyre case must not return");
        assert!(e.ents.iter().any(|en| en.kind == EK_GUN4), "an unowned gun still drops");
        assert!(e.ents.iter().any(|en| matches!(en.kind, EK_AMMO | EK_MED)));
    }

    #[test]
    fn chimera_bolts_hurt_enemies_and_not_the_shooter() {
        let mut e = arena();
        e.set_w(9);
        e.weapon = 10;
        e.mag[10] = 5;
        e.health = 100;
        let target = e.spawn(EK_HUSK, 7.5, 4.5).unwrap();
        e.ents[target].hp = 80;
        e.ents[target].stun = 3.0;
        e.fire();
        for _ in 0..40 { e.tick(1.0 / 60.0); }
        assert_eq!(e.health, 100, "the specimen fan must not kill its owner");
        assert!(e.ents[target].hp < 80, "the fan still damages the target");
    }
}

#[cfg(test)]
mod sound_tests {
    use super::*;
    fn arena() -> Engine {
        let mut e = Engine::new(160, 100); e.map.fill(0);
        for ent in &mut e.ents {ent.kind = EK_NONE;}
        e.px = 4.5; e.py = 4.5; e.sound_cues.clear(); e
    }
    #[test]
    fn sound_abi_and_queue_are_bounded() {
        assert_eq!(std::mem::size_of::<sound::SoundCue>(),16);
        let mut e = arena();
        for n in 0..100 {e.sound(3,0,n as f32,4.0);}
        assert_eq!(e.sound_cues.len(),sound::CAP);
        assert_eq!(e.sound_cues[0].x,0.0);
        assert_eq!(e.sound_cues[sound::CAP-1].x,63.0);
        e.tick(1.0/60.0); assert!(e.sound_cues.is_empty());
    }
    #[test]
    fn world_events_preserve_material_position_and_pickup_type() {
        let mut e=arena(); e.map[4*MAP_W+6]=8;
        assert!(e.open_door_at(6,4,false));
        assert_eq!(e.sound_cues[0].kind,1.0); assert_eq!(e.sound_cues[0].x,6.5);
        assert!(!e.open_door_at(6,4,false));assert_eq!(e.sound_cues.len(),1);
        for (kind,sound) in [(EK_MED,8.0),(EK_ARMOR,9.0),(EK_AMMO,10.0)] {
            e.pickup(kind);assert_eq!(e.sound_cues.last().unwrap().kind,sound);
        }
        let i=e.spawn_with_skin(EK_HUSK,SKIN_RIFLEMAN,8.5,4.5).unwrap();
        e.hurt_ent(i,1,4.5,4.5); assert!(e.sound_cues.iter().any(|s|s.kind==4.0&&s.x==8.5));
        let i=e.spawn_with_skin(EK_BRUTE,SKIN_LOADER,9.5,4.5).unwrap();
        e.hurt_ent(i,1,4.5,4.5);assert!(e.sound_cues.iter().any(|s|s.kind==3.0&&s.x==9.5));
        e.explode(7.0,5.0,1.0,1.0);assert!(e.sound_cues.iter().any(|s|s.kind==2.0&&s.x==7.0&&s.y==5.0));
    }
    #[test]
    fn scenery_destruction_is_not_an_enemy_death_voice() {
        let mut e=arena();let i=e.spawn(EK_CRATE,8.5,4.5).unwrap();
        e.hurt_ent(i,10000,4.5,4.5);
        assert!(e.sound_cues.iter().any(|s|s.kind==7.0&&s.x==8.5));
    }
}
