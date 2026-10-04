import { migrateGameStorage, readGameStorage } from "../../lib/storage-migration.ts";
import { DEFAULT_GFX, DEFAULT_RES, RES_MODES, type GfxOpts, type ResMode } from "../../game/types.ts";
import { WEAPON_SHEETS } from "../../game/weapon-assets.ts";

export type Screen = "menu" | "play" | "pause" | "dead" | "win";

export type Score = { name: string; wave: number; kills: number; time: number };

export type Vol = { master: number; music: number; sfx: number; menu: number };

export const DEFAULT_VOL: Vol = { master: 0.85, music: 0.42, sfx: 0.75, menu: 0.7 };

import { CAMPAIGN_EXPANSION } from "../../game/campaign25.ts";

export const SECTORS = [
  { code: "NADIR–7A", name: "UPPER WORKS", rewardSlot: 8, bossTitle: "VAULT MASTER", bossName: "MALIK VEYRAN" },
  { code: "NADIR–7B", name: "CRYOGENIC FOUNDRY", rewardSlot: 9, bossTitle: "FORGE WARDEN", bossName: "HECATE–9" },
  { code: "NADIR–7C", name: "BIOFORGE DEPTHS", rewardSlot: 10, bossTitle: "SPECIMEN PRIME", bossName: "CHIMERA–9" },
  { code: "NADIR–7D", name: "DATA SPINE", rewardSlot: 12, bossTitle: "PREDICTIVE CORE", bossName: "ORACLE–7" },
  { code: "NADIR–7E", name: "REACTOR SINK", rewardSlot: 13, bossTitle: "REACTOR WARDEN", bossName: "GRAVEMIND–4" },
  { code: "NADIR–7F", name: "NULL ARCHIVE", rewardSlot: 11, bossTitle: "MEMORY CUSTODIAN", bossName: "NULL ARCHIVIST" },
  { code: "NADIR–7G", name: "CRYO RESERVE", rewardSlot: 14, bossTitle: "COOLANT SENTINEL", bossName: "HALCYON–3" },
  { code: "NADIR–7H", name: "SIGNAL CRYPT", rewardSlot: 15, bossTitle: "TRANSMISSION GHOST", bossName: "RELAY–0" },
  { code: "NADIR–7I", name: "SIEGE YARD", rewardSlot: 16, bossTitle: "HEAVY ASSET", bossName: "TITAN–12" },
  { code: "NADIR–7J", name: "COMMAND BUNKER", rewardSlot: 17, bossTitle: "SITE DIRECTOR", bossName: "DIRECTOR KEST" },
  { code: "NADIR–7K", name: "OBSIDIAN VAULT", rewardSlot: 18, bossTitle: "ECHO KEEPER", bossName: "MNEMOSYNE–6" },
  ...CAMPAIGN_EXPANSION.map((boss, i) => ({ code: `NADIR–${i + 12}`, name: boss.sector, rewardSlot: i + 19, bossTitle: boss.ability.toUpperCase(), bossName: boss.boss })),
] as const;

export function sectorForWave(wave: number) {
  return SECTORS[(Math.max(1, wave || 1) - 1) % SECTORS.length]!;
}

/** Every boss alternates its death line on the next twenty-five-sector cycle. */
export function bossDeathVariantForWave(wave: number) {
  const index = Math.max(0, wave - 1);
  return (index % SECTORS.length + Math.floor(index / SECTORS.length)) % 2;
}

