import { describe, it, beforeEach } from "node:test";
import assert from "node:assert/strict";
import {
  machineryHint,
  fmtTime,
  gpuEnabled,
  loadBoard,
  loadGfx,
  loadCheckpoint,
  loadRes,
  loadVol,
  radioCopy,
  saveBoard,
  sheetPos,
  WEAPONS,
  gridPos,
  sectorForWave,
  bossRewardForWave,
  bossDeathVariantForWave,
} from "./data.ts";
import { DEFAULT_GFX, RES_MODES } from "../../game/types.ts";
import { WEAPON_SHEETS, WEAPON_THUMBNAILS } from "../../game/weapon-assets.ts";

function installMemoryStorage(initial: Record<string, string> = {}) {
  const store = new Map<string, string>(Object.entries(initial));
  const stub = {
    getItem: (k: string) => (store.has(k) ? store.get(k)! : null),
    setItem: (k: string, v: string) => {
      store.set(k, String(v));
    },
    removeItem: (k: string) => {
      store.delete(k);
    },
    clear: () => store.clear(),
    _store: store,
  };
  Object.defineProperty(globalThis, "localStorage", { value: stub, configurable: true });
  return stub;
}

beforeEach(() => {
  installMemoryStorage();
});

describe("fmtTime", () => {
  it("formats seconds as m:ss", () => {
    assert.equal(fmtTime(0), "0:00");
    assert.equal(fmtTime(5_000), "0:05");
    assert.equal(fmtTime(65_000), "1:05");
    assert.equal(fmtTime(600_000), "10:00");
  });

  it("clamps negative input to zero", () => {
    assert.equal(fmtTime(-1_000), "0:00");
  });
});

describe("sectorForWave", () => {
  it("cycles through twenty-five distinct sectors and bosses", () => {
    assert.deepEqual(Array.from({ length: 11 }, (_, i) => sectorForWave(i + 1).name), [
      "UPPER WORKS",
      "CRYOGENIC FOUNDRY",
      "BIOFORGE DEPTHS",
      "DATA SPINE",
      "REACTOR SINK",
      "NULL ARCHIVE",
      "CRYO RESERVE",
      "SIGNAL CRYPT",
      "SIEGE YARD",
      "COMMAND BUNKER",
      "OBSIDIAN VAULT",
    ]);
    assert.equal(new Set(Array.from({ length: 25 }, (_, i) => sectorForWave(i + 1).bossName)).size, 25);
    assert.equal(sectorForWave(26).name, "UPPER WORKS");
    assert.equal(sectorForWave(51).name, "UPPER WORKS");
  });
});

describe("radioCopy", () => {
  it("has two matching death lines for every boss across cycles", () => {
    const lines = Array.from({ length: 50 }, (_, i) => radioCopy(8, i + 1));
    for (const line of lines) {
      assert.ok(line && line.speaker.length > 0 && line.text.length > 0);
    }
    assert.equal(new Set(lines.map((l) => l!.text)).size, 50);
    assert.equal(bossDeathVariantForWave(1), 0);
    assert.equal(bossDeathVariantForWave(26), 1);
  });
});

describe("sheetPos", () => {
  it("maps frame cells to 2x2 background positions", () => {
    assert.equal(sheetPos(0), "0% 0%");
    assert.equal(sheetPos(1), "100% 0%");
    assert.equal(sheetPos(2), "0% 100%");
    assert.equal(sheetPos(3), "100% 100%");
  });
});

describe("gridPos", () => {
  it("addresses all eight cells in a four-by-two reload sheet", () => {
    assert.equal(gridPos(0, 4, 2), "0% 0%");
    assert.equal(gridPos(3, 4, 2), "100% 0%");
    assert.equal(gridPos(4, 4, 2), "0% 100%");
    assert.equal(gridPos(7, 4, 2), "100% 100%");
  });
});

