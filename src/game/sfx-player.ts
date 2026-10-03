import { asset } from "@/lib/asset";
import { VoiceGate } from "./voice-gate";
import type { SoundIntent } from "./sfx-director";
export type SfxClip = { id: string; url: string; fallback: string; vocal: boolean };
type LiveSound = {
  source: AudioBufferSourceNode;
  nodes: AudioNode[];
  group: string;
  vocal: boolean;
  released: boolean;
  loopKey?: number;
  panner?: PannerNode;
  releaseVoice?: () => void;
  clipId: string;
  gain: GainNode;
  baseGain: number;
};
/** Decoded local recordings with bounded concurrency, MP3 compatibility and shared mixer routing. */
export class SfxPlayer {
  private buffers = new Map<string, AudioBuffer>();
  private vocals = new Set<string>();
  private active: LiveSound[] = [];
  private loadPromise: Promise<void> | null = null;
  private closed = false;
  private muted = false;
  private failed: string[] = [];
  private fallback = new Set<string>();
  private count = 0;
  private played: string[] = [];
  private next = new Map<string, number>();
  spatial = true;
  listenerPosition = { x: 0, y: 0 };
  constructor(
    private ctx: AudioContext,
    private bus: GainNode,
    private voiceGate: VoiceGate,
  ) {}
  async load(): Promise<void> {
    this.loadPromise ??= (async () => {
      const r = await fetch(asset("/game/sfx/v2/manifest.json"));
      if (!r.ok) throw new Error(`Sound effects manifest missing (${r.status})`);
      const manifest = (await r.json()) as { clips: SfxClip[] };
      let index = 0;
      await Promise.all(
        Array.from({ length: 6 }, async () => {
          while (index < manifest.clips.length && !this.closed) {
            const clip = manifest.clips[index++]!;
            let decoded: AudioBuffer | null = null;
            for (const [n, url] of [clip.url, clip.fallback].entries()) {
              try {
                const response = await fetch(asset(url));
                if (!response.ok) throw new Error(`${response.status}`);
                decoded = await this.ctx.decodeAudioData(await response.arrayBuffer());
                if (n) this.fallback.add(clip.id);
                break;
              } catch {
                /* MP3 fallback is attempted once, during loading only. */
              }
            }
            if (!decoded) {
              this.failed.push(clip.id);
              continue;
            }
            this.buffers.set(clip.id, decoded);
            if (clip.vocal) this.vocals.add(clip.id);
          }
        }),
      );
      if (this.failed.length)
        throw new Error(`Sound effects unavailable: ${this.failed.join(", ")}`);
    })();
    return this.loadPromise;
  }
  diagnostics() {
    return {
      loaded: this.buffers.size,
      failed: [...this.failed],
      fallback: [...this.fallback],
      active: this.active.length,
      played: this.count,
      recent: [...this.played],
      state: this.ctx.state,
    };
  }
  setMuted(muted: boolean) {
    this.muted = muted;
    if (muted) this.stop();
  }
  private release(sound: LiveSound) {
    if (sound.released) return;
    sound.released = true;
    sound.source.disconnect();
    for (const node of sound.nodes) node.disconnect();
    this.active = this.active.filter((s) => s !== sound);
    sound.releaseVoice?.();
    sound.releaseVoice = undefined;
  }
  stop(group?: string) {
    for (const sound of [...this.active])
      if (!group || sound.group === group) {
        try {
          sound.source.stop();
        } catch {}
        this.release(sound);
      }
    if (!group) this.next.clear();
  }
  updateLoops(cues: { id: number; kind: number; x: number; y: number }[]) {
    const present = new Set(cues.map((c) => c.id));
    for (const s of [...this.active])
      if (s.loopKey !== undefined && !present.has(s.loopKey)) {
        s.source.stop();
        this.release(s);
      }
    for (const cue of cues) {
      const id = ["travel-rocket", "travel-energy", "travel-acid", "travel-flame"][cue.kind - 14];
      if (!id) continue;
      let live = this.active.find((s) => s.loopKey === cue.id);
      if (live && live.clipId !== id) {
        live.source.stop();
        this.release(live);
        live = undefined;
      }
      if (live) {
        const p = live.panner;
        const x = this.spatial ? cue.x : this.listenerPosition.x,
          y = this.spatial ? cue.y : this.listenerPosition.y;
        if (p) {
          if (p.positionX) {
            p.positionX.value = x;
            p.positionZ.value = y;
          } else p.setPosition(x, 0, y);
        }
        const attenuation = this.spatial ? 1 : 1 / (1 + Math.hypot(cue.x - x, cue.y - y) / 3);
        live.gain.gain.setTargetAtTime(
          live.baseGain * attenuation * (this.voiceGate.busy ? 0.55 : 1),
          this.ctx.currentTime,
          0.04,
        );
        continue;
      }
      if (this.active.filter((s) => s.loopKey !== undefined).length >= 6) break;
      this.play(
        { id, x: cue.x, y: cue.y, gain: cue.kind === 17 ? 0.25 : 0.35, group: "travel" },
        cue.id,
      );
    }
  }
  play(intent: SoundIntent, loopKey?: number): boolean {
    const buffer = this.buffers.get(intent.id);
    if (this.closed || this.muted || !buffer || this.ctx.state !== "running") return false;
    const vocal = this.vocals.has(intent.id);
    if (vocal && this.voiceGate.busy) return false;
    const group = intent.group ?? "player",
      t = this.ctx.currentTime;
    // Bound bursts per source/category as well as total nodes. UI and player shots get priority.
    const key = `${intent.id}:${intent.x ?? "local"}:${intent.y ?? "local"}`;
    if (t < (this.next.get(key) ?? 0)) return false;
    this.next.set(key, t + (group === "hazard" ? 0.22 : group === "world" ? 0.045 : 0.018));
    if (this.next.size > 512) for (const [id, end] of this.next) if (end < t) this.next.delete(id);
    if (
      this.active.length >= 24 ||
      this.active.filter((s) => s.group === group).length >=
        (group === "enemy" ? 8 : group === "hazard" ? 4 : 12)
    ) {
      if (group !== "player" && group !== "ui") return false;
      const oldest =
        this.active.find((s) => s.group !== "ui" && s.group !== "player") ?? this.active[0];
      if (oldest) {
        oldest.source.stop();
        this.release(oldest);
      }
    }
    const source = this.ctx.createBufferSource(),
      gain = this.ctx.createGain();
    source.buffer = buffer;
    source.loop = loopKey !== undefined;
    source.playbackRate.value = Math.max(0.75, Math.min(1.25, intent.rate ?? 1));
    gain.gain.value =
      Math.max(0, Math.min(1, intent.gain ?? 0.75)) * (this.voiceGate.busy && !vocal ? 0.55 : 1);
    source.connect(gain);
    const nodes: AudioNode[] = [gain];
    let tail: AudioNode = gain;
    let panner: PannerNode | undefined;
    if (intent.x !== undefined && intent.y !== undefined) {
      const p = this.ctx.createPanner();
      panner = p;
      p.panningModel = this.spatial ? "HRTF" : "equalpower";
      p.distanceModel = "inverse";
      p.refDistance = 2;
      p.maxDistance = 24;
      p.rolloffFactor = 1.2;
      const x = this.spatial ? intent.x : this.listenerPosition.x;
      const y = this.spatial ? intent.y : this.listenerPosition.y;
      if (p.positionX) {
        p.positionX.value = x;
        p.positionY.value = 0;
        p.positionZ.value = y;
      } else p.setPosition(x, 0, y);
      const filter = this.ctx.createBiquadFilter();
      filter.type = "lowpass";
      filter.frequency.value = intent.sight === false ? 1100 : 14000;
      if (!this.spatial) gain.gain.value /= 1 + Math.hypot(intent.x - x, intent.y - y) / 3;
      gain.connect(filter);
      filter.connect(p);
      tail = p;
      nodes.push(filter, p);
    }
    tail.connect(this.bus);
    const sound: LiveSound = {
      source,
      nodes,
      group,
      vocal,
      released: false,
      loopKey,
      panner,
      clipId: intent.id,
      gain,
      baseGain: Math.max(0, Math.min(1, intent.gain ?? 0.75)),
    };
    this.active.push(sound);
    source.onended = () => this.release(sound);
    const start = () => {
      source.start();
      this.count++;
      this.played.push(intent.id);
      if (this.played.length > 64) this.played.shift();
    };
    if (vocal) {
      if (
        !this.voiceGate.tryStart((done) => {
          sound.releaseVoice = done;
          start();
        })
      ) {
        this.release(sound);
        return false;
      }
    } else start();
    return true;
  }
  close() {
    this.closed = true;
    this.stop();
    this.buffers.clear();
  }
}
