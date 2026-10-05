import { DEFAULT_SENSITIVITY, normalizeSensitivity } from "./input-settings";
import { automapCells } from "./automap-data";
import BOSS_ARENAS from "./boss-arena-data.json";
import { SECTOR_ENEMIES } from "./sector-enemies";
import { SECTOR_SLUGS, SECTOR_SURFACES } from "./sector-assets";
import { CAMPAIGN_EXPANSION } from "./campaign25";
import { asset } from "@/lib/asset";
import { createAudio, type GameAudio } from "./audio";
import { createBlitter, type BlitKind, type Blitter } from "./blit";
import { WorldEffects, sectorEffect } from "./sector-effects";
import {
  ENEMY_ANIM_COUNT,
  ENEMY_PROJECTILE_COUNT,
  ENEMY_TEX_BASE,
  readWorldFrame,
  TEX_N,
  T_GUN3,
  T_GUN4,
  T_GUN5,
  T_GUN6,
  T_GUN7,
  T_GUN8,
  T_GUN9,
  T_GUN10,
  T_GUN11,
  T_GUN12,
  T_GUN13,
  T_GUN14,
  T_GUN15,
  T_GUN16,
  T_GUN17,
  T_GUN18,
  T_GUN19,
  T_EXPANSION_CASE,
  T_PROJECTILE_NEW,
  T_IMPACT_NEW,
  T_ENEMY_PROJECTILE,
  T_BOSS_PROJECTILE,
  T_BOSS_ARENA,
  T_BOSS_ENTRANCE,
  T_CASING,
  T_CASING_SHELL,
  T_PLAYER_MISSILE,
  T_SECTOR_SURFACE,
  T_SECTOR_PROP,
  T_PROP_REACTOR,
  T_PROP_SERVER,
  T_PROP_AC,
  T_PROP_VENT,
  T_PROP_WLIGHT_C,
  T_PROP_WLIGHT_W,
  T_PROP_BEACON,
  T_DOOR,
  T_TECH,
  T_ORDNANCE,
} from "./gpu-world";
import { HUD_SIZE } from "./hud-abi";
import { SAVE_AMMO_BASE, SAVE_SIZE, SAVE_SLOTS } from "./save-abi";
import { keySpriteAlpha } from "./sprite-alpha";
import { WEAPON_SHEETS, WEAPON_THUMBNAILS } from "./weapon-assets";
import {
  readEnemyCues,
  readBars,
  type BarCue,
  type EnemyCue,
  type EnemyOptions,
  type EnemySubtitle,
} from "./enemy-presentation";
import { DEFAULT_GFX, DEFAULT_HUD, type GfxOpts, type HudState, type ResMode } from "./types";
import { bossDeathVariantForWave, type RunSave } from "@/components/game/data";

export { HUD_SIZE };

type MapSnapshot = { width: number; height: number; map: number[]; doors: number[]; x: number; y: number; nodeX: number; nodeY: number; enemies: EnemyCue[] };

type WasmExports = {
  memory: WebAssembly.Memory;
  hs_init: (w: number, h: number) => number;
  hs_restart: () => void;
  hs_next_wave: () => void;
  hs_select_weapon: (slot: number) => void;
  hs_resize: (w: number, h: number) => void;
  hs_fb_ptr: () => number;
  hs_fb_w: () => number;
  hs_fb_h: () => number;
  hs_hud_ptr: () => number;
  hs_hud_size: () => number;
  hs_events: () => number;
  hs_sound_count: () => number;
  hs_prepare_sound_loops: () => number;
  hs_sound_loops: () => number;
  hs_sound_cues: () => number;
  hs_ev_weapon: () => number;
  hs_tex_ptr: (id: number) => number;
  hs_tex_size: () => number;
  hs_textures_ready: () => void;
  hs_input: (bits: number, mx: number, my: number) => void;
  hs_qa: (bits: number, enabled: number) => void;
  hs_qa_armory: () => void;
  hs_fire_patches: () => number;
  hs_tick: (dt: number) => void;
  hs_render: () => void;
  hs_prepare_gpu: () => void;
  hs_gpu_view: () => number;
  hs_gpu_cols: () => number;
  hs_gpu_sprites: () => number;
  hs_gpu_sprite_count: () => number;
  hs_floor_ptr: () => number;
  hs_light_ptr: () => number;
  hs_smoke_ptr: () => number;
  hs_theme_ptr: (slot: number) => number;
  hs_apply_theme: (wave: number) => void;
  hs_yaw: () => number;
  hs_speed: () => number;
  hs_spread: () => number;
  hs_x: () => number;
  hs_y: () => number;
  hs_prepare_enemies: () => number;
  hs_enemy_cues: () => number;
  hs_prepare_bars: () => number;
  hs_bars: () => number;
  hs_qa_end: (state: number) => void;
  hs_qa_heal: () => void;
  hs_qa_reward: () => void;
  hs_qa_boss: (phase: number) => void;
  hs_qa_objective: () => void;
  hs_qa_tactical: (kind: number, index: number) => void;
  hs_save_ptr: () => number;
  hs_save_size: () => number;
  hs_load_ptr: () => number;
  hs_load_run: () => void;
  hs_map_ptr: () => number;
  hs_door_ptr: () => number;
  hs_map_w: () => number;
  hs_map_h: () => number;
};

const IN = {
  W: 1,
  S: 2,
  A: 4,
  D: 8,
  FIRE: 16,
  SPRINT: 32,
  USE: 64,
  W1: 128,
  W2: 256,
  W3: 512,
  TURNL: 1024,
  TURNR: 2048,
  RELOAD: 4096,
  W4: 8192,
  W5: 16384,
  W6: 32768,
  W7: 65536,
  W8: 131072,
  W9: 262144,
  W10: 524288,
  W11: 1048576,
  W12: 2097152,
  W13: 4194304,
  W14: 8388608,
  W15: 16777216,
  W16: 33554432,
  W17: 67108864,
  W18: 134217728,
  W19: 268435456,
};

const CODE_BITS: Record<string, number> = {
  KeyW: IN.W,
  ArrowUp: IN.W,
  KeyS: IN.S,
  ArrowDown: IN.S,
  KeyA: IN.A,
  KeyD: IN.D,
  ControlLeft: IN.FIRE,
  ControlRight: IN.FIRE,
  Space: IN.FIRE,
  ShiftLeft: IN.SPRINT,
  ShiftRight: IN.SPRINT,
  KeyE: IN.USE,
  Digit1: IN.W1,
  Digit2: IN.W2,
  Digit3: IN.W3,
  Digit4: IN.W4,
  Digit5: IN.W5,
  Digit6: IN.W6,
  Digit7: IN.W7,
  Digit8: IN.W8,
  Digit9: IN.W9,
  Digit0: IN.W10,
  Minus: IN.W11,
  Equal: IN.W12,
  BracketLeft: IN.W13,
  BracketRight: IN.W14,
  Backslash: IN.W15,
  Semicolon: IN.W16,
  Quote: IN.W17,
  Backquote: IN.W18,
  Comma: IN.W19,
  ArrowLeft: IN.TURNL,
  KeyQ: IN.TURNL,
  ArrowRight: IN.TURNR,
  KeyR: IN.RELOAD,
};

const ENEMY_SKINS = [
  "rifleman",
  "breacher",
  "subject",
  "hazmat",
  "gunner",
  "loader",
  "vatbrute",
  "marksman",
  "hornet",
  "hound",
  "spitter",
  "martyr",
  "veyran",
  "hecate",
  "chimera",
  "oracle",
  "gravemind",
  "archivist",
  "halcyon",
  "relay",
  "titan",
  "kest",
  "mnemosyne",
  ...CAMPAIGN_EXPANSION.map((boss) => boss.slug),
  ...SECTOR_ENEMIES.map((enemy) => enemy.slug),
] as const;
const ENEMY_ANIMATIONS = ["idle", "move", "pain", "fire", "reload", "dead", "special"] as const;

