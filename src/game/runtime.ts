import { createAudio, type GameAudio } from "./audio";
import { createBlitter, type BlitKind, type Blitter } from "./blit";
import { readWorldFrame, TEX_N } from "./gpu-world";
import { HUD_SIZE } from "./hud-abi";
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
  ArrowLeft: IN.TURNL,
  KeyQ: IN.TURNL,
  ArrowRight: IN.TURNR,
  KeyR: IN.RELOAD,
};

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
  { id: 18, src: "/game/wall_tech_tile2x2.png" },
  { id: 19, src: "/game/wall_hazard_tile2x2.png" },
  { id: 20, src: "/game/spr_lamp.png" },
  { id: 21, src: "/game/spr_crate.png" },
  { id: 22, src: "/game/spr_impact.png" },
  { id: 23, src: "/game/spr_muzzle.png" },
  { id: 24, src: "/game/spr_flame.png" },
  { id: 25, src: "/game/spr_chain.png" },
  { id: 26, src: "/game/wall_pipes.png" },
  { id: 27, src: "/game/spr_gun.png" },
  { id: 28, src: "/game/floor_seal.png" },
];

const UI_CRITICAL = [
  "/game/weap_pistol.png",
  "/game/weap_pistol_fire.png",
  "/game/menu.jpg",
];

