import { describe, it, beforeEach } from "node:test";
import assert from "node:assert/strict";
import {
  fmtTime,
  gpuEnabled,
  loadBoard,
  loadGfx,
  loadRes,
  loadVol,
  saveBoard,
  sheetPos,
  WEAPONS,
  gridPos,
  sectorForWave,
} from "./data.ts";
import { DEFAULT_GFX, RES_MODES } from "../../game/types.ts";

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
  it("cycles through three distinct sectors and bosses", () => {
    assert.deepEqual([1, 2, 3].map((wave) => sectorForWave(wave).name), [
      "UPPER WORKS",
      "CRYOGENIC FOUNDRY",
      "BIOFORGE DEPTHS",
    ]);
    assert.equal(new Set([1, 2, 3].map((wave) => sectorForWave(wave).bossName)).size, 3);
    assert.equal(sectorForWave(4).name, "UPPER WORKS");
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
  it("has eleven entries with unique ids and art paths", () => {
    assert.equal(WEAPONS.length, 11);
    assert.deepEqual(
      WEAPONS.map((w) => w.id),
      [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    );
    for (const w of WEAPONS) {
      assert.ok(w.name.length > 0);
      for (const art of [w.idle, w.fire, w.reload]) {
        assert.match(art, /^\/game\/weap_.+\.png$/);
      }
      assert.ok(w.magSize > 0);
      assert.ok(w.lowAmmoAt > 0 && w.lowAmmoAt < w.magSize);
    }
    assert.deepEqual(
      WEAPONS.map((w) => w.magSize),
      [12, 8, 36, 5, 4, 10, 90, 6, 4, 14, 5],
    );
  });
});

describe("loadVol", () => {
  it("returns defaults when nothing is stored", () => {
    assert.deepEqual(loadVol(), { master: 0.85, music: 0.42, sfx: 0.75 });
  });

  it("merges stored values over defaults", () => {
    installMemoryStorage({ "hellscan-vol": JSON.stringify({ master: 0.5 }) });
    assert.deepEqual(loadVol(), { master: 0.5, music: 0.42, sfx: 0.75 });
  });

  it("falls back to defaults on corrupt JSON", () => {
    installMemoryStorage({ "hellscan-vol": "{" });
    assert.deepEqual(loadVol(), { master: 0.85, music: 0.42, sfx: 0.75 });
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
    installMemoryStorage({ "hellscan-board": "not json" });
    assert.deepEqual(loadBoard(), []);
  });
});

describe("gpuEnabled", () => {
  it("is opt-in (defaults to false)", () => {
    assert.equal(gpuEnabled(), false);
    installMemoryStorage({ "hellscan-gpu": "1" });
    assert.equal(gpuEnabled(), true);
  });
});

describe("loadRes", () => {
  it("defaults to the 640x400 mode", () => {
    assert.equal(loadRes().id, "640");
  });

  it("restores a stored mode and rejects unknown ids", () => {
    installMemoryStorage({ "hellscan-res": "1280" });
    assert.equal(loadRes().id, "1280");
    installMemoryStorage({ "hellscan-res": "bogus" });
    assert.equal(loadRes().id, "640");
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
    installMemoryStorage({ "hellscan-gfx": JSON.stringify({ bloom: false }) });
    assert.deepEqual(loadGfx(), { ...DEFAULT_GFX, bloom: false });
  });
});
