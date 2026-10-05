import { enemyCombatSkin } from "./sector-enemies.ts";
/** Pure event decisions, independent of Web Audio and the rendering frame rate. */
export type WeaponSoundState = {
  weapon: number;
  ammo: number;
  reloading: number;
  weapFrame: number;
  state: number;
  wave: number;
};
export type CombatSoundCue = {
  id: number;
  skin: number;
  hp: number;
  anim: number;
  x: number;
  y: number;
  sight: boolean;
  distance: number;
};
export type SoundIntent = {
  id: string;
  gain?: number;
  rate?: number;
  x?: number;
  y?: number;
  sight?: boolean;
  group?: string;
};
export const ENEMY_WEAPONS = [
  2, 1, 24, 7, 6, 16, 13, 3, 5, 24, 10, 4, 8, 9, 10, 12, 13, 11, 14, 15, 16, 17, 18, 19, 20, 21, 22,
  23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
] as const;
export const WEAPON_HAZARDS = [
  "electric",
  "pressure",
  "acid",
  "phase",
  "magnetic",
  "electric",
  "cryo",
  "electric",
  "pressure",
  "electric",
  "phase",
  "ember",
  "pressure",
  "phase",
  "magnetic",
  "biological",
  "biological",
  "ember",
  "pressure",
  "phase",
  "ember",
  "pressure",
  "phase",
  "cryo",
  "electric",
] as const;
export function enemyFamily(skin: number): "human" | "creature" | "robot" {
  skin = enemyCombatSkin(skin);
  if ([2, 6, 9, 10, 14, 24, 28].includes(skin)) return "creature";
  if (
    [
      5, 8, 11, 13, 15, 16, 17, 18, 19, 20, 22, 23, 25, 26, 27, 29, 30, 31, 32, 33, 34, 35, 36,
    ].includes(skin)
  )
    return "robot";
  return "human";
}
export function weaponMechanism(weapon: number): [string, string, string] {
  return weapon <= 4 || weapon === 6 || weapon === 17 || weapon === 26
    ? ["mag-out", "mag-in", "chamber"]
    : ["cell-out", "cell-in", "charge"];
}
export function worldIntent(kind: number, variant = 0): SoundIntent | null {
  const id = (
    {
      1: "door",
      2: "explosion",
      3: "hit-metal",
      4: "hit-flesh",
      5: "break-glass",
      6: "break-metal",
      7: "break-wood",
      8: "pickup-health",
      9: "pickup-armor",
      10: "pickup-ammo",
      11: "pickup-weapon",
      12: "hazard-ember",
      21: "empty",
      22: "ui-error",
      23: variant === 1 ? "casing-shell" : "casing-brass",
      13: variant === 3 ? "hazard-acid" : variant === 1 ? "hazard-electric" : "hit-stone",
    } as Record<number, string>
  )[kind];
  return id
    ? {
        id,
        gain: kind === 2 ? 0.75 : kind === 23 ? 0.24 : 0.65,
        rate: kind === 1 && variant === 1 ? 0.9 : 1,
        group: kind === 12 ? "hazard" : "world",
      }
    : null;
}
export class SfxDirector {
  private weapon: WeaponSoundState | null = null;
  private enemies = new Map<number, { skin: number; hp: number; anim: number; next: number }>();
  private phase = 0;
  pause() {
    this.weapon = null;
  }
  reset() {
    this.weapon = null;
    this.enemies.clear();
    this.phase = 0;
  }
  weaponFrame(h: WeaponSoundState): SoundIntent[] {
    const p = this.weapon;
    this.weapon = { ...h };
    if (h.state !== 0) return [];
    if (!p || h.wave !== p.wave) return [];
    if (h.weapon !== p.weapon) return [{ id: "equip", gain: 0.5, group: "handling" }];
    const result: SoundIntent[] = [];
    if (h.weapon === 1) {
      // Ammo is authoritative: one insert per actual shell, including the final one.
      if (p.reloading > 0 && h.ammo > p.ammo)
        result.push({ id: "shell", gain: 0.7, group: "reload" });
      if (p.reloading > 0 && h.reloading <= 0 && h.ammo > p.ammo)
        result.push({ id: "pump", gain: 0.65, group: "reload" });
      return result;
    }
    if (h.reloading <= 0) return [];
    const stage = (frame: number) => (frame <= 11 ? 0 : frame <= 12 ? 1 : 2);
    const now = stage(h.weapFrame),
      prev = p.reloading > 0 ? stage(p.weapFrame) : -1;
    // Catch stages crossed by a slow frame; never schedule future stages after an interruption.
    for (let n = prev + 1; n <= now; n++)
      result.push({
        id: weaponMechanism(h.weapon)[n]!,
        gain: 0.65,
        rate: 1 - (h.weapon % 4) * 0.025,
        group: "reload",
      });
    return result;
  }
  enemyFrame(cues: CombatSoundCue[], time: number): SoundIntent[] {
    const result: SoundIntent[] = [],
      present = new Set<number>();
    for (const e of cues) {
      const skin = enemyCombatSkin(e.skin);
      present.add(e.id);
      const old = this.enemies.get(e.id),
        p = old?.skin === e.skin ? old : undefined;
      const place = { x: e.x, y: e.y, sight: e.sight, rate: 0.96 + (e.skin % 5) * 0.02 };
      if (p && p.hp > 0 && e.hp <= 0) {
        result.push({ id: "body-thud", gain: 0.4, ...place, group: "enemy" });
        result.push({
          id: `death-${enemyFamily(e.skin)}${e.id % 3}`,
          gain: 0.55,
          ...place,
          group: "enemy",
        });
      } else if (e.hp > 0) {
        if (p && e.hp < p.hp && time >= p.next) {
          result.push({
            id: `pain-${enemyFamily(e.skin)}${e.id % 3}`,
            gain: 0.45,
            ...place,
            group: "enemy",
          });
          p.next = time + 0.3;
        }
        if (e.anim === 3 && (!p || p.anim !== 3))
          result.push({
            id: `fire${ENEMY_WEAPONS[enemyCombatSkin(e.skin)] ?? 2}`,
            gain: 0.45,
            ...place,
            group: "enemy",
          });
        if (e.anim === 6 && (!p || p.anim !== 6))
          result.push({
            id: `hazard-${skin >= 12 ? (WEAPON_HAZARDS[skin - 12] ?? "electric") : skin === 11 ? "ember" : "biological"}`,
            gain: 0.5,
            ...place,
            group: "hazard",
          });
        if (e.anim === 4 && p && p.anim !== 4)
          result.push({
            id: weaponMechanism(ENEMY_WEAPONS[enemyCombatSkin(e.skin)] ?? 2)[1],
            gain: 0.3,
            ...place,
            group: "enemy",
          });
      }
      this.enemies.set(e.id, { skin: e.skin, hp: e.hp, anim: e.anim, next: p?.next ?? 0 });
    }
    for (const id of this.enemies.keys()) if (!present.has(id)) this.enemies.delete(id);
    return result;
  }
  bossPhase(phase: number, skin: number, position?: { x: number; y: number }): SoundIntent[] {
    const previous = this.phase;
    this.phase = phase;
    return phase > previous && phase > 0
      ? [
          {
            id: `hazard-${WEAPON_HAZARDS[skin - 12] ?? "electric"}`,
            gain: 0.6,
            ...position,
            group: "hazard",
          },
        ]
      : [];
  }
}