const UI_DEFERRED = [
  "/game/weap_pistol_reload.png",
  "/game/weap_shotgun.png",
  "/game/weap_shotgun_fire.png",
  "/game/weap_shotgun_reload.png",
  "/game/weap_ripper.png",
  "/game/weap_ripper_fire.png",
  "/game/weap_ripper_reload.png",
  "/game/weap_lance.png",
  "/game/weap_lance_fire.png",
  "/game/weap_lance_reload.png",
  "/game/weap_pyre.png",
  "/game/weap_pyre_fire.png",
  "/game/weap_pyre_reload.png",
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

function keySpriteAlpha(data: Uint8ClampedArray, size: number) {
  const n = size * size;
  const seen = new Uint8Array(n);
  const q: number[] = [];
  const isKey = (i: number) => {
    const o = i * 4;
    const r = data[o];
    const g = data[o + 1];
    const b = data[o + 2];
    const a = data[o + 3];
    if (a < 16) return true;
    if (r > 220 && b > 220 && g < 40) return true;
    const mx = r > g ? (r > b ? r : b) : g > b ? g : b;
    const mn = r < g ? (r < b ? r : b) : g < b ? g : b;
    return mx < 28 || (mx < 42 && mx - mn < 10);
  };
  const seed = (x: number, y: number) => {
    const i = y * size + x;
    if (isKey(i)) q.push(i);
  };
  for (let x = 0; x < size; x++) {
    seed(x, 0);
    seed(x, size - 1);
  }
  for (let y = 0; y < size; y++) {
    seed(0, y);
    seed(size - 1, y);
  }
  while (q.length) {
    const i = q.pop()!;
    if (seen[i]) continue;
    seen[i] = 1;
    if (!isKey(i)) continue;
    data[i * 4 + 3] = 0;
    const x = i % size;
    const y = (i / size) | 0;
    if (x > 0) q.push(i - 1);
    if (x + 1 < size) q.push(i + 1);
    if (y > 0) q.push(i - size);
    if (y + 1 < size) q.push(i + size);
  }
  for (let i = 0; i < n; i++) {
    if (data[i * 4 + 3] === 0) continue;
    const x = i % size;
    const y = (i / size) | 0;
    let edge = false;
    if (x > 0 && data[(i - 1) * 4 + 3] === 0) edge = true;
    if (x + 1 < size && data[(i + 1) * 4 + 3] === 0) edge = true;
    if (y > 0 && data[(i - size) * 4 + 3] === 0) edge = true;
    if (y + 1 < size && data[(i + size) * 4 + 3] === 0) edge = true;
    if (!edge) continue;
    const r = data[i * 4];
    const g = data[i * 4 + 1];
    const b = data[i * 4 + 2];
    if (r + g + b < 90) data[i * 4 + 3] = 0;
  }
}

export type RuntimeHooks = {
  onHud: (hud: HudState, fps: number, resolution: string) => void;
  onState: (state: number) => void;
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
    if (this.aborted) { this.blit.dispose(); return; }
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
    await this.uploadTextures();
    // Critical weapon/menu art blocks readiness; the rest streams in during
    // idle time so first paint isn't held hostage by ~8MB of weapon sheets.
    await preloadImages(UI_CRITICAL);
    const idle = window.requestIdleCallback ?? ((cb: () => void) => window.setTimeout(cb, 1500));
    idle(() => void preloadImages(UI_DEFERRED));
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
    }
  }

  setSens(v: number) {
    this.sens = v;
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
    this.clearInput();
    this.accumulator = 0;
    this.wasm?.hs_restart();
    this.hud = { ...DEFAULT_HUD };
    this.prevHud = { ...DEFAULT_HUD };
    this.audio.setBoss(false);
  }

  nextWave() {
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
  }

  requestLock() {
    if (window.matchMedia("(pointer: coarse)").matches || !this.canvas.requestPointerLock) return;
    this.ignoreMouse = 3;
    this.lookX = 0;
    this.lookY = 0;
    const el = this.canvas;
    const req = el.requestPointerLock as (opts?: { unadjustedMovement?: boolean }) => Promise<void> | void;
    try {
      const p = req.call(el, { unadjustedMovement: true });
      if (p && typeof (p as Promise<void>).catch === "function") {
        (p as Promise<void>).catch(() => {
          try { void Promise.resolve(el.requestPointerLock()).catch(() => {}); } catch { /* unavailable */ }
        });
      }
    } catch {
      try { void Promise.resolve(el.requestPointerLock()).catch(() => {}); } catch { /* unavailable */ }
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

  nextWeapon() {
    const owned = [true, this.hud.hasW2, this.hud.hasW3, this.hud.hasW4, this.hud.hasW5];
    for (let offset = 1; offset <= 5; offset++) {
      const next = (this.hud.weapon + offset) % 5;
      if (owned[next]) {
        this.weaponPulse = [IN.W1, IN.W2, IN.W3, IN.W4, IN.W5][next];
        return;
      }
    }
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
    const loaded = await preloadImages(TEX_FILES.map((t) => t.src));
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
        const sprite = id === 27 || (id >= 8 && id <= 14) || (id >= 20 && id <= 25);
        if (sprite) keySpriteAlpha(pixels.data, size);
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
    if (!this.playing || (e.target instanceof HTMLElement &&
      (e.target.isContentEditable || /^(INPUT|SELECT|TEXTAREA)$/.test(e.target.tagName)))) return;
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
    };
  }

  private loop = (t: number) => {
    if (!this.running || !this.wasm || !this.blit) return;
    this.raf = requestAnimationFrame(this.loop);
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
      this.wasm.hs_input(bits, (this.lookX + this.touchLookX) * this.sens,
        (this.lookY + this.touchLookY) * this.sens);
      this.lookX = this.lookY = this.touchLookX = this.touchLookY = 0;
      this.wasm.hs_tick(step);
      // Lightweight event drain per substep: avoids decoding the full HUD
      // struct N times per frame. Full HUD is decoded once below.
      this.sfxFromEvents(this.wasm.hs_events(), this.wasm.hs_ev_weapon());
      this.accumulator -= step;
    }
    const hud = this.readHud();
    const fx = { muzzle: hud.muzzle, hurt: hud.hurt, time: t * 0.001 };
    const w = this.wasm.hs_fb_w();
    const h = this.wasm.hs_fb_h();
    let presented = false;
    if (this.gpuReady && this.blit.drawWorld) {
      this.wasm.hs_prepare_gpu();
      const mem = this.wasm.memory.buffer;
      const frame = readWorldFrame(
        mem,
        this.wasm.hs_gpu_view(),
        this.wasm.hs_gpu_cols(),
        this.wasm.hs_gpu_sprites(),
        this.wasm.hs_gpu_sprite_count(),
        this.wasm.hs_floor_ptr(),
        this.wasm.hs_light_ptr(),
        w,
      );
      presented = this.blit.drawWorld(frame, fx);
    }
    if (!presented) {
      this.wasm.hs_render();
      const ptr = this.wasm.hs_fb_ptr();
      const buf = this.wasm.memory.buffer;
      const len = w * h * 4;
      if (!this.fbView || this.fbBuf !== buf || this.fbLen !== len) {
        this.fbView = new Uint8Array(buf, ptr, len);
        this.fbBuf = buf;
        this.fbLen = len;
      } else if ((this.fbView as Uint8Array).byteOffset !== ptr) {
        this.fbView = new Uint8Array(buf, ptr, len);
        this.fbLen = len;
      }
      this.blit.draw(this.fbView as Uint8Array, w, h, fx);
    }

    this.hud = hud;
    this.hooks.onHud(hud, this.fps, `${w} × ${h}`);
    if (hud.state !== this.prevHud.state) this.hooks.onState(hud.state);
    this.prevHud = hud;
  };
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
    })().catch((error) => { wasmModule = null; throw error; });
  }
  const instance = await WebAssembly.instantiate(await wasmModule, {});
  return instance.exports as unknown as WasmExports;
}

declare global {
  interface Window {
    __controlsTest?: {
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
    };
  }
}
