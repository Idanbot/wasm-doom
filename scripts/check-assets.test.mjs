import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { deflateSync } from "node:zlib";
import {
  checkAssets,
  checkBudgetFiles,
  isSnakePng,
  readPngInfo,
  validateManifest,
  validateMetadataSchema,
  validateReadyFiles,
} from "./check-assets.mjs";

const execFileAsync = promisify(execFile);
const scriptPath = join(dirname(fileURLToPath(import.meta.url)), "check-assets.mjs");

// Minimal CRC32 for building valid PNG fixtures.
function crc32(buf) {
  let table = crc32.table;
  if (!table) {
    table = crc32.table = new Int32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
      table[n] = c;
    }
  }
  let crc = -1;
  for (const b of buf) crc = table[(crc ^ b) & 0xff] ^ (crc >>> 8);
  return (crc ^ -1) >>> 0;
}

function chunk(type, data) {
  const t = Buffer.from(type, "ascii");
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([t, data])));
  return Buffer.concat([len, t, data, crc]);
}

/** Build a minimal valid PNG: width x height, given IHDR color type. */
function makePng(width, height, colorType = 6) {
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = colorType;
  const channels = colorType === 6 ? 4 : colorType === 2 ? 3 : 1;
  const raw = Buffer.alloc(height * (1 + width * channels));
  return Buffer.concat([sig, chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw)), chunk("IEND", Buffer.alloc(0))]);
}

function writeManifest(root, entries) {
  const m = { enemies: [], weapons: [], textures: [], props: [], ...entries };
  writeFileSync(join(root, "art", "blacksite-manifest.json"), JSON.stringify(m));
  return m;
}

function makeRoot() {
  const root = mkdtempSync(join(tmpdir(), "assets-"));
  mkdirSync(join(root, "art", "atlases"), { recursive: true });
  mkdirSync(join(root, "public", "game"), { recursive: true });
  return root;
}

test("isSnakePng accepts lowercase snake_case PNGs only", () => {
  assert.equal(isSnakePng("spr_rifleman.png"), true);
  assert.equal(isSnakePng("wall_hazard_bi.png"), true);
  assert.equal(isSnakePng("Spr_Rifleman.png"), false);
  assert.equal(isSnakePng("rifleman.png.exe"), false);
  assert.equal(isSnakePng("rifleman.jpg"), false);
});

test("readPngInfo parses dimensions and alpha", () => {
  assert.deepEqual(readPngInfo(makePng(256, 256, 6)), { width: 256, height: 256, colorType: 6, hasAlpha: true });
  assert.equal(readPngInfo(makePng(640, 640, 2)).hasAlpha, false);
  assert.throws(() => readPngInfo(Buffer.from("nope")), /not a PNG/);
});

test("validateMetadataSchema accepts the spec §35 example", () => {
  const meta = {
    id: "enemy_rifleman",
    frameWidth: 256,
    frameHeight: 256,
    origin: { x: 128, y: 236 },
    animations: { idle: { start: 0, frames: 2, fps: 3 }, walk: { start: 2, frames: 8, fps: 8 } },
  };
  assert.deepEqual(validateMetadataSchema(meta), []);
  assert.ok(validateMetadataSchema({}).length > 0);
  assert.ok(validateMetadataSchema({ ...meta, animations: { idle: { start: 0, frames: 0, fps: 3 } } }).length > 0);
});

test("validateManifest rejects duplicates and bad enums", () => {
  const base = {
    enemies: [{ specId: "enemy_x", engineKind: "EK_HUSK", textureSlot: 8, file: "spr_x.png", sheet: "2x2", status: "planned" }],
    weapons: [], textures: [], props: [],
  };
  assert.deepEqual(validateManifest(base).errors, []);
  const dup = structuredClone(base);
  dup.props = [{ specId: "enemy_x", file: "spr_y.png", size: 256, status: "planned", textureSlot: 21 }];
  assert.match(validateManifest(dup).errors.join("\n"), /duplicate specId/);
  const bad = structuredClone(base);
  bad.enemies[0].status = "eventually";
  bad.enemies[0].engineKind = "EK_NOPE";
  const errs = validateManifest(bad).errors.join("\n");
  assert.match(errs, /bad status/);
  assert.match(errs, /unknown engineKind/);
});