describe("WEAPONS", () => {
  it("has thirty-three entries with v2 5x5 sheets and reserve caps", () => {
    assert.equal(WEAPONS.length, 33);
    assert.deepEqual(
      WEAPONS.map((w) => w.id),
      Array.from({ length: 33 }, (_, i) => i),
    );
    assert.deepEqual(WEAPONS.map((weapon) => weapon.sheet), WEAPON_SHEETS);
    assert.equal(WEAPON_THUMBNAILS.length, WEAPONS.length);
    for (const w of WEAPONS) {
      assert.ok(w.name.length > 0);
      assert.match(w.sheet, /^\/game\/draft\/v2\/weap_.+_5x5\.png$/);
      assert.ok(w.magSize > 0);
      assert.ok(w.lowAmmoAt > 0 && w.lowAmmoAt < w.magSize);
      assert.ok(w.reserve >= w.magSize);
    }
    assert.deepEqual(
      WEAPONS.slice(0,19).map((w) => w.magSize),
      [12, 8, 36, 5, 4, 10, 90, 6, 4, 14, 5, 9, 4, 8, 6, 24, 3, 30, 12],
    );
    assert.deepEqual(
      WEAPONS.slice(0,19).map((w) => w.reserve),
      [120, 48, 216, 20, 16, 80, 450, 36, 24, 84, 30, 54, 24, 48, 36, 144, 18, 180, 72],
    );
  });
});

describe("loadVol", () => {
  it("returns defaults when nothing is stored", () => {
    assert.deepEqual(loadVol(), { master: 0.85, music: 0.42, sfx: 0.75, menu: 0.7 });
  });

  it("merges stored values over defaults", () => {
    installMemoryStorage({ "blacksite-vol": JSON.stringify({ master: 0.5 }) });
    assert.deepEqual(loadVol(), { master: 0.5, music: 0.42, sfx: 0.75, menu: 0.7 });
  });

  it("falls back to defaults on corrupt JSON", () => {
    installMemoryStorage({ "blacksite-vol": "{" });
    assert.deepEqual(loadVol(), { master: 0.85, music: 0.42, sfx: 0.75, menu: 0.7 });
  });

  it("keeps corrupt persisted channel values out of Web Audio gains", () => {
    installMemoryStorage({ "blacksite-vol": '{"master":"loud","music":1e999,"sfx":-4,"menu":2}' });
    assert.deepEqual(loadVol(), { master: 0.85, music: 0.42, sfx: 0, menu: 1 });
    installMemoryStorage({ "blacksite-vol": '{"master":0,"music":0,"sfx":0,"menu":0}' });
    assert.deepEqual(loadVol(), { master: 0, music: 0, sfx: 0, menu: 0 });
  });
});

describe("leaderboard", () => {
  it("starts empty and keeps the best 10 runs", () => {
    assert.deepEqual(loadBoard(), []);
    let board: ReturnType<typeof loadBoard> = [];
    for (let i = 1; i <= 12; i++) {
      board = saveBoard({ name: `P${i}`, wave: i, kills: i, time: i * 1000 });
    }
    assert.equal(board.length, 10);
    // Sorted by wave desc, so waves 12..3 survive.
    assert.deepEqual(
      board.map((r) => r.wave),
      [12, 11, 10, 9, 8, 7, 6, 5, 4, 3],
    );
  });

  it("sanitizes names to 12 chars on load", () => {
    saveBoard({ name: "A very long marine name", wave: 2, kills: 1, time: 500 });
    assert.equal(loadBoard()[0]!.name, "A very long ");
  });

  it("ignores corrupt storage", () => {
    installMemoryStorage({ "blacksite-board": "not json" });
    assert.deepEqual(loadBoard(), []);
  });
});

describe("gpuEnabled", () => {
  it("is opt-in (defaults to false)", () => {
    assert.equal(gpuEnabled(), false);
    installMemoryStorage({ "blacksite-gpu": "1" });
    assert.equal(gpuEnabled(), true);
  });
});

