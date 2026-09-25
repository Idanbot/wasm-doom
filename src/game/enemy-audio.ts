import { asset } from "@/lib/asset";
import {
  DEFAULT_ENEMY_OPTIONS,
  VoiceDirector,
  subtitleFor,
  type EnemyCue,
  type EnemyOptions,
  type EnemySubtitle,
  type VoiceLine,
  type VoiceProfile,
} from "./enemy-presentation.ts";

type SpatialSound = {
  id: number;
  skin: number;
  source: AudioScheduledSourceNode;
  panner: PannerNode;
  filter: BiquadFilterNode;
  gain: GainNode;
  end: number;
  line?: VoiceLine;
  profile?: VoiceProfile;
};

type ListenerPose = { x: number; y: number; fx: number; fy: number };

/**
 * Some browsers (notably Firefox) never implemented the AudioParam-based
 * `AudioListener.positionX/...` properties — only the legacy
 * `setPosition()`/`setOrientation()` methods. Feature-detect once per call
 * site and fall back, or positioning throws every frame and kills the game.
 */
function writeListenerPose(ctx: AudioContext, pose: ListenerPose) {
  const l = ctx.listener;
  if (l.positionX && l.positionY && l.positionZ && l.forwardX && l.upX) {
    const t = ctx.currentTime;
    l.positionX.setValueAtTime(pose.x, t);
    l.positionY.setValueAtTime(0, t);
    l.positionZ.setValueAtTime(pose.y, t);
    l.forwardX.setValueAtTime(pose.fx, t);
    l.forwardY.setValueAtTime(0, t);
    l.forwardZ.setValueAtTime(pose.fy, t);
    l.upX.setValueAtTime(0, t);
    l.upY.setValueAtTime(1, t);
    l.upZ.setValueAtTime(0, t);
  } else {
    l.setPosition(pose.x, 0, pose.y);
    l.setOrientation(pose.fx, 0, pose.fy, 0, 1, 0);
  }
}

function writePannerPosition(panner: PannerNode, x: number, y: number, z: number) {
  if (panner.positionX && panner.positionY && panner.positionZ) {
    const t = panner.context.currentTime;
    panner.positionX.setValueAtTime(x, t);
    panner.positionY.setValueAtTime(y, t);
    panner.positionZ.setValueAtTime(z, t);
  } else {
    panner.setPosition(x, y, z);
  }
}

function readPannerCoord(
  panner: PannerNode,
  axis: "positionX" | "positionY" | "positionZ",
): number {
  const param = panner[axis];
  return param ? param.value : 0;
}

