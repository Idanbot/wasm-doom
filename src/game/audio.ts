import { CAMPAIGN_EXPANSION } from "./campaign25";
import { asset } from "@/lib/asset";
import { fetchAsset, preloadAssets, type AssetProgress } from "./asset-loading";
import { EnemyAudio } from "./enemy-audio";
import { VoiceGate } from "./voice-gate";
import { SfxPlayer } from "./sfx-player";
import { SfxDirector, worldIntent, type WeaponSoundState } from "./sfx-director";
import {
  DEFAULT_ENEMY_OPTIONS,
  type EnemyCue,
  type EnemyOptions,
  type EnemySubtitle,
} from "./enemy-presentation";

export type GameAudio = {
  previewEnemy: (skin: number) => void;
  enemyDiagnostics: () => ReturnType<EnemyAudio["diagnostics"]> | null;
  prepareEnemies: (onProgress?: (done: number, total: number) => void) => Promise<void>;
  prepareMedia: (onProgress?: AssetProgress) => Promise<void>;
  setEnemyOptions: (options: EnemyOptions) => void;
  updateEnemies: (
    enemies: EnemyCue[],
    player: { x: number; y: number; yaw: number },
  ) => EnemySubtitle[];
  clearEnemies: (reset?: boolean) => void;
  advanceSectorVoices: () => void;
  dispose: () => void;
  unlock: () => void;
  setMuted: (m: boolean) => void;
  setVolumes: (master: number, music: number, sfx: number, menu: number) => void;
  setMusic: (on: boolean) => void;
  setMenuBed: (on: boolean) => void;
  bossKill: (sector: number, variant: number) => void;
  setBoss: (on: boolean) => void;
  hushBoss: () => void;
  dropBoss: () => void;
  radio: (line?: number) => void;
  ui: (kind?: "click" | "confirm" | "back" | "error" | "transition") => void;
  updateWeapon: (hud: WeaponSoundState & { bossPhase: number }) => void;
  world: (kind: number, variant: number, x: number, y: number) => void;
  sectorAccent: (sector: number) => void;
  updateLoops: (cues: { id: number; kind: number; x: number; y: number }[]) => void;
  sfxDiagnostics: () =>
    (ReturnType<SfxPlayer["diagnostics"]> & { levels: { master: number; sfx: number } }) | null;
  fire: (weapon: number) => void;
  hurt: () => void;
  die: () => void;
  empty: (weapon?: number) => void;
  foot: () => void;
};