const TEX_FILES: { id: number; src: string }[] = [
  { id: 0, src: "/game/wall_brick.png" },
  { id: 1, src: "/game/wall_metal.png" },
  { id: 2, src: "/game/wall_flesh.png" },
  { id: 3, src: "/game/wall_bone.png" },
  { id: 4, src: "/game/wall_door.png" },
  { id: 5, src: "/game/floor_grate.png" },
  { id: 6, src: "/game/floor_concrete.png" },
  { id: 7, src: "/game/ceil_pipes.png" },
  { id: 8, src: "/game/spr_husk.png" },
  { id: 9, src: "/game/spr_brute.png" },
  { id: 10, src: "/game/spr_wraith.png" },
  { id: 11, src: "/game/spr_med.png" },
  { id: 12, src: "/game/spr_ammo.png" },
  { id: 13, src: "/game/spr_armor.png" },
  { id: 14, src: "/game/spr_barrel.png" },
  { id: 15, src: "/game/spr_ball.png" },
  { id: T_ORDNANCE, src: "/game/spr_ordnance.png" },
  { id: 18, src: "/game/wall_tech_tile2x2.png" },
  { id: 19, src: "/game/wall_hazard_tile2x2.png" },
  { id: 20, src: "/game/spr_lamp.png" },
  { id: 21, src: "/game/spr_crate.png" },
  { id: 22, src: "/game/spr_impact.png" },
  { id: 23, src: "/game/spr_muzzle.png" },
  { id: 24, src: "/game/spr_flame.png" },
  { id: 25, src: "/game/spr_chain.png" },
  { id: 26, src: "/game/wall_pipes.png" },
  { id: 27, src: "/game/spr_gun_br12.png" },
  { id: 28, src: "/game/floor_override.png" },
  { id: T_GUN3, src: "/game/spr_gun_kx9.png" },
  { id: T_GUN4, src: "/game/spr_gun_mr4.png" },
  { id: T_GUN5, src: "/game/spr_gun_vlk6.png" },
  { id: T_GUN6, src: "/game/spr_gun_ax12.png" },
  { id: T_GUN7, src: "/game/spr_gun_m91.png" },
  { id: T_GUN8, src: "/game/spr_gun_hx8.png" },
  { id: T_GUN7 + 1, src: "/game/environment/props/datacenter_terminal_console.png" },
  { id: T_GUN7 + 2, src: "/game/environment/props/foundry_terminal_console.png" },
  { id: T_GUN7 + 3, src: "/game/environment/props/biotech_terminal_console.png" },
  { id: T_GUN9, src: "/game/spr_gun_vr9.png" },
  { id: T_GUN10, src: "/game/spr_gun_hc9.png" },
  { id: T_GUN11, src: "/game/spr_gun_cm9.png" },
  { id: T_GUN12, src: "/game/spr_gun_ar6.png" },
  { id: T_GUN13, src: "/game/spr_gun_or7.png" },
  { id: T_GUN14, src: "/game/spr_gun_gs4.png" },
  { id: T_GUN15, src: "/game/spr_gun_cr3.png" },
  { id: T_GUN16, src: "/game/spr_gun_sr0.png" },
  { id: T_GUN17, src: "/game/spr_gun_ts12.png" },
  { id: T_GUN18, src: "/game/spr_gun_ks8.png" },
  { id: T_GUN19, src: "/game/spr_gun_mn6.png" },
  { id: T_PROP_REACTOR, src: "/game/spr_prop_reactor.png" },
  { id: T_PROP_SERVER, src: "/game/spr_prop_server.png" },
  { id: T_PROP_AC, src: "/game/spr_prop_ac.png" },
  { id: T_PROP_VENT, src: "/game/spr_prop_vent.png" },
  { id: T_PROP_WLIGHT_C, src: "/game/spr_prop_worklight_cyan.png" },
  { id: T_PROP_WLIGHT_W, src: "/game/spr_prop_worklight_white.png" },
  { id: T_PROP_BEACON, src: "/game/spr_prop_beacon.png" },
  ...CAMPAIGN_EXPANSION.map((boss, i) => ({
    id: T_EXPANSION_CASE + i,
    src: `/game/spr_gun_${boss.weapon}.png`,
  })),
  ...["rocket-forward", "ion-bolt", "cryo-shard", "spore-cluster", "phase-orb"].map((name, i) => ({
    id: T_PROJECTILE_NEW + i,
    src: `/game/fx25/${name}.png`,
  })),
  ...["ember-impact", "pressure-impact", "magnetic-impact", "solar-impact"].map((name, i) => ({
    id: T_IMPACT_NEW + i,
    src: `/game/fx25/${name}.png`,
  })),
  ...BOSS_ARENAS.map((boss,i) => ({
    id: T_BOSS_ARENA + i,
    src: `/game/boss-arenas/${boss.slug}.png`,
  })),
  ...Array.from({ length: 25 }, (_, i) => ({
    id: T_BOSS_PROJECTILE + i,
    src: `/game/projectiles/boss_weapon_${i + 8}.png`,
  })),
  ...ENEMY_SKINS.slice(0, ENEMY_PROJECTILE_COUNT).map((skin, i) => ({
    id: T_ENEMY_PROJECTILE + i,
    src: `/game/projectiles/enemy_${skin}.png`,
  })),
  ...["tactical", "siege", "naval"].map((model, i) => ({
    id: T_PLAYER_MISSILE + i,
    src: `/game/projectiles/missile_${model}_outgoing.png`,
  })),
  ...SECTOR_SLUGS.flatMap((sector, i) =>
    SECTOR_SURFACES.map((name, j) => ({
      id: T_SECTOR_SURFACE + i * 5 + j,
      src: `/game/sectors/${sector}/${name}.png`,
    })),
  ),
  ...SECTOR_SLUGS.flatMap((sector, i) =>
    [1, 2, 3].map((prop, j) => ({
      id: T_SECTOR_PROP + i * 3 + j,
      src: `/game/sectors/${sector}/prop_${prop}.png`,
    })),
  ),
  ...ENEMY_SKINS.flatMap((skin, skinIndex) =>
    ENEMY_ANIMATIONS.map((animation, animationIndex) => ({
      id: ENEMY_TEX_BASE + skinIndex * ENEMY_ANIM_COUNT + animationIndex,
      src: `/game/enemy_${skin}_${animation}.png`,
    })),
  ),
  ...Array.from({length:8},(_,i)=>({id:T_BOSS_ENTRANCE+i,src:`/game/fx/boss-entry-${i}.png`})),
  { id: T_CASING, src: "/game/fx/casing-hd.png" },
  { id: T_CASING_SHELL, src: "/game/fx/shell-hd.png" },
];

const UI_CRITICAL = [
  ...WEAPON_THUMBNAILS,
  ...WEAPON_SHEETS,
  "/game/ui/hud-panel.webp",
  "/game/ui/menu-reactor.webp",
  "/game/ui-plaque.svg",
];
const weaponSheetPaths = new Set<string>(WEAPON_SHEETS);
const weaponSheetCache = new Map<string, HTMLImageElement>();
const pendingWeaponSheets = new Set<string>();

export function weaponSheetImage(src: string): HTMLImageElement | null {
  const cached = weaponSheetCache.get(src);
  if (cached) return cached;
  if (!pendingWeaponSheets.has(src)) {
    pendingWeaponSheets.add(src);
    const image = new Image();
    image.onload = () => {
      weaponSheetCache.set(src, image);
      pendingWeaponSheets.delete(src);
    };
    image.onerror = () => pendingWeaponSheets.delete(src);
    image.src = asset(src);
  }
  return null;
}

/**
 * Theme order shared with theme_index/engine theme slots in
 * engine/src/lib.rs: walls 0-7, then doors 8-15.
 */
export const THEME_ORDER = [
  "hangar",
  "plaza",
  "security",
  "datacenter",
  "foundry",
  "biotech",
  "nuclear",
  "vault",
] as const;

const THEME_FILES: { slot: number; src: string }[] = [
  ...THEME_ORDER.map((theme, i) => ({ slot: i, src: `/game/theme/wall_${theme}.png` })),
  ...THEME_ORDER.map((theme, i) => ({ slot: 8 + i, src: `/game/theme/door_${theme}.png` })),
];

