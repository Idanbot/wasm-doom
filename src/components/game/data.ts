import { DEFAULT_GFX, RES_MODES, type GfxOpts, type ResMode } from "../../game/types.ts";

export type Screen = "menu" | "play" | "pause" | "dead" | "win";

export type Score = { name: string; wave: number; kills: number; time: number };

export type Vol = { master: number; music: number; sfx: number };

export const WEAPONS = [
  { id: 0, name: "Sidearm", role: "Precision · conserve ammo", idle: "/game/weap_pistol.png", fire: "/game/weap_pistol_fire.png", reload: "/game/weap_pistol_reload.png" },
  { id: 1, name: "Scattergun", role: "Close range · stagger", idle: "/game/weap_shotgun.png", fire: "/game/weap_shotgun_fire.png", reload: "/game/weap_shotgun_reload.png" },
  { id: 2, name: "Ripper", role: "Short bursts · control spread", idle: "/game/weap_ripper.png", fire: "/game/weap_ripper_fire.png", reload: "/game/weap_ripper_reload.png" },
  { id: 3, name: "Lance", role: "Pierces 3 · line them up", idle: "/game/weap_lance.png", fire: "/game/weap_lance_fire.png", reload: "/game/weap_lance_reload.png" },
  { id: 4, name: "Pyre", role: "Lingering fire · hold doors", idle: "/game/weap_pyre.png", fire: "/game/weap_pyre_fire.png", reload: "/game/weap_pyre_reload.png" },
];

export function sheetPos(cell: number) {
  return `${(cell & 1) * 100}% ${((cell >> 1) & 1) * 100}%`;
}

export function loadVol(): Vol {
  try {
    const raw = localStorage.getItem("hellscan-vol");
    if (!raw) return { master: 0.85, music: 0.42, sfx: 0.75 };
    const v = JSON.parse(raw) as Partial<Vol>;
    return { master: v.master ?? 0.85, music: v.music ?? 0.42, sfx: v.sfx ?? 0.75 };
  } catch {
    return { master: 0.85, music: 0.42, sfx: 0.75 };
  }
}

export function fmtTime(ms: number) {
  const s = Math.max(0, Math.floor(ms / 1000));
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${m}:${r.toString().padStart(2, "0")}`;
}

export function loadBoard(): Score[] {
  try {
    const raw = localStorage.getItem("hellscan-board");
    if (raw) {
      const v = JSON.parse(raw) as Score[];
      if (Array.isArray(v)) {
        return v
          .map((r) => ({
            name: (r.name || "MARINE").slice(0, 12),
            wave: r.wave || 1,
            kills: r.kills || 0,
            time: r.time || 0,
          }))
          .slice(0, 10);
      }
    }
  } catch {
    /* ignore */
  }
  return [];
}

export function saveBoard(entry: Score): Score[] {
  const next = [...loadBoard(), entry]
    .sort((a, b) => b.wave - a.wave || b.kills - a.kills || a.time - b.time)
    .slice(0, 10);
  try {
    localStorage.setItem("hellscan-board", JSON.stringify(next));
  } catch {
    /* ignore */
  }
  return next;
}

export function gpuEnabled() {
  try {
    return localStorage.getItem("hellscan-gpu") === "1";
  } catch {
    return false;
  }
}

export function loadRes(): ResMode {
  try {
    const id = localStorage.getItem("hellscan-res");
    const hit = RES_MODES.find((r) => r.id === id);
    if (hit) return hit;
  } catch {
    /* ignore */
  }
  return RES_MODES[1]!;
}

export function loadGfx(): GfxOpts {
  try {
    const raw = localStorage.getItem("hellscan-gfx");
    if (raw) {
      const v = JSON.parse(raw) as Partial<GfxOpts>;
      return { crt: v.crt ?? DEFAULT_GFX.crt, bloom: v.bloom ?? DEFAULT_GFX.bloom, fog: v.fog ?? DEFAULT_GFX.fog };
    }
  } catch {
    /* ignore */
  }
  return { ...DEFAULT_GFX };
}