// v2 viewmodels: one 5x5 sheet per gun (25 cells of 512x384).
// Cells: 0 full, 1 half, 2 low, 3 empty, 4 no magazine, 5 dry fire,
// 6-9 pickup, 10-14 reload, 15-19 fire, 20-24 alt-fire (unused: no
// alt-fire mechanic ships, boss sheets carry them as spares).
// `reserve` mirrors the engine pickup caps for the ammo-state frames.
export const WEAPONS = [
  { id: 0, name: "MK23-S", role: "Suppressed precision · 12 rounds", magSize: 12, lowAmmoAt: 3, reserve: 120, sheet: WEAPON_SHEETS[0]! },
  { id: 1, name: "BR-12 BREAKER", role: "8-shot breacher · heavy stagger", magSize: 8, lowAmmoAt: 2, reserve: 48, sheet: WEAPON_SHEETS[1]! },
  { id: 2, name: "KX-9 VECTOR", role: "36-round PDW · controlled burst", magSize: 36, lowAmmoAt: 9, reserve: 216, sheet: WEAPON_SHEETS[2]! },
  { id: 3, name: "MR-4 LONGBOW", role: "Magnetic penetrator · pierces 3", magSize: 5, lowAmmoAt: 1, reserve: 20, sheet: WEAPON_SHEETS[3]! },
  { id: 4, name: "VLK-6 WARDEN", role: "Guided micro-missile · blast radius", magSize: 4, lowAmmoAt: 1, reserve: 16, sheet: WEAPON_SHEETS[4]! },
  { id: 5, name: "AX-12 VOLT", role: "Electrical carbine · precision shock", magSize: 10, lowAmmoAt: 2, reserve: 80, sheet: WEAPON_SHEETS[5]! },
  { id: 6, name: "M91 CYCLONE", role: "Rotary cannon · sustained suppression", magSize: 90, lowAmmoAt: 22, reserve: 450, sheet: WEAPON_SHEETS[6]! },
  { id: 7, name: "HX-8 PYRE", role: "Incendiary projector · leaves a burn", magSize: 6, lowAmmoAt: 2, reserve: 36, sheet: WEAPON_SHEETS[7]! },
  { id: 8, name: "VR-9 OVERRIDE", role: "Veyran rail · pierces the lane, then bursts", magSize: 4, lowAmmoAt: 1, reserve: 24, sheet: WEAPON_SHEETS[8]! },
  { id: 9, name: "HC-9 FORGE", role: "HECATE cutter · wide beam and impact splash", magSize: 14, lowAmmoAt: 3, reserve: 84, sheet: WEAPON_SHEETS[9]! },
  { id: 10, name: "CM-9 CHIMERA", role: "Specimen fan · acid bursts and a short pool", magSize: 5, lowAmmoAt: 1, reserve: 30, sheet: WEAPON_SHEETS[10]! },
  { id: 11, name: "AR-6 ARCHIVE", role: "Archivist rail · precise amber pulse", magSize: 9, lowAmmoAt: 2, reserve: 54, sheet: WEAPON_SHEETS[11]! },
  { id: 12, name: "OR-7 PREDICTOR", role: "Oracle rail · accurate double pulse", magSize: 4, lowAmmoAt: 1, reserve: 24, sheet: WEAPON_SHEETS[12]! },
  { id: 13, name: "GS-4 SINK", role: "Reactor scatter · dense pressure burst", magSize: 8, lowAmmoAt: 2, reserve: 48, sheet: WEAPON_SHEETS[13]! },
  { id: 14, name: "CR-3 RIME", role: "Coolant launcher · three slowing slugs", magSize: 6, lowAmmoAt: 2, reserve: 36, sheet: WEAPON_SHEETS[14]! },
  { id: 15, name: "SR-0 RELAY", role: "Signal carbine · rapid narrow burst", magSize: 24, lowAmmoAt: 6, reserve: 144, sheet: WEAPON_SHEETS[15]! },
  { id: 16, name: "TS-12 TITAN", role: "Siege cannon · heavy explosive shell", magSize: 3, lowAmmoAt: 1, reserve: 18, sheet: WEAPON_SHEETS[16]! },
  { id: 17, name: "KS-8 KEST", role: "Director rifle · precise triple burst", magSize: 30, lowAmmoAt: 8, reserve: 180, sheet: WEAPON_SHEETS[17]! },
  { id: 18, name: "MN-6 ECHO", role: "Memory lance · pierces three targets", magSize: 12, lowAmmoAt: 3, reserve: 72, sheet: WEAPON_SHEETS[18]! },
  ...CAMPAIGN_EXPANSION.map((boss, i) => ({ id: i + 19, name: boss.gun, role: boss.ability, magSize: boss.mag, lowAmmoAt: Math.max(1, Math.floor(boss.mag * .25)), reserve: boss.mag * 6, sheet: WEAPON_SHEETS[i + 19]! })),
];