function decodeImage(src: string, timeoutMs = 30000): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const img = new Image();
    let settled = false;
    const finish = (value: HTMLImageElement | null) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timer);
      resolve(value);
    };
    const timer = window.setTimeout(() => finish(null), timeoutMs);
    img.onload = () => {
      img.decode().then(
        () => {
          if (weaponSheetPaths.has(src)) weaponSheetCache.set(src, img);
          finish(img);
        },
        () => finish(null),
      );
    };
    img.onerror = () => finish(null);
    img.src = asset(src);
  });
}

async function preloadImages(
  urls: string[],
  limit = 4,
  onItem?: (done: number, total: number) => void,
  timeoutMs = 30000,
) {
  const out: (HTMLImageElement | null)[] = new Array(urls.length);
  let cursor = 0;
  let done = 0;
  onItem?.(0, urls.length);
  async function worker() {
    while (cursor < urls.length) {
      const i = cursor;
      cursor += 1;
      out[i] = await decodeImage(urls[i]!, timeoutMs);
      done += 1;
      onItem?.(done, urls.length);
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, urls.length) }, () => worker()));
  return out;
}

export type RuntimeHooks = {
  onSubtitles?: (subtitles: EnemySubtitle[]) => void;
  onHud: (hud: HudState, fps: number, resolution: string) => void;
  onState: (state: number) => void;
  /** Arsenal images that failed to decode (procedural fallback in use). */
  onAssetError?: (failed: string[]) => void;
  /** Fired once when the frame loop throws (e.g. a WASM trap from a
   * version-skewed binary). Without this the game would freeze silently
   * on a stale HUD with no game-over ever arriving. */
  onError?: (message: string) => void;
  onLoad?: (progress: { ratio: number; label: string }) => void;
};

export class BlacksiteRuntime {
  private canvas: HTMLCanvasElement;
  private hooks: RuntimeHooks;
  private wasm: WasmExports | null = null;
  private blit: Blitter | null = null;
  private audio: GameAudio;
  private worldEffects = new WorldEffects();
  private keys = new Set<string>();
  private bits = 0;
  private lookX = 0;
  private lookY = 0;
  private touchMoveX = 0;
  private touchMoveY = 0;
  private touchLookX = 0;
  private touchLookY = 0;
  private ignoreMouse = 0;
  private fireHeld = false;
  private running = false;
  private aborted = false;
  private playing = false;
  private raf = 0;
  private last = 0;
  private accumulator = 0;
  private resolution: ResMode | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private fps = 0;
  private fpsAccum = 0;
  private fpsFrames = 0;
  private hud: HudState = { ...DEFAULT_HUD };
  private prevHud = { ...DEFAULT_HUD };
  private prevRadioSeq = 0;
  private lastEnemies: EnemyCue[] = [];
  private lastBars: BarCue[] = [];
  private qaBits = 0;
  private qaOn = false;
  private sens = DEFAULT_SENSITIVITY;
  private usePulse = 0;
  private reloadPulse = 0;
  private weaponPulse = 0;
  muted = false;
  renderer: BlitKind = "canvas2d";
  private volumes = { master: 0.85, music: 0.42, sfx: 0.75, menu: 0.7 };
  private gfx: GfxOpts = { ...DEFAULT_GFX };
  private requireGpu = false;
  private fbView: Uint8Array<ArrayBuffer> | null = null;
  private fbBuf: ArrayBuffer | null = null;
  private fbLen = 0;
  private gpuReady = false;
  private assetsReady = false;
  private renderDirty = true;
  private atlasRevision = -1;
  private recovering: Promise<BlitKind> | null = null;
  private contextLost = false;
  private recoveryAttempts = 0;
  private rendererWaitSince = 0;
  private contextTimer: ReturnType<typeof setTimeout> | null = null;
  private lastThemeWave = 1;

  constructor(canvas: HTMLCanvasElement, hooks: RuntimeHooks) {
    this.canvas = canvas;
    this.hooks = hooks;
    this.audio = createAudio();
  }
  async boot(res: ResMode, opts?: { requireGpu?: boolean }) {
    const report = (ratio: number, label: string) => this.hooks.onLoad?.({ ratio, label });
    report(0.02, "Engine");
    const wasm = await loadWasm();
    if (this.aborted) return;
    this.wasm = wasm;
    this.requireGpu = opts?.requireGpu ?? false;
    wasm.hs_init(res.w, res.h);
    const hudSize = wasm.hs_hud_size();
    if (hudSize !== HUD_SIZE) {
      throw new Error(
        `HUD ABI mismatch: WASM reports ${hudSize} bytes, TS expects ${HUD_SIZE}. Rebuild both sides.`,
      );
    }
    report(0.1, "Renderer");
    this.blit = await createBlitter(this.canvas, { requireGpu: this.requireGpu });
    if (this.aborted) {
      this.blit.dispose();
      return;
    }
    this.renderer = this.blit.kind;
    this.blit.setGfx(this.gfx);
    this.setResolution(res);
    this.resizeObserver = new ResizeObserver(() => {
      if (this.resolution) this.setResolution(this.resolution);
    });
    this.resizeObserver.observe(this.canvas);
    this.bind();
    this.installQa();
    this.running = true;
    this.last = performance.now();
    this.loop(this.last);
    const slices = { tex: 0, ui: 0, voice: 0, media: 0 };
    const paint = (label: string) => {
      report(
        Math.min(
          0.99,
          0.12 + 0.4 * slices.tex + 0.25 * slices.ui + 0.15 * slices.voice + 0.08 * slices.media,
        ),
        label,
      );
    };
    await Promise.all([
      this.uploadTextures((done, total) => {
        slices.tex = total ? done / total : 1;
        paint("World textures");
      }),
      // Deployment waits for every viewmodel, wheel thumbnail and HUD image.
      preloadImages(
        UI_CRITICAL,
        2,
        (done, total) => {
          slices.ui = total ? done / total : 1;
          paint("Arsenal & interface");
        },
        30000,
      ).then((imgs) => {
        const failed = UI_CRITICAL.filter((_, i) => !imgs[i]);
        if (failed.length) {
          this.hooks.onAssetError?.(failed);
          throw new Error(`Arsenal assets could not load: ${failed.join(", ")}`);
        }
      }),
      this.audio.prepareEnemies((done, total) => {
        slices.voice = total ? done / total : 1;
        paint("Enemy voices");
      }),
      this.audio.prepareMedia().then(() => {
        slices.media = 1;
        paint("Sound & music");
      }),
      document.fonts.ready,
    ]);
    this.assetsReady = true;
    this.renderDirty = true;
    report(1, "Ready");
    if (this.aborted) {
      this.running = false;
      cancelAnimationFrame(this.raf);
    }
  }

  setResolution(res: ResMode) {
    this.resolution = res;
    if (!this.wasm || !this.blit) return;
    const aspect = (this.canvas.clientWidth || res.w) / (this.canvas.clientHeight || res.h);
    // Cap internal width at 2560: beyond that the CPU framebuffer alone
    // costs 33MB+ and the GPU path gains nothing CSS can't upscale.
    const raw = Math.min(2560, Math.max(192, Math.round(Math.min(res.w, res.h * aspect))));
    const w = raw - (raw % 64);
    const h = Math.max(100, Math.round(w / aspect));
    if (this.wasm.hs_fb_w() !== w || this.wasm.hs_fb_h() !== h) this.wasm.hs_resize(w, h);
    this.fbView = null;
    if (!this.recovering && !this.contextLost) this.blit.resize(w, h);
    // Display resizing clears the canvas, including while simulation is paused.
    // Texture mipmaps do not depend on framebuffer size; keep resident assets.
    this.renderDirty = true;
    this.canvas.dataset.resolution = `${w} × ${h}`;
  }

