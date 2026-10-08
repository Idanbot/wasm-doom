//! The WASM ABI surface.
//!
//! Every `#[no_mangle]` export lives here, along with the save/load blob. The
//! rule for this module is that it only validates and translates arguments:
//! simulation lives in the `sim` modules, rendering in `render`, and lighting in
//! `lighting`. Anything that needs to reason about game state belongs there
//! instead of growing a branch here.
//!
//! Bounds checking, NaN rejection and pointer-null returns are deliberate: this
//! is the only place untrusted values enter the engine, and `panic = "abort"`
//! turns a panic into an unrecoverable WASM trap.

use super::*;
use super::render::{gpu_scratch, GpuCol, GpuSprite, GpuView};

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
pub(crate) struct RunSave {
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

pub(crate) fn capture_save(e: &Engine) -> RunSave {
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

pub(crate) fn apply_save(e: &mut Engine, s: &RunSave) {
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

static mut AGENT_ITEMS: [agent_view::ItemCue; ENT_N] = [agent_view::ItemCue { id:0.0, category:0.0,x:0.0,y:0.0,screen_x:0.0,distance:0.0,slot:0.0 }; ENT_N];
#[no_mangle]
pub extern "C" fn hs_prepare_agent_items()->i32 {
    // Same single-threaded scratch contract as enemy presentation cues.
    let out=unsafe { &mut *core::ptr::addr_of_mut!(AGENT_ITEMS) };
    agent_view::snapshot(eng(),out) as i32
}
#[no_mangle]
pub extern "C" fn hs_agent_items()->*const agent_view::ItemCue {core::ptr::addr_of!(AGENT_ITEMS).cast()}
