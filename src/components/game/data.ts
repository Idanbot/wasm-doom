import { DEFAULT_GFX, DEFAULT_RES, RES_MODES, type GfxOpts, type ResMode } from "../../game/types.ts";

export type Screen = "menu" | "play" | "pause" | "dead" | "win";

export type Score = { name: string; wave: number; kills: number; time: number };

export type Vol = { master: number; music: number; sfx: number };

export const SECTORS = [
  { code: "NADIR–7A", name: "UPPER WORKS", bossTitle: "VAULT MASTER", bossName: "MALIK VEYRAN" },
  { code: "NADIR–7B", name: "CRYOGENIC FOUNDRY", bossTitle: "FORGE WARDEN", bossName: "HECATE–9" },
  { code: "NADIR–7C", name: "BIOFORGE DEPTHS", bossTitle: "SPECIMEN PRIME", bossName: "CHIMERA–9" },
] as const;

export function sectorForWave(wave: number) {
  return SECTORS[(Math.max(1, wave || 1) - 1) % SECTORS.length]!;
}

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
  {
    id: 7,
    name: "HX-8 PYRE",
    role: "Incendiary projector · leaves a burn",
    magSize: 6,
    lowAmmoAt: 2,
    idle: "/game/weap_hx8.png",
    fire: "/game/weap_hx8_fire.png",
    reload: "/game/weap_hx8_reload.png",
  },
  {
    id: 8,
    name: "VR-9 OVERRIDE",
    role: "Veyran rail · pierces the lane, then bursts",
    magSize: 4,
    lowAmmoAt: 1,
    idle: "/game/weap_vr9.png",
    fire: "/game/weap_vr9_fire.png",
    reload: "/game/weap_vr9_reload.png",
  },
  {
    id: 9,
    name: "HC-9 FORGE",
    role: "HECATE cutter · wide beam and impact splash",
    magSize: 14,
    lowAmmoAt: 3,
    idle: "/game/weap_hc9.png",
    fire: "/game/weap_hc9_fire.png",
    reload: "/game/weap_hc9_reload.png",
  },
  {
    id: 10,
    name: "CM-9 CHIMERA",
    role: "Specimen fan · acid bursts and a short pool",
    magSize: 5,
    lowAmmoAt: 1,
    idle: "/game/weap_cm9.png",
    fire: "/game/weap_cm9_fire.png",
    reload: "/game/weap_cm9_reload.png",
  },
];

const HANDLER = [
  "Ward. Restore the lab node before the vault console will answer.",
  "Coolant node is still live. Vent it, or the foundry cooks you with the warden.",
  "Specimen lock first. Containment will not open until the ring is purged.",
] as const;

const LOCKDOWN = [
  "MALIK: Vault doors sealed. You are the remaining variable.",
  "MALIK: Forge circuit closed. The floor is now part of the weapon.",
  "MALIK: Containment sealed. The specimen is authorized to finish this.",
] as const;

const NODE_DONE = [
  "Power restored. The vault will take a USE now.",
  "Coolant dumped. Override is armed.",
  "Specimen lock purged. Containment will answer.",
] as const;

const MEMOS: Record<number, string> = {
  10: "Memo 14-C: the coffee machine needs three signatures. The weapons program needs none.",
  11: "Handler: if the posters start agreeing with each other, leave.",
  14: "Forge log: HECATE-9 requested more drones. The request was approved by HECATE-9.",
  15: "Procurement: do not stand on the electrical floor. The floor was not informed.",
  18: "Lab note: subject 9 asked to be filed as equipment. Request pending.",
  19: "MALIK: biological leadership remains an operational vulnerability.",
};

export function radioCopy(line: number, wave: number) {
  const sector = (Math.max(1, wave) - 1) % 3;
  if (line === 1) return { speaker: "HANDLER", text: HANDLER[sector]! };
  if (line === 2) return { speaker: "MALIK", text: LOCKDOWN[sector]! };
  if (line === 3) return { speaker: "MALIK", text: "Command surface exposed. That window will not last." };
  if (line === 4) return { speaker: "HANDLER", text: NODE_DONE[sector]! };
  if (line === 5) return { speaker: "HANDLER", text: "Cache behind the panel. Someone signed for it twice." };
  if (line === 6) {
    return {
      speaker: "HANDLER",
      text: [
        "Veyran dropped the Override rail. Take it, and the sector opens.",
        "HECATE's forge cutter is on the deck. That is the way out.",
        "The specimen case is Chimera's sprayer. Pick it up to leave.",
      ][sector]!,
    };
  }
  const memo = MEMOS[line];
  if (memo) return { speaker: line >= 18 ? "LAB" : line >= 14 ? "FORGE" : "ARCHIVE", text: memo };
  return null;
}

export function missionLine(hud: { prompt: number; objective: number; wave: number }) {
  const sector = (Math.max(1, hud.wave) - 1) % 3;
  if (hud.prompt === 3) return "INITIATE OVERRIDE";
  if (hud.prompt === 13) return ["USE THE LAB NODE", "VENT THE COOLANT", "PURGE THE LOCK"][sector]!;
  if (hud.prompt === 15 || hud.objective === 0) {
    return ["RESTORE THE LAB NODE", "VENT THE COOLANT NODE", "PURGE THE SPECIMEN LOCK"][sector]!;
  }
  if (hud.prompt === 6) return "REACH THE OVERRIDE";
  return "ELIMINATE THE SIGNAL";
}

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
    const id = localStorage.getItem("blacksite-res");
    const hit = RES_MODES.find((r) => r.id === id);
    if (hit) return hit;
  } catch {
    /* ignore */
  }
  return DEFAULT_RES;
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

export type RunSave = {
  wave: number;
  health: number;
  armor: number;
  weapon: number;
  flags: number;
  kills: number;
  secrets: number;
  elapsedMs: number;
  ammo: number[];
  mag: number[];
};

const CHECK_KEY = "hellscan-checkpoint";

export function loadCheckpoint(): RunSave | null {
  try {
    const raw = localStorage.getItem(CHECK_KEY);
    if (!raw) return null;
    const v = JSON.parse(raw) as Partial<RunSave>;
    if (!v || !Array.isArray(v.ammo) || v.ammo.length !== 8 || !Array.isArray(v.mag) || v.mag.length !== 8) {
      return null;
    }
    return {
      wave: Math.min(12, Math.max(1, v.wave || 1)),
      health: v.health || 100,
      armor: v.armor || 0,
      weapon: v.weapon || 0,
      flags: v.flags || 0,
      kills: v.kills || 0,
      secrets: v.secrets || 0,
      elapsedMs: v.elapsedMs || 0,
      ammo: v.ammo,
      mag: v.mag,
    };
  } catch {
    return null;
  }
}

export function saveCheckpoint(save: RunSave) {
  try {
    localStorage.setItem(CHECK_KEY, JSON.stringify(save));
  } catch {
    /* ignore */
  }
}

export function clearCheckpoint() {
  try {
    localStorage.removeItem(CHECK_KEY);
  } catch {
    /* ignore */
  }
}