  setPlaying(v: boolean) {
    const changed = this.playing !== v;
    this.playing = v;
    this.renderDirty = true;
    this.clearInput();
    this.accumulator = 0;
    this.last = performance.now();
    this.lookX = 0;
    this.lookY = 0;
    this.touchLookX = 0;
    this.touchLookY = 0;
    if (v) {
      this.audio.unlock();
      this.audio.setMusic(true);
    } else {
      this.audio.setMusic(false);
      this.audio.clearEnemies();
      this.hooks.onSubtitles?.([]);
    }
    if (changed) this.audio.ui("transition");
  }

  setSens(v: number) {
    this.sens = normalizeSensitivity(v);
  }

  setEnemyOptions(options: EnemyOptions) {
    this.audio.setEnemyOptions(options);
    if (!options.subtitles) this.hooks.onSubtitles?.([]);
  }
  previewEnemy(skin: number) {
    this.audio.previewEnemy(skin);
  }

  setMuted(v: boolean) {
    this.muted = v;
    this.audio.setMuted(v);
  }

  setMenuBed(on: boolean) {
    this.audio.setMenuBed(on);
  }

  setVolumes(master: number, music: number, sfx: number, menu: number) {
    this.volumes = { master, music, sfx, menu };
    this.audio.setVolumes(master, music, sfx, menu);
  }

  setGfx(g: GfxOpts) {
    this.gfx = { ...g };
    this.blit?.setGfx(this.gfx);
    this.renderDirty = true;
  }

  /** Switch WebGPU enforcement without restarting the sim or canvas. */
  async switchRenderer(requireGpu: boolean): Promise<BlitKind> {
    if (!this.wasm || !this.blit || this.aborted) return this.renderer;
    if (requireGpu === this.requireGpu && this.blit.kind === this.renderer) return this.renderer;
    this.requireGpu = requireGpu;
    return this.rebuildRenderer();
  }

  private rebuildRenderer(): Promise<BlitKind> {
    if (this.recovering) return this.recovering;
    const work = async () => {
      // Dispose before configuring the replacement: WebGPU shares the canvas
      // context, so disposing afterwards would unconfigure the new renderer.
      this.blit?.dispose();
      this.gpuReady = false;
      const next = await createBlitter(this.canvas, { requireGpu: this.requireGpu });
      if (this.aborted) { next.dispose(); return this.renderer; }
      this.blit = next;
      this.renderer = next.kind;
      next.setGfx(this.gfx);
      if (this.resolution) this.setResolution(this.resolution);
      next.resize(this.wasm!.hs_fb_w(), this.wasm!.hs_fb_h());
      if (this.assetsReady) this.pushAtlas();
      this.fbView = this.fbBuf = null;
      this.fbLen = 0;
      this.renderDirty = true;
      this.last = performance.now();
      this.accumulator = 0;
      return this.renderer;
    };
    this.recovering = work().finally(() => { this.recovering = null; });
    return this.recovering;
  }

  private recoverRenderer() {
    if (this.recovering || this.contextLost || this.aborted) return;
    if (++this.recoveryAttempts > 2) {
      this.running = false;
      cancelAnimationFrame(this.raf);
      this.hooks.onError?.("Graphics recovery failed. Your level has not been restarted.");
      return;
    }
    void this.rebuildRenderer().catch(err => {
      if (!this.aborted) {
        this.running = false;
        cancelAnimationFrame(this.raf);
        this.hooks.onError?.(err instanceof Error ? err.message : String(err));
      }
    });
  }

  private onContextLost = (event: Event) => {
    event.preventDefault(); // Allow WebGL to restore its context.
    this.contextLost = true;
    this.renderDirty = true;
    this.clearInput();
    if (this.contextTimer) clearTimeout(this.contextTimer);
    this.contextTimer = setTimeout(() => {
      if (!this.aborted && this.contextLost) {
        this.hooks.onError?.("The graphics context could not be restored. Your level has not been restarted.");
      }
    }, 8000);
  };

  private onContextRestored = () => {
    this.contextLost = false;
    if (this.contextTimer) clearTimeout(this.contextTimer);
    this.contextTimer = null;
    this.recoverRenderer();
  };

  restart() {
    this.worldEffects.reset();
    this.audio.clearEnemies(true);
    this.clearInput();
    this.accumulator = 0;
    this.wasm?.hs_restart();
    this.refreshThemeLayers(1);
    this.hud = { ...DEFAULT_HUD };
    this.prevHud = { ...DEFAULT_HUD };
    this.prevRadioSeq = 0;
    this.audio.setBoss(false);
  }

  nextWave() {
    this.worldEffects.reset();
    this.audio.advanceSectorVoices();
    this.wasm?.hs_next_wave();
    this.prevRadioSeq = 0;
    this.audio.setBoss(false);
  }

  /** Local QA starts from a clean sector and grants all thirty-three weapons. */
  startQaRun(level: number) {
    if (!import.meta.env.DEV) return;
    this.restart();
    const sector = Number.isInteger(level) && level >= 1 && level <= 25 ? level : 1;
    for (let wave = 1; wave < sector; wave++) this.nextWave();
    this.wasm?.hs_qa(0, 1);
    this.wasm?.hs_qa_armory();
    this.wasm?.hs_qa(0, 0);
  }

  stop() {
    this.aborted = true;
    if (this.contextTimer) clearTimeout(this.contextTimer);
    this.running = false;
    cancelAnimationFrame(this.raf);
    this.unbind();
    this.resizeObserver?.disconnect();
    this.blit?.dispose();
    this.audio.setMusic(false);
    this.audio.setBoss(false);
    this.audio.dispose();
  }

  requestLock() {
    if (window.matchMedia("(pointer: coarse)").matches || !this.canvas.requestPointerLock) return;
    this.ignoreMouse = 3;
    this.lookX = 0;
    this.lookY = 0;
    const el = this.canvas;
    const req = el.requestPointerLock as (opts?: {
      unadjustedMovement?: boolean;
    }) => Promise<void> | void;
    try {
      const p = req.call(el, { unadjustedMovement: true });
      if (p && typeof (p as Promise<void>).catch === "function") {
        (p as Promise<void>).catch(() => {
          try {
            void Promise.resolve(el.requestPointerLock()).catch(() => {});
          } catch {
            /* unavailable */
          }
        });
      }
    } catch {
      try {
        void Promise.resolve(el.requestPointerLock()).catch(() => {});
      } catch {
        /* unavailable */
      }
    }
  }

  setTouchMove(x: number, y: number) {
    this.touchMoveX = x;
    this.touchMoveY = y;
  }

  setTouchLook(dx: number, dy: number) {
    // Touch drags need more gain than a mouse to feel responsive.
    this.touchLookX += dx * 1.4;
    this.touchLookY += dy * 1.4;
  }

  setFireHeld(v: boolean) {
    this.fireHeld = v;
  }

  pulseUse() {
    this.usePulse = 10;
  }

  pulseReload() {
    this.reloadPulse = 10;
  }

  /** Cycle owned weapons. dir = +1 (wheel down / touch) or -1 (wheel up). */
  cycleWeapon(dir = 1) {
    const owned = [
      true,
      this.hud.hasW2,
      this.hud.hasW3,
      this.hud.hasW4,
      this.hud.hasW5,
      this.hud.hasW6,
      this.hud.hasW7,
      this.hud.hasW8,
      this.hud.hasW9,
      this.hud.hasW10,
      this.hud.hasW11,
      this.hud.hasW12,
      this.hud.hasW13,
      this.hud.hasW14,
      this.hud.hasW15,
      this.hud.hasW16,
      this.hud.hasW17,
      this.hud.hasW18,
      this.hud.hasW19,
      ...CAMPAIGN_EXPANSION.map((_, i) => (this.hud.extraWeapons & (1 << i)) !== 0),
    ];
    const count = owned.length;
    const step = dir >= 0 ? 1 : count - 1;
    for (let n = 1; n <= count; n++) {
      const next = (this.hud.weapon + step * n) % count;
      if (owned[next]) {
        this.wasm?.hs_select_weapon(next);
        return;
      }
    }
  }

  nextWeapon() {
    this.cycleWeapon(1);
  }