test("ready entries require real 256px alpha PNGs", () => {
  const root = makeRoot();
  writeManifest(root, {
    enemies: [{ specId: "enemy_x", engineKind: "EK_HUSK", textureSlot: 8, file: "spr_x.png", sheet: "2x2", status: "ready" }],
  });
  const gameDir = join(root, "public", "game");
  const atlasesDir = join(root, "art", "atlases");
  const read = () => JSON.parse(readFileSync(join(root, "art", "blacksite-manifest.json"), "utf8"));
  // Missing file fails.
  assert.match(validateReadyFiles(read(), gameDir, atlasesDir).errors.join("\n"), /missing delivered file/);
  // Wrong size fails.
  writeFileSync(join(gameDir, "spr_x.png"), makePng(128, 128, 6));
  assert.match(validateReadyFiles(read(), gameDir, atlasesDir).errors.join("\n"), /must be 256x256/);
  // No alpha fails.
  writeFileSync(join(gameDir, "spr_x.png"), makePng(256, 256, 2));
  assert.match(validateReadyFiles(read(), gameDir, atlasesDir).errors.join("\n"), /no alpha/);
  // Correct file passes (metadata warning only).
  writeFileSync(join(gameDir, "spr_x.png"), makePng(256, 256, 6));
  const good = validateReadyFiles(read(), gameDir, atlasesDir);
  assert.deepEqual(good.errors, []);
  assert.equal(good.checked, 1);
});

test("ready viewmodel weapons accept 800x480 frames and 4x2 reload sheets", () => {
  const root = makeRoot();
  const files = ["weap_test.png", "weap_test_fire.png", "weap_test_reload.png"];
  writeManifest(root, {
    weapons: [
      {
        specId: "weapon_test",
        engineSlot: 0,
        files,
        sheet: "2x2",
        reloadSheet: "4x2",
        status: "ready",
      },
    ],
  });
  const gameDir = join(root, "public", "game");
  writeFileSync(join(gameDir, files[0]), makePng(800, 480));
  writeFileSync(join(gameDir, files[1]), makePng(1600, 960));
  writeFileSync(join(gameDir, files[2]), makePng(3200, 960));
  const result = validateReadyFiles(
    JSON.parse(readFileSync(join(root, "art", "blacksite-manifest.json"), "utf8")),
    gameDir,
    join(root, "art", "atlases"),
  );
  assert.deepEqual(result.errors, []);
  assert.equal(result.checked, 3);
});

test("budgets flag tiny pickup cases and warn on huge viewmodels", () => {
  const root = makeRoot();
  const gameDir = join(root, "public", "game");
  writeFileSync(join(gameDir, "spr_gun_test.png"), Buffer.alloc(30_000));
  writeFileSync(join(gameDir, "weap_test_fire.png"), Buffer.alloc(2_100_000));
  writeFileSync(join(gameDir, "weap_test.png"), Buffer.alloc(100_000));
  const result = checkBudgetFiles(gameDir);
  assert.match(result.errors.join("\n"), /spr_gun_test\.png.*regenerate/);
  assert.match(result.warnings.join("\n"), /weap_test_fire\.png.*2\.0MB/);
  writeFileSync(join(gameDir, "spr_gun_test.png"), Buffer.alloc(60_000));
  writeFileSync(join(gameDir, "weap_test_fire.png"), Buffer.alloc(500_000));
  const clean = checkBudgetFiles(gameDir);
  assert.deepEqual(clean.errors, []);
  assert.deepEqual(clean.warnings, []);
});

test("budgets pass against the real repo tree", () => {
  const gameDir = join(dirname(fileURLToPath(import.meta.url)), "..", "public", "game");
  assert.deepEqual(checkBudgetFiles(gameDir).errors, []);
});

test("cli passes against the real repo (all planned)", async () => {
  const { stdout } = await execFileAsync(process.execPath, [scriptPath]);
  assert.match(stdout, /manifest ok/);
});