describe("loadRes", () => {
  it("defaults to 1080p", () => {
    assert.equal(loadRes().id, "1080");
  });

  it("restores a stored mode and rejects unknown ids", () => {
    installMemoryStorage({ "blacksite-res": "1280" });
    assert.equal(loadRes().id, "1280");
    installMemoryStorage({ "blacksite-res": "bogus" });
    assert.equal(loadRes().id, "1080");
  });

  it("only offers widths that keep WebGPU rows 256-byte aligned", () => {
    for (const mode of RES_MODES) {
      assert.equal(mode.w % 64, 0, `${mode.label} width must be a multiple of 64`);
    }
  });
});

describe("loadGfx", () => {
  it("defaults to all effects on", () => {
    assert.deepEqual(loadGfx(), DEFAULT_GFX);
  });

  it("merges stored toggles over defaults", () => {
    installMemoryStorage({ "blacksite-gfx": JSON.stringify({ bloom: false }) });
    assert.deepEqual(loadGfx(), { ...DEFAULT_GFX, bloom: false });
  });

  it("accepts only boolean graphics toggles from storage", () => {
    installMemoryStorage({ "blacksite-gfx": '{"crt":"false","bloom":0,"fog":false}' });
    assert.deepEqual(loadGfx(), { ...DEFAULT_GFX, fog:false });
  });
});

describe("checkpoint validation", () => {
  it("restores matching legacy inventories without inventing ammo", () => {
    for (const length of [8,11,12,18,19,33]) {
      installMemoryStorage({"blacksite-checkpoint":JSON.stringify({wave:26,health:57,weapon:1,flags:1,
        ammo:Array(length).fill(2),mag:Array(length).fill(0)})});
      const save=loadCheckpoint()!;
      assert.equal(save.wave,26); assert.equal(save.health,57); assert.equal(save.weapon,1);
      assert.equal(save.ammo.length,33); assert.equal(save.mag.length,33);
      assert.deepEqual(save.ammo.slice(0,length),Array(length).fill(2));
      assert.deepEqual(save.ammo.slice(length),Array(33-length).fill(0));
      assert.ok(save.mag.every(n=>n===0));
    }
  });

  it("normalizes malformed checkpoint fields before writing the WASM ABI", () => {
    const ammo: unknown[]=Array(33).fill(1e100), mag: unknown[]=Array(33).fill(1e100);
    ammo[0]=-1; ammo[1]="50"; ammo[2]=null; mag[0]=-2; mag[1]={bad:true};
    installMemoryStorage({"blacksite-checkpoint":JSON.stringify({wave:"bad",health:0,armor:1000,weapon:999,
      flags:-1,extraWeapons:-1,kills:-2,secrets:"bad",elapsedMs:1e100,ammo,mag})});
    const save=loadCheckpoint()!;
    assert.equal(save.wave,1); assert.equal(save.health,1); assert.equal(save.armor,100);
    assert.equal(save.weapon,0); assert.equal(save.flags,0); assert.equal(save.extraWeapons,0);
    assert.equal(save.kills,0); assert.equal(save.secrets,0); assert.equal(save.elapsedMs,2147483647);
    for (const weapon of WEAPONS) {
      assert.ok(Number.isInteger(save.ammo[weapon.id]) && save.ammo[weapon.id]!>=0 && save.ammo[weapon.id]!<=weapon.reserve);
      assert.ok(Number.isInteger(save.mag[weapon.id]) && save.mag[weapon.id]!>=0 && save.mag[weapon.id]!<=weapon.magSize);
    }
    assert.deepEqual(save.ammo.slice(0,3),[0,0,0]); assert.deepEqual(save.mag.slice(0,2),[0,0]);
  });

  it("rejects mismatched arrays and never restores an unowned selected gun", () => {
    installMemoryStorage({"blacksite-checkpoint":JSON.stringify({ammo:Array(8).fill(0),mag:Array(11).fill(0)})});
    assert.equal(loadCheckpoint(),null);
    for (const weapon of [1,18,19,32]) {
      installMemoryStorage({"blacksite-checkpoint":JSON.stringify({weapon,flags:0,extraWeapons:0,
        ammo:Array(33).fill(0),mag:Array(33).fill(0)})});
      assert.equal(loadCheckpoint()!.weapon,0);
    }
    installMemoryStorage({"blacksite-checkpoint":JSON.stringify({weapon:32,extraWeapons:1<<13,
      ammo:Array(33).fill(0),mag:Array(33).fill(0)})});
    assert.equal(loadCheckpoint()!.weapon,32);
    installMemoryStorage({"blacksite-checkpoint":JSON.stringify({weapon:999,extraWeapons:1<<13,
      ammo:Array(33).fill(0),mag:Array(33).fill(0)})});
    assert.equal(loadCheckpoint()!.weapon,0);
    installMemoryStorage({"blacksite-checkpoint":JSON.stringify({weapon:32,flags:1e100,extraWeapons:1e100,
      ammo:Array(33).fill(0),mag:Array(33).fill(0)})});
    const invalidBits=loadCheckpoint()!;
    assert.equal(invalidBits.flags,0); assert.equal(invalidBits.extraWeapons,0); assert.equal(invalidBits.weapon,0);
  });
});