  exportSave(): RunSave | null {
    const wasm = this.wasm;
    if (!wasm) return null;
    const ptr = wasm.hs_save_ptr();
    const dv = new DataView(wasm.memory.buffer, ptr, wasm.hs_save_size());
    const size = wasm.hs_save_size();
    // Legacy 8-slot checkpoints (mag at 64) are still readable via loadSave.
    const slots = size >= SAVE_SIZE ? SAVE_SLOTS : 8;
    const magBase = SAVE_AMMO_BASE + slots * 4;
    if (size >= SAVE_SIZE && size !== SAVE_SIZE) {
      throw new Error(`RunSave ABI mismatch: WASM reports ${size} bytes, TS expects ${SAVE_SIZE}.`);
    }
    return {
      wave: dv.getInt32(0, true),
      health: dv.getInt32(4, true),
      armor: dv.getInt32(8, true),
      weapon: dv.getInt32(12, true),
      flags: dv.getInt32(16, true),
      extraWeapons: size >= SAVE_SIZE ? dv.getUint32(296, true) : 0,
      kills: dv.getInt32(20, true),
      secrets: dv.getInt32(24, true),
      elapsedMs: dv.getInt32(28, true),
      ammo: Array.from({ length: SAVE_SLOTS }, (_, i) =>
        i < slots ? dv.getInt32(SAVE_AMMO_BASE + i * 4, true) : 0,
      ),
      mag: Array.from({ length: SAVE_SLOTS }, (_, i) =>
        i < slots ? dv.getInt32(magBase + i * 4, true) : 0,
      ),
    };
  }

  loadSave(save: RunSave) {
    const wasm = this.wasm;
    if (!wasm) return;
    const ptr = wasm.hs_load_ptr();
    const dv = new DataView(wasm.memory.buffer, ptr, wasm.hs_save_size());
    const size = wasm.hs_save_size();
    const slots = size >= SAVE_SIZE ? SAVE_SLOTS : 8;
    const magBase = SAVE_AMMO_BASE + slots * 4;
    const ammo = [...save.ammo, ...Array(SAVE_SLOTS).fill(0)].slice(0, slots);
    const mag = [...save.mag, ...Array(SAVE_SLOTS).fill(0)].slice(0, slots);
    dv.setInt32(0, save.wave, true);
    dv.setInt32(4, save.health, true);
    dv.setInt32(8, save.armor, true);
    dv.setInt32(12, save.weapon, true);
    dv.setInt32(16, save.flags, true);
    dv.setInt32(20, save.kills, true);
    dv.setInt32(24, save.secrets, true);
    dv.setInt32(28, save.elapsedMs, true);
    ammo.forEach((n, i) => dv.setInt32(SAVE_AMMO_BASE + i * 4, n, true));
    mag.forEach((n, i) => dv.setInt32(magBase + i * 4, n, true));
    dv.setUint32(296, save.extraWeapons || 0, true);
    wasm.hs_load_run();
    this.refreshThemeLayers(save.wave);
    this.hud = this.readHud();
  }

  readMap(): Uint8Array | null {
    const wasm = this.wasm;
    if (!wasm) return null;
    const count = wasm.hs_map_w() * wasm.hs_map_h();
    return automapCells(
      new Uint8Array(wasm.memory.buffer, wasm.hs_map_ptr(), count),
      new Float32Array(wasm.memory.buffer, wasm.hs_door_ptr(), count),
    );
  }

  getEnemies(): EnemyCue[] {
    return this.lastEnemies;
  }

  getBars(): BarCue[] {
    return this.lastBars;
  }

  getHud() {
    return this.hud;
  }

  private async uploadTextures(onItem?: (done: number, total: number) => void) {
    const wasm = this.wasm;
    if (!wasm) return;
    const size = wasm.hs_tex_size();
    const scratch = document.createElement("canvas");
    scratch.width = size;
    scratch.height = size;
    const ctx = scratch.getContext("2d", { willReadFrequently: true });
    if (!ctx) throw new Error("Unable to prepare world textures");
    // Enemy animation layers are independent 256px files; decode them in a
    // wider batch so the first playable frame is not gated by four-at-a-time
    // image loads.
    const loaded = await preloadImages(
      TEX_FILES.map((t) => t.src),
      12,
      onItem,
    );
    const missing = TEX_FILES.filter((_, i) => !loaded[i]).map((t) => t.src);
    if (missing.length) throw new Error(`World textures could not load: ${missing.join(", ")}`);
    for (let i = 0; i < TEX_FILES.length; i++) {
      const img = loaded[i];
      const id = TEX_FILES[i]!.id;
      if (!img) continue;
      try {
        ctx.imageSmoothingEnabled = false;
        ctx.clearRect(0, 0, size, size);
        const wall = id <= 7 || id === 18 || id === 19 || id === 26;
        if (wall && (img.width < size || img.height < size)) {
          const tw = Math.max(1, img.width);
          const th = Math.max(1, img.height);
          for (let y = 0; y < size; y += th) {
            for (let x = 0; x < size; x += tw) {
              ctx.drawImage(img, x, y, tw, th);
            }
          }
        } else {
          ctx.drawImage(img, 0, 0, size, size);
        }

        const pixels = ctx.getImageData(0, 0, size, size);
        const sprite =
          id === 15 ||
          id === 27 ||
          (id >= 8 && id <= 14) ||
          (id >= 20 && id <= 25) ||
          (id >= ENEMY_TEX_BASE && id < T_SECTOR_SURFACE) ||
          id >= T_SECTOR_PROP;
        if (sprite) {
          // Generated VFX use intentional dark cores and smoke; preserve
          // those pixels instead of applying the legacy black-key cleanup.
          keySpriteAlpha(
            pixels.data,
            size,
            id === 15 || (id >= 22 && id <= 24) || id >= ENEMY_TEX_BASE,
          );
        }
        const ptr = wasm.hs_tex_ptr(id);
        const view = new Uint8Array(wasm.memory.buffer, ptr, size * size * 4);
        view.set(pixels.data);
      } catch {
        throw new Error(`World texture could not be prepared: ${TEX_FILES[i]!.src}`);
      }
    }
    await this.uploadThemeVariants(ctx, size);
    wasm.hs_apply_theme(1);
    wasm.hs_textures_ready();
    this.pushAtlas();
  }

  /** Decode the 16 per-theme wall/door layers into engine staging memory. */
  private async uploadThemeVariants(
    ctx: CanvasRenderingContext2D,
    size: number,
  ): Promise<string[]> {
    const wasm = this.wasm;
    if (!wasm) return [];
    const loaded = await preloadImages(
      THEME_FILES.map((t) => t.src),
      4,
    );
    const failed: string[] = [];
    for (let i = 0; i < THEME_FILES.length; i++) {
      const img = loaded[i];
      const { slot, src } = THEME_FILES[i]!;
      if (!img) {
        failed.push(src);
        continue;
      }
      try {
        ctx.imageSmoothingEnabled = false;
        ctx.clearRect(0, 0, size, size);
        ctx.drawImage(img, 0, 0, size, size);
        const pixels = ctx.getImageData(0, 0, size, size);
        const ptr = wasm.hs_theme_ptr(slot);
        new Uint8Array(wasm.memory.buffer, ptr, size * size * 4).set(pixels.data);
      } catch {
        failed.push(src);
      }
    }
    if (failed.length) throw new Error(`Sector textures could not load: ${failed.join(", ")}`);
    return failed;
  }

  /**
   * Re-apply the wave theme on the CPU side and push the two swapped
   * atlas layers to the GPU. The canvas2d fallback reads engine memory
   * directly and needs no upload.
   */
  private refreshThemeLayers(wave: number) {
    const wasm = this.wasm;
    const blit = this.blit;
    if (!wasm || !blit?.drawWorld) return;
    wasm.hs_apply_theme(wave);
    const size = wasm.hs_tex_size();
    for (const id of [T_TECH, T_DOOR]) {
      const ptr = wasm.hs_tex_ptr(id);
      blit.uploadAtlasLayer?.(id, new Uint8Array(wasm.memory.buffer, ptr, size * size * 4));
    }
    this.lastThemeWave = wave;
  }

