import { createAudio, type GameAudio } from "./audio";
import { createBlitter, type BlitKind, type Blitter } from "./blit";
import {
  ENEMY_ANIM_COUNT,
  ENEMY_TEX_BASE,
  readWorldFrame,
  TEX_N,
  T_GUN3,
  T_GUN4,
  T_GUN5,
  T_GUN6,
  T_GUN7,
  T_ORDNANCE,
} from "./gpu-world";
import { HUD_SIZE } from "./hud-abi";
import { keySpriteAlpha } from "./sprite-alpha";
import {
  readEnemyCues,
  type EnemyCue,
  type EnemyOptions,
  type EnemySubtitle,
} from "./enemy-presentation";
import { DEFAULT_GFX, DEFAULT_HUD, type GfxOpts, type HudState, type ResMode } from "./types";

export { HUD_SIZE };

type WasmExports = {
  memory: WebAssembly.Memory;
  hs_init: (w: number, h: number) => number;
  hs_restart: () => void;
  hs_next_wave: () => void;
  hs_resize: (w: number, h: number) => void;
  hs_fb_ptr: () => number;
  hs_fb_w: () => number;
  hs_fb_h: () => number;
  hs_hud_ptr: () => number;
  hs_hud_size: () => number;
  hs_events: () => number;
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
  hs_yaw: () => number;
  hs_speed: () => number;
  hs_spread: () => number;
  hs_x: () => number;
  hs_y: () => number;
  hs_prepare_enemies: () => number;
  hs_enemy_cues: () => number;
  hs_qa_end: (state: number) => void;
  hs_qa_boss: (phase: number) => void;
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
  ...ENEMY_SKINS.flatMap((skin, skinIndex) =>
    ENEMY_ANIMATIONS.map((animation, animationIndex) => ({
      id: ENEMY_TEX_BASE + skinIndex * ENEMY_ANIM_COUNT + animationIndex,
      src: `/game/enemy_${skin}_${animation}.png`,
    })),
  ),
];

const UI_CRITICAL = ["/game/weap_mk23s.png", "/game/weap_mk23s_fire.png", "/game/menu.jpg"];

const UI_DEFERRED = [
  "/game/weap_mk23s_reload.png",
  "/game/weap_br12.png",
  "/game/weap_br12_fire.png",
  "/game/weap_br12_reload.png",
  "/game/weap_kx9.png",
  "/game/weap_kx9_fire.png",
  "/game/weap_kx9_reload.png",
  "/game/weap_mr4.png",
  "/game/weap_mr4_fire.png",
  "/game/weap_mr4_reload.png",
  "/game/weap_vlk6.png",
  "/game/weap_vlk6_fire.png",
  "/game/weap_vlk6_reload.png",
  "/game/weap_ax12.png",
  "/game/weap_ax12_fire.png",
  "/game/weap_ax12_reload.png",
  "/game/weap_m91.png",
  "/game/weap_m91_fire.png",
  "/game/weap_m91_reload.png",
];

function decodeImage(src: string): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const img = new Image();
    let settled = false;
    const finish = (value: HTMLImageElement | null) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timer);
      resolve(value);
    };
    const timer = window.setTimeout(() => finish(null), 8000);
    img.onload = () => finish(img);
    img.onerror = () => finish(null);
    img.src = src;
  });
}

async function preloadImages(urls: string[], limit = 4) {
  const out: (HTMLImageElement | null)[] = new Array(urls.length);
  let cursor = 0;
  async function worker() {
    while (cursor < urls.length) {
      const i = cursor;
      cursor += 1;
      out[i] = await decodeImage(urls[i]!);
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, urls.length) }, () => worker()));
  return out;
}

export type RuntimeHooks = {
  onSubtitles?: (subtitles: EnemySubtitle[]) => void;
  onHud: (hud: HudState, fps: number, resolution: string) => void;
  onState: (state: number) => void;
  /** Fired once when the frame loop throws (e.g. a WASM trap from a
   * version-skewed binary). Without this the game would freeze silently
   * on a stale HUD with no game-over ever arriving. */
  onError?: (message: string) => void;
};

export class HellscanRuntime {
  private canvas: HTMLCanvasElement;
  private hooks: RuntimeHooks;
  private wasm: WasmExports | null = null;
  private blit: Blitter | null = null;
  private audio: GameAudio;
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
  private lastEnemies: EnemyCue[] = [];
  private qaBits = 0;
  private qaOn = false;
  private sens = 1;
  private usePulse = 0;
  private reloadPulse = 0;
  private weaponPulse = 0;
  muted = false;
  renderer: BlitKind = "canvas2d";
  private volumes = { master: 0.85, music: 0.42, sfx: 0.75 };
  private gfx: GfxOpts = { ...DEFAULT_GFX };
  private requireGpu = false;
  private fbView: Uint8Array | null = null;
  private fbBuf: ArrayBuffer | null = null;
  private fbLen = 0;
  private gpuReady = false;