export function bossRewardForWave(wave: number) {
  return WEAPONS[sectorForWave(wave).rewardSlot]!;
}

const HANDLER = [
  "Ward. Restore the lab node before the vault console will answer.",
  "Coolant node is still live. Vent it, or the foundry cooks you with the warden.",
  "Specimen lock first. Containment will not open until the ring is purged.",
  "The data spine is feeding Oracle. Cut its prediction node before the core override.",
  "Reactor sink ahead. Isolate the coolant relay before you challenge the warden.",
  "The Null Archive is sealing its memory stacks. Cut the index node before the custodian wakes.",
  "Cryo Reserve is above pressure. Isolate the coolant manifold before HALCYON comes online.",
  "The Signal Crypt is broadcasting your position. Kill its relay node before RELAY finds you.",
  "The Siege Yard still has a live hydraulic spine. Shut it down before TITAN wakes.",
  "Director Kest is locking Command. Cut the command uplink and open the bunker.",
  "Cut the memory bus before MNEMOSYNE wakes. Dodge its echo mines and pulse rings.",
] as const;

const LOCKDOWN = [
  "MALIK: Vault doors sealed. You are the remaining variable.",
  "MALIK: Forge circuit closed. The floor is now part of the weapon.",
  "MALIK: Containment sealed. The specimen is authorized to finish this.",
  "ORACLE–7: Your route has already been calculated. Core doors sealed.",
  "GRAVEMIND–4: Reactor doors sealed. This place dies with me.",
  "NULL ARCHIVIST: Memory vault closed. Your route has been erased.",
  "HALCYON–3: Reserve sealed. Coolant pressure rising.",
  "RELAY–0: Transmission contained. You have nowhere left to hide.",
  "TITAN–12: Siege gate sealed. Structural load accepted.",
  "DIRECTOR KEST: Command doors closed. No extraction for you.",
  "MNEMOSYNE–6: Your memory belongs to the vault. Echo sequence armed.",
] as const;

const NODE_DONE = [
  "Power restored. The vault will take a USE now.",
  "Coolant dumped. Override is armed.",
  "Specimen lock purged. Containment will answer.",
  "Prediction node offline. Oracle can no longer close the route.",
  "Coolant relay isolated. The reactor override is live.",
  "Index node severed. The memory vault override is live.",
  "Coolant manifold isolated. Cryo override is live.",
  "Signal relay cut. The crypt override is live.",
  "Hydraulic spine offline. The siege gate override is live.",
  "Command uplink severed. The bunker override is live.",
  "Memory bus severed. The vault override is live.",
] as const;