  uiSound(kind: "click" | "confirm" | "back" | "error" | "transition" = "click") {
    this.audio.ui(kind);
  }

  private onUiClick = (event: MouseEvent) => {
    const button =
      event.target instanceof Element ? event.target.closest("button, [role=button]") : null;
    if (!button || button.hasAttribute("disabled") || button.closest(".touch-layer")) return;
    const text = button.textContent?.toLowerCase() ?? "";
    this.audio.ui(
      /deploy|enter |retry|resume/.test(text)
        ? "confirm"
        : /back|close|abort/.test(text)
          ? "back"
          : "click",
    );
  };
  private bind() {
    this.canvas.addEventListener("webglcontextlost", this.onContextLost);
    this.canvas.addEventListener("webglcontextrestored", this.onContextRestored);
    document.addEventListener("click", this.onUiClick);
    window.addEventListener("keydown", this.onKeyDown);
    window.addEventListener("keyup", this.onKeyUp);
    window.addEventListener("blur", this.onBlur);
    document.addEventListener("visibilitychange", this.onVis);
    document.addEventListener("mousemove", this.onMouse);
    document.addEventListener("pointerlockchange", this.onLock);
    this.canvas.addEventListener("mousedown", this.onMouseDown);
    window.addEventListener("mouseup", this.onMouseUp);
  }

  private unbind() {
    this.canvas.removeEventListener("webglcontextlost", this.onContextLost);
    this.canvas.removeEventListener("webglcontextrestored", this.onContextRestored);
    document.removeEventListener("click", this.onUiClick);
    window.removeEventListener("keydown", this.onKeyDown);
    window.removeEventListener("keyup", this.onKeyUp);
    window.removeEventListener("blur", this.onBlur);
    document.removeEventListener("visibilitychange", this.onVis);
    document.removeEventListener("mousemove", this.onMouse);
    document.removeEventListener("pointerlockchange", this.onLock);
    this.canvas.removeEventListener("mousedown", this.onMouseDown);
    window.removeEventListener("mouseup", this.onMouseUp);
  }

