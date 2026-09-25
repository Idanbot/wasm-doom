mod combat;
mod voices;
mod consts;
mod enemies;
mod events;
mod hud;
mod map;
mod types;

use consts::*;
use enemies::{default_skin, enemy_def, is_hostile_kind, skin_def};
use events::*;
pub use hud::{Hud, HUD_OFFSETS, HUD_SIZE};
use types::{AmbushTrigger, Ent, FxCmd};

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
    ammo: [i32; 7],
    mag: [i32; 7],
    weapon: i32,
    has_w2: bool,
    has_w3: bool,
    has_w4: bool,
    has_w5: bool,
    has_w6: bool,
    has_w7: bool,
    cooldown: f32,
    reload_t: f32,
    reload_dur: f32,
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
    ambush: Vec<AmbushTrigger>,
}

/// Single engine instance. WASM is single-threaded; raw-ref access keeps
/// `static_mut_refs` clippy-clean. All entry points run on the same JS
/// thread, so `eng()` exclusivity holds.
static mut E: Option<Engine> = None;

fn eng() -> &'static mut Engine {
    #[allow(static_mut_refs)]
    unsafe {
        (*(&raw mut E)).as_mut().expect("engine")
    }
}

fn eng_init(w: usize, h: usize) {
    #[allow(static_mut_refs)]
    unsafe {
        E = Some(Engine::new(w, h));
    }
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

fn segment_distance_sq(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let t = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy).max(1e-8)).clamp(0.0, 1.0);
    (x - ax - dx * t).powi(2) + (y - ay - dy * t).powi(2)
}

fn set_anim(e: &mut Ent, state: u8, lock: f32) {
    if e.anim != state {
        e.anim = state;
        e.anim_time = 0.0;
    }
    e.anim_lock = e.anim_lock.max(lock);
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
    let fps = match e.anim {
        ANIM_IDLE => 2.2,
        ANIM_MOVE => 8.0,
        ANIM_PAIN => 7.0,
        ANIM_FIRE => 9.0,
        ANIM_RELOAD => 5.5,
        ANIM_DEAD => 3.2,
        ANIM_SPECIAL => 5.0,
        _ => 4.0,
    };
    ((e.anim_time * fps) as i32).rem_euclid(count)
}

fn sprite_style(e: &Ent) -> (usize, f32, bool, i32) {
    if e.kind == EK_OVERRIDE_CONSOLE {
        let texture = match e.skin {
            SKIN_CONSOLE_FOUNDRY => T_CONSOLE_FOUNDRY,
            SKIN_CONSOLE_BIOFORGE => T_CONSOLE_BIOFORGE,
            _ => T_CONSOLE_UPPER,
        };
        return (texture, 0.95, false, 0);
    }
    if let Some(skin) = skin_def(e.skin) {
        return (skin.texture + (e.anim as usize).min(ENEMY_ANIM_COUNT - 1), if e.kind == EK_BOSS { skin.scale.max(1.9) } else { skin.scale }, true, anim_frame(e));
    }
    // VFX cells are distinct effects, not animation frames. Animate scale
    // over lifetime without cycling flames into smoke or muzzle flashes.
    match e.kind {
        EK_PROJ => return (T_ORDNANCE, 0.24, true, if e.effect_tick == 3.0 { 3 } else { 0 }),
        // The generated rear-view missile has a narrower silhouette than the
        // old fireball, so give it enough projected size to read in motion.
        EK_BOLT => return (T_ORDNANCE, 0.42, true, 1),
        EK_SMOKE => return (T_FLAME, 0.22 + e.frame.min(0.8) * 0.4, true, 1),
        EK_FLAME | EK_FIREPATCH => return (T_FLAME, 0.58 + (e.frame * 7.0).sin() * 0.035, true, 0),
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
        EK_MED | EK_AMMO | EK_ARMOR | EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 | EK_GUN6 | EK_GUN7
    )
}