  constructor(canvas: HTMLCanvasElement, hooks: RuntimeHooks) {
    this.canvas = canvas;
    this.hooks = hooks;
    this.audio = createAudio();
  }
  async boot(res: ResMode, opts?: { requireGpu?: boolean }) {
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
    // Every firing/reload sheet must be cached before play: switching a CSS
    // background to an unloaded sheet otherwise hides the weapon mid-shot.
    await Promise.all([
      this.uploadTextures(),
      preloadImages([...UI_CRITICAL, ...UI_DEFERRED]),
      this.audio.prepareEnemies(),
    ]);
    if (this.aborted) {
      this.running = false;
      cancelAnimationFrame(this.raf);
    }
  }

  setResolution(res: ResMode) {
    this.resolution = res;
    if (!this.wasm || !this.blit) return;
    const aspect = (this.canvas.clientWidth || res.w) / (this.canvas.clientHeight || res.h);
    const w = Math.max(160, Math.round(Math.min(res.w, res.h * aspect)));
    const h = Math.max(100, Math.round(w / aspect));
    this.wasm.hs_resize(w, h);
    this.blit.resize(w, h);
    this.canvas.dataset.resolution = `${w} × ${h}`;
  }

  setPlaying(v: boolean) {
    this.playing = v;
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
  }

  setSens(v: number) {
    this.sens = v;
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

  setVolumes(master: number, music: number, sfx: number) {
    this.volumes = { master, music, sfx };
    this.audio.setVolumes(master, music, sfx);
  }

  setGfx(g: GfxOpts) {
    this.gfx = { ...g };
    this.blit?.setGfx(this.gfx);
  }

  /** Switch WebGPU enforcement without restarting the sim or canvas. */
  async switchRenderer(requireGpu: boolean): Promise<BlitKind> {
    if (!this.wasm || !this.blit || this.aborted) return this.renderer;
    if (requireGpu === this.requireGpu && this.blit.kind === this.renderer) return this.renderer;
    this.requireGpu = requireGpu;
    const old = this.blit;
    const next = await createBlitter(this.canvas, { requireGpu });
    if (this.aborted) {
      next.dispose();
      return this.renderer;
    }
    this.blit = next;
    this.renderer = next.kind;
    next.setGfx(this.gfx);
    if (this.resolution) this.setResolution(this.resolution);
    old.dispose();
    this.fbView = null;
    this.fbBuf = null;
    this.fbLen = 0;
    return this.renderer;
  }
  restart() {
    this.audio.clearEnemies(true);
    this.clearInput();
    this.accumulator = 0;
    this.wasm?.hs_restart();
    this.hud = { ...DEFAULT_HUD };
    this.prevHud = { ...DEFAULT_HUD };
    this.audio.setBoss(false);
  }

  nextWave() {
    this.audio.clearEnemies(true);
    this.wasm?.hs_next_wave();
    this.audio.setBoss(false);
  }

  stop() {
    this.aborted = true;
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
    this.touchLookX += dx;
    this.touchLookY += dy;
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
    ];
    const step = dir >= 0 ? 1 : 6;
    for (let n = 1; n <= 7; n++) {
      const next = (this.hud.weapon + step * n) % 7;
      if (owned[next]) {
        this.weaponPulse = [IN.W1, IN.W2, IN.W3, IN.W4, IN.W5, IN.W6, IN.W7][next]!;
        return;
      }
    }
  }

  nextWeapon() {
    this.cycleWeapon(1);
  }

  getHud() {
    return this.hud;
  }