  private onKeyDown = (e: KeyboardEvent) => {
    if (
      !this.playing ||
      (e.target instanceof HTMLElement &&
        (e.target.isContentEditable || /^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName)))
    )
      return;
    if (e.repeat) return;
    if (CODE_BITS[e.code] !== undefined) e.preventDefault();
    this.keys.add(e.code);
    this.audio.unlock();
  };

  private onKeyUp = (e: KeyboardEvent) => {
    this.keys.delete(e.code);
  };

  private clearInput() {
    this.keys.clear();
    this.fireHeld = false;
    this.lookX = 0;
    this.lookY = 0;
    this.touchMoveX = this.touchMoveY = 0;
    this.touchLookX = this.touchLookY = 0;
    this.usePulse = this.reloadPulse = 0;
    this.weaponPulse = 0;
  }

  private onBlur = () => {
    this.clearInput();
    this.accumulator = 0;
  };

  private onVis = () => {
    if (document.hidden) {
      this.onBlur();
    }
    this.last = performance.now();
    this.audio.unlock();
  };

  private onLock = () => {
    this.lookX = 0;
    this.lookY = 0;
    this.ignoreMouse = 3;
  };

  private onMouse = (e: MouseEvent) => {
    if (!this.playing || document.pointerLockElement !== this.canvas) return;
    if (this.ignoreMouse > 0) {
      this.ignoreMouse -= 1;
      return;
    }
    this.lookX += e.movementX || 0;
    this.lookY += e.movementY || 0;
  };

  private onMouseDown = (e: MouseEvent) => {
    if (e.button === 0) this.fireHeld = true;
    this.audio.unlock();
  };

  private onMouseUp = (e: MouseEvent) => {
    if (e.button === 0) this.fireHeld = false;
  };

  private bitsFromKeys(): number {
    let bits = this.weaponPulse;
    this.weaponPulse = 0;
    for (const code of this.keys) {
      bits |= CODE_BITS[code] ?? 0;
    }
    if (this.fireHeld) bits |= IN.FIRE;
    if (this.usePulse > 0) {
      bits |= IN.USE;
      this.usePulse -= 1;
    }
    if (this.reloadPulse > 0) {
      bits |= IN.RELOAD;
      this.reloadPulse -= 1;
    }
    if (this.touchMoveY < -0.25) bits |= IN.W;
    if (this.touchMoveY > 0.25) bits |= IN.S;
    if (this.touchMoveX < -0.25) bits |= IN.A;
    if (this.touchMoveX > 0.25) bits |= IN.D;
    return bits;
  }

  private readHud(): HudState {
    const wasm = this.wasm!;
    const ptr = wasm.hs_hud_ptr();
    const dv = new DataView(wasm.memory.buffer, ptr, HUD_SIZE);
    return {
      health: dv.getInt32(0, true),
      armor: dv.getInt32(4, true),
      ammo: dv.getInt32(8, true),
      weapon: dv.getInt32(12, true),
      kills: dv.getInt32(16, true),
      living: dv.getInt32(20, true),
      state: dv.getInt32(24, true),
      prompt: dv.getInt32(28, true),
      hasW2: dv.getInt32(32, true) !== 0,
      hasW3: dv.getInt32(36, true) !== 0,
      secrets: dv.getInt32(40, true),
      elapsedMs: dv.getInt32(44, true),
      shake: dv.getFloat32(48, true),
      muzzle: dv.getFloat32(52, true),
      hurt: dv.getFloat32(56, true),
      bob: dv.getFloat32(60, true),
      kick: dv.getFloat32(64, true),
      hitmarker: dv.getFloat32(68, true),
      spread: wasm.hs_spread(),
      yaw: dv.getFloat32(72, true),
      speed: dv.getFloat32(76, true),
      x: wasm.hs_x(),
      y: wasm.hs_y(),
      reserve: dv.getInt32(88, true),
      reloading: dv.getFloat32(92, true),
      weapFrame: dv.getInt32(96, true),
      hasW4: dv.getInt32(100, true) !== 0,
      hasW5: dv.getInt32(104, true) !== 0,
      events: dv.getUint32(108, true),
      evWeapon: dv.getInt32(112, true),
      wave: dv.getInt32(116, true),
      bossHealth: dv.getInt32(120, true),
      bossMaxHealth: dv.getInt32(124, true),
      bossPhase: dv.getInt32(128, true),
      hasW6: dv.getInt32(132, true) !== 0,
      hasW7: dv.getInt32(136, true) !== 0,
      hasW8: dv.getInt32(140, true) !== 0,
      hasW9: dv.getInt32(168, true) !== 0,
      hasW10: dv.getInt32(172, true) !== 0,
      hasW11: dv.getInt32(176, true) !== 0,
      hasW12: dv.getInt32(192, true) !== 0,
      hasW13: dv.getInt32(196, true) !== 0,
      hasW14: dv.getInt32(200, true) !== 0,
      hasW15: dv.getInt32(204, true) !== 0,
      hasW16: dv.getInt32(208, true) !== 0,
      hasW17: dv.getInt32(212, true) !== 0,
      hasW18: dv.getInt32(216, true) !== 0,
      hasW19: dv.getInt32(220, true) !== 0,
      extraWeapons: dv.getUint32(224, true),
      objective: dv.getInt32(144, true),
      radioSeq: dv.getInt32(148, true),
      radioLine: dv.getInt32(152, true),
      vuln: dv.getFloat32(156, true),
      nodeX: dv.getFloat32(160, true),
      nodeY: dv.getFloat32(164, true),
      splash: dv.getFloat32(188, true),
    };
  }

  private loop = (t: number) => {
    if (!this.running || !this.wasm || !this.blit) return;
    this.raf = requestAnimationFrame(this.loop);
    try {
      this.frame(t);
    } catch (err) {
      // Never freeze silently on a stale HUD: stop the loop and surface
      // the fault so the player gets an error screen, not a dead game
      // with no game-over.
      this.running = false;
      cancelAnimationFrame(this.raf);
      this.hooks.onError?.(err instanceof Error ? err.message : String(err));
    }
  };

  /** Restart the rAF loop after a fault (used when re-entering play). No-op while running. */
  kick() {
    if (this.running || !this.wasm || !this.blit || this.aborted) return;
    this.running = true;
    this.accumulator = 0;
    this.last = performance.now();
    this.raf = requestAnimationFrame(this.loop);
  }

  private frame(t: number) {
    // Re-narrowed here because loop() delegates across a method boundary.
    const wasm = this.wasm;
    const blit = this.blit;
    if (!wasm || !blit) return;
    if (typeof document !== "undefined" && document.hidden) return;

    if (!this.assetsReady || this.contextLost || this.recovering) { this.last = t; return; }
    if (!blit.isReady()) {
      this.renderDirty = true;
      this.last = t;
      // Give WebGPU's own device recovery time to finish before rebuilding.
      if (!this.rendererWaitSince) this.rendererWaitSince = t;
      if (t - this.rendererWaitSince > 1500 && !this.contextLost) {
        this.rendererWaitSince = 0;
        this.recoverRenderer();
      }
      return;
    }
    this.rendererWaitSince = 0;
    if (this.atlasRevision !== blit.revision()) this.renderDirty = true;
    const live = this.playing || this.qaOn;
    if (!live) {
      if (this.renderDirty) this.presentFrame(this.readHud(), t);
      this.last = t;
      return;
    }

    const dt = Math.min((t - this.last) / 1000, 0.08);
    this.last = t;
    this.fpsAccum += dt;
    this.fpsFrames += 1;
    if (this.fpsAccum >= 0.25) {
      this.fps = this.fpsFrames / this.fpsAccum;
      this.fpsAccum = 0;
      this.fpsFrames = 0;
    }

    // Fixed simulation steps keep weapons, physics and AI independent of display refresh.
    const step = 1 / 60;
    this.accumulator += dt;
    while (this.accumulator >= step) {
      const bits = this.qaOn ? this.qaBits : this.bitsFromKeys();
      wasm.hs_input(
        bits,
        (this.lookX + this.touchLookX) * this.sens,
        (this.lookY + this.touchLookY) * this.sens,
      );
      this.lookX = this.lookY = this.touchLookX = this.touchLookY = 0;
      wasm.hs_tick(step);
      const effectHud = new DataView(wasm.memory.buffer, wasm.hs_hud_ptr(), HUD_SIZE);
      const effectWave = effectHud.getInt32(116, true);
      if (effectHud.getInt32(24, true) === 0 && this.worldEffects.tick(step, effectWave)) {
        const profile = sectorEffect(effectWave);
        this.audio.sectorAccent(profile.index);
      }
      // Drain world cues and the compact weapon state on every fixed step.
      // The complete HUD is decoded once below.
      this.sfxFromEvents(wasm.hs_events(), wasm.hs_ev_weapon());
      const soundCount = wasm.hs_sound_count();
      const soundData = new Float32Array(wasm.memory.buffer, wasm.hs_sound_cues(), soundCount * 4);
      for (let n = 0; n < soundCount; n++) {
        if (soundData[n * 4] === 24) this.worldEffects.bossEntrance(soundData[n*4+2]!,soundData[n*4+3]!,soundData[n*4+1]!);
        if (soundData[n * 4] === 2) this.worldEffects.explosion(soundData[n * 4 + 2]!, soundData[n * 4 + 3]!);
        this.audio.world(
          soundData[n * 4]!,
          soundData[n * 4 + 1]!,
          soundData[n * 4 + 2]!,
          soundData[n * 4 + 3]!,
        );
      }
      const soundHud = new DataView(wasm.memory.buffer, wasm.hs_hud_ptr(), HUD_SIZE);
      this.audio.updateWeapon({
        ammo: soundHud.getInt32(8, true),
        weapon: soundHud.getInt32(12, true),
        state: soundHud.getInt32(24, true),
        reloading: soundHud.getFloat32(92, true),
        weapFrame: soundHud.getInt32(96, true),
        wave: soundHud.getInt32(116, true),
        bossPhase: soundHud.getInt32(128, true),
      });
      this.accumulator -= step;
    }
    const hud = this.readHud();
    // Wave transitions swap the wall/door theme inside the engine; push
    // the two layers before presenting so no frame shows the old theme.
    if (hud.wave !== this.lastThemeWave) this.refreshThemeLayers(hud.wave);
    this.presentFrame(hud, t);
    const w = wasm.hs_fb_w(), h = wasm.hs_fb_h();

    // Radio text persists across frames and sector transitions. Boss death
    // audio is driven by the one-tick engine event in sfxFromEvents instead.
    if (hud.radioSeq !== this.prevRadioSeq) {
      this.prevRadioSeq = hud.radioSeq;
      if (hud.radioSeq !== 0) {
        if (hud.radioLine !== 8) this.audio.radio(hud.radioLine);
      }
    }
    this.hud = hud;
    const count = wasm.hs_prepare_enemies();
    // Cached for the QA probe (`getEnemies`) so browser tests can assert
    // move/fire animation states without touching WASM memory.
    const enemies = readEnemyCues(wasm.memory.buffer, wasm.hs_enemy_cues(), count);
    this.lastEnemies = enemies;
    const barCount = wasm.hs_prepare_bars();
    this.lastBars = readBars(wasm.memory.buffer, wasm.hs_bars(), barCount);
    const subtitles = this.audio.updateEnemies(enemies, hud);
    const loopCount = wasm.hs_prepare_sound_loops();
    const loopData = new Float32Array(wasm.memory.buffer, wasm.hs_sound_loops(), loopCount * 4);
    const loops = Array.from({ length: loopCount }, (_, n) => ({
      kind: loopData[n * 4]!,
      id: loopData[n * 4 + 1]!,
      x: loopData[n * 4 + 2]!,
      y: loopData[n * 4 + 3]!,
    }));
    this.audio.updateLoops(loops);
    this.hooks.onSubtitles?.(subtitles);
    this.hooks.onHud(hud, this.fps, `${w} × ${h}`);
    if (hud.state !== this.prevHud.state) this.hooks.onState(hud.state);
    this.prevHud = hud;
  }
  private presentFrame(hud: HudState, t: number) {
    const wasm = this.wasm!, blit = this.blit!;
    if (this.atlasRevision !== blit.revision()) this.pushAtlas();
    const boss = -1;
    const w = wasm.hs_fb_w();
    const h = wasm.hs_fb_h();
    const fx = { muzzle: hud.muzzle, hurt: hud.hurt, time: this.worldEffects.time, boss,
      entrance: this.worldEffects.projectEntrance(hud,w/h), sector: hud.wave, yaw: hud.yaw, shocks: this.worldEffects.project(hud, w / h) };
    let presented = false;
    if (this.gpuReady && blit.drawWorld) {
      wasm.hs_prepare_gpu();
      const mem = wasm.memory.buffer;
      const frame = readWorldFrame(
        mem,
        wasm.hs_gpu_view(),
        wasm.hs_gpu_cols(),
        wasm.hs_gpu_sprites(),
        wasm.hs_gpu_sprite_count(),
        wasm.hs_floor_ptr(),
        wasm.hs_light_ptr(),
        wasm.hs_smoke_ptr(),
        w,
      );
      presented = blit.drawWorld(frame, fx);
    }
    if (!presented) {
      wasm.hs_render();
      const ptr = wasm.hs_fb_ptr();
      const buf = wasm.memory.buffer;
      const len = w * h * 4;
      if (!this.fbView || this.fbBuf !== buf || this.fbLen !== len) {
        this.fbView = new Uint8Array(buf, ptr, len);
        this.fbBuf = buf;
        this.fbLen = len;
      } else if (this.fbView.byteOffset !== ptr) {
        this.fbView = new Uint8Array(buf, ptr, len);
        this.fbLen = len;
      }
      presented = blit.draw(this.fbView, w, h, fx);
    }

    this.renderDirty = !presented;
    if (presented) this.recoveryAttempts = 0;
    else this.recoverRenderer();
  }

  private pushAtlas() {
    const wasm = this.wasm;
    if (!wasm || !this.blit) return;
    this.atlasRevision = this.blit.revision();
    if (!this.blit.uploadAtlas) return;
    const size = wasm.hs_tex_size();
    const layers = new Uint8Array(TEX_N * size * size * 4);
    for (let id = 0; id < TEX_N; id++) {
      const ptr = wasm.hs_tex_ptr(id);
      layers.set(new Uint8Array(wasm.memory.buffer, ptr, size * size * 4), id * size * size * 4);
    }
    this.blit.uploadAtlas(layers);
    this.gpuReady = true;
    this.atlasRevision = this.blit.revision();
  }

  private sfxFromEvents(events: number, evWeapon: number) {
    try {
      if (this.hud.state !== 0) this.audio.setBoss(false);
      const ev = events | 0;
      if (!ev) return;
      if (ev & 1) this.audio.fire(evWeapon);
      if (ev & 2) this.audio.empty(evWeapon);
      if (ev & 16 && !(ev & 128)) this.audio.hurt();
      if (ev & 128) this.audio.die();
      if (ev & 512) this.audio.foot();
      if (ev & 2048) this.audio.setBoss(true);
      if (ev & 8192) this.audio.hushBoss();
      if (ev & 16384) this.audio.dropBoss();
      if (ev & 65536)
        this.audio.bossKill((this.hud.wave - 1) % 25, bossDeathVariantForWave(this.hud.wave));
    } catch {
      /* keep the sim running if a sound fails */
    }
  }

  private installQa() {
    // Debug hooks (grantWeapons/triggerEnd/nextWave) are dev-only. They
    // power the Playwright smokes against `npm run dev`; exposing them in
    // production lets anyone skip sectors on the live site.
    if (!import.meta.env.DEV) return;
    const mapCodes = (codes: string[]) => {
      let bits = 0;
      for (const c of codes) bits |= CODE_BITS[c] ?? 0;
      return bits;
    };
    window.__controlsTest = {
      getEnemyAudio: () => this.audio.enemyDiagnostics(),
      getSfxAudio: () => this.audio.sfxDiagnostics(),
      getWorldEffects: () => ({ time: this.worldEffects.time, wave: this.hud.wave,
        signature: sectorEffect(this.hud.wave).name, entrance:this.worldEffects.projectEntrance(this.hud,1.6) }),
      getEnemies: () => this.lastEnemies.map((e) => ({ ...e })),
      getYaw: () => this.wasm?.hs_yaw() ?? 0,
      getSpeed: () => this.wasm?.hs_speed() ?? 0,
      getX: () => this.wasm?.hs_x() ?? 0,
      getY: () => this.wasm?.hs_y() ?? 0,
      setKeys: (codes: string[]) => {
        this.qaBits = mapCodes(codes);
        if (codes.length === 0) {
          this.qaOn = false;
          this.wasm?.hs_qa(0, 0);
          return;
        }
        this.qaOn = true;
        this.playing = true;
        this.wasm?.hs_qa(this.qaBits, 1);
      },
      setSteer: () => {},
      look: (mx: number, my: number) => {
        this.lookX += mx;
        this.lookY += my;
      },
      getAmmo: () => this.hud.ammo,
      getReserve: () => this.hud.reserve,
      getReloading: () => this.hud.reloading,
      getWeapon: () => this.hud.weapon,
      selectWeapon: (slot: number) => this.wasm?.hs_select_weapon(slot),
      getWeaponFrame: () => this.hud.weapFrame,
      getSpread: () => this.hud.spread,
      getFirePatches: () => this.wasm?.hs_fire_patches() ?? 0,
      grantWeapons: () => {
        this.wasm?.hs_qa(0, 1);
        this.wasm?.hs_qa_armory();
        this.wasm?.hs_qa(this.qaBits, this.qaOn ? 1 : 0);
      },
      triggerEnd: (state: 1 | 2) => this.wasm?.hs_qa_end(state),
      heal: () => {
        // setKeys([]) disables QA mode; re-enable so the heal lands.
        this.qaOn = true;
        this.wasm?.hs_qa(this.qaBits, 1);
        this.wasm?.hs_qa_heal();
      },
      getArena: () => {
        const w = this.wasm;
        if (!w) return { warning: 0, active: 0 };
        const floor = new Uint8Array(w.memory.buffer, w.hs_floor_ptr(), 48 * 32);
        return { warning: floor.filter(v => v === 3).length, active: floor.filter(v => v === 4).length };
      },
      triggerBoss: (phase = 0) => this.wasm?.hs_qa_boss(phase),
      dropBossReward: () => {
        this.wasm?.hs_qa(0, 1);
        this.wasm?.hs_qa_reward();
      },
      visitObjective: () => this.wasm?.hs_qa_objective(),
      visitSecret: () => { this.qaOn=true;this.wasm?.hs_qa(this.qaBits,1);this.wasm?.hs_qa_tactical(0,0); },
      visitMachinery: (index=0) => { this.qaOn=true;this.wasm?.hs_qa(this.qaBits,1);this.wasm?.hs_qa_tactical(1,index); },
      getSecrets: () => this.hud.secrets,
      getMapState: () => {
        const w = this.wasm;
        if (!w) return null;
        const width = w.hs_map_w(), height = w.hs_map_h(), count = width * height;
        return { width, height,
          map: Array.from(new Uint8Array(w.memory.buffer, w.hs_map_ptr(), count)),
          doors: Array.from(new Float32Array(w.memory.buffer, w.hs_door_ptr(), count)),
          x: this.hud.x, y: this.hud.y, nodeX: this.hud.nodeX, nodeY: this.hud.nodeY,
          enemies: this.lastEnemies,
        };
      },
      nextWave: () => this.nextWave(),
    };
  }
}