export class EnemyAudio {
  private options = { ...DEFAULT_ENEMY_OPTIONS };
  private voiceBus: GainNode;
  private profiles = new Map<number, VoiceProfile>();
  private buffers = new Map<string, AudioBuffer>();
  private active: SpatialSound[] = [];
  private director = new VoiceDirector();
  private previous = new Map<number, { anim: number; hp: number; next: number }>();
  private loaded: Promise<void> | null = null;
  private closed = false;
  private subtitles: EnemySubtitle[] = [];
  /** Last written listener pose. Doubles as the fallback read path where
   * AudioListener AudioParams do not exist. */
  private pose: ListenerPose = { x: 0, y: 0, fx: 1, fy: 0 };
  constructor(
    private ctx: AudioContext,
    master: GainNode,
    private sfxBus: GainNode,
  ) {
    this.voiceBus = ctx.createGain();
    this.voiceBus.connect(master);
    this.configure(this.options);
  }
  async load(onProgress?: (done: number, total: number) => void) {
    this.loaded ??= (async () => {
      const response = await fetch(asset("/game/voices/manifest.json"));
      if (!response.ok) throw new Error("Enemy voice manifest could not load");
      const manifest = (await response.json()) as { enemies: VoiceProfile[] };
      for (const p of manifest.enemies) this.profiles.set(p.skin, p);
      const lines = manifest.enemies.flatMap((p) => p.lines);
      let index = 0;
      let done = 0;
      onProgress?.(0, lines.length);
      await Promise.all(
        Array.from({ length: 4 }, async () => {
          while (index < lines.length && !this.closed) {
            const line = lines[index++]!;
            const r = await fetch(asset(line.url));
            if (!r.ok) throw new Error(`Enemy voice missing: ${line.id}`);
            const decoded = await this.ctx.decodeAudioData(await r.arrayBuffer());
            this.buffers.set(line.id, decoded);
            done += 1;
            onProgress?.(done, lines.length);
          }
        }),
      );
    })();
    return this.loaded;
  }
  configure(options: EnemyOptions) {
    this.options = { ...options };
    this.voiceBus.gain.setTargetAtTime(
      Math.min(1.3, options.voices * 1.25),
      this.ctx.currentTime,
      0.02,
    );
    if (!options.subtitles) this.subtitles = [];
  }
  captions() {
    return this.subtitles;
  }
  diagnostics() {
    return {
      state: this.ctx.state,
      loaded: this.buffers.size,
      voices: this.active
        .filter((s) => s.line)
        .map((s) => ({
          id: s.id,
          text: s.line!.text,
          x: readPannerCoord(s.panner, "positionX"),
          y: readPannerCoord(s.panner, "positionZ"),
          panning: s.panner.panningModel,
          filter: s.filter.frequency.value,
        })),
      listener: {
        x: this.pose.x,
        y: this.pose.y,
        dx: this.pose.fx,
        dy: this.pose.fy,
      },
      volume: this.voiceBus.gain.value,
    };
  }
  preview(skin: number) {
    this.silence();
    void this.ctx.resume();
    const profile = this.profiles.get(skin);
    if (!profile?.lines[0]) return;
    this.speak(
      {
        id: -1,
        skin,
        anim: 0,
        hp: 1,
        x: this.pose.x + this.pose.fx * 2,
        y: this.pose.y + this.pose.fy * 2,
        screenX: -10,
        screenY: -10,
        sight: true,
        distance: 2,
      },
      profile,
      profile.lines[0],
    );
  }
  silence(reset = false) {
    for (const sound of [...this.active]) {
      try {
        sound.source.stop();
      } catch {}
      this.dispose(sound);
    }
    this.active = [];
    this.subtitles = [];
    if (reset) {
      this.director.reset();
      this.previous.clear();
    }
  }
  close() {
    this.closed = true;
    this.silence(true);
    this.voiceBus.disconnect();
  }
  private dispose(sound: SpatialSound) {
    sound.source.disconnect();
    sound.panner.disconnect();
    sound.filter.disconnect();
    sound.gain.disconnect();
    this.active = this.active.filter((s) => s !== sound);
  }
  private place(sound: SpatialSound, enemy: EnemyCue) {
    const t = this.ctx.currentTime;
    writePannerPosition(sound.panner, enemy.x, 0, enemy.y);
    sound.filter.frequency.setTargetAtTime(enemy.sight ? 11000 : 750, t, 0.04);
    sound.gain.gain.setTargetAtTime((enemy.sight ? 1 : 0.18) * (sound.line ? 1.1 : 0.24), t, 0.04);
    sound.panner.panningModel = this.options.spatial ? "HRTF" : "equalpower";
    // Spatial off means centered sound; distance attenuation stays useful.
    if (!this.options.spatial) {
      writePannerPosition(
        sound.panner,
        this.pose.x + this.pose.fx * enemy.distance,
        0,
        this.pose.y + this.pose.fy * enemy.distance,
      );
    }
  }
  private connect(
    source: AudioScheduledSourceNode,
    enemy: EnemyCue,
    duration: number,
    line?: VoiceLine,
    profile?: VoiceProfile,
  ) {
    const panner = this.ctx.createPanner();
    panner.distanceModel = "inverse";
    panner.refDistance = 2;
    panner.maxDistance = 20;
    panner.rolloffFactor = 1.1;
    const filter = this.ctx.createBiquadFilter();
    filter.type = "lowpass";
    const gain = this.ctx.createGain();
    gain.gain.value = 0;
    source.connect(filter);
    filter.connect(gain);
    gain.connect(panner);
    panner.connect(line ? this.voiceBus : this.sfxBus);
    const sound = {
      id: enemy.id,
      skin: enemy.skin,
      source,
      filter,
      gain,
      panner,
      end: this.ctx.currentTime + duration,
      line,
      profile,
    };
    this.place(sound, enemy);
    this.active.push(sound);
    source.onended = () => this.dispose(sound);
    source.start();
    return sound;
  }
  private speak(enemy: EnemyCue, profile: VoiceProfile, line: VoiceLine) {
    const buffer = this.buffers.get(line.id);
    if (!buffer || this.active.some((s) => s.line)) return;
    const source = this.ctx.createBufferSource();
    source.buffer = buffer;
    // Slower machines and commander delivery improve intelligibility, while
    // small human variation keeps the roster from sharing one cadence.
    const rate =
      enemy.skin === 12
        ? 0.88
        : [5, 8, 11].includes(enemy.skin)
          ? 0.93
          : 0.96 + (enemy.skin % 3) * 0.025;
    source.playbackRate.value = rate;
    this.connect(source, enemy, buffer.duration / rate, line, profile);
  }
  private enemySound(enemy: EnemyCue, pain: boolean) {
    if (this.active.filter((s) => !s.line).length >= 6 || enemy.distance > 16) return;
    const oscillator = this.ctx.createOscillator();
    const biological = [2, 6, 10, 11].includes(enemy.skin);
    oscillator.type = biological ? "sawtooth" : enemy.skin === 9 ? "square" : "triangle";
    const base =
      enemy.skin === 6
        ? 55
        : enemy.skin === 10
          ? 420
          : enemy.skin === 9
            ? 140
            : biological
              ? 95
              : 190;
    const duration = biological ? 0.5 : 0.18;
    oscillator.frequency.setValueAtTime(base * (pain ? 1.6 : 1), this.ctx.currentTime);
    oscillator.frequency.exponentialRampToValueAtTime(base * 0.45, this.ctx.currentTime + duration);
    const sound = this.connect(oscillator, enemy, duration);
    sound.gain.gain.setTargetAtTime(0.001, this.ctx.currentTime + duration * 0.4, duration * 0.18);
    oscillator.stop(this.ctx.currentTime + duration);
  }
  update(enemies: EnemyCue[], player: { x: number; y: number; yaw: number }) {
    const t = this.ctx.currentTime;
    this.pose = { x: player.x, y: player.y, fx: Math.cos(player.yaw), fy: Math.sin(player.yaw) };
    writeListenerPose(this.ctx, this.pose);
    const byId = new Map(enemies.map((e) => [e.id, e]));
    this.subtitles = [];
    for (const sound of [...this.active]) {
      const enemy = byId.get(sound.id);
      if (!enemy || enemy.hp <= 0 || enemy.skin !== sound.skin || t >= sound.end) {
        try {
          sound.source.stop();
        } catch {}
        this.dispose(sound);
        continue;
      }
      this.place(sound, enemy);
      if (this.options.subtitles && sound.line && sound.profile) {
        const caption = subtitleFor(enemy, sound.profile, sound.line);
        if (caption) this.subtitles.push(caption);
      }
    }
    if (this.ctx.state !== "running") return;
    if (!this.active.some((s) => s.line)) {
      const chosen = this.director.choose(enemies, this.profiles, t);
      if (chosen) this.speak(chosen.enemy, chosen.profile, chosen.line);
    }
    for (const enemy of enemies) {
      if (enemy.hp <= 0) {
        this.previous.delete(enemy.id);
        continue;
      }
      const previous = this.previous.get(enemy.id);
      const nonverbal = [2, 6, 9, 10, 11].includes(enemy.skin);
      if (
        previous &&
        ((enemy.anim === 3 && previous.anim !== 3) ||
          enemy.hp < previous.hp ||
          (nonverbal && enemy.sight && t > previous.next))
      ) {
        this.enemySound(enemy, enemy.hp < previous.hp);
        previous.next = t + 4 + (enemy.id % 4);
      }
      this.previous.set(enemy.id, {
        hp: enemy.hp,
        anim: enemy.anim,
        next: previous?.next ?? t + 1.5 + (enemy.id % 4),
      });
    }
    for (const id of this.previous.keys()) if (!byId.has(id)) this.previous.delete(id);
  }
}
