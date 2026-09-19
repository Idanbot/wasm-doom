mod consts;
mod events;
mod hud;
mod types;

use consts::*;
use events::*;
pub use hud::{Hud, HUD_OFFSETS, HUD_SIZE};
use types::{Ent, FxCmd};

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
    ammo: [i32; 5],
    mag: [i32; 5],
    weapon: i32,
    has_w2: bool,
    has_w3: bool,
    has_w4: bool,
    has_w5: bool,
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
    matches!(k, EK_HUSK | EK_BRUTE | EK_WRAITH | EK_BARREL | EK_BOSS)
}

fn is_pickup(k: u8) -> bool {
    matches!(
        k,
        EK_MED | EK_AMMO | EK_ARMOR | EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5
    )
}

fn is_weapon_item(k: u8) -> bool {
    matches!(k, EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5)
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
                kind: 0,
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                hp: 0,
                timer: 0.0,
                frame: 0.0,
                radius: 0.25,
                flash: 0.0,
                stun: 0.0,
                effect_tick: 0.0,
                zoff: 0.0,
            }; ENT_N],
            px: 4.5,
            py: 6.5,
            pa: 0.0,
            pitch: 0.0,
            pr: 0.22,
            health: 100,
            armor: 0,
            ammo: [36, 0, 0, 0, 0],
            mag: [12, 0, 0, 0, 0],
            weapon: 0,
            has_w2: false,
            has_w3: false,
            has_w4: false,
            has_w5: false,
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
                x: 4.5,
                y: 6.5,
                reserve: 36,
                reloading: 0.0,
                weap_frame: 0,
                has_w4: 0,
                has_w5: 0,
                events: 0,
                ev_weapon: 0,
                wave: 1,
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

    fn on_seal(&self) -> bool {
        let x = self.px.floor() as i32;
        let y = self.py.floor() as i32;
        x >= SEAL_X && y >= SEAL_Y && x < SEAL_X + SEAL_W && y < SEAL_Y + SEAL_H
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
        self.map.fill(1);
        self.floor.fill(0);
        self.decal.fill(0);
        // start hangar — riveted steel with hazard strips
        self.room(1, 1, 15, 12, 1, 0);
        // metal lab — live CRT walls
        self.room(18, 2, 13, 11, 6, 0);
        // brick chapel
        self.room(33, 1, 14, 14, 2, 1);
        // barracks — hazard / caution
        self.room(1, 15, 16, 13, 7, 1);
        // flesh pit
        self.room(20, 16, 16, 13, 3, 0);
        // vault
        self.room(37, 18, 10, 11, 5, 1);
        // secret alcove off chapel
        self.room(40, 1, 7, 5, 2, 1);

        self.hall_h(15, 18, 6, 16);
        self.hall_h(30, 33, 7, 31);
        self.hall_v(7, 12, 15, 13);
        self.hall_h(16, 20, 21, 18);
        self.hall_h(35, 37, 23, 36);
        // secret door in chapel north
        self.set_cell(42, 5, 9);
        self.set_cell(43, 5, 9);

        self.pillar(38, 6, 2);
        self.pillar(24, 6, 6);
        self.pillar(26, 21, 3);
        self.pillar(8, 20, 7);

        // hangar hazard bulkheads
        self.set_cell(1, 5, 7);
        self.set_cell(1, 6, 7);
        self.set_cell(1, 7, 7);
        self.set_cell(15, 4, 7);
        self.set_cell(15, 5, 7);
        // lab monitor bulk
        self.set_cell(18, 5, 6);
        self.set_cell(18, 6, 6);
        self.set_cell(30, 8, 6);
        // chapel bone niches
        self.set_cell(33, 8, 5);
        self.set_cell(46, 8, 5);

        self.set_cell(42, 23, 10);
        self.set_cell(43, 23, 10);
        self.set_cell(42, 24, 10);
        self.set_cell(43, 24, 10);

        self.mix_edge(1, 1, 15, 12, &[1, 7, 4, 1, 6, 7]);
        self.mix_edge(18, 2, 13, 11, &[6, 1, 6, 4, 6]);
        self.mix_edge(33, 1, 14, 14, &[2, 5, 2, 2, 5]);
        self.mix_edge(1, 15, 16, 13, &[7, 1, 4, 7, 1]);
        self.mix_edge(20, 16, 16, 13, &[3, 2, 3, 3]);
        self.mix_edge(37, 18, 10, 11, &[1, 6, 4, 1]);
        self.mix_edge(40, 1, 7, 5, &[2, 5, 2]);
        for j in SEAL_Y..SEAL_Y + SEAL_H {
            for i in SEAL_X..SEAL_X + SEAL_W {
                if self.cell(i, j) == 0 {
                    let idx = j as usize * MAP_W + i as usize;
                    if idx < self.floor.len() {
                        self.floor[idx] = 2;
                    }
                }
            }
        }
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
                        T_GUN => {
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
        let (hp, radius, zoff) = match kind {
            EK_HUSK => (28, 0.28, 0.0),
            EK_BRUTE => (78, 0.38, 0.0),
            EK_WRAITH => (20, 0.26, -70.0),
            EK_BOSS => (520, 0.62, 8.0),
            EK_MED | EK_AMMO | EK_ARMOR | EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 => (1, 0.22, 34.0),
            EK_BARREL => (14, 0.3, 78.0),
            EK_PROJ => (1, 0.12, -10.0),
            EK_RAY => (1, 0.1, -6.0),
            EK_BOLT => (1, 0.14, 12.0),
            EK_GIB => (1, 0.08, 0.0),
            EK_LAMP => (1, 0.16, -118.0),
            EK_CRATE => (1, 0.32, 86.0),
            EK_CHAIN => (1, 0.12, -104.0),
            EK_IMPACT => (1, 0.1, -6.0),
            EK_SPARK => (1, 0.06, 0.0),
            EK_SMOKE => (1, 0.1, -4.0),
            EK_FLAME => (1, 0.14, 42.0),
            _ => (1, 0.2, 0.0),
        };
        for (i, e) in self.ents.iter_mut().enumerate() {
            if e.kind == 0 {
                *e = Ent {
                    kind,
                    x,
                    y,
                    vx: 0.0,
                    vy: 0.0,
                    hp,
                    timer: 0.4,
                    frame: 0.0,
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
                kind: f.kind,
                x: f.x,
                y: f.y,
                vx: f.vx,
                vy: f.vy,
                hp,
                timer: f.timer,
                frame: 0.0,
                radius,
                flash: 0.0,
                stun: 0.0,
                effect_tick: 0.0,
                zoff: if f.zoff != 0.0 { f.zoff } else { zdef },
            };
        }
        self.fx_n = 0;
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
        self.ents = [Ent {
            kind: 0,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            hp: 0,
            timer: 0.0,
            frame: 0.0,
            radius: 0.25,
            flash: 0.0,
            stun: 0.0,
            effect_tick: 0.0,
            zoff: 0.0,
        }; ENT_N];
        self.spawn(EK_MED, 12.5, 9.5);
        self.spawn(EK_LAMP, 6.5, 4.5);
        self.spawn(EK_LAMP, 12.5, 8.5);
        self.spawn(EK_CRATE, 3.5, 9.5);
        self.spawn(EK_CRATE, 9.5, 10.6);
        self.spawn(EK_FLAME, 7.5, 2.6);
        self.spawn(EK_FLAME, 11.5, 10.4);
        self.spawn(EK_CHAIN, 8.5, 3.4);
        self.spawn(EK_GUN2, 24.5, 4.5);
        self.spawn(EK_AMMO, 28.5, 10.5);
        self.spawn(EK_LAMP, 21.5, 4.5);
        self.spawn(EK_LAMP, 28.5, 8.5);
        self.spawn(EK_CRATE, 20.5, 10.5);
        self.spawn(EK_FLAME, 25.5, 3.4);
        self.spawn(EK_ARMOR, 35.5, 12.5);
        self.spawn(EK_AMMO, 44.5, 12.5);
        // Introduce the Lance before the chapel's aligned firing line.
        self.spawn(EK_GUN4, 35.5, 8.5);
        self.spawn(EK_LAMP, 36.5, 5.5);
        self.spawn(EK_CHAIN, 40.5, 8.5);
        self.spawn(EK_FLAME, 38.5, 3.5);
        self.spawn(EK_GUN3, 43.5, 2.7);
        self.spawn(EK_MED, 41.5, 2.7);
        self.spawn(EK_BARREL, 4.5, 17.5);
        self.spawn(EK_BARREL, 14.5, 18.5);
        self.spawn(EK_BARREL, 10.5, 25.5);
        self.spawn(EK_MED, 3.5, 25.5);
        self.spawn(EK_LAMP, 8.5, 18.5);
        self.spawn(EK_CRATE, 14.5, 22.5);
        self.spawn(EK_FLAME, 3.5, 18.6);
        self.spawn(EK_AMMO, 33.5, 18.5);
        // The pit entrance gives the Pyre a doorway to hold against approaching brutes.
        self.spawn(EK_GUN5, 22.5, 19.5);
        self.spawn(EK_LAMP, 25.5, 22.5);
        self.spawn(EK_CHAIN, 22.5, 18.5);
        self.spawn(EK_CHAIN, 30.5, 24.5);
        self.spawn(EK_FLAME, 28.5, 18.4);
        self.spawn(EK_MED, 39.5, 26.5);
        self.spawn(EK_LAMP, 40.5, 22.5);
        self.spawn_hostiles(1);
    }

    fn spawn_hostiles(&mut self, mult: i32) {
        let copies = mult.max(1);
        for n in 0..copies {
            for &(k, x, y) in &HOSTILES {
                let jx = if n == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
                let jy = if n == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
                let nx = x + jx;
                let ny = y + jy;
                if !self.blocked(nx.floor() as i32, ny.floor() as i32) {
                    if self.spawn(k, nx, ny).is_none() {
                        return;
                    }
                } else if self.spawn(k, x, y).is_none() {
                    return;
                }
            }
        }
    }

    fn next_wave(&mut self) {
        self.wave = (self.wave + 1).min(12);
        let shift = (self.wave - 1).clamp(0, 8);
        let mult = 1i32 << shift;
        self.health = 100;
        self.iframes = 1.4;
        self.px = 4.5;
        self.py = 6.5;
        self.pa = 0.0;
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
            self.ammo[4] = self.ammo[4].max(60);
        }
        for e in self.ents.iter_mut() {
            if matches!(
                e.kind,
                EK_HUSK | EK_BRUTE | EK_WRAITH | EK_PROJ | EK_RAY | EK_BOLT | EK_BOSS | EK_FIREPATCH
            ) {
                e.kind = 0;
            }
        }
        self.hell = false;
        self.light_dirty = true;
        self.boss_spawned = false;
        self.boss_intro = 0.0;
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
        let hp = (480.0 * 1.5f32.powi((self.wave - 1).max(0))).round() as i32;
        let fx = self.pa.cos();
        let fy = self.pa.sin();
        let spots = [
            (41.5, 22.5),
            (42.7, 24.6),
            (self.px + fx * 4.6, self.py + fy * 4.6),
            (self.px + fx * 3.2 - fy * 2.4, self.py + fy * 3.2 + fx * 2.4),
        ];
        for (x, y) in spots {
            if self.blocked(x.floor() as i32, y.floor() as i32) {
                continue;
            }
            if let Some(i) = self.spawn(EK_BOSS, x, y) {
                self.ents[i].hp = hp;
                break;
            }
        }
        for _ in 0..14 {
            let a = self.rnd() * core::f32::consts::TAU;
            let r = 0.5 + self.rnd() * 1.9;
            self.spawn_timed(
                EK_FLAME,
                self.px + a.cos() * r,
                self.py + a.sin() * r,
                2.4,
                18.0,
            );
        }
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

    fn sample_mip(&self, id: usize, u: i32, v: i32, level: usize, mix: u32) -> u32 {
        let sample = |level: usize| {
            if level == 0 { return self.sample(id, u, v); }
            let size = TEX >> level;
            let x = ((u & TEXM) as usize) >> level;
            let y = ((v & TEXM) as usize) >> level;
            self.mipmaps[level - 1][id * size * size + y * size + x]
        };
        let a = sample(level);
        let b = sample((level + 1).min(8));
        let rb = (((a & 0x00ff00ff) * (256 - mix) + (b & 0x00ff00ff) * mix) >> 8) & 0x00ff00ff;
        let g = (((a & 0x0000ff00) * (256 - mix) + (b & 0x0000ff00) * mix) >> 8) & 0x0000ff00;
        rb | g | (a & 0xff000000)
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

    fn is_boss_door(&self, x: i32, y: i32) -> bool {
        x == 36 && (y == 23 || y == 24)
    }

    fn wall_tex(&self, c: u8, x: i32, y: i32) -> usize {
        if (c == 8 || c == 9) && self.is_boss_door(x, y) {
            return T_SEAL;
        }
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
            T_SEAL => T_SEAL,
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
        if c == 9 && self.secrets == 0 {
            self.secrets = 1;
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
        self.shake = (self.shake + 0.8).min(1.0);
        self.events |= EV_EXPLODE;
        let pd = ((self.px - x).powi(2) + (self.py - y).powi(2)).sqrt();
        if pd < radius && self.los(x, y, self.px, self.py) {
            let fall = 1.0 - pd / radius;
            self.damage_player((dmg * fall) as i32);
        }
        let mut hits: Vec<(usize, i32)> = Vec::new();
        for (i, e) in self.ents.iter().enumerate() {
            if !solid_kind(e.kind) {
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
            if e.kind == 0 {
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
                self.hitmarker = 1.0;
                self.events |= EV_HIT;
                return;
            }
            e.kind = 0;
        }
        self.hitmarker = 1.0;
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
            if e.kind == 0 || !solid_kind(e.kind) {
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
            if !solid_kind(e.kind) { continue; }
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
                self.muzzle_sprite();
            }
            1 => {
                self.cooldown = 0.62;
                self.muzzle = 1.0;
                self.kick = 1.4;
                self.shake = (self.shake + 0.38).min(1.0);
                for _ in 0..8 {
                    let a = self.pa + (self.rnd() - 0.5) * 0.22;
                    self.hitscan(a, 7, 11.0);
                }
                self.eject_casing();
                self.muzzle_sprite();
            }
            2 => {
                self.cooldown = 0.065;
                self.muzzle = 1.0;
                self.kick = 0.7;
                self.shake = (self.shake + 0.08).min(1.0);
                self.spread = (self.spread + 0.028).min(0.24);
                let a = self.pa + (self.rnd() - 0.5) * (0.03 + self.spread);
                self.hitscan(a, 8, 20.0);
                if (self.rng & 1) == 0 {
                    self.eject_casing();
                }
                self.muzzle_sprite();
            }
            3 => {
                self.cooldown = 0.85;
                self.muzzle = 1.0;
                self.kick = 1.1;
                self.shake = (self.shake + 0.22).min(1.0);
                self.fire_lance();
                self.muzzle_sprite();
            }
            _ => {
                self.cooldown = 0.07;
                self.muzzle = 0.7;
                self.kick = 0.4;
                self.shake = (self.shake + 0.05).min(1.0);
                for _ in 0..2 {
                    let a = self.pa + (self.rnd() - 0.5) * 0.12;
                    let spd = 8.2 + self.rnd() * 2.4;
                    let x = self.px + a.cos() * 0.15;
                    let y = self.py + a.sin() * 0.15;
                    if let Some(i) = self.spawn(EK_BOLT, x, y) {
                        self.ents[i].vx = a.cos() * spd;
                        self.ents[i].vy = a.sin() * spd;
                        self.ents[i].timer = 0.42 + self.rnd() * 0.12;
                        self.ents[i].zoff = 10.0 + self.rnd() * 8.0;
                    }
                }
            }
        }
    }

    fn muzzle_sprite(&mut self) {
        let x = self.px + self.pa.cos() * 0.42;
        let y = self.py + self.pa.sin() * 0.42;
        self.spawn_timed(EK_SPARK, x, y, 0.08, 12.0);
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
        self.queue_fx(EK_SPARK, x, y, rx * jx + pax, ry * jy + pay, 0.35, 12.0);
    }

    fn enemy_shoot(&mut self, i: usize) {
        let (x, y, kind) = (self.ents[i].x, self.ents[i].y, self.ents[i].kind);
        let a = (self.py - y).atan2(self.px - x);
        let sp = if kind == EK_WRAITH { 7.2 } else { 5.4 };
        let zoff = if kind == EK_WRAITH { -50.0 } else { -8.0 };
        self.spawn(EK_PROJ, x, y);
        if let Some(e) = self.ents.iter_mut().rev().find(|e| e.kind == EK_PROJ && e.vx == 0.0) {
            e.vx = a.cos() * sp;
            e.vy = a.sin() * sp;
            e.timer = 2.4;
            e.zoff = zoff;
        }
        self.queue_fx(EK_SPARK, x, y, 0.0, 0.0, 0.12, zoff);
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
                    self.ammo[1] = (self.ammo[1] + 8).min(40);
                }
                if self.has_w3 {
                    self.ammo[2] = (self.ammo[2] + 40).min(200);
                }
                if self.has_w4 {
                    self.ammo[3] = (self.ammo[3] + 4).min(16);
                }
                if self.has_w5 {
                    self.ammo[4] = (self.ammo[4] + 24).min(80);
                }
                self.events |= EV_PICK_SILVER;
            }
            EK_ARMOR => {
                self.armor = (self.armor + 50).min(100);
                self.events |= EV_PICK_SILVER;
            }
            EK_GUN2 => {
                self.has_w2 = true;
                self.ammo[1] = (self.ammo[1] + 6).min(40);
                if self.mag[1] <= 0 {
                    self.mag[1] = MAG_SZ[1];
                }
                self.weapon = 1;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN3 => {
                self.has_w3 = true;
                self.ammo[2] = (self.ammo[2] + 48).min(200);
                if self.mag[2] <= 0 {
                    self.mag[2] = MAG_SZ[2];
                }
                self.weapon = 2;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN4 => {
                self.has_w4 = true;
                self.ammo[3] = (self.ammo[3] + 6).min(16);
                if self.mag[3] <= 0 {
                    self.mag[3] = MAG_SZ[3];
                }
                self.weapon = 3;
                self.reload_t = 0.0;
                self.events |= EV_PICK_GOLD;
            }
            EK_GUN5 => {
                self.has_w5 = true;
                self.ammo[4] = (self.ammo[4] + 40).min(80);
                if self.mag[4] <= 0 {
                    self.mag[4] = MAG_SZ[4];
                }
                self.weapon = 4;
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
                || (self.has_w2 && self.ammo[1] < 40)
                || (self.has_w3 && self.ammo[2] < 200)
                || (self.has_w4 && self.ammo[3] < 16)
                || (self.has_w5 && self.ammo[4] < 80),
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
            self.wpn_latched = bits & (IN_W1 | IN_W2 | IN_W3 | IN_W4 | IN_W5);

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
            if self.cell(self.px.floor() as i32, self.py.floor() as i32) == 10 {
                self.maybe_spawn_boss();
            }
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
            if !matches!(e.kind, EK_HUSK | EK_BRUTE | EK_WRAITH | EK_BOSS) {
                e.frame += dt;
            }
            match e.kind {
                EK_HUSK | EK_BRUTE | EK_WRAITH | EK_BOSS => {
                    if pstate != 0 {
                        continue;
                    }
                    if e.stun > 0.0 {
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
                    let spd = match e.kind {
                        EK_BRUTE => 2.15,
                        EK_WRAITH => 2.7,
                        EK_BOSS => 1.28,
                        _ => 1.7,
                    };
                    let hold = match e.kind {
                        EK_BRUTE => 0.95,
                        EK_WRAITH => 3.2,
                        EK_BOSS => 1.45,
                        _ => 3.6,
                    };
                    let (ex, ey, kind) = (e.x, e.y, e.kind);
                    let _ = e;
                    let clear = dist < 16.0 && self.los(ex, ey, px, py);
                    let e = &mut self.ents[i];
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
                    } else if kind == EK_WRAITH {
                        e.x += -dy / dist * spd * dt;
                        e.y += dx / dist * spd * dt;
                    }
                    let moved = ((e.x - ex0).powi(2) + (e.y - ey0).powi(2)).sqrt();
                    e.frame += dt * 1.4 + moved * 6.5;
                    if kind == EK_BRUTE && clear && dist < 1.15 && e.timer <= 0.0 {
                        melee.push((i, 14));
                        e.timer = 0.75;
                    }
                    if kind == EK_BOSS && clear && dist < 1.5 && e.timer <= 0.0 {
                        let bd = (22.0 * 1.2f32.powi((self.wave - 1).max(0))).round().max(1.0) as i32;
                        melee.push((i, bd));
                        e.timer = 0.7;
                    }
                    if kind != EK_BRUTE && kind != EK_BOSS && clear && dist < 10.0 && e.timer <= 0.0 {
                        shots.push(i);
                        e.timer = if kind == EK_WRAITH { 0.9 } else { 1.15 };
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
                    e.x += e.vx * dt;
                    e.y += e.vy * dt;
                    e.timer -= dt;
                    let (ex, ey, dead) = (e.x, e.y, e.timer <= 0.0);
                    let _ = e;
                    if dead || self.blocked(ex.floor() as i32, ey.floor() as i32) {
                        if self.blocked(ex.floor() as i32, ey.floor() as i32) {
                            self.spawn_timed(EK_IMPACT, ex, ey, 0.22, -6.0);
                        }
                        self.ents[i].kind = 0;
                    } else if pstate == 0 {
                        let d = (ex - px).powi(2) + (ey - py).powi(2);
                        if d < 0.22 {
                            self.ents[i].kind = 0;
                            melee.push((i, 11));
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
                            if !solid_kind(target.kind) { continue; }
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
                        self.ignite(old_x, old_y);
                        self.spawn_timed(EK_SMOKE, ex, ey, 0.2, 8.0);
                    } else {
                        let mut hit = None;
                        for (j, o) in self.ents.iter().enumerate() {
                            if j == i || !solid_kind(o.kind) {
                                continue;
                            }
                            let d2 = (o.x - ex).powi(2) + (o.y - ey).powi(2);
                            if d2 < (o.radius + 0.2).powi(2) {
                                hit = Some(j);
                                break;
                            }
                        }
                        if let Some(j) = hit {
                            self.ents[i].kind = 0;
                            self.hurt_ent(j, 3, ex, ey);
                            self.ignite(ex, ey);
                            self.spawn_timed(EK_SMOKE, ex, ey, 0.16, 10.0);
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
            if self.ents[i].kind != 0 {
                self.enemy_shoot(i);
            }
        }
        for (_i, dmg) in melee {
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
            if matches!(e.kind, EK_HUSK | EK_BRUTE | EK_WRAITH | EK_BOSS) {
                living += 1;
            }
        }
        if !self.boss_spawned && self.state == 0 {
            if self.boss_intro <= 0.0 {
                if living == 0 && self.on_seal() {
                    self.boss_intro = 6.0;
                    self.events |= EV_BOSS;
                }
            } else {
                let prev = self.boss_intro;
                self.boss_intro -= dt;
                if prev > 1.0 && self.boss_intro <= 1.0 {
                    self.hell = true;
                    self.shake = (self.shake + 0.35).min(1.0);
                    self.events |= EV_BOSS_HUSH;
                }
                if self.boss_intro <= 0.0 {
                    self.boss_intro = 0.0;
                    self.maybe_spawn_boss();
                }
                living = 1;
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
        if self.cell(self.px.floor() as i32, self.py.floor() as i32) == 10 {
            prompt = 3;
        }

        let weap_frame = if self.reload_t > 0.0 {
            let p = (1.0 - self.reload_t / self.reload_dur.max(0.05)).clamp(0.0, 0.999);
            5 + (p * 4.0) as i32
        } else if self.muzzle > 0.08 {
            if self.weapon == 2 || self.weapon == 4 {
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
            if !matches!(e.kind, EK_PROJ | EK_RAY | EK_BOLT | EK_FIREPATCH)
                || (e.x - self.px).powi(2) + (e.y - self.py).powi(2) > 100.0 { continue; }
            let rgb = if e.kind == EK_RAY { [0.06, 0.38, 0.65] } else { [0.36, 0.10, 0.02] };
            self.add_light(&mut grid, e.x, e.y, 2.4, rgb);
            count += 1;
            if count == 12 { break; }
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
            // Contiguous writes; projection, mip choice and fog are shared by the row.
            for x in 0..w {
                if x % 8 == 0 {
                    let light = self.light_at(start_x + step_x * x as f32, start_y + step_y * x as f32);
                    let next = self.light_at(start_x + step_x * (x + 8) as f32, start_y + step_y * (x + 8) as f32);
                    for c in 0..3 {
                        gain[c] = ((shade + light[c]).clamp(0.18, 1.8) * 65536.0) as i32;
                        let target = ((shade + next[c]).clamp(0.18, 1.8) * 65536.0) as i32;
                        gain_step[c] = (target - gain[c]) / 8;
                    }
                } else { for c in 0..3 { gain[c] += gain_step[c]; } }
                if (y as i32) >= walls[x][0] && (y as i32) <= walls[x][1] { continue; }
                let fx = start_x + step_x * x as f32;
                let fy = start_y + step_y * x as f32;
                let tx = (fx * TEX as f32).floor() as i32;
                let ty = (fy * TEX as f32).floor() as i32;
                let mx = fx.floor() as i32;
                let my = fy.floor() as i32;
                let mut color = if floor {
                    let style = if mx >= 0 && my >= 0 && mx < MAP_W as i32 && my < MAP_H as i32 {
                        self.floor[my as usize * MAP_W + mx as usize]
                    } else { 0 };
                    let tid = if style == 1 { T_CONC } else { T_GRATE };
                    if style == 2 {
                        let u = ((fx - SEAL_X as f32) / SEAL_W as f32 * TEX as f32) as i32;
                        let v = ((fy - SEAL_Y as f32) / SEAL_H as f32 * TEX as f32) as i32;
                        self.sample_mip(T_SEAL, u, v, level.saturating_sub(3), mix)
                    } else { self.sample_mip(tid, tx, ty, level, mix) }
                } else {
                    self.sample_mip(if self.hell { T_SKULL } else { T_CEIL }, tx, ty, level, mix)
                };
                let r = (((color & 255) * gain[0] as u32) >> 16).min(255);
                let g = ((((color >> 8) & 255) * gain[1] as u32) >> 16).min(255);
                let b = ((((color >> 16) & 255) * gain[2] as u32) >> 16).min(255);
                color = r | (g << 8) | (b << 16);
                self.fb[y * w + x] = color | alpha;
            }
        }
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
                for y in draw0..=draw1 {
                    let mut tex_y = (tex_pos as i32) & TEXM;
                    tex_pos += step;
                    if tid == T_FLESH {
                        tex_y = (tex_y + ((tnow * 7.0).sin() * 4.0) as i32) & TEXM;
                    } else if tid == T_TECH {
                        tex_y = (tex_y + (tnow * 26.0) as i32) & TEXM;
                    } else if tid == T_PIPES {
                        tex_y = (tex_y + (tnow * 10.0) as i32) & TEXM;
                    } else if tid == T_SKULL {
                        tex_y = (tex_y + ((tnow * 3.5).sin() * 3.0) as i32) & TEXM;
                    } else if tid == T_BRICK {
                        let drip = Self::nbit(3, tex_x, 0);
                        if drip > 0.84 {
                            tex_y = (tex_y + (tnow * (10.0 + drip * 18.0)) as i32) & TEXM;
                        }
                    }
                    let mut col = self.sample(tid, tex_x, tex_y);
                    if tid == T_TECH && (tex_y & 7) == 0 {
                        col = Self::shade(col, 1.35);
                    }
                    if (tid == T_METAL || tid == T_HAZARD) && (tex_y & 31) < 3 && (hash & 1) == 0 {
                        col = Self::blend(col, Self::pack(255, 140, 40, 255), 0.35);
                    }
                    if dec > 0 {
                        let splat = self.sample(T_SPLAT, tex_x, tex_y.wrapping_add(dec as i32 * 17));
                        if ((splat >> 24) & 255) > 24 {
                            col = Self::blend(col, splat, 0.28 + dec as f32 * 0.18);
                        }
                    }
                    col = Self::shade_rgb(col, dist_mul, light);
                    if tid == T_TECH && (tex_y & 31) < 2 {
                        col = Self::blend(col, Self::pack(64, 190, 224, 255), 0.62);
                    }
                    col = Self::fog(col, perp);
                    let yy = y + shy;
                    let xx = x as i32 + shx;
                    if yy >= 0 && yy < h as i32 && xx >= 0 && xx < w as i32 {
                        self.fb[yy as usize * w + xx as usize] = col;
                    }
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
            let (tid, scale, sheet4) = match e.kind {
                EK_HUSK => (T_HUSK, 0.95, true),
                EK_BRUTE => (T_BRUTE, 1.25, true),
                EK_BOSS => (T_BRUTE, 2.45, true),
                EK_WRAITH => (T_WRAITH, 0.7, true),
                EK_PROJ => (T_BALL, 0.28, false),
                EK_RAY => (T_BALL, 0.10, false),
                EK_FIREPATCH => (T_FLAME, 0.60, true),
                EK_BOLT => (T_FLAME, 0.36, true),
                EK_MED => (T_MED, 0.38, false),
                EK_AMMO => (T_AMMO, 0.4, false),
                EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 => (T_GUN, 0.46, false),
                EK_ARMOR => (T_ARMOR, 0.48, false),
                EK_BARREL => (T_BARREL, 0.72, false),
                EK_GIB => (T_SPLAT, 0.18, false),
                EK_LAMP => (T_LAMP, 0.52, false),
                EK_CRATE => (T_CRATE, 0.62, false),
                EK_CHAIN => (T_CHAIN, 1.05, false),
                EK_IMPACT => (T_IMPACT, 0.48, true),
                EK_SPARK => (T_MUZZLEFX, 0.05, true),
                EK_SMOKE => (T_FLAME, 0.28, true),
                EK_FLAME => (T_FLAME, 0.82, true),
                _ => (T_SPLAT, 0.3, false),
            };
            let sprite_h = (h as f32 / ty * scale).abs();
            let voff = e.zoff / ty;
            let ds_y = (-sprite_h * 0.5 + horizon + voff) as i32;
            let de_y = (sprite_h * 0.5 + horizon + voff) as i32;
            let sprite_w = sprite_h;
            let screen_x = (w as f32 / 2.0) * (1.0 + tx / ty);
            let ds_x = (-sprite_w / 2.0 + screen_x) as i32;
            let de_x = (sprite_w / 2.0 + screen_x) as i32;
            let flash = e.flash > 0.0;
            let fr = if sheet4 {
                ((e.frame * 4.0) as i32).rem_euclid(4)
            } else {
                0
            };
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
                        let r8 = col & 255;
                        let g8 = (col >> 8) & 255;
                        let b8 = (col >> 16) & 255;
                        if item && lum < 48 {
                            continue;
                        }
                        if matches!(e.kind, EK_BOLT | EK_PROJ | EK_SPARK | EK_SMOKE | EK_IMPACT)
                            && (lum < 70 || (e.kind == EK_IMPACT && g8 + 18 < r8 && g8 + 18 < b8 && r8.max(b8) < 130))
                        {
                            continue;
                        }
                        if flash || e.kind == EK_BOSS {
                            col = if e.kind == EK_BOSS {
                                Self::blend(col, Self::pack(255, 36, 24, 255), 0.42)
                            } else {
                                Self::pack(255, 220, 220, 255)
                            };
                        }
                    }
                    col = Self::fog(Self::shade_rgb(col, glow, sprite_light), ty);
                    let yy = y + shy;
                    let xx = stripe + shx;
                    if yy >= 0 && yy < h as i32 && xx >= 0 && xx < w as i32 {
                        self.fb[yy as usize * w + xx as usize] = col;
                    }
                }
            }
        }

    }
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
    e.mag = MAG_SZ;
    e.ammo = [120, 40, 200, 16, 80];
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
}