fn is_weapon_item(k: u8) -> bool {
    matches!(k, EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 | EK_GUN6 | EK_GUN7)
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
                radius: 0.25,
                flash: 0.0,
                stun: 0.0,
                effect_tick: 0.0,
                zoff: 0.0,
            }; ENT_N],
            px: map::PLAYER_START.0,
            py: map::PLAYER_START.1,
            pa: map::PLAYER_START.2,
            pitch: 0.0,
            pr: 0.22,
            health: 100,
            armor: 0,
            ammo: [36, 0, 0, 0, 0, 0, 0],
            mag: [12, 0, 0, 0, 0, 0, 0],
            weapon: 0,
            has_w2: false,
            has_w3: false,
            has_w4: false,
            has_w5: false,
            has_w6: false,
            has_w7: false,
            cooldown: 0.0,
            reload_t: 0.0,
            reload_dur: 1.0,
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
            events: 0,
            ev_weapon: 0,
            foot_acc: 0.0,
            flow_dir: vec![0; MAP_CELLS],
            flow_dist: vec![0xFFFF; MAP_CELLS],
            flow_q: vec![0; MAP_CELLS],
            flow_age: 1.0,
            fx_q: [FxCmd {
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
            ambush: Vec::new(),
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
        [4.0, 5.2, 4.6][map::level_index(self.wave)]
    }

    fn boss_intro_effect(&mut self, stage: u8) {
        let sector = map::level_index(self.wave);
        let (x, y) = map::boss_spots(self.wave)[0];
        match sector {
            0 => {
                self.shake = (self.shake + if stage == 1 { 0.32 } else { 0.17 }).min(1.0);
                for n in 0..8 {
                    let a = n as f32 * core::f32::consts::TAU / 8.0;
                    self.spawn_timed(EK_SPARK, x + a.cos() * 1.2, y + a.sin() * 1.2, 0.55, -16.0);
                }
            }
            1 => {
                self.shake = (self.shake + 0.22).min(1.0);
                for n in 0..12 {
                    let a = n as f32 * core::f32::consts::TAU / 12.0;
                    self.spawn_timed(EK_SPARK, x + a.cos() * 1.8, y + a.sin() * 1.8, 0.65, -30.0);
                }
            }
            _ => {
                self.shake = (self.shake + 0.15).min(1.0);
                for n in 0..9 {
                    let a = n as f32 * core::f32::consts::TAU / 9.0;
                    self.spawn_timed(EK_SMOKE, x + a.cos() * 1.2, y + a.sin() * 1.2, 0.9, -5.0);
                }
            }
        }
        if stage == 1 { self.events |= EV_BOSS_HUSH; } else { self.events |= EV_DOOR; }
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
    }

    fn tex_at_mut(&mut self, id: usize) -> &mut [u32] {
        let o = id * TEX * TEX;
        &mut self.tex[o..o + TEX * TEX]
    }

    fn put_tex(slot: &mut [u32], x: usize, y: usize, c: u32) {
        slot[(y & (TEX - 1)) * TEX + (x & (TEX - 1))] = c;
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
        let legs = (cx.abs() - 8).abs() < 7 && y >= 96 && y < 122;
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
        let (hp, radius, zoff) = match enemy_def(kind) {
            Some(d) => (d.hp, d.radius, d.zoff),
            None => (1, 0.2, 0.0),
        };
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
                    radius,
                    flash: 0.0,
                    stun: 0.0,
                    effect_tick: 0.0,
                    zoff,
                };
                return Some(i);
            }
        }
        None
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

    fn queue_fx(&mut self, kind: u8, x: f32, y: f32, vx: f32, vy: f32, timer: f32, zoff: f32) {
        if self.fx_n >= FX_CAP {
            return;
        }
        self.fx_q[self.fx_n] = FxCmd {
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
                aim: 0.0,
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
                skin: SKIN_NONE,
                radius,
                flash: 0.0,
                stun: 0.0,
                effect_tick: f.variant as f32,
                zoff: if f.zoff != 0.0 { f.zoff } else { zdef },
            };
        }
        self.fx_n = 0;
    }

    fn effect(&mut self, kind: u8, variant: u8, x: f32, y: f32, life: f32, zoff: f32) {
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

    fn spawn_hostiles(&mut self, mult: i32) {
        let copies = mult.max(1);
        for n in 0..copies {
            for &(k, skin, x, y) in map::hostiles(self.wave) {
                let jx = if n == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
                let jy = if n == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
                let nx = x + jx;
                let ny = y + jy;
                if !self.blocked(nx.floor() as i32, ny.floor() as i32) {
                    if self.spawn_with_skin(k, skin, nx, ny).is_none() {
                        return;
                    }
                } else if self.spawn_with_skin(k, skin, x, y).is_none() {
                    return;
                }
            }
        }
    }

    fn next_wave(&mut self) {
        self.wave = (self.wave + 1).min(12);
        // Capped at 4x (56 hostiles + ambushes): the uncapped 1<<8 shift
        // filled all 192 entity slots with hostiles and starved FX.
        let shift = (self.wave - 1).clamp(0, 2);
        let mult = 1i32 << shift;
        self.health = 100;
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
        self.mag[0] = MAG_SZ[0];
        self.ammo[0] = self.ammo[0].max(36);
        if self.has_w2 {
            self.mag[1] = MAG_SZ[1];
            self.ammo[1] = self.ammo[1].max(12);
        }
        if self.has_w3 {
            self.mag[2] = MAG_SZ[2];
            self.ammo[2] = self.ammo[2].max(48);
        }
        if self.has_w4 {
            self.mag[3] = MAG_SZ[3];
            self.ammo[3] = self.ammo[3].max(8);
        }
        if self.has_w5 {
            self.mag[4] = MAG_SZ[4];
            self.ammo[4] = self.ammo[4].max(12);
        }
        if self.has_w6 {
            self.mag[5] = MAG_SZ[5];
            self.ammo[5] = self.ammo[5].max(16);
        }
        if self.has_w7 {
            self.mag[6] = MAG_SZ[6];
            self.ammo[6] = self.ammo[6].max(160);
        }
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
        // Capped so late waves stay killable: 480 / 720 / 1080 / 1620, then 2200.
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
                break;
            }
        }
        let sector = map::level_index(self.wave);
        let entry = map::boss_spots(self.wave)[0];
        let burst = [14, 22, 10][sector];
        for _ in 0..burst {
            let a = self.rnd() * core::f32::consts::TAU;
            let r = 0.5 + self.rnd() * 1.6;
            self.spawn_timed(
                if sector == 2 { EK_SMOKE } else { EK_SPARK },
                entry.0 + a.cos() * r,
                entry.1 + a.sin() * r,
                if sector == 2 { 1.1 } else { 0.7 },
                if sector == 1 { -32.0 } else { -16.0 },
            );
        }
        let escorts = match map::level_index(self.wave) {
            1 => [(EK_WRAITH, SKIN_HORNET, -2.4, -1.6), (EK_MARTYR, SKIN_MARTYR, 2.4, -1.6), (EK_HUSK, SKIN_GUNNER, -2.8, 1.8), (EK_BRUTE, SKIN_HAZMAT, 2.8, 1.8)],
            2 => [(EK_HUSK, SKIN_HOUND, -2.4, -1.6), (EK_WRAITH, SKIN_SPITTER, 2.4, -1.6), (EK_HUSK, SKIN_SUBJECT, -2.8, 1.8), (EK_BRUTE, SKIN_VATBRUTE, 2.8, 1.8)],
            _ => [(EK_WRAITH, SKIN_HORNET, -2.4, -1.6), (EK_WRAITH, SKIN_MARKSMAN, 2.4, -1.6), (EK_HUSK, SKIN_RIFLEMAN, -2.8, 1.8), (EK_BRUTE, SKIN_LOADER, 2.8, 1.8)],
        };
        for &(kind, skin, ox, oy) in &escorts[..(2 + self.wave.min(2) as usize)] {
            let x = entry.0 + ox;
            let y = entry.1 + oy;
            if !self.blocked(x.floor() as i32, y.floor() as i32) {
                let _ = self.spawn_with_skin(kind, skin, x, y);
            }
        }
    }

    fn boss_max_health(&self) -> i32 {
        (480.0 * 1.5f32.powi((self.wave - 1).max(0))).min(2200.0).round() as i32
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
            let prev_size = size * 2;
            let source = if level == 1 { &self.tex } else { &self.mipmaps[level - 2] };
            let mut pixels = vec![0u32; TEX_N * size * size];
            for id in 0..TEX_N {
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
            self.mipmaps.push(pixels);
        }
    }

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
        let id = match c {
            1 => T_METAL,
            2 => T_BRICK,
            3 => T_FLESH,
            4 => T_PIPES,
            5 => T_SKULL,
            6 => T_TECH,
            7 => T_HAZARD,
            8 => T_DOOR,
            9 => T_SECRET,
            _ => T_METAL,
        };
        if !self.hell {
            return id;
        }
        match id {
            T_OVERRIDE => T_OVERRIDE,
            T_BRICK | T_METAL | T_TECH | T_PIPES | T_DOOR => T_FLESH,
            T_GRATE | T_CONC | T_HAZARD => T_SKULL,
            T_CEIL | T_SECRET => T_SKULL,
            _ => T_FLESH,
        }
    }

    fn open_door_at(&mut self, cx: i32, cy: i32, force: bool) -> bool {
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
        self.events |= EV_DOOR;
        if c == 9 {
            self.secrets += 1;
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

    fn damage_player(&mut self, dmg: i32) {
        if self.iframes > 0.0 || self.state != 0 {
            return;
        }
        let mut d = dmg;
        if self.armor > 0 {
            let soak = (d * 2) / 3;
            let take = soak.min(self.armor);
            self.armor -= take;
            d -= take;
        }
        self.health -= d.max(1);
        self.iframes = 0.35;
        self.hurt = 1.0;
        self.shake = (self.shake + 0.55).min(1.0);
        self.events |= EV_HURT;
        if self.health <= 0 {
            self.health = 0;
            self.state = 1;
            self.events |= EV_DIE;
        }
    }

    fn explode(&mut self, x: f32, y: f32, radius: f32, dmg: f32) {
        self.effect(EK_IMPACT, 2, x, y, 0.38, 10.0);
        self.spawn_timed(EK_SMOKE, x, y, 0.75, 12.0);
        self.shake = (self.shake + 0.8).min(1.0);
        self.events |= EV_EXPLODE;
        let pd = ((self.px - x).powi(2) + (self.py - y).powi(2)).sqrt();
        if pd < radius && self.los(x, y, self.px, self.py) {
            let fall = 1.0 - pd / radius;
            self.damage_player((dmg * fall) as i32);
        }
        let mut hits: Vec<(usize, i32)> = Vec::new();
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !solid_kind(e.kind) {
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
        if i >= ENT_N {
            return;
        }
        let kind;
        let x;
        let y;
        {
            let e = &mut self.ents[i];
            if e.kind == 0 || e.hp <= 0 {
                return;
            }
            e.hp -= dmg;
            e.flash = 0.12;
            let dx = e.x - hx;
            let dy = e.y - hy;
            let l = (dx * dx + dy * dy).sqrt().max(0.01);
            e.vx += dx / l * 1.6;
            e.vy += dy / l * 1.6;
            kind = e.kind;
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
        if kind == EK_BARREL || kind == EK_MARTYR {
            self.light_dirty = true;
            self.explode(x, y, 2.6, 55.0);
        }
        if kind == EK_BOSS {
            self.state = 2;
        }
    }

    fn hitscan(&mut self, ang: f32, dmg: i32, maxd: f32) -> bool {
        let dx = ang.cos();
        let dy = ang.sin();
        let mut best_t = maxd;
        let mut best_e: Option<usize> = None;
        for (i, e) in self.ents.iter().enumerate() {
            if e.kind == 0 || e.hp <= 0 || !solid_kind(e.kind) {
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
                self.hurt_ent(i, damage, self.px, self.py);
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
            if e.hp <= 0 || !solid_kind(e.kind) { continue; }
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
        hits[..count].sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        for (n, &(distance, i)) in hits[..count].iter().take(3).enumerate() {
            if self.ents[i].kind == EK_NONE { continue; }
            let barrel = self.ents[i].kind == EK_BARREL;
            self.hurt_ent(i, [160, 112, 78][n], self.px, self.py);
            if barrel || n == 2 { end = distance; break; }
        }
        // Short-lived tracer sprites have no gameplay collision; the beam resolves once.
        let segments = ((end / 0.5).ceil() as usize).clamp(1, 24);
        for n in 0..segments {
            let t = 0.25 + (end - 0.25).max(0.0) * n as f32 / segments as f32;
            if t >= end { break; }
            self.spawn_timed(EK_RAY, self.px + dx * t, self.py + dy * t, 0.12, 6.0);
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
                e.timer = 2.2;
                return;
            }
        }
        if count >= 12 { return; }
        if let Some(i) = self.spawn(EK_FIREPATCH, x, y) {
            self.ents[i].timer = 2.2;
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
        self.reload_dur = RELOAD_T[w];
        self.reload_t = RELOAD_T[w];
        self.events |= EV_RELOAD;
    }

    fn finish_reload(&mut self) {
        let w = self.weapon as usize;
        let need = MAG_SZ[w] - self.mag[w];
        let take = need.min(self.ammo[w]).max(0);
        self.mag[w] += take;
        self.ammo[w] -= take;
        self.reload_t = 0.0;
    }

    fn fire(&mut self) {
        if self.state != 0 || self.cooldown > 0.0 || self.reload_t > 0.0 {
            return;
        }
        let w = self.weapon as usize;
        if self.mag[w] <= 0 {
            if self.ammo[w] > 0 {
                self.begin_reload();
            } else {
                self.cooldown = 0.22;
                self.ev_weapon = self.weapon;
                self.events |= EV_EMPTY;
            }
            return;
        }
        self.mag[w] -= 1;
        self.events |= EV_FIRE;
        self.ev_weapon = self.weapon;
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
                if (self.rng & 1) == 0 {
                    self.eject_casing();
                }
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
            _ => {
                self.cooldown = 0.058;
                self.muzzle = 1.0;
                self.kick = 0.52;
                self.shake = (self.shake + 0.055).min(1.0);
                self.spread = (self.spread + 0.014).min(0.17);
                let a = self.pa + (self.rnd() - 0.5) * (0.035 + self.spread);
                self.hitscan(a, 7, 24.0);
                if (self.rng & 3) == 0 { self.eject_casing(); }
            }
        }
    }

    fn eject_casing(&mut self) {
        let rx = -self.pa.sin();
        let ry = self.pa.cos();
        let x = self.px + rx * 0.18;
        let y = self.py + ry * 0.18;
        let jx = 1.2 + self.rnd();
        let jy = 1.2 + self.rnd();
        let pax = self.pa.cos() * 0.2;
        let pay = self.pa.sin() * 0.2;
        let slot = self.fx_n;
        self.queue_fx(EK_SPARK, x, y, rx * jx + pax, ry * jy + pay, 0.35, 12.0);
        if self.fx_n > slot { self.fx_q[slot].variant = 5; }
    }

    fn enemy_shoot(&mut self, i: usize) {
        let shooter = self.ents[i];
        let (x, y) = (shooter.x, shooter.y);
        let role = combat::profile(shooter.skin, shooter.kind);
        set_anim(&mut self.ents[i], ANIM_FIRE, 0.24);
        let sp = if shooter.skin == SKIN_MARKSMAN { 8.0 } else { 5.4 };
        let zoff = if shooter.skin == SKIN_HORNET { -35.0 } else { -8.0 };
        for n in 0..role.pellets {
            let a = shooter.aim + (n as f32 - (role.pellets - 1) as f32 * 0.5) * role.spread;
            if let Some(index) = self.spawn(EK_PROJ, x, y) {
                let e = &mut self.ents[index];
                e.vx = a.cos() * sp;
                e.vy = a.sin() * sp;
                e.timer = 2.8;
                e.hp = role.damage;
                e.effect_tick = if shooter.skin == SKIN_SPITTER { 3.0 } else { 0.0 };
                e.zoff = zoff;
            }
        }
        self.effect(EK_SPARK, if role.pellets > 1 { 1 } else { 0 }, x, y, 0.12, zoff);
    }

    fn pickup(&mut self, kind: u8) {
        match kind {
            EK_MED => {
                self.health = (self.health + 35).min(100);
                self.events |= EV_PICK_SILVER;
            }
            EK_AMMO => {
                self.ammo[0] = (self.ammo[0] + 18).min(120);
                if self.has_w2 {
                    self.ammo[1] = (self.ammo[1] + 10).min(48);
                }
                if self.has_w3 {
                    self.ammo[2] = (self.ammo[2] + 45).min(216);
                }
                if self.has_w4 {
                    self.ammo[3] = (self.ammo[3] + 5).min(20);
                }
                if self.has_w5 {
                    self.ammo[4] = (self.ammo[4] + 2).min(16);
                }
                if self.has_w6 {
                    self.ammo[5] = (self.ammo[5] + 15).min(80);
                }
                if self.has_w7 {
                    self.ammo[6] = (self.ammo[6] + 90).min(450);
                }
                self.events |= EV_PICK_SILVER;
            }
            EK_ARMOR => {
                self.armor = (self.armor + 50).min(100);
                self.events |= EV_PICK_SILVER;
            }
            EK_GUN2 => {
                self.has_w2 = true;
                self.ammo[1] = (self.ammo[1] + 8).min(48);
                if self.mag[1] <= 0 {
                    self.mag[1] = MAG_SZ[1];
                }
                self.weapon = 1;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN3 => {
                self.has_w3 = true;
                self.ammo[2] = (self.ammo[2] + 54).min(216);
                if self.mag[2] <= 0 {
                    self.mag[2] = MAG_SZ[2];
                }
                self.weapon = 2;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN4 => {
                self.has_w4 = true;
                self.ammo[3] = (self.ammo[3] + 7).min(20);
                if self.mag[3] <= 0 {
                    self.mag[3] = MAG_SZ[3];
                }
                self.weapon = 3;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN5 => {
                self.has_w5 = true;
                self.ammo[4] = (self.ammo[4] + 4).min(16);
                if self.mag[4] <= 0 {
                    self.mag[4] = MAG_SZ[4];
                }
                self.weapon = 4;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN6 => {
                self.has_w6 = true;
                self.ammo[5] = (self.ammo[5] + 20).min(80);
                if self.mag[5] <= 0 { self.mag[5] = MAG_SZ[5]; }
                self.weapon = 5;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN7 => {
                self.has_w7 = true;
                self.ammo[6] = (self.ammo[6] + 180).min(450);
                if self.mag[6] <= 0 { self.mag[6] = MAG_SZ[6]; }
                self.weapon = 6;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            _ => {}
        }
    }

    fn needs_pickup(&self, kind: u8) -> bool {
        match kind {
            EK_MED => self.health < 100,
            EK_ARMOR => self.armor < 100,
            EK_AMMO => self.ammo[0] < 120
                || (self.has_w2 && self.ammo[1] < 48)
                || (self.has_w3 && self.ammo[2] < 216)
                || (self.has_w4 && self.ammo[3] < 20)
                || (self.has_w5 && self.ammo[4] < 16)
                || (self.has_w6 && self.ammo[5] < 80)
                || (self.has_w7 && self.ammo[6] < 450),
            _ => true,
        }
    }

    fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.08);
        self.time += dt;
        self.events = 0;
        self.ev_weapon = 0;
        if self.state == 0 {
            self.elapsed += dt;
        }
        self.cooldown = (self.cooldown - dt).max(0.0);
        if self.reload_t > 0.0 {
            self.reload_t = (self.reload_t - dt).max(0.0);
            if self.reload_t <= 0.0 {
                self.finish_reload();
            }
        } else {
            let w = self.weapon as usize;
            if self.mag[w] <= 0 && self.ammo[w] > 0 && self.cooldown <= 0.0 {
                self.begin_reload();
            }
        }
        self.iframes = (self.iframes - dt).max(0.0);
        self.shake = (self.shake - dt * 2.2).max(0.0);
        self.muzzle = (self.muzzle - dt * 8.0).max(0.0);
        self.hurt = (self.hurt - dt * 2.6).max(0.0);
        self.kick = (self.kick - dt * 6.0).max(0.0);
        self.hitmarker = (self.hitmarker - dt * 4.0).max(0.0);
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

            if bits & IN_W1 != 0 && self.wpn_latched & IN_W1 == 0 {
                self.weapon = 0;
                self.reload_t = 0.0;
            }
            if bits & IN_W2 != 0 && self.has_w2 && self.wpn_latched & IN_W2 == 0 {
                self.weapon = 1;
                self.reload_t = 0.0;
            }
            if bits & IN_W3 != 0 && self.has_w3 && self.wpn_latched & IN_W3 == 0 {
                self.weapon = 2;
                self.reload_t = 0.0;
            }
            if bits & IN_W4 != 0 && self.has_w4 && self.wpn_latched & IN_W4 == 0 {
                self.weapon = 3;
                self.reload_t = 0.0;
            }
            if bits & IN_W5 != 0 && self.has_w5 && self.wpn_latched & IN_W5 == 0 {
                self.weapon = 4;
                self.reload_t = 0.0;
            }
            if bits & IN_W6 != 0 && self.has_w6 && self.wpn_latched & IN_W6 == 0 {
                self.weapon = 5;
                self.reload_t = 0.0;
            }
            if bits & IN_W7 != 0 && self.has_w7 && self.wpn_latched & IN_W7 == 0 {
                self.weapon = 6;
                self.reload_t = 0.0;
            }
            self.wpn_latched = bits & (IN_W1 | IN_W2 | IN_W3 | IN_W4 | IN_W5 | IN_W6 | IN_W7);

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
            let sprint = if bits & IN_SPRINT != 0 { 1.55 } else { 1.0 };
            let speed = 3.35 * sprint;
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

            if bits & IN_USE != 0 {
                if !self.use_latched {
                    self.open_nearby_doors(true);
                    let clear = !self.ents.iter().any(|e| e.hp > 0 && is_hostile_kind(e.kind));
                    if clear && self.near_override() && !self.boss_spawned && self.boss_intro <= 0.0 {
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
            let e = &mut self.ents[i];
            e.flash = (e.flash - dt).max(0.0);
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
                    let spd = role.speed;
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
                    e.frame += dt * 1.4 + moved * 6.5;
                    // Locomotion bob so chasers read as moving, not sliding:
                    // floaters hover, ground units step. Assigned, never
                    // accumulated, from the skin's base height.
                    let base_z = skin_def(e.skin).map(|s| s.zoff).unwrap_or(0.0);
                    if e.skin == SKIN_HORNET || e.kind == EK_MARTYR {
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
                }
                EK_PROJ => {
                    let (old_x, old_y) = (e.x, e.y);
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.timer -= dt;
                    let (ex, ey, dead, damage, variant) = (e.x, e.y, e.timer <= 0.0, e.hp, e.effect_tick);
                    let _ = e;
                    if dead || !self.los(old_x, old_y, ex, ey) {
                        if !dead {
                            self.effect(EK_IMPACT, if variant == 3.0 { 3 } else { 1 }, old_x, old_y, 0.22, -6.0);
                        }
                        self.ents[i].kind = 0;
                    } else if pstate == 0 {
                        let d = segment_distance_sq(px, py, old_x, old_y, ex, ey);
                        if d < 0.22 {
                            self.ents[i].kind = 0;
                            melee.push((i, damage));
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
                    let (x, y, pulse) = (e.x, e.y, e.effect_tick <= 0.0);
                    if e.timer <= 0.0 { e.kind = EK_NONE; continue; }
                    if pulse { e.effect_tick = 0.25; }
                    let _ = e;
                    if pulse && pstate == 0 {
                        for j in 0..ENT_N {
                            let target = self.ents[j];
                            if target.hp <= 0 || !solid_kind(target.kind) { continue; }
                            let range = target.radius + 0.7;
                            if (target.x - x).powi(2) + (target.y - y).powi(2) < range * range
                                && self.los(x, y, target.x, target.y) {
                                self.hurt_ent(j, 5, target.x, target.y);
                            }
                        }
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
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.vx *= 0.88;
                    e.vy *= 0.88;
                    e.zoff += if e.kind == EK_SPARK { -40.0 * dt } else { 8.0 * dt };
                    e.timer -= dt;
                    if e.timer <= 0.0 {
                        e.kind = 0;
                    }
                }
                EK_BOLT => {
                    let (old_x, old_y) = (e.x, e.y);
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.timer -= dt;
                    e.frame += dt * 6.0;
                    let (ex, ey, dead) = (e.x, e.y, e.timer <= 0.0);
                    let _ = e;
                    if dead || !self.los(old_x, old_y, ex, ey) {
                        self.ents[i].kind = 0;
                        self.explode(old_x, old_y, 2.4, 65.0);
                    } else {
                        let mut hit = None;
                        for (j, o) in self.ents.iter().enumerate() {
                            if j == i || o.hp <= 0 || !solid_kind(o.kind) {
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
                            self.hurt_ent(j, 24, ex, ey);
                            self.explode(ex, ey, 2.4, 65.0);
                        }
                    }
                }
                EK_LAMP => {
                    e.zoff = -118.0 + (self.time * 2.6 + e.x).sin() * 5.0;
                }
                EK_FLAME => {
                    e.zoff = 48.0 + (self.time * 14.0 + e.x).sin() * 6.0;
                    e.frame += dt * 2.0;
                }
                EK_CHAIN => {
                    e.zoff = -104.0 + (self.time * 2.2 + e.x).sin() * 6.0;
                }
                EK_CRATE => {
                    e.zoff = 86.0;
                }
                EK_BARREL => {
                    e.vx *= 0.8;
                    e.vy *= 0.8;
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.zoff = 78.0;
                }
                EK_MED | EK_AMMO | EK_ARMOR | EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 => {
                    e.zoff = 32.0 + (self.time * 2.5 + e.x * 1.7).sin() * 9.0;
                }
                _ => {}
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
            self.damage_player(dmg);
        }

        if (self.rng & 31) == 0 {
            let mut puffs: Vec<(f32, f32, f32)> = Vec::new();
            for e in self.ents.iter() {
                if e.kind == EK_LAMP || e.kind == EK_FLAME {
                    puffs.push((
                        e.x,
                        e.y,
                        if e.kind == EK_FLAME { 14.0 } else { -60.0 },
                    ));
                }
            }
            for (x, y, z) in puffs {
                if self.rnd() > 0.45 {
                    self.spawn_timed(EK_SMOKE, x, y, 0.5, z);
                }
            }
        }

        // push enemies out of walls / each other lightly skipped

        self.flush_fx();

        let mut living = 0;
        for e in self.ents.iter() {
            if e.hp > 0 && is_hostile_kind(e.kind) {
                living += 1;
            }
        }
        if !self.boss_spawned && self.state == 0 {
            if self.boss_intro > 0.0 {
                let prev = self.boss_intro;
                self.boss_intro -= dt;
                let half = self.boss_intro_duration() * 0.5;
                if prev > half && self.boss_intro <= half { self.boss_intro_effect(0); }
                if prev > 1.0 && self.boss_intro <= 1.0 {
                    self.hell = true;
                    self.boss_intro_effect(1);
                }
                if self.boss_intro <= 0.0 {
                    self.boss_intro = 0.0;
                    self.maybe_spawn_boss();
                }
                living = 1;
            }
        }

        if self.boss_spawned && self.state == 0 {
            let max_hp = self.boss_max_health();
            let boss_hp = self.ents.iter().find(|e| e.kind == EK_BOSS && e.hp > 0).map(|e| e.hp).unwrap_or(0);
            let phase = if boss_hp > 0 && boss_hp * 3 <= max_hp { 2 } else if boss_hp > 0 && boss_hp * 3 <= max_hp * 2 { 1 } else { 0 };
            if phase > self.boss_phase {
                self.boss_phase = phase;
                self.shake = (self.shake + 0.5).min(1.0);
                self.events |= EV_EXPLODE;
                let support = match (map::level_index(self.wave), phase) {
                    (1, 1) => [(EK_WRAITH, SKIN_HORNET), (EK_HUSK, SKIN_GUNNER)],
                    (1, _) => [(EK_MARTYR, SKIN_MARTYR), (EK_WRAITH, SKIN_HORNET)],
                    (2, 1) => [(EK_HUSK, SKIN_HOUND), (EK_WRAITH, SKIN_SPITTER)],
                    (2, _) => [(EK_BRUTE, SKIN_VATBRUTE), (EK_HUSK, SKIN_HOUND)],
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
                        let count = if map::level_index(self.wave) == 1 { 8 } else { 6 };
                        for n in 0..count {
                            let a = n as f32 * core::f32::consts::TAU / count as f32 + phase as f32 * 0.18;
                            if let Some(i) = self.spawn(EK_PROJ, bx, by) {
                                let speed = if map::level_index(self.wave) == 1 { 6.7 } else { 4.1 };
                                self.ents[i].vx = a.cos() * speed;
                                self.ents[i].vy = a.sin() * speed;
                                self.ents[i].timer = 3.2;
                                self.ents[i].hp = if phase == 2 { 18 } else { 12 };
                                self.ents[i].effect_tick = if map::level_index(self.wave) == 2 { 3.0 } else { 0.0 };
                            }
                        }
                    }
                }
            }
        }


        let mut prompt = 0;
        let fx = self.px + self.pa.cos();
        let fy = self.py + self.pa.sin();
        let fc = self.cell(fx.floor() as i32, fy.floor() as i32);
        if fc == 8 || fc == 9 {
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
        if living == 0 && !self.boss_spawned && self.boss_intro <= 0.0 && self.state == 0 {
            prompt = 6;
        }
        if living == 0 && self.near_override() && !self.boss_spawned && self.boss_intro <= 0.0 {
            prompt = 3;
        }
        if self.boss_intro > 0.0 {
            prompt = 7 + map::level_index(self.wave) as i32 * 2
                + if self.boss_intro <= self.boss_intro_duration() * 0.5 { 1 } else { 0 };
        }

        let weap_frame = if self.reload_t > 0.0 {
            let p = (1.0 - self.reload_t / self.reload_dur.max(0.05)).clamp(0.0, 0.999);
            5 + (p * if self.weapon == 0 { 4.0 } else { 8.0 }) as i32
        } else if self.muzzle > 0.08 {
            if self.weapon == 2 {
                1 + ((self.time * 18.0) as i32).rem_euclid(4)
            } else if self.muzzle > 0.72 {
                1
            } else if self.muzzle > 0.48 {
                2
            } else if self.muzzle > 0.24 {
                3
            } else {
                4
            }
        } else {
            0
        };

        let boss_health = self.ents.iter()
            .find(|e| e.kind == EK_BOSS && e.hp > 0)
            .map(|e| e.hp)
            .unwrap_or(0);
        self.hud = Hud {
            health: self.health,
            armor: self.armor,
            ammo: self.mag[wpn],
            weapon: self.weapon,
            kills: self.kills,
            living,
            state: self.state,
            prompt,
            has_w2: if self.has_w2 { 1 } else { 0 },
            has_w3: if self.has_w3 { 1 } else { 0 },
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
            has_w4: if self.has_w4 { 1 } else { 0 },
            has_w5: if self.has_w5 { 1 } else { 0 },
            events: self.events,
            ev_weapon: self.ev_weapon,
            wave: self.wave,
            boss_health,
            boss_max_health: if boss_health > 0 { self.boss_max_health() } else { 0 },
            boss_phase: if boss_health > 0 { self.boss_phase as i32 } else { 0 },
            has_w6: if self.has_w6 { 1 } else { 0 },
            has_w7: if self.has_w7 { 1 } else { 0 },
        };
    }

    fn put(&mut self, x: i32, y: i32, c: u32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= self.w || y >= self.h {
            return;
        }
        self.fb[y * self.w + x] = c;
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
                let falloff = (1.0 - d2 / (radius * radius)).powi(2) / (1.0 + d2 * 0.3);
                for channel in 0..3 { grid[cy * MAP_W + cx][channel] += rgb[channel] * falloff; }
            }
        }
    }

    fn update_lighting(&mut self) {
        if self.light_dirty {
            let mut grid = vec![[0.0; 3]; MAP_CELLS];
            for y in 1..MAP_H - 1 {
                for x in 1..MAP_W - 1 {
                    // Broad contact shading around room edges, retained under local lights.
                    let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter()
                        .filter(|&&(dx, dy)| self.blocked(x as i32 + dx, y as i32 + dy)).count();
                    grid[y * MAP_W + x] = [-0.045 * neighbors as f32; 3];
                }
            }
            for e in &self.ents {
                if e.kind == EK_LAMP {
                    self.add_light(&mut grid, e.x, e.y, 4.5, [0.70, 0.36, 0.12]);
                } else if e.kind == EK_FLAME {
                    self.add_light(&mut grid, e.x, e.y, 3.0, [0.55, 0.16, 0.035]);
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
                                y as f32 + 0.5 + dy as f32 * 0.6, 3.5, [0.08, 0.35, 0.52]);
                        }
                    }
                }
            }
            self.static_light = grid;
            self.light_dirty = false;
        }
        let mut grid = std::mem::take(&mut self.light_grid);
        grid.copy_from_slice(&self.static_light);
        let mut count = 0;
        for e in &self.ents {
            if !matches!(e.kind, EK_PROJ | EK_RAY | EK_BOLT | EK_FIREPATCH | EK_IMPACT)
                || (e.x - self.px).powi(2) + (e.y - self.py).powi(2) > 100.0 { continue; }
            let rgb = if e.kind == EK_PROJ && e.effect_tick == 3.0 { [0.12, 0.45, 0.025] }
                else if matches!(e.kind, EK_RAY | EK_PROJ) { [0.06, 0.38, 0.65] }
                else { [0.36, 0.10, 0.02] };
            self.add_light(&mut grid, e.x, e.y, 2.4, rgb);
            count += 1;
            if count == 12 { break; }
        }
        if self.muzzle > 0.05 {
            let power = self.muzzle * if self.weapon == 0 { 0.2 } else { 0.55 };
            self.add_light(&mut grid, self.px, self.py, 3.0, [power, power * 0.55, power * 0.18]);
        }
        self.light_grid = grid;
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
        if !floor {
            return (if self.hell { T_SKULL } else { T_CEIL }, tx, ty, level);
        }
        let mx = fx.floor() as i32;
        let my = fy.floor() as i32;
        let style = if mx >= 0 && my >= 0 && mx < MAP_W as i32 && my < MAP_H as i32 {
            self.floor[my as usize * MAP_W + mx as usize]
        } else {
            0
        };
        if style == 2 {
            return (T_OVERRIDE, tx, ty, level.saturating_sub(2));
        }
        (if style == 1 { T_CONC } else { T_GRATE }, tx, ty, level)
    }

    fn plane_texel(&self, floor: bool, fx: f32, fy: f32, level: usize, mix: u32) -> u32 {
        let (id, u, v, lvl) = self.plane_sample(floor, fx, fy, level);
        self.sample_mip(id, u, v, lvl, mix)
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
            _p1: 0.0,
            _p2: 0.0,
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
            if tid == T_METAL && hash % 7 == 0 { tid = T_HAZARD; }
            if tid == T_BRICK && hash % 5 == 0 { tid = T_SKULL; }
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
                flash: if e.flash > 0.0 { 1.0 } else if is_hostile_kind(e.kind) && e.effect_tick > 0.0 { 2.0 } else { 0.0 },
                kind: e.kind as f32,
            };
            n += 1;
        }
        scratch.sprite_n = n;
        scratch.view.sprite_n = n as f32;
    }

    fn render(&mut self) {
        self.update_lighting();
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
            if tid == T_METAL && hash % 7 == 0 {
                tid = T_HAZARD;
            }
            if tid == T_BRICK && hash % 5 == 0 {
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
        order[..count].sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
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
            let glow = if e.kind == EK_LAMP {
                1.1 + 0.45 * (tnow * 13.0 + e.x).sin().abs()
            } else if matches!(e.kind, EK_FLAME | EK_BOLT | EK_FIREPATCH) {
                1.3 + 0.45 * (tnow * 18.0 + e.x).sin().abs()
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
                        if rad < 0.9 || rad > 1.0 {
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
                        let core = (1.0 - rad).powf(1.7);
                        col = Self::pack(
                            (40.0 + core * 215.0) as u32,
                            (170.0 + core * 85.0) as u32,
                            255,
                            255,
                        );
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


#[repr(C)]
#[derive(Clone, Copy)]
struct GpuView {
    px: f32, py: f32, dir_x: f32, dir_y: f32,
    plane_x: f32, plane_y: f32, horizon: f32, time: f32,
    hell: f32, muzzle: f32, w: f32, h: f32,
    plane_len: f32, sprite_n: f32, _p1: f32, _p2: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GpuCol {
    perp: f32, tex_x: f32, light_r: f32, light_g: f32,
    light_b: f32, draw0: f32, draw1: f32, line_h: f32,
    ds_full: f32, tex: f32, side: f32, dec: f32,
    hash: f32, hit: f32, z: f32, _pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GpuSprite {
    x: f32, y: f32, zoff: f32, scale: f32,
    tex: f32, frame: f32, flash: f32, kind: f32,
}

struct GpuScratch {
    enemy_cues: [voices::EnemyCue; ENT_N],
    cols: Vec<GpuCol>,
    sprites: Vec<GpuSprite>,
    sprite_n: usize,
    view: GpuView,
}

fn gpu_scratch() -> &'static mut GpuScratch {
    static mut G: Option<GpuScratch> = None;
    unsafe {
        if G.is_none() {
            G = Some(GpuScratch {
                enemy_cues: [voices::EnemyCue::default(); ENT_N],
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
pub extern "C" fn hs_prepare_enemies() -> i32 {
    voices::snapshot(eng(), &mut gpu_scratch().enemy_cues) as i32
}

#[no_mangle]
pub extern "C" fn hs_enemy_cues() -> *const voices::EnemyCue {
    gpu_scratch().enemy_cues.as_ptr()
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
pub extern "C" fn hs_init(w: i32, h: i32) -> i32 {
    eng_init(w as usize, h as usize);
    0
}

#[no_mangle]
pub extern "C" fn hs_restart() {
    let e = eng();
    let textures = std::mem::take(&mut e.tex);
    *e = Engine::with_textures(e.w, e.h, Some(textures));
}

#[no_mangle]
pub extern "C" fn hs_next_wave() {
    eng().next_wave();
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
    let id = id.clamp(0, TEX_N as i32 - 1) as usize;
    let o = id * TEX * TEX;
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

#[no_mangle]
pub extern "C" fn hs_input(bits: u32, mx: f32, my: f32) {
    let e = eng();
    e.bits = bits;
    e.mx = mx;
    e.my = my;
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
    e.has_w2 = true;
    e.has_w3 = true;
    e.has_w4 = true;
    e.has_w5 = true;
    e.has_w6 = true;
    e.has_w7 = true;
    e.mag = MAG_SZ;
    e.ammo = [120, 40, 200, 16, 24, 48, 320];
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
    e.boss_spawned = false;
    e.boss_phase = 0;
    e.maybe_spawn_boss();
    let max_hp = e.boss_max_health();
    if let Some(boss) = e.ents.iter_mut().find(|enemy| enemy.kind == EK_BOSS && enemy.hp > 0) {
        boss.hp = match phase.clamp(0, 2) {
            1 => max_hp * 2 / 3,
            2 => max_hp / 3,
            _ => max_hp,
        };
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
    fn next_wave_stays_within_entity_budget() {
        let mut e = Engine::new(160, 100);
        for _ in 0..11 { e.next_wave(); }
        assert_eq!(e.wave, 12);
        assert!(map::living_hostiles(&e) <= 60, "wave spawn must leave FX slots free");
        e.maybe_spawn_boss();
        let boss = e.ents.iter().find(|x| x.kind == EK_BOSS).expect("boss spawns");
        assert!(boss.hp <= 2200, "late-wave boss must stay killable");
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
        for (kind, texture, frame) in [(EK_SMOKE, T_FLAME, 1), (EK_FIREPATCH, T_FLAME, 0), (EK_BOLT, T_ORDNANCE, 1)] {
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
        assert_eq!(e.ents.iter().filter(|e| e.kind == EK_FIREPATCH).count(), 0);
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
        e.tex[T_BRICK * TEX * TEX..(T_BRICK + 1) * TEX * TEX].fill(Engine::pack(200, 0, 0, 255));
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
        assert!(e.events & EV_EMPTY != 0);
        assert_eq!(e.reload_t, 0.0);
        assert_eq!(e.mag[0], 0);
    }

    #[test]
    fn held_weapon_key_does_not_cancel_reload() {
        let mut e = arena();
        e.has_w2 = true;
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
        e.has_w2 = true;
        e.has_w3 = true;
        e.has_w4 = true;
        e.has_w5 = true;
        e.has_w6 = true;
        e.has_w7 = true;
        for (expected, bit) in [IN_W1, IN_W2, IN_W3, IN_W4, IN_W5, IN_W6, IN_W7]
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
        for bit in [IN_W6, IN_W7] {
            e.bits = bit;
            e.tick(1.0 / 60.0);
            assert_eq!(e.weapon, 0, "locked late-game weapons must not switch");
            e.bits = 0;
            e.tick(1.0 / 60.0);
        }
        e.pickup(EK_GUN6);
        assert!(e.has_w6);
        assert_eq!(e.weapon, 5);
        e.pickup(EK_GUN7);
        assert!(e.has_w7);
        assert_eq!(e.weapon, 6);
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
        assert_eq!(e.ents[boss].anim, ANIM_DEAD);
        assert_eq!(e.state, 2, "the vault master is the win");
        assert_eq!(e.kills, 1);
        for _ in 0..45 { e.tick(1.0 / 60.0); }
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
        assert_eq!(e.state, 2, "killing the boss ends the level");
        e.next_wave();
        assert_eq!((e.state, e.wave), (0, 2));
        assert!(!e.boss_spawned && !e.hell);
        assert!(map::living_hostiles(&e) > 0, "the next level spawns its cast");
    }

    #[test]
    fn next_wave_restores_the_body_and_keeps_found_gear() {
        let mut e = arena();
        e.health = 11;
        e.state = 1;
        e.has_w4 = true;
        e.weapon = 3;
        e.ammo[3] = 1;
        e.mag[3] = 0;
        let _med = e.spawn(EK_MED, 8.5, 8.5).unwrap();
        e.spawn(EK_HUSK, 9.5, 8.5).unwrap();
        e.next_wave();
        assert_eq!(e.wave, 2);
        assert_eq!(e.health, 100);
        assert_eq!(e.state, 0);
        assert!(e.has_w4);
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
        assert_eq!(e.wave, 12, "waves cap at 12");
        assert!(e.ents.iter().any(|en| en.kind == EK_MED));
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
    fn ammo_crates_restock_the_complete_seven_weapon_arsenal() {
        let mut e = arena();
        e.has_w2 = true;
        e.has_w3 = true;
        e.has_w4 = true;
        e.has_w5 = true;
        e.has_w6 = true;
        e.has_w7 = true;
        e.ammo = [0; 7];
        let item = e.spawn(EK_AMMO, e.px, e.py).unwrap();
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents[item].kind, EK_NONE, "a useful ammo crate is collected");
        assert_eq!(e.ammo, [18, 10, 45, 5, 2, 15, 90]);

        e.ammo = [120, 48, 216, 20, 16, 80, 450];
        let full = e.spawn(EK_AMMO, e.px, e.py).unwrap();
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents[full].kind, EK_AMMO, "a full arsenal leaves supplies available");
    }

    #[test]
    fn override_requires_use_then_spawns_the_sector_boss() {
        let mut e = arena();
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
        assert_eq!(bosses[0].hp, 720);
        assert_eq!(bosses[0].skin, SKIN_GUNNER);
        assert!(e.boss_spawned);
        assert!(e.hell);
        e.tick(1.0 / 60.0);
        assert_eq!(e.ents.iter().filter(|en| en.kind == EK_BOSS).count(), 1);
    }

    #[test]
    fn override_use_requires_a_clear_path_to_the_console() {
        let mut e = arena();
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
        assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.kind == EK_SMOKE));
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
        foundry.ents[boss].hp = 400;
        foundry.tick(1.0 / 60.0);
        assert_eq!(foundry.ents.iter().filter(|e| e.kind == EK_PROJ).count(), 8);
        assert!(foundry.ents.iter().any(|e| e.skin == SKIN_GUNNER && e.kind == EK_BOSS));

        let mut bioforge = arena();
        bioforge.wave = 3;
        bioforge.boss_spawned = true;
        let boss = bioforge.spawn_with_skin(EK_BOSS, map::boss_skin(3), 9.5, 4.5).unwrap();
        bioforge.ents[boss].hp = 700;
        bioforge.tick(1.0 / 60.0);
        assert_eq!(bioforge.ents.iter().filter(|e| e.kind == EK_PROJ && e.effect_tick == 3.0).count(), 6);
        assert!(bioforge.ents.iter().any(|e| e.skin == SKIN_VATBRUTE && e.kind == EK_BOSS));
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
        assert_eq!(e.wall_tex(2, 1, 1), T_BRICK);
        assert_eq!(e.wall_tex(8, 36, 18), T_DOOR);
        e.hell = true;
        assert_eq!(e.wall_tex(2, 1, 1), T_FLESH);
        assert_eq!(e.wall_tex(6, 1, 1), T_FLESH, "tech becomes flesh, not skull");
        assert_eq!(e.wall_tex(7, 1, 1), T_SKULL, "hazard striping becomes skull");
        assert_eq!(e.wall_tex(9, 1, 1), T_SKULL);
        assert_eq!(e.wall_tex(8, 36, 18), T_FLESH);
        assert_eq!(e.wall_tex(9, 36, 19), T_SKULL);
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
        assert_eq!(std::mem::size_of::<GpuCol>() / 4, 16);
        assert_eq!(std::mem::size_of::<GpuView>() / 4, 16);
        assert_eq!(std::mem::size_of::<GpuSprite>() / 4, 8);
    }
}
