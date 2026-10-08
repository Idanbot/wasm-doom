// Fixed-size parallel tables (MAG_SZ/RESERVE_CAP/AMMO_PICKUP/IN_W_SLOT and the
// light/sprite scratch grids) are indexed by slot throughout; the `zip` rewrite
// Clippy suggests reads worse and hides the slot invariant.
#![allow(clippy::needless_range_loop)]

#[cfg(test)]
mod spatial_checks;
mod abi;
mod helpers;
use helpers::*;
mod lighting;
mod render;
mod sim;
mod textures;
mod boss_arena;
mod boss_attacks;
mod tactical;
mod tactical_roles;
mod combat;
mod campaign;
mod field;
mod voices;
mod agent_view;
mod sound;
mod consts;
mod enemies;
mod events;
mod hud;
mod map;
mod layouts;
mod types;

#[cfg(test)]
mod testutil;
#[cfg(test)]
mod engine_tests;
#[cfg(test)]
mod sim_tests;

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
    /// Radius multiplier for this cloud. Firing and blast smoke uses a smaller
    /// puff than authored arena smoke; see `spawn_smoke_cloud_scaled`.
    scale: f32,
    vx: f32,
    vy: f32,
}

impl Smoke {
    const DEAD: Self = Self { x: 0.0, y: 0.0, age: -1.0, scale: 1.0, vx: 0.0, vy: 0.0 };
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
    boss_attack: boss_attacks::AttackState,
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
                boss_attack_state: 0,
                boss_attack_t: 0.0,
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
            boss_attack: boss_attacks::AttackState::new(),
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


    // Test-only probe into the mipmap chain (see shade_and_mip_blend).
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
            if self.ents[i].kind == EK_BOSS && self.ents[i].hp > 0 && self.tick_boss_attack(i, dt) {
                let e = &mut self.ents[i];
                e.flash = (e.flash-dt).max(0.0);
                e.bar_t = (e.bar_t-dt).max(0.0);
                advance_anim(e, dt);
                continue;
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
                // Phase transitions schedule the boss's own warned pattern.
                if let Some(i)=self.ents.iter().position(|e|e.kind==EK_BOSS&&e.hp>0) {
                    self.boss_attack.stage=0;
                    self.boss_attack.remaining=0.0;
                    self.ents[i].effect_tick=0.0;
                    self.ents[i].timer=0.6;
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

        if !self.ents.iter().any(|e|e.kind==EK_BOSS&&e.hp>0) || self.state!=0 {
            self.boss_attack = boss_attacks::AttackState::new();
            self.boss_vuln=0.0;
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
            boss_attack_state: self.boss_attack.stage,
            boss_attack_t: self.boss_attack.remaining,
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



}


#[cfg(test)]
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
