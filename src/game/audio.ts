export type GameAudio = {
  unlock: () => void;
  setMuted: (m: boolean) => void;
  setVolumes: (master: number, music: number, sfx: number) => void;
  setMusic: (on: boolean) => void;
  setBoss: (on: boolean) => void;
  hushBoss: () => void;
  dropBoss: () => void;
  fire: (weapon: number) => void;
  hit: () => void;
  kill: () => void;
  hurt: () => void;
  pickup: (gold?: boolean) => void;
  door: () => void;
  die: () => void;
  explode: () => void;
  reload: () => void;
  empty: (weapon?: number) => void;
  foot: () => void;
};

const BPM = 136;
const BEAT = 60 / BPM;

export function createAudio(): GameAudio {
  let ctx: AudioContext | null = null;
  let master: GainNode | null = null;
  let sfx: GainNode | null = null;
  let music: GainNode | null = null;
  let muted = false;
  let masterV = 0.85;
  let musicV = 0.42;
  let sfxV = 0.75;
  let noise: AudioBuffer | null = null;
  let musicOn = false;
  let musicTimer: number | null = null;
  let nextHitAt = 0;
  let musicNext = 0;
  let musicBar = 0;
  let musicGen = 0;
  let bossMode = false;
  let guitarBus: GainNode | null = null;
  let drumBus: GainNode | null = null;
  let bed: HTMLAudioElement | null = null;
  let bossBed: HTMLAudioElement | null = null;
  let bedFailed = false;
  let bossVol = 0;
  let bossFade = 0;
  const buffers: Record<string, AudioBuffer> = {};
  let sfxLoadStarted = false;
  const SFX_URLS: Record<string, string> = {
    fire0: "/game/sfx/fire0.ogg",
    fire1: "/game/sfx/fire1.ogg",
    fire2: "/game/sfx/fire2.ogg",
    fire3: "/game/sfx/fire3.ogg",
    fire4: "/game/sfx/fire4.ogg",
    reload: "/game/sfx/reload.ogg",
    empty: "/game/sfx/empty.ogg",
    empty1: "/game/sfx/empty1.ogg",
    empty2: "/game/sfx/empty2.ogg",
    hit: "/game/sfx/hit.ogg",
    hitFlesh: "/game/sfx/hit_flesh.ogg",
    death: "/game/sfx/death.ogg",
    deathThud: "/game/sfx/death_thud.ogg",
    pickup: "/game/sfx/pickup.ogg",
    pickupGold: "/game/sfx/pickup_gold.ogg",
    boom: "/game/sfx/boom.ogg",
    door: "/game/sfx/door.ogg?v=2",
    hurt: "/game/sfx/hurt.ogg",
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
    master.connect(ctx.destination);
    applyGains();
    const data = new Float32Array(ctx.sampleRate * 1.2);
    for (let i = 0; i < data.length; i++) {
      const t = i / ctx.sampleRate;
      data[i] = (Math.random() * 2 - 1) * (0.7 + 0.3 * Math.sin(t * 18));
    }
    noise = ctx.createBuffer(1, data.length, ctx.sampleRate);
    noise.getChannelData(0).set(data);
    loadSfx();
  }

  function resume() {
    ensure();
    if (ctx && ctx.state === "suspended") void ctx.resume();
  }

  function applyGains() {
    if (!ctx || !master || !sfx || !music) return;
    const t = ctx.currentTime;
    master.gain.setTargetAtTime(muted ? 0 : Math.max(0.0001, masterV), t, 0.04);
    sfx.gain.setTargetAtTime(muted ? 0 : Math.max(0.0001, sfxV * 1.25), t, 0.04);
    const mv = muted || !musicOn ? 0 : Math.max(0.0001, musicV * 0.55);
    music.gain.setTargetAtTime(mv, t, 0.08);
    if (bed) bed.volume = muted || !musicOn ? 0 : 0.85;
    if (bossBed) bossBed.volume = muted || !musicOn ? 0 : bossVol;
  }
  function loadSfx() {
    if (!ctx || sfxLoadStarted) return;
    sfxLoadStarted = true;
    for (const [key, url] of Object.entries(SFX_URLS)) {
      void fetch(url)
        .then((r) => (r.ok ? r.arrayBuffer() : Promise.reject(new Error(url))))
        .then((buf) => ctx!.decodeAudioData(buf.slice(0)))
        .then((audio) => {
          buffers[key] = audio;
        })
        .catch(() => {
          /* synth fallback */
        });
    }
  }

  const pool: Record<string, HTMLAudioElement[]> = {};

  function playBuffer(name: string, vol = 1, rate = 1) {
    const buf = buffers[name];
    if (!buf || !ctx || !sfx || muted) return false;
    try {
      const src = ctx.createBufferSource();
      src.buffer = buf;
      src.playbackRate.value = Math.max(0.5, Math.min(2, rate));
      const g = ctx.createGain();
      g.gain.value = Math.min(1, Math.max(0, vol));
      src.connect(g);
      g.connect(sfx);
      src.start();
      return true;
    } catch {
      return false;
    }
  }

  function playFile(name: string, vol = 1, rate = 1) {
    const url = SFX_URLS[name];
    if (!url || muted) return false;
    // Prefer decoded WebAudio buffers: no element churn, sample-accurate.
    if (playBuffer(name, Math.min(1, vol * sfxV), rate)) return true;
    try {
      const idle = pool[name] ?? (pool[name] = []);
      let el = idle.pop();
      if (!el) {
        el = new Audio(url);
        el.preload = "auto";
      } else {
        el.src = url;
      }
      el.volume = Math.min(1, Math.max(0, vol * sfxV * masterV));
      el.playbackRate = rate !== 1 ? Math.max(0.5, Math.min(2, rate)) : 1;
      const release = () => {
        try {
          el!.pause();
        } catch {
          /* ignore */
        }
        const list = pool[name] ?? (pool[name] = []);
        if (list.length < 4) list.push(el!);
      };
      el.onended = release;
      const p = el.play();
      if (p && typeof p.catch === "function") {
        void p.catch(() => {
          release();
        });
        return true;
      }
      release();
      return true;
    } catch {
      return false;
    }
  }

  function sample(name: string, vol = 1, rate = 1) {
    return playFile(name, vol, rate);
  }

  function beep(freq: number, dur: number, type: OscillatorType, vol: number, slide = 0, dest?: GainNode) {
    if (!ctx || muted) return;
    const bus = dest ?? sfx;
    if (!bus) return;
    const t = ctx.currentTime;
    const o = ctx.createOscillator();
    const g = ctx.createGain();
    o.type = type;
    o.frequency.setValueAtTime(freq, t);
    if (slide) o.frequency.exponentialRampToValueAtTime(Math.max(30, freq + slide), t + dur);
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(vol, t + 0.008);
    g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
    o.connect(g);
    g.connect(bus);
    o.start(t);
    o.stop(t + dur + 0.03);
  }

  function burst(dur: number, vol: number, rate = 1, freq = 900, type: BiquadFilterType = "bandpass") {
    if (!ctx || !sfx || !noise || muted) return;
    const t = ctx.currentTime;
    const src = ctx.createBufferSource();
    src.buffer = noise;
    src.playbackRate.value = rate * (0.92 + Math.random() * 0.16);
    const g = ctx.createGain();
    const f = ctx.createBiquadFilter();
    f.type = type;
    f.frequency.value = freq;
    f.Q.value = 0.7;
    g.gain.setValueAtTime(vol, t);
    g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
    src.connect(f);
    f.connect(g);
    g.connect(sfx);
    src.start(t);
    src.stop(t + dur);
  }

  function metallic(freq: number, dur: number, vol: number) {
    beep(freq, dur, "square", vol * 0.45, -freq * 0.4);
    beep(freq * 1.53, dur * 0.7, "triangle", vol * 0.25, -freq * 0.2);
    burst(dur * 0.5, vol * 0.35, 1.4, freq * 2, "highpass");
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

  function noiseAt(dest: GainNode, when: number, dur: number, vol: number, freq: number, type: BiquadFilterType, rate = 1) {
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
    noiseAt(dest, when, open ? 0.08 : 0.03, open ? 0.07 : 0.045, open ? 7000 : 9000, "highpass", 2.4);
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
    const el = new Audio(url);
    el.loop = true;
    // Long-form tracks (bgm is ~31min) must stream, never preload fully.
    el.preload = "none";
    el.crossOrigin = "anonymous";
    try {
      const node = ctx.createMediaElementSource(el);
      node.connect(music);
      return el;
    } catch {
      return null;
    }
  }

  function ensureBed() {
    if (!ctx || !music || bedFailed) return;
    if (!bed) bed = hookBed("/game/bgm.ogg");
    if (!bossBed) bossBed = hookBed("/game/boss.ogg");
    if (!bed && !bossBed) bedFailed = true;
  }

  function playEl(el: HTMLAudioElement | null, fromStart = false) {
    if (!el) return;
    if (fromStart) el.currentTime = 0;
    const play = el.play();
    if (play && typeof play.catch === "function") void play.catch(() => {});
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
    startSynth();
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
    unlock() {
      resume();
    },
    setMuted(m) {
      muted = m;
      applyGains();
    },
    setVolumes(masterVol, musicVol, sfxVol) {
      masterV = masterVol;
      musicV = musicVol;
      sfxV = sfxVol;
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
    fire(weapon) {
      resume();
      const jitter = 0.94 + Math.random() * 0.12;
      if (weapon === 4) {
        // The launcher needs a short mechanical thump, not the old looping
        // flamethrower sample. Detonation has its own spatially later event.
        beep(90, 0.16, "sine", 0.22, -55);
        burst(0.11, 0.34 * jitter, 0.7, 550, "lowpass");
        metallic(650, 0.055, 0.08);
        return;
      }
      // File OR synth, never both — the sample already carries the transient.
      if (sample(`fire${weapon}`, weapon === 1 ? 1.35 : 1.2, jitter)) return;
      if (weapon === 1) {
        burst(0.22, 0.55 * jitter, 0.55, 160, "lowpass");
        burst(0.08, 0.28, 1.8, 1800, "highpass");
        beep(70, 0.14, "sawtooth", 0.16, -40);
      } else if (weapon === 2) {
        burst(0.05, 0.28 * jitter, 1.8, 1400, "bandpass");
        beep(240, 0.04, "square", 0.06, -90);
      } else if (weapon === 3) {
        beep(620, 0.22, "sawtooth", 0.07, -280);
        burst(0.18, 0.2, 1.1, 2200, "bandpass");
      } else {
        burst(0.07, 0.24 * jitter, 2.1, 1100, "bandpass");
        beep(380, 0.05, "square", 0.07, -160);
      }
    },
    hit() {
      resume();
      const now = ctx?.currentTime ?? 0;
      if (now < nextHitAt) return;
      nextHitAt = now + 0.04;
      if (sample("hit", 0.85, 0.94 + Math.random() * 0.12)) {
        sample("hitFlesh", 0.55, 0.9 + Math.random() * 0.2);
        return;
      }
      beep(980, 0.035, "square", 0.06, 220);
      metallic(420, 0.06, 0.05);
    },
    kill() {
      resume();
      if (sample("death", 0.9, 0.92 + Math.random() * 0.16)) {
        sample("deathThud", 0.8, 0.88 + Math.random() * 0.2);
        return;
      }
      beep(90, 0.35, "sawtooth", 0.2, -50);
      burst(0.32, 0.28, 0.45, 110, "lowpass");
    },
    hurt() {
      resume();
      if (sample("hurt", 1.35, 0.92 + Math.random() * 0.1)) return;
      burst(0.22, 0.35, 0.45, 140, "lowpass");
      beep(110, 0.2, "sawtooth", 0.16, -50);
    },
    pickup(gold = false) {
      resume();
      if (gold) {
        if (sample("pickupGold", 1.35, 0.96 + Math.random() * 0.08)) return;
        beep(523, 0.1, "triangle", 0.09, 80);
        beep(784, 0.14, "square", 0.07, 60);
      } else {
        if (sample("pickup", 1.3, 0.96 + Math.random() * 0.08)) return;
        beep(660, 0.07, "square", 0.08, 120);
        beep(880, 0.09, "triangle", 0.06, 90);
      }
    },
    door() {
      resume();
      if (sample("door", 1.2, 0.94 + Math.random() * 0.08)) return;
      burst(0.28, 0.2, 0.35, 140, "lowpass");
      metallic(180, 0.22, 0.08);
    },
    die() {
      resume();
      beep(70, 0.55, "sawtooth", 0.24, -40);
      burst(0.5, 0.3, 0.3, 90, "lowpass");
    },
    explode() {
      resume();
      if (sample("boom", 1.4, 0.92 + Math.random() * 0.1)) return;
      burst(0.45, 0.7, 0.35, 70, "lowpass");
      beep(48, 0.42, "sine", 0.28, -18);
      beep(72, 0.28, "sawtooth", 0.16, -30);
    },
    reload() {
      resume();
      if (sample("reload", 0.95, 0.96 + Math.random() * 0.08)) return;
      metallic(240, 0.08, 0.08);
      window.setTimeout(() => metallic(180, 0.1, 0.07), 90);
      window.setTimeout(() => burst(0.06, 0.12, 1.1, 700, "bandpass"), 180);
    },
    empty(weapon = 0) {
      resume();
      const click = weapon === 1 ? "empty1" : weapon === 2 || weapon === 4 ? "empty2" : "empty";
      if (sample(click, 0.9, 0.97 + Math.random() * 0.06)) return;
      if (weapon === 1) {
        burst(0.07, 0.16, 0.7, 320, "bandpass");
        beep(70, 0.08, "square", 0.08, -12);
        metallic(150, 0.09, 0.1);
      } else if (weapon === 2) {
        beep(260, 0.028, "square", 0.07, -50);
        burst(0.028, 0.11, 3.4, 3200, "highpass");
        metallic(420, 0.04, 0.05);
      } else if (weapon === 3) {
        beep(980, 0.04, "square", 0.06, -520);
        beep(140, 0.07, "sawtooth", 0.05, -40);
        burst(0.05, 0.09, 1.5, 1700, "bandpass");
      } else if (weapon === 4) {
        burst(0.09, 0.13, 1.7, 850, "highpass");
        beep(64, 0.07, "triangle", 0.06, -8);
        metallic(190, 0.06, 0.07);
      } else {
        beep(190, 0.032, "square", 0.08, -28);
        burst(0.03, 0.11, 2.9, 2500, "highpass");
        metallic(340, 0.045, 0.07);
      }
    },
    foot() {
      resume();
      burst(0.07, 0.12, 0.55 + Math.random() * 0.2, 180, "lowpass");
      beep(70 + Math.random() * 20, 0.05, "triangle", 0.03, -10);
    },
  };
}