export function createAudio(loadSignal?: AbortSignal): GameAudio {
  let ctx: AudioContext | null = null;
  let enemies: EnemyAudio | null = null;
  let effects: SfxPlayer | null = null;
  const director = new SfxDirector();
  let sector = 1;
  let lastPlayer = { x: 0, y: 0 };
  let lastWeapon = -1;
  let bossPosition: { x: number; y: number } | undefined;
  const voiceGate = new VoiceGate();
  let enemyOptions = { ...DEFAULT_ENEMY_OPTIONS };
  let master: GainNode | null = null;
  let sfx: GainNode | null = null;
  let music: GainNode | null = null;
  let muted = false;
  let masterV = 0.85;
  let musicV = 0.42;
  let sfxV = 0.75;
  let menuV = 0.7;
  let noise: AudioBuffer | null = null;
  let musicOn = false;
  let bossMode = false;
  let bed: HTMLAudioElement | null = null;
  let bossBed: HTMLAudioElement | null = null;
  let menuBed: HTMLAudioElement | null = null;
  let menuBedWanted = false;
  let bedFailed = false;
  let bossVol = 0;
  let bossFade = 0;
  const buffers: Record<string, AudioBuffer> = {};
  let sfxLoadPromise: Promise<void> | null = null;
  let musicLoadPromise: Promise<void> | null = null;
  const MUSIC_URLS = [
    asset("/game/music/menu.mp3"),
    asset("/game/music/bgm-remix.mp3"),
    asset("/game/music/boss.mp3"),
  ];
  const SFX_URLS: Record<string, string> = {
    ...Object.fromEntries(
      [
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
        ...CAMPAIGN_EXPANSION.map((b) => b.slug),
      ].flatMap((id, sector) =>
        [0, 1].map((variant) => [
          `bossKill${sector}_${variant}`,
          asset(`/game/voices/boss-${id}${variant ? "-v2" : ""}.mp3`),
        ]),
      ),
    ),
  };

  function ensure() {
    if (ctx) return;
    type WebkitAudioWindow = Window & { webkitAudioContext?: typeof AudioContext };
    const audioWindow = window as WebkitAudioWindow;
    const AC = window.AudioContext || audioWindow.webkitAudioContext;
    if (!AC) return;
    ctx = new AC({ latencyHint: "interactive" });
    master = ctx.createGain();
    sfx = ctx.createGain();
    music = ctx.createGain();
    sfx.connect(master);
    music.connect(master);
    const limiter = ctx.createDynamicsCompressor();
    limiter.threshold.value = -8;
    limiter.knee.value = 12;
    limiter.ratio.value = 4;
    limiter.attack.value = 0.005;
    limiter.release.value = 0.15;
    master.connect(limiter);
    limiter.connect(ctx.destination);
    effects = new SfxPlayer(ctx, sfx, voiceGate, loadSignal);
    effects.spatial = enemyOptions.spatial;
    enemies = new EnemyAudio(ctx, master, sfx, voiceGate, loadSignal);
    enemies.configure(enemyOptions);
    applyGains();
    const data = new Float32Array(ctx.sampleRate * 1.2);
    for (let i = 0; i < data.length; i++) {
      const t = i / ctx.sampleRate;
      data[i] = (Math.random() * 2 - 1) * (0.7 + 0.3 * Math.sin(t * 18));
    }
    noise = ctx.createBuffer(1, data.length, ctx.sampleRate);
    noise.getChannelData(0).set(data);
  }

  function resume() {
    ensure();
    if (ctx && ctx.state === "suspended") void ctx.resume();
  }

  function applyGains() {
    if (!ctx || !master || !sfx || !music) return;
    const t = ctx.currentTime;
    master.gain.setTargetAtTime(muted ? 0 : masterV, t, 0.04);
    sfx.gain.setTargetAtTime(muted ? 0 : sfxV, t, 0.04);
    const mv = muted || !musicOn ? 0 : musicV * 0.55;
    music.gain.setTargetAtTime(mv, t, 0.08);
    if (bed) bed.volume = muted || !musicOn ? 0 : 0.85;
    if (bossBed) bossBed.volume = muted || !musicOn ? 0 : bossVol;
    if (menuBed) menuBed.volume = muted ? 0 : menuV;
  }
  function loadSfx(onProgress?: AssetProgress): Promise<void> {
    if (!ctx) return Promise.resolve();
    const entries = Object.entries(SFX_URLS);
    sfxLoadPromise ??= preloadAssets(entries.map(([, url]) => url), (url, onAttempt) =>
      fetchAsset(url, async response => ctx!.decodeAudioData(await response.arrayBuffer()),
        { signal: loadSignal, onAttempt }), 4, onProgress).then(decoded => {
      for (let i = 0; i < entries.length; i++) if (decoded[i]) buffers[entries[i]![0]] = decoded[i]!;
      // Exhausted legacy clips keep the existing synth fallback.
    });
    return sfxLoadPromise;
  }

  function preloadMusic(onProgress?: AssetProgress): Promise<void> {
    musicLoadPromise ??= preloadAssets(MUSIC_URLS, (url, onAttempt) => fetchAsset(url,
      async (response, signal) => {
        const blob = await response.blob();
        // Validate MP3 metadata without expanding long music beds into PCM.
        const url = URL.createObjectURL(blob);
        const element = new Audio();
        try {
          await new Promise<void>((resolve, reject) => {
            const cancel = () => reject(signal.reason);
            signal.addEventListener("abort", cancel, { once: true });
            const finish = (error?: Error) => {
              signal.removeEventListener("abort", cancel);
              if (error) reject(error); else resolve();
            };
            element.onloadedmetadata = () => Number.isFinite(element.duration) && element.duration > 0
              ? finish() : finish(new Error("Invalid music duration"));
            element.onerror = () => finish(new Error("Music could not decode"));
            element.preload = "metadata";
            element.src = url;
            if (signal.aborted) cancel(); else element.load();
          });
          return blob.size;
        } finally {
          element.onloadedmetadata = element.onerror = null;
          element.removeAttribute("src"); element.load();
          URL.revokeObjectURL(url);
        }
      }, { signal: loadSignal, onAttempt, timeoutMs: 60000 }), 2, onProgress).then(loaded => {
      const failed = MUSIC_URLS.filter((_, i) => !loaded[i]);
      if (failed.length) throw new Error(`Music could not load: ${failed.join(", ")}`);
      for (const element of [menuBed, bed, bossBed]) {
        if (!element?.error) continue;
        element.load();
        if (element === menuBed && menuBedWanted) playEl(element);
      }
    });
    return musicLoadPromise;
  }

  function playVoiceClip(name: string, done: () => void) {
    const buffer = buffers[name];
    if (buffer && ctx && sfx && !muted) {
      const source = ctx.createBufferSource();
      const gain = ctx.createGain();
      source.buffer = buffer;
      gain.gain.value = 1;
      source.connect(gain);
      gain.connect(sfx);
      source.onended = () => {
        source.disconnect();
        gain.disconnect();
        done();
      };
      source.start();
      return;
    }
    const url = SFX_URLS[name];
    if (!url || muted) {
      done();
      return;
    }
    const element = new Audio(url);
    element.volume = Math.min(1, 1.35 * sfxV * masterV);
    element.onended = done;
    void element.play().catch(done);
  }

  function hookBed(url: string): HTMLAudioElement | null {
    if (!ctx || !music) return null;
    const el = new Audio();
    el.loop = true;
    el.preload = "auto";
    el.src = url;
    try {
      const node = ctx.createMediaElementSource(el);
      node.connect(music);
      return el;
    } catch {
      return null;
    }
  }

  // Menu bed bypasses the music gain (gated by musicOn) and hangs off
  // master, so the standby screen has its own bed while the field is quiet.
  // preload="none": the 21MB bed streams on first actual play instead of
  // fetching on every page load (which stalls headless runs and burns
  // mobile data when the player goes straight into the field).
  function hookMasterBed(url: string): HTMLAudioElement | null {
    if (!ctx || !master) return null;
    const el = new Audio();
    el.loop = true;
    el.preload = "none";
    el.src = url;
    try {
      const node = ctx.createMediaElementSource(el);
      node.connect(master);
      return el;
    } catch {
      return null;
    }
  }

  function ensureBed() {
    if (!ctx || !music || bedFailed) return;
    if (!bed) bed = hookBed(asset("/game/music/bgm-remix.mp3"));
    if (!bossBed) bossBed = hookBed(asset("/game/music/boss.mp3"));
    if (!bed && !bossBed) bedFailed = true;
  }

  function playEl(el: HTMLAudioElement | null, fromStart = false) {
    if (!el) return;
    const start = () => {
      try {
        if (fromStart && el.readyState >= 1) el.currentTime = 0;
      } catch {
        /* metadata not in yet; play from the start anyway */
      }
      const play = el.play();
      if (play && typeof play.catch === "function") {
        void play.catch(() => {
          if (el === bossBed && bed && musicOn) playEl(bed);
        });
      }
    };
    // Calling play directly preserves the gesture's autoplay permission.
    // Loading an unplayed media element first can hold its range request open
    // and block the full-file preload of the same music URL in Chromium.
    start();
  }

  function setBossElVol(v: number) {
    bossVol = Math.max(0, Math.min(1, v));
    if (bossBed) bossBed.volume = muted || !musicOn ? 0 : bossVol;
  }

  function stopBossFade() {
    if (bossFade) {
      cancelAnimationFrame(bossFade);
      bossFade = 0;
    }
  }

  function fadeBoss(from: number, to: number, ms: number) {
    stopBossFade();
    setBossElVol(from);
    const t0 = performance.now();
    const tick = (now: number) => {
      const p = Math.min(1, (now - t0) / ms);
      setBossElVol(from + (to - from) * p);
      if (p < 1) bossFade = requestAnimationFrame(tick);
      else bossFade = 0;
    };
    bossFade = requestAnimationFrame(tick);
  }

  function startGate() {
    if (!ctx || !music) return;
    ensureBed();
    menuBed?.pause();
    if (bossMode && bossBed) {
      bed?.pause();
      playEl(bossBed);
      return;
    }
    if (bed) {
      bossBed?.pause();
      playEl(bed);
      return;
    }
    // Every background bed is an MP3; no generated fallback music.
  }

  function stopGate() {
    bed?.pause();
    bossBed?.pause();
  }

  return {
    previewEnemy(skin) {
      resume();
      enemies?.preview(skin);
    },
    enemyDiagnostics: () => enemies?.diagnostics() ?? null,
    async prepareEnemies(onProgress?: (done: number, total: number) => void) {
      ensure();
      await enemies?.load(onProgress);
    },
    async prepareMedia(onProgress) {
      ensure();
      const progress = [0, 0, 0];
      const slice = (index: number): AssetProgress => (done, total) => {
        progress[index] = total ? done / total : 1;
        onProgress?.(progress.reduce((sum, value) => sum + value, 0), 3);
      };
      await Promise.all([loadSfx(slice(0)), effects?.load(slice(1)), preloadMusic(slice(2))]);
      onProgress?.(3, 3);
    },
    setEnemyOptions(options) {
      enemyOptions = { ...options };
      enemies?.configure(options);
      if (effects && effects.spatial !== options.spatial) {
        effects.spatial = options.spatial;
        effects.stop("travel");
      }
    },
    updateEnemies(cues, player) {
      lastPlayer = { x: player.x, y: player.y };
      const boss = cues.find((c) => c.skin >= 12 && c.hp > 0);
      bossPosition = boss ? { x: boss.x, y: boss.y } : undefined;
      if (effects) effects.listenerPosition = lastPlayer;
      enemies?.update(cues, player);
      for (const intent of director.enemyFrame(cues, ctx?.currentTime ?? 0)) effects?.play(intent);
      return enemies?.captions() ?? [];
    },
    clearEnemies(reset = false) {
      enemies?.silence(reset);
      effects?.stop();
      if (reset) {
        director.reset();
        bossPosition = undefined;
      } else director.pause();
    },
    advanceSectorVoices() {
      enemies?.advanceSector();
      effects?.stop();
      director.reset();
    },
    dispose() {
      musicOn = false;
      voiceGate.clearPending();
      stopGate();
      enemies?.close();
      effects?.close();
      if (ctx) void ctx.close();
    },
    unlock() {
      resume();
      // Autoplay policy may have blocked the standby bed before the first
      // gesture; retry it on every unlock while the menu wants it.
      if (menuBedWanted && menuBed && menuBed.paused) playEl(menuBed);
    },
    setMuted(m) {
      muted = m;
      effects?.setMuted(m);
      applyGains();
    },
    setVolumes(masterVol, musicVol, sfxVol, menuVol) {
      masterV = masterVol;
      musicV = musicVol;
      sfxV = sfxVol;
      menuV = menuVol ?? menuV;
      applyGains();
    },
    setMusic(on) {
      resume();
      musicOn = on;
      applyGains();
      if (on) {
        startGate();
      } else {
        stopGate();
      }
    },
    setMenuBed(on) {
      resume();
      menuBedWanted = on;
      if (on) {
        if (!menuBed) menuBed = hookMasterBed(asset("/game/music/menu.mp3"));
        if (menuBed) {
          bed?.pause();
          bossBed?.pause();
          menuBed.volume = muted ? 0 : menuV;
          playEl(menuBed);
        }
        return;
      }
      menuBed?.pause();
    },
    setBoss(on) {
      const was = bossMode;
      bossMode = on;
      if (!on) {
        stopBossFade();
        setBossElVol(0);
        if (was && musicOn) {
          bossBed?.pause();
          if (bossBed) bossBed.currentTime = 0;
          playEl(bed);
        }
        return;
      }
      if (!musicOn) return;
      ensureBed();
      if (!bossBed) {
        playEl(bed);
        return;
      }
      bed?.pause();
      setBossElVol(0);
      playEl(bossBed, true);
      fadeBoss(0, 1, 5000);
    },
    hushBoss() {
      if (!bossMode) return;
      stopBossFade();
      bossBed?.pause();
    },
    dropBoss() {
      if (!musicOn) return;
      bossMode = true;
      ensureBed();
      bed?.pause();
      stopBossFade();
      setBossElVol(1);
      playEl(bossBed, false);
    },
    sfxDiagnostics: () =>
      effects
        ? {
            ...effects.diagnostics(),
            levels: { master: muted ? 0 : masterV, sfx: muted ? 0 : sfxV },
          }
        : null,
    ui(kind = "click") {
      resume();
      effects?.play({ id: `ui-${kind}`, gain: 0.6, group: "ui" });
    },
    updateWeapon(hud) {
      sector = hud.wave;
      if (hud.weapon !== lastWeapon) effects?.stop("reload");
      lastWeapon = hud.weapon;
      for (const intent of director.weaponFrame(hud)) effects?.play(intent);
      for (const intent of director.bossPhase(
        hud.bossPhase,
        12 + ((hud.wave - 1) % 25),
        bossPosition ?? lastPlayer,
      ))
        effects?.play(intent);
    },
    updateLoops: (cues) => effects?.updateLoops(cues),
    world(kind, variant, x, y) {
      const intent = worldIntent(kind, variant);
      if (intent) effects?.play(kind === 21 || kind === 22 ? intent : { ...intent, x, y });
    },
    sectorAccent(index) {
      effects?.play({ id: `sector-${index + 1}`, gain: 0.22, group: "ambience" });
    },
    radio(line = 0) {
      resume();
      effects?.play({ id: `terminal${Math.abs(line) % 3}`, gain: 0.55, group: "world" });
    },
    bossKill(sector, variant) {
      resume();
      const clip = sector >= 0 && sector < 25 ? `bossKill${sector}_${variant === 1 ? 1 : 0}` : null;
      if (clip) voiceGate.enqueue((done) => playVoiceClip(clip, done));
    },
    fire(weapon) {
      resume();
      effects?.stop("reload");
      effects?.play({ id: `fire${weapon}`, gain: 0.8, rate: 0.98 + Math.random() * 0.04 });
    },
    // Spatial impacts, deaths, doors and pickups are driven by entity/world events.
    hurt() {
      resume();
      effects?.play({ id: `pain-human${Math.floor(Math.random() * 3)}`, gain: 0.6 });
    },
    die() {
      resume();
      effects?.play({ id: "body-thud", gain: 0.6 });
      effects?.play({ id: "death-human2", gain: 0.65 });
    },
    empty(weapon = 0) {
      resume();
      effects?.play({ id: "empty", gain: 0.65, rate: 1 - (weapon % 3) * 0.04 });
    },
    foot() {
      resume();
      const surfaces = [
        "concrete",
        "metal",
        "concrete",
        "soft",
        "metal",
        "soft",
        "metal",
        "metal",
        "concrete",
        "metal",
        "metal",
        "metal",
        "metal",
        "concrete",
        "metal",
        "metal",
        "soft",
        "metal",
        "concrete",
        "soft",
        "metal",
        "metal",
        "metal",
        "metal",
        "metal",
      ];
      const surface = surfaces[(sector - 1) % 25] ?? "metal";
      effects?.play({
        id: `foot-${surface}${Math.floor(Math.random() * 3)}`,
        gain: 0.5,
        rate: 0.96 + Math.random() * 0.08,
      });
    },
  };
}