  private async uploadTextures() {
    const wasm = this.wasm;
    if (!wasm) return;
    const size = wasm.hs_tex_size();
    const scratch = document.createElement("canvas");
    scratch.width = size;
    scratch.height = size;
    const ctx = scratch.getContext("2d", { willReadFrequently: true });
    if (!ctx) return;
    // Enemy animation layers are independent 256px files; decode them in a
    // wider batch so the first playable frame is not gated by four-at-a-time
    // image loads.
    const loaded = await preloadImages(
      TEX_FILES.map((t) => t.src),
      12,
    );
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
          id >= ENEMY_TEX_BASE;
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
        /* keep procedural */
      }
    }
    wasm.hs_textures_ready();
    this.pushAtlas();
  }

  private bind() {
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

    const live = this.playing || this.qaOn;
    if (!live) {
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
      // Lightweight event drain per substep: avoids decoding the full HUD
      // struct N times per frame. Full HUD is decoded once below.
      this.sfxFromEvents(wasm.hs_events(), wasm.hs_ev_weapon());
      this.accumulator -= step;
    }
    const hud = this.readHud();
    const fx = { muzzle: hud.muzzle, hurt: hud.hurt, time: t * 0.001 };
    const w = wasm.hs_fb_w();
    const h = wasm.hs_fb_h();
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
      } else if ((this.fbView as Uint8Array).byteOffset !== ptr) {
        this.fbView = new Uint8Array(buf, ptr, len);
        this.fbLen = len;
      }
      blit.draw(this.fbView as Uint8Array, w, h, fx);
    }

    this.hud = hud;
    const count = wasm.hs_prepare_enemies();
    // Cached for the QA probe (`getEnemies`) so browser tests can assert
    // move/fire animation states without touching WASM memory.
    const enemies = readEnemyCues(wasm.memory.buffer, wasm.hs_enemy_cues(), count);
    this.lastEnemies = enemies;
    const subtitles = this.audio.updateEnemies(enemies, hud);
    this.hooks.onSubtitles?.(subtitles);
    this.hooks.onHud(hud, this.fps, `${w} × ${h}`);
    if (hud.state !== this.prevHud.state) this.hooks.onState(hud.state);
    this.prevHud = hud;
  }
  private pushAtlas() {
    const wasm = this.wasm;
    if (!wasm || !this.blit?.uploadAtlas) return;
    const size = wasm.hs_tex_size();
    const layers = new Uint8Array(TEX_N * size * size * 4);
    for (let id = 0; id < TEX_N; id++) {
      const ptr = wasm.hs_tex_ptr(id);
      layers.set(new Uint8Array(wasm.memory.buffer, ptr, size * size * 4), id * size * size * 4);
    }
    this.blit.uploadAtlas(layers);
    this.gpuReady = true;
  }

  private sfxFromEvents(events: number, evWeapon: number) {
    try {
      if (this.hud.state !== 0) this.audio.setBoss(false);
      const ev = events | 0;
      if (!ev) return;
      if (ev & 1) this.audio.fire(evWeapon);
      if (ev & 2) this.audio.empty(evWeapon);
      if (ev & 4) this.audio.reload();
      if (ev & 8) this.audio.hit();
      if (ev & 16) this.audio.hurt();
      if (ev & 32) this.audio.pickup(false);
      if (ev & 64) this.audio.pickup(true);
      if (ev & 128) this.audio.die();
      if (ev & 256) this.audio.explode();
      if (ev & 512) this.audio.foot();
      if (ev & 1024) this.audio.door();
      if (ev & 2048) this.audio.setBoss(true);
      if (ev & 4096) this.audio.kill();
      if (ev & 8192) this.audio.hushBoss();
      if (ev & 16384) this.audio.dropBoss();
    } catch {
      /* keep the sim running if a sound fails */
    }
  }

  private installQa() {
    if (!import.meta.env.DEV && new URLSearchParams(location.search).get("qa") !== "1") return;
    const mapCodes = (codes: string[]) => {
      let bits = 0;
      for (const c of codes) bits |= CODE_BITS[c] ?? 0;
      return bits;
    };
    window.__controlsTest = {
      getEnemyAudio: () => this.audio.enemyDiagnostics(),
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
      getSpread: () => this.hud.spread,
      getFirePatches: () => this.wasm?.hs_fire_patches() ?? 0,
      grantWeapons: () => {
        this.wasm?.hs_qa(0, 1);
        this.wasm?.hs_qa_armory();
        this.wasm?.hs_qa(this.qaBits, this.qaOn ? 1 : 0);
      },
      triggerEnd: (state: 1 | 2) => this.wasm?.hs_qa_end(state),
      triggerBoss: (phase = 0) => this.wasm?.hs_qa_boss(phase),
      nextWave: () => this.nextWave(),
    };
  }
}

let wasmModule: Promise<WebAssembly.Module> | null = null;

async function loadWasm(): Promise<WasmExports> {
  // Share compiled code, never mutable memory between runtime mounts.
  if (!wasmModule) {
    wasmModule = (async () => {
      const res = await fetch("/hellscan.wasm", { cache: "no-cache" });
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
      getSpread?: () => number;
      getFirePatches?: () => number;
      grantWeapons?: () => void;
      triggerEnd?: (state: 1 | 2) => void;
      triggerBoss?: (phase?: number) => void;
      nextWave?: () => void;
    };
  }
}