const MEMOS: Record<number, string> = {
  10: "Memo 14-C: the coffee machine needs three signatures. The weapons program needs none.",
  11: "Handler: if the posters start agreeing with each other, leave.",
  14: "Forge log: HECATE-9 requested more drones. The request was approved by HECATE-9.",
  15: "Procurement: do not stand on the electrical floor. The floor was not informed.",
  18: "Lab note: subject 9 asked to be filed as equipment. Request pending.",
  19: "MALIK: biological leadership remains an operational vulnerability.",
  22: "Archive: Oracle compares every intrusion to the routes that failed before it.",
  23: "Operator note: unplugging the racks did not stop the core from answering.",
  26: "Reactor log: the warden locked himself inside before the containment alarm.",
  27: "Handler: pressure is climbing. Keep the coolant corridor clear.",
  30: "Archive memo: records marked deleted remain on the other side of the wall.",
  31: "Custodian log: the last human operator signed out thirty years ago.",
  34: "Cryo log: coolant pressure was used to suppress the whole reserve shift.",
  35: "Maintenance: HALCYON's thaw sequence is longer than its targeting cycle.",
  38: "Signal log: RELAY broadcasts on frequencies that are not in the manual.",
  39: "Handler: the crypt keeps replaying voices of people who never entered it.",
  42: "Siege record: TITAN was parked with its cannon still loaded.",
  43: "Hydraulics inspection failed. Inspector reassigned to hydraulics.",
  46: "Command memo: Kest approved every containment loss personally.",
  47: "Handler: the bunker leads to the memory vault. Keep moving.",
  50: "Vault log: MNEMOSYNE preserves every failed intrusion as a training echo.",
  51: "Handler: those mines are moving memories. Keep your distance when they charge.",
};