describe("boss reward labels", () => {
  it("matches all twenty-five sector rewards, including repeated Obsidian Vault cycles", () => {
    assert.deepEqual(Array.from({length: 25}, (_, i) => bossRewardForWave(i + 1).id), [8, 9, 10, 12, 13, 11, 14, 15, 16, 17, 18, ...Array.from({length:14}, (_,i) => 19+i)]);
    assert.equal(bossRewardForWave(11).name, "MN-6 ECHO");
    assert.equal(bossRewardForWave(36).name, "MN-6 ECHO");
  });
});

describe("tactical machinery hints",()=>{
 it("names each sector's actual prop and its gameplay effect",()=>{
  const hints=Array.from({length:25},(_,i)=>[120,121,122].map(p=>machineryHint(p,i+1)));
  assert.equal(new Set(hints.flat()).size,75);
  for(const row of hints)for(const hint of row)assert.ok(hint && hint.includes(" — "));
  assert.equal(machineryHint(123,1),"SCUFFED SERVICE PANEL — USE E");
  assert.equal(machineryHint(19,1),null);
  assert.equal(machineryHint(120,1),machineryHint(120,26));
 });
 it("keeps recovered logs and machinery reports separate from every terminal ID",()=>{
  for(let wave=1;wave<=25;wave++){
   for(const line of [200,201,202,203,204,205,206])assert.equal(radioCopy(line,wave)?.speaker,"HANDLER");
   for(const line of [210,211,212])assert.equal(radioCopy(line,wave)?.speaker,"RECOVERED LOG");
  }
 });
});

describe("difficulty leaderboard partitions", () => {
  it("keeps ten records independently for each preset and preserves run metadata", () => {
    installMemoryStorage({});
    for (const difficulty of ["recruit", "operative", "veteran", "blacksite"] as const) {
      for (let wave = 1; wave <= 12; wave++)
        saveBoard({ name: "TEST", wave, kills: wave, time: wave * 1000, difficulty });
    }
    const board = loadBoard();
    assert.equal(board.length, 40);
    for (const difficulty of ["recruit", "operative", "veteran", "blacksite"]) {
      assert.deepEqual(
        board.filter((row) => row.difficulty === difficulty).map((row) => row.wave),
        [12, 11, 10, 9, 8, 7, 6, 5, 4, 3],
      );
    }
  });
  it("migrates legacy records to Operative and ignores corrupt rows", () => {
    installMemoryStorage({
      "blacksite-board": JSON.stringify([null, { name: "LEGACY", wave: 3, kills: 8, time: 1000 }]),
    });
    const board = loadBoard();
    assert.equal(board.length, 1);
    assert.equal(board[0]!.difficulty, "operative");
  });
});