let wasmModule: Promise<WebAssembly.Module> | null = null;

async function loadWasm(): Promise<WasmExports> {
  // Share compiled code, never mutable memory between runtime mounts.
  if (!wasmModule) {
    wasmModule = (async () => {
      const res = await fetch(asset("/blacksite.wasm"), { cache: "no-cache" });
      if (!res.ok) throw new Error(`Unable to load engine (${res.status})`);
      return WebAssembly.compile(await res.arrayBuffer());
    })().catch((error) => {
      wasmModule = null;
      throw error;
    });
  }
  const instance = await WebAssembly.instantiate(await wasmModule, {});
  return instance.exports as unknown as WasmExports;
}

declare global {
  interface Window {
    __controlsTest?: {
      getEnemyAudio: () => ReturnType<GameAudio["enemyDiagnostics"]>;
      getSfxAudio: () => ReturnType<GameAudio["sfxDiagnostics"]>;
      getWorldEffects: () => { time: number; wave: number; signature: string; entrance?: ReturnType<WorldEffects["projectEntrance"]> };
      getEnemies?: () => EnemyCue[];
      getYaw: () => number;
      getSpeed: () => number;
      getX?: () => number;
      getY?: () => number;
      setKeys?: (codes: string[]) => void;
      setSteer?: (v: number) => void;
      look?: (mx: number, my: number) => void;
      getAmmo?: () => number;
      getReserve?: () => number;
      getReloading?: () => number;
      getWeapon?: () => number;
      selectWeapon?: (slot: number) => void;
      getWeaponFrame?: () => number;
      getSpread?: () => number;
      getFirePatches?: () => number;
      grantWeapons?: () => void;
      triggerEnd?: (state: 1 | 2) => void;
      heal?: () => void;
      getArena?: () => { warning: number; active: number };
      triggerBoss?: (phase?: number) => void;
      dropBossReward?: () => void;
      visitObjective?: () => void;
      visitSecret?: () => void;
      visitMachinery?: (index?: number) => void;
      getSecrets?: () => number;
      getMapState?: () => MapSnapshot | null;
      nextWave?: () => void;
    };
  }
}