export function radioCopy(line: number, wave: number) {
  const sector = (Math.max(1, wave) - 1) % SECTORS.length;
  if (sector >= 11) {
    const boss = CAMPAIGN_EXPANSION[sector - 11]!;
    if (line === 1) return { speaker: "HANDLER", text: `${boss.sector}. ${boss.boss} controls this sector. Disable the node and initiate the override.` };
    if (line === 2) return { speaker: boss.boss, text: `${boss.ability}. You have entered my sector.` };
    if (line === 3) return { speaker: boss.boss, text: "Armor circuit exposed. Recalibrating defenses." };
    if (line === 4) return { speaker: "HANDLER", text: "Node isolated. The override console is live." };
    if (line === 6) return { speaker: "HANDLER", text: `${boss.gun} is on the deck. Claim the golden case to leave.` };
    if (line === 8) return { speaker: boss.boss, text: bossDeathVariantForWave(wave) ? `My systems are gone. Take ${boss.gun}. Carry it beyond this sector.` : `Core failure. ${boss.gun} released. The sector is yours.` };
    return line >= 10 ? { speaker: "ARCHIVE", text: `${boss.sector}: ${boss.ability}. Keep cover between you and the firing lanes.` } : null;
  }
  if (line === 1) return { speaker: "HANDLER", text: HANDLER[sector]! };
  if (line === 2) return { speaker: "MALIK", text: LOCKDOWN[sector]! };
  if (line === 3) return {
    speaker: ["MALIK", "HECATE–9", "CHIMERA–9", "ORACLE–7", "GRAVEMIND–4", "NULL ARCHIVIST", "HALCYON–3", "RELAY–0", "TITAN–12", "DIRECTOR KEST", "MNEMOSYNE–6"][sector]!,
    text: [
      "Command surface exposed. That window will not last.",
      "Shield circuit open. Discharge imminent.",
      "Containment breached. The specimen is vulnerable.",
      "Prediction buffer severed. I cannot see your next move.",
      "Containment pressure falling. My armor is open.",
      "Shield memory fractured. This record will not survive.",
      "Coolant shield fractured. The cold will not hold.",
      "Carrier lost. Signal lattice exposed.",
      "Hydraulic plate open. Cannon pressure falling.",
      "Armor breached. The chain of command ends here.",
      "Core exposed. I cannot preserve this memory.",
    ][sector]!,
  };
  if (line === 4) return { speaker: "HANDLER", text: NODE_DONE[sector]! };
  if (line === 6) {
    return {
      speaker: "HANDLER",
      text: [
        "Veyran dropped the Override rail. Take it, and the sector opens.",
        "HECATE's forge cutter is on the deck. That is the way out.",
        "The specimen case is Chimera's sprayer. Pick it up to leave.",
        "Oracle's Predictor rail is down. Take the case and clear the spine.",
        "The warden dropped a Sink cannon. Take it and leave the reactor.",
        "The Archivist dropped an Archive rail. Take it and clear the vault.",
        "HALCYON dropped a Rime launcher. Claim the case and leave the reserve.",
        "RELAY dropped a signal carbine. Claim the case and clear the crypt.",
        "TITAN dropped a siege cannon. Claim the case and clear the yard.",
        "Kest dropped her rifle. Claim it and leave Command.",
        "MNEMOSYNE dropped the Echo lance. Claim it and leave the vault.",
      ][sector]!,
    };
  }
  if (line === 8) {
    // Speaker and copy mirror the two variants in art/boss-voices.json.
    const first = [
      { speaker: "MALIK", text: "My vault... my blood. Take the rail, intruder. Let it remember who built this place." },
      { speaker: "HECATE–9", text: "Core failure. Warden protocol terminated. Forge cutter released. Do not let it cool." },
      { speaker: "CHIMERA–9", text: "You broke the cage. The toxin is yours now. Breathe carefully." },
      { speaker: "ORACLE–7", text: "Prediction failed. The Predictor rail is yours." },
      { speaker: "GRAVEMIND–4", text: "Containment... lost. Take the Sink cannon. Seal the reactor." },
      { speaker: "NULL ARCHIVIST", text: "Archive integrity lost. The rail is yours. Do not write me back." },
      { speaker: "HALCYON–3", text: "Coolant pressure collapsing. The Rime launcher is yours. Do not let it thaw." },
      { speaker: "RELAY–0", text: "Signal lost. The Relay carbine is unbound." },
      { speaker: "TITAN–12", text: "Hydraulics failed. Siege cannon released. The gate is yours." },
      { speaker: "DIRECTOR KEST", text: "You have cut the chain of command. The Kest rifle is yours." },
      { speaker: "MNEMOSYNE–6", text: "Memory core erased. Take the Echo lance. Let the vault forget." },
    ];
    const second = [
      { speaker: "MALIK", text: "My vault is yours. I buried the truth beneath that rail. Do not let them seal it again." },
      { speaker: "HECATE–9", text: "Safety locks have failed. My forge cutter is yours. Keep clear of the discharge." },
      { speaker: "CHIMERA–9", text: "Containment was never meant to save you. The compound still lives in the sprayer." },
      { speaker: "ORACLE–7", text: "I saw every outcome except this one. The Predictor rifle is yours." },
      { speaker: "GRAVEMIND–4", text: "Pressure gone. Take the Sink cannon. Keep the core from swallowing you." },
      { speaker: "NULL ARCHIVIST", text: "Last record: I was here. Take the Archive rail and carry it out." },
      { speaker: "HALCYON–3", text: "Containment ice gone. Keep the cold between you and the next chamber." },
      { speaker: "RELAY–0", text: "No carrier. No command. Take my transmitter and make your own route." },
      { speaker: "TITAN–12", text: "Load-bearing systems gone. Take the Titan cannon before the ceiling follows." },
      { speaker: "DIRECTOR KEST", text: "I built this place to outlast us. Take the rifle. Prove me wrong." },
      { speaker: "MNEMOSYNE–6", text: "This is my final memory. The Echo lance is yours. Carry it beyond these walls." },
    ];
    return (bossDeathVariantForWave(wave) === 0 ? first : second)[sector]!;
  }
  const memo = MEMOS[line];
  if (memo) return { speaker: line >= 18 ? "LAB" : line >= 14 ? "FORGE" : "ARCHIVE", text: memo };
  return null;
}

