import { DEFAULT_GFX, RES_MODES, type GfxOpts, type ResMode } from "../../game/types.ts";

export type Screen = "menu" | "play" | "pause" | "dead" | "win";

export type Score = { name: string; wave: number; kills: number; time: number };

export type Vol = { master: number; music: number; sfx: number };

export const WEAPONS = [
  {
    id: 0,
    name: "MK23-S",
    role: "Suppressed precision · 12 rounds",
    magSize: 12,
    lowAmmoAt: 3,
    idle: "/game/weap_mk23s.png",
    fire: "/game/weap_mk23s_fire.png",
    reload: "/game/weap_mk23s_reload.png",
  },
  {
    id: 1,
    name: "BR-12 BREAKER",
    role: "8-shot breacher · heavy stagger",
    magSize: 8,
    lowAmmoAt: 2,
    idle: "/game/weap_br12.png",
    fire: "/game/weap_br12_fire.png",
    reload: "/game/weap_br12_reload.png",
  },
  {
    id: 2,
    name: "KX-9 VECTOR",
    role: "36-round PDW · controlled burst",
    magSize: 36,
    lowAmmoAt: 9,
    idle: "/game/weap_kx9.png",
    fire: "/game/weap_kx9_fire.png",
    reload: "/game/weap_kx9_reload.png",
  },
  {
    id: 3,
    name: "MR-4 LONGBOW",
    role: "Magnetic penetrator · pierces 3",
    magSize: 5,
    lowAmmoAt: 1,
    idle: "/game/weap_mr4.png",
    fire: "/game/weap_mr4_fire.png",
    reload: "/game/weap_mr4_reload.png",
  },
  {
    id: 4,
    name: "VLK-6 WARDEN",
    role: "Guided micro-missile · blast radius",
    magSize: 4,
    lowAmmoAt: 1,
    idle: "/game/weap_vlk6.png",
    fire: "/game/weap_vlk6_fire.png",
    reload: "/game/weap_vlk6_reload.png",
  },
  {
    id: 5,
    name: "AX-12 VOLT",
    role: "Electrical carbine · precision shock",
    magSize: 10,
    lowAmmoAt: 2,
    idle: "/game/weap_ax12.png",
    fire: "/game/weap_ax12_fire.png",
    reload: "/game/weap_ax12_reload.png",
  },
  {
    id: 6,
    name: "M91 CYCLONE",
    role: "Rotary cannon · sustained suppression",
    magSize: 90,
    lowAmmoAt: 22,
    idle: "/game/weap_m91.png",
    fire: "/game/weap_m91_fire.png",
    reload: "/game/weap_m91_reload.png",
  },
];

export function sheetPos(cell: number) {
  return gridPos(cell, 2, 2);
}

export function gridPos(cell: number, columns: number, rows: number) {
  const x = columns <= 1 ? 0 : ((cell % columns) * 100) / (columns - 1);
  const y = rows <= 1 ? 0 : (Math.floor(cell / columns) * 100) / (rows - 1);
  return `${x}% ${y}%`;
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
      return {
        crt: v.crt ?? DEFAULT_GFX.crt,
        bloom: v.bloom ?? DEFAULT_GFX.bloom,
        fog: v.fog ?? DEFAULT_GFX.fog,
      };
    }
  } catch {
    /* ignore */
  }
  return { ...DEFAULT_GFX };
}
