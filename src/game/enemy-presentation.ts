export type EnemyCue = {
  id: number;
  skin: number;
  anim: number;
  hp: number;
  x: number;
  y: number;
  screenX: number;
  screenY: number;
  sight: boolean;
  distance: number;
};

export type BarCue = {
  screenX: number;
  screenY: number;
  frac: number;
  layer: number;
  fade: number;
};

export function readBars(memory: ArrayBuffer, ptr: number, count: number): BarCue[] {
  const n = Math.max(0, Math.min(48, count));
  const data = new Float32Array(memory, ptr, n * 5);
  return Array.from({ length: n }, (_, i) => {
    const o = i * 5;
    return {
      screenX: data[o]!,
      screenY: data[o + 1]!,
      frac: data[o + 2]!,
      layer: data[o + 3]!,
      fade: data[o + 4]!,
    };
  });
}
export type EnemySubtitle = { id: number; name: string; text: string; x: number; y: number };
export type VoiceLine = { id: string; cue: string; text: string; url: string; duration: number };
export type VoiceProfile = {
  skin: number;
  id: string;
  name: string;
  style: string;
  speaker: string | null;
  lines: VoiceLine[];
};
export type EnemyOptions = {
  subtitles: boolean;
  subtitleSize: number;
  voices: number;
  spatial: boolean;
  showStats: boolean;
};
export const DEFAULT_ENEMY_OPTIONS: EnemyOptions = {
  subtitles: true,
  subtitleSize: 1,
  voices: 1,
  spatial: true,
  showStats: false,
};

export function readEnemyCues(memory: ArrayBuffer, ptr: number, count: number): EnemyCue[] {
  const data = new Float32Array(memory, ptr, Math.min(192, count) * 10);
  return Array.from({ length: Math.min(192, count) }, (_, i) => {
    const o = i * 10;
    return {
      id: data[o]!,
      skin: data[o + 1]!,
      anim: data[o + 2]!,
      hp: data[o + 3]!,
      x: data[o + 4]!,
      y: data[o + 5]!,
      screenX: data[o + 6]!,
      screenY: data[o + 7]!,
      sight: data[o + 8] === 1,
      distance: data[o + 9]!,
    };
  });
}

export function subtitleFor(
  enemy: EnemyCue,
  profile: VoiceProfile,
  line: VoiceLine,
): EnemySubtitle | null {
  // Never reveal enemies behind walls or behind the camera with captions.
  if (
    enemy.hp <= 0 ||
    !enemy.sight ||
    enemy.distance > 16 ||
    enemy.screenX < 0.06 ||
    enemy.screenX > 0.94 ||
    enemy.screenY < 0.06 ||
    enemy.screenY > 0.85
  )
    return null;
  return {
    id: enemy.id,
    name: profile.name,
    text: line.text,
    x: enemy.screenX * 100,
    y: enemy.screenY * 100,
  };
}

export function loadEnemyOptions(): EnemyOptions {
  try {
    const data = JSON.parse(localStorage.getItem("blacksite-enemy-options") || "{}");
    const finite = (v: unknown, fallback: number, min: number, max: number) =>
      typeof v === "number" && Number.isFinite(v) ? Math.max(min, Math.min(max, v)) : fallback;
    return {
      subtitles: typeof data.subtitles === "boolean" ? data.subtitles : true,
      spatial: typeof data.spatial === "boolean" ? data.spatial : true,
      showStats: data.showStats === true,
      subtitleSize: finite(data.subtitleSize, 1, 0.85, 1.4),
      voices: finite(data.voices, 1, 0, 1),
    };
  } catch {
    return { ...DEFAULT_ENEMY_OPTIONS };
  }
}

/** Deterministic per-enemy rotation, with priorities and explicit chatter limits. */
export class VoiceDirector {
  private seen = new Map<
    number,
    { hp: number; anim: number; skin: number; next: number; engaged: boolean; serial: number }
  >();
  private nextGlobal = 0;
  private rotations = new Map<string, number>();
  reset() {
    this.seen.clear();
    this.nextGlobal = 0;
    this.rotations.clear();
  }
  choose(
    enemies: EnemyCue[],
    profiles: Map<number, VoiceProfile>,
    now: number,
  ): { enemy: EnemyCue; profile: VoiceProfile; line: VoiceLine } | null {
    let chosen: {
      enemy: EnemyCue;
      profile: VoiceProfile;
      cue: string;
      priority: number;
      serial: number;
    } | null = null;
    const live = new Set(enemies.filter((e) => e.hp > 0).map((e) => e.id));
    for (const id of this.seen.keys()) if (!live.has(id)) this.seen.delete(id);
    for (const enemy of enemies) {
      const profile = profiles.get(enemy.skin);
      if (enemy.hp <= 0 || !profile?.lines.length) continue;
      let state = this.seen.get(enemy.id);
      if (!state || state.skin !== enemy.skin) {
        state = {
          hp: enemy.hp,
          anim: enemy.anim,
          skin: enemy.skin,
          next: now + 0.6 + (enemy.id % 3) * 0.15,
          engaged: false,
          serial: 0,
        };
        this.seen.set(enemy.id, state);
      }
      const pain = enemy.hp < state.hp;
      const attacking = (enemy.anim === 6 || enemy.anim === 3) && enemy.anim !== state.anim;
      state.hp = enemy.hp;
      state.anim = enemy.anim;
      if (now < state.next || now < this.nextGlobal || enemy.distance > 14 || !enemy.sight)
        continue;
      const cue = !state.engaged
        ? "alert"
        : pain
          ? "pain"
          : attacking || (enemy.skin === 11 && enemy.distance < 3)
            ? "attack"
            : "taunt";
      if (cue === "taunt" && now < state.next + 4) continue;
      const priority =
        ([12, 13, 14].includes(enemy.skin) ? 10 : 0) +
        (cue === "alert" ? 4 : cue === "pain" ? 3 : cue === "attack" ? 2 : 1) -
        enemy.distance * 0.04;
      if (!chosen || priority > chosen.priority)
        chosen = { enemy, profile, cue, priority, serial: state.serial };
    }
    if (!chosen) return null;
    const matching = chosen.profile.lines.filter((line) => line.cue === chosen.cue);
    const key = `${chosen.enemy.skin}:${chosen.cue}`;
    const rotation = this.rotations.get(key) ?? chosen.enemy.id;
    const line = matching[rotation % matching.length] ?? chosen.profile.lines[0]!;
    this.rotations.set(key, rotation + 1);
    const state = this.seen.get(chosen.enemy.id)!;
    state.engaged = true;
    state.serial++;
    state.next = now + 8 + (chosen.enemy.id % 5);
    this.nextGlobal = now + Math.max(2.6, line.duration + 0.6);
    return { enemy: chosen.enemy, profile: chosen.profile, line };
  }
}
