import { CAMPAIGN_EXPANSION } from "./campaign25";
import { asset } from "@/lib/asset";
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
  prepareMedia: () => Promise<void>;
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
  updateLoops: (cues: { id: number; kind: number; x: number; y: number }[]) => void;
  sfxDiagnostics: () =>
    (ReturnType<SfxPlayer["diagnostics"]> & { levels: { master: number; sfx: number } }) | null;
  fire: (weapon: number) => void;
  hurt: () => void;
  die: () => void;
  empty: (weapon?: number) => void;
  foot: () => void;
};

const BPM = 136;
const BEAT = 60 / BPM;

export function createAudio(): GameAudio {
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
  let musicTimer: number | null = null;
  let musicNext = 0;
  let musicBar = 0;
  let musicGen = 0;
  let bossMode = false;
  let guitarBus: GainNode | null = null;
  let drumBus: GainNode | null = null;
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
    effects = new SfxPlayer(ctx, sfx, voiceGate);
    effects.spatial = enemyOptions.spatial;
    enemies = new EnemyAudio(ctx, master, sfx, voiceGate);
    enemies.configure(enemyOptions);
    applyGains();
    const data = new Float32Array(ctx.sampleRate * 1.2);
    for (let i = 0; i < data.length; i++) {
      const t = i / ctx.sampleRate;
      data[i] = (Math.random() * 2 - 1) * (0.7 + 0.3 * Math.sin(t * 18));
    }
    noise = ctx.createBuffer(1, data.length, ctx.sampleRate);
    noise.getChannelData(0).set(data);
    void loadSfx();
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
  function loadSfx(): Promise<void> {
    if (!ctx) return Promise.resolve();
    sfxLoadPromise ??= Promise.all(
      Object.entries(SFX_URLS).map(([key, url]) =>
        fetch(url)
          .then((r) => (r.ok ? r.arrayBuffer() : Promise.reject(new Error(url))))
          .then((buf) => ctx!.decodeAudioData(buf.slice(0)))
          .then((audio) => {
            buffers[key] = audio;
          })
          .catch(() => {
            /* synth fallback */
          }),
      ),
    ).then(() => undefined);
    return sfxLoadPromise;
  }

  function preloadMusic(): Promise<void> {
    musicLoadPromise ??= Promise.all(
      MUSIC_URLS.map(async (url) => {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`Music could not load: ${url}`);
        await response.arrayBuffer();
      }),
    ).then(() => undefined);
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

  function toneAt(
    dest: GainNode,
    when: number,
    freq: number,
    dur: number,
    vol: number,
    type: OscillatorType,
    slide = 0,
  ) {
    if (!ctx) return;
    const o = ctx.createOscillator();
    const g = ctx.createGain();
    o.type = type;
    o.frequency.setValueAtTime(freq, when);
    if (slide) o.frequency.exponentialRampToValueAtTime(Math.max(28, freq + slide), when + dur);
    g.gain.setValueAtTime(0.0001, when);
    g.gain.exponentialRampToValueAtTime(Math.max(0.0002, vol), when + 0.01);
    g.gain.exponentialRampToValueAtTime(0.0001, when + dur);
    o.connect(g);
    g.connect(dest);
    o.start(when);
    o.stop(when + dur + 0.04);
  }

  function noiseAt(
    dest: GainNode,
    when: number,
    dur: number,
    vol: number,
    freq: number,
    type: BiquadFilterType,
    rate = 1,
  ) {
    if (!ctx || !noise) return;
    const src = ctx.createBufferSource();
    src.buffer = noise;
    src.playbackRate.value = rate;
    const g = ctx.createGain();
    const f = ctx.createBiquadFilter();
    f.type = type;
    f.frequency.value = freq;
    f.Q.value = 0.85;
    g.gain.setValueAtTime(vol, when);
    g.gain.exponentialRampToValueAtTime(0.0001, when + dur);
    src.connect(f);
    f.connect(g);
    g.connect(dest);
    src.start(when);
    src.stop(when + dur);
  }

  function chug(dest: GainNode, when: number, accent: boolean) {
    const vol = accent ? 0.28 : 0.16;
    toneAt(dest, when, 82.41, 0.07, vol, "sawtooth", -28);
    toneAt(dest, when, 123.47, 0.055, vol * 0.7, "sawtooth", -34);
    toneAt(dest, when, 164.81, 0.04, vol * 0.28, "square", -40);
  }

  function kick(dest: GainNode, when: number) {
    toneAt(dest, when, 150, 0.14, 0.28, "sine", -115);
    noiseAt(dest, when, 0.06, 0.1, 80, "lowpass", 0.4);
  }

  function snare(dest: GainNode, when: number) {
    noiseAt(dest, when, 0.09, 0.16, 1800, "bandpass", 1.6);
    toneAt(dest, when, 180, 0.06, 0.08, "triangle", -80);
  }

  function hat(dest: GainNode, when: number, open: boolean) {
    noiseAt(
      dest,
      when,
      open ? 0.08 : 0.03,
      open ? 0.07 : 0.045,
      open ? 7000 : 9000,
      "highpass",
      2.4,
    );
  }

  function lead(dest: GainNode, when: number, freq: number, dur: number) {
    toneAt(dest, when, freq, dur, 0.09, "sawtooth", -freq * 0.04);
    toneAt(dest, when, freq * 1.997, dur, 0.045, "square", -freq * 0.03);
    toneAt(dest, when, freq * 0.5, dur * 0.8, 0.03, "triangle");
  }

  function scheduleBar(start: number, bar: number) {
    if (!guitarBus || !drumBus) return;
    const eighth = BEAT / 2;
    const sixteenth = BEAT / 4;
    const phase = bar & 7;

    for (let beat = 0; beat < 4; beat++) {
      const base = start + beat * BEAT;
      chug(guitarBus, base, true);
      chug(guitarBus, base + sixteenth, false);
      chug(guitarBus, base + sixteenth * 3, false);
      if (bossMode) {
        chug(guitarBus, base + sixteenth * 2, false);
        toneAt(guitarBus, base, 55, 0.08, 0.08, "sawtooth", -20);
      }
    }
    if (phase === 4 || phase === 5) {
      toneAt(guitarBus, start + BEAT * 3 + sixteenth * 2, 116.54, 0.18, 0.14, "sawtooth", -10);
      toneAt(guitarBus, start + BEAT * 3 + sixteenth * 2, 233.08, 0.14, 0.07, "square", -14);
    }

    kick(drumBus, start);
    snare(drumBus, start + BEAT);
    kick(drumBus, start + BEAT * 2);
    if (bossMode) {
      kick(drumBus, start + BEAT * 0.5);
      kick(drumBus, start + BEAT * 2.5);
      kick(drumBus, start + BEAT * 3 + sixteenth * 2);
    } else if (phase === 7) {
      kick(drumBus, start + BEAT * 2 + sixteenth * 2);
    }
    snare(drumBus, start + BEAT * 3);

    for (let i = 0; i < 8; i++) {
      hat(drumBus, start + i * eighth, i === 7 && (phase & 1) === 1);
    }

    const E3 = 164.81;
    const Fs3 = 185.0;
    const G3 = 196.0;
    const A3 = 220.0;
    const Bb3 = 233.08;
    const B3 = 246.94;
    const D4 = 293.66;
    const E4 = 329.63;
    const G4 = 392.0;

    if (phase === 2 || phase === 3 || bossMode) {
      const riff = [
        [0, E3, 0.16],
        [1, G3, 0.12],
        [2, E3, 0.12],
        [3, Bb3, 0.16],
        [4, A3, 0.12],
        [5, G3, 0.12],
        [6, Fs3, 0.12],
        [7, E3, 0.2],
      ] as const;
      for (const [i, f, d] of riff) lead(guitarBus, start + i * eighth, f, d);
    } else if (phase === 6) {
      const riff = [
        [0, E4, 0.12],
        [1, E4, 0.1],
        [2, D4, 0.1],
        [3, E4, 0.16],
        [4, G4, 0.14],
        [5, Fs3 * 2, 0.12],
        [6, E4, 0.12],
        [7, D4, 0.18],
      ] as const;
      for (const [i, f, d] of riff) lead(guitarBus, start + i * eighth, f, d);
    } else if (phase === 7) {
      const riff = [
        [0, B3, 0.14],
        [1, A3, 0.12],
        [2, G3, 0.12],
        [3, Fs3, 0.14],
        [4, E3, 0.16],
        [6, Bb3, 0.2],
        [7, E3, 0.26],
      ] as const;
      for (const [i, f, d] of riff) lead(guitarBus, start + i * eighth, f, d);
    }
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
    stopSynth();
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

  function startSynth() {
    if (!ctx || !music) return;
    musicGen += 1;
    if (!guitarBus) {
      const dist = ctx.createWaveShaper();
      const n = 2048;
      const curveArr = new Float32Array(n);
      for (let i = 0; i < n; i++) {
        const x = (i / (n - 1)) * 2 - 1;
        curveArr[i] = Math.tanh(x * 8.5);
      }
      dist.curve = curveArr;
      dist.oversample = "4x";
      const lp = ctx.createBiquadFilter();
      lp.type = "lowpass";
      lp.frequency.value = 1750;
      lp.Q.value = 1.1;
      guitarBus = ctx.createGain();
      guitarBus.gain.value = 0.0001;
      guitarBus.connect(dist);
      dist.connect(lp);
      lp.connect(music);
      drumBus = ctx.createGain();
      drumBus.gain.value = 0.55;
      drumBus.connect(music);
    }
    const t = ctx.currentTime;
    guitarBus.gain.cancelScheduledValues(t);
    guitarBus.gain.setTargetAtTime(0.9, t, 0.08);
    if (drumBus) {
      drumBus.gain.cancelScheduledValues(t);
      drumBus.gain.setTargetAtTime(0.55, t, 0.08);
    }
    musicBar = 0;
    musicNext = t + 0.05;
    pumpMusic();
  }

  function pumpMusic() {
    if (musicTimer != null) window.clearTimeout(musicTimer);
    if (!musicOn || !ctx || (bed && !bedFailed)) return;
    const now = ctx.currentTime;
    const barLen = BEAT * 4;
    while (musicNext < now + barLen * 2.2) {
      scheduleBar(musicNext, musicBar);
      musicNext += barLen;
      musicBar += 1;
    }
    musicTimer = window.setTimeout(pumpMusic, 160);
  }

  function stopSynth() {
    if (musicTimer != null) window.clearTimeout(musicTimer);
    musicTimer = null;
    const t = ctx?.currentTime ?? 0;
    const gen = ++musicGen;
    if (guitarBus && ctx) guitarBus.gain.setTargetAtTime(0.0001, t, 0.12);
    if (drumBus && ctx) drumBus.gain.setTargetAtTime(0.0001, t, 0.12);
    window.setTimeout(() => {
      if (gen !== musicGen) return;
      guitarBus = null;
      drumBus = null;
    }, 400);
  }

  function stopGate() {
    bed?.pause();
    bossBed?.pause();
    stopSynth();
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
    async prepareMedia() {
      ensure();
      await Promise.all([loadSfx(), effects?.load(), preloadMusic()]);
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
          stopSynth();
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
      if (intent) effects?.play(kind >= 21 ? intent : { ...intent, x, y });
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