export function missionLine(hud: { prompt: number; objective: number; wave: number }) {
  const sector = (Math.max(1, hud.wave) - 1) % SECTORS.length;
  if (sector >= 11) {
    if (hud.prompt === 3) return "INITIATE OVERRIDE";
    if (hud.prompt === 13 || hud.prompt === 15 || hud.objective === 0) return "ISOLATE THE SECTOR NODE";
    if (hud.prompt === 6) return "REACH THE OVERRIDE";
    return `ELIMINATE ${SECTORS[sector]!.bossName}`;
  }
  if (hud.prompt === 3) return "INITIATE OVERRIDE";
  if (hud.prompt === 13) return ["USE THE LAB NODE", "VENT THE COOLANT", "PURGE THE LOCK", "CUT THE PREDICTION NODE", "ISOLATE THE COOLANT RELAY", "SEVER THE INDEX NODE", "ISOLATE THE MANIFOLD", "CUT THE SIGNAL RELAY", "SHUT DOWN HYDRAULICS", "CUT THE COMMAND UPLINK", "SEVER THE MEMORY BUS"][sector]!;
  if (hud.prompt === 15 || hud.objective === 0) {
    return ["RESTORE THE LAB NODE", "VENT THE COOLANT NODE", "PURGE THE SPECIMEN LOCK", "CUT THE PREDICTION NODE", "ISOLATE THE COOLANT RELAY", "SEVER THE INDEX NODE", "ISOLATE THE MANIFOLD", "CUT THE SIGNAL RELAY", "SHUT DOWN HYDRAULICS", "CUT THE COMMAND UPLINK", "SEVER THE MEMORY BUS"][sector]!;
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
    const raw = readGameStorage("blacksite-vol");
    if (!raw) return { ...DEFAULT_VOL };
    const v = JSON.parse(raw) as Partial<Vol>;
    return {
      master: v.master ?? DEFAULT_VOL.master,
      music: v.music ?? DEFAULT_VOL.music,
      sfx: v.sfx ?? DEFAULT_VOL.sfx,
      menu: v.menu ?? DEFAULT_VOL.menu,
    };
  } catch {
    return { ...DEFAULT_VOL };
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
    const raw = readGameStorage("blacksite-board");
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
    localStorage.setItem("blacksite-board", JSON.stringify(next));
  } catch {
    /* ignore */
  }
  return next;
}

export function gpuEnabled() {
  try {
    return readGameStorage("blacksite-gpu") === "1";
  } catch {
    return false;
  }
}

export function loadRes(): ResMode {
  try {
    const id = readGameStorage("blacksite-res");
    const hit = RES_MODES.find((r) => r.id === id);
    if (hit) return hit;
  } catch {
    /* ignore */
  }
  return DEFAULT_RES;
}

export function loadGfx(): GfxOpts {
  try {
    const raw = readGameStorage("blacksite-gfx");
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
  extraWeapons?: number;
  kills: number;
  secrets: number;
  elapsedMs: number;
  ammo: number[];
  mag: number[];
};

const CHECK_KEY = "blacksite-checkpoint";

export function loadCheckpoint(): RunSave | null {
  try {
    const raw = readGameStorage(CHECK_KEY);
    if (!raw) return null;
    const v = JSON.parse(raw) as Partial<RunSave>;
    if (!v || !Array.isArray(v.ammo) || !Array.isArray(v.mag)) {
      return null;
    }
    // Old 8- and 11-slot checkpoints remain loadable after adding AR-6.
    if (![8, 11, 12, 18, 19, 33].includes(v.ammo.length)) return null;
    if (![8, 11, 12, 18, 19, 33].includes(v.mag.length)) return null;
    const padSlots = (a: number[]) => [...a, ...Array(33).fill(0)].slice(0, 33);
    return {
      wave: Math.min(2147483647, Math.max(1, Math.floor(v.wave || 1))),
      health: v.health || 100,
      armor: Math.min(100, Math.max(0, v.armor || 0)),
      weapon: v.weapon || 0,
      flags: v.flags || 0,
      extraWeapons: (v.extraWeapons || 0) & 16383,
      kills: v.kills || 0,
      secrets: v.secrets || 0,
      elapsedMs: v.elapsedMs || 0,
      ammo: padSlots(v.ammo),
      mag: padSlots(v.mag),
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
    migrateGameStorage(localStorage);
    localStorage.removeItem(CHECK_KEY);
  } catch {
    /* ignore */
  }
}
