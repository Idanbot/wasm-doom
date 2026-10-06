import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, statSync, readdirSync } from "node:fs";
import { createHash } from "node:crypto";
const read = (p) => JSON.parse(readFileSync(p, "utf8"));
const truePeaks = read("art/audio/true-peaks.json");
const art = read("art/audio/manifest.json"),
  runtime = read("public/game/sfx/v2/manifest.json");
const hash = (p) => createHash("sha256").update(readFileSync(p)).digest("hex");
test("all 33 gunshots, mechanisms and event families have local licensed audio with fallback", () => {
  const ids = new Set(runtime.clips.map((c) => c.id));
  assert.equal(ids.size, runtime.clips.length);
  // Compared against the authored art manifest, not a restated literal.
  assert.equal(ids.size, art.clips.length);
  for (let n = 1; n <= 25; n++) assert.ok(ids.has(`sector-${n}`));
  for (const id of ["casing-brass", "casing-shell"]) assert.ok(ids.has(id));
  for (let n = 0; n < 33; n++) assert.ok(ids.has(`fire${n}`));
  for (const name of [
    "mag-out",
    "mag-in",
    "chamber",
    "shell",
    "pump",
    "empty",
    "cell-out",
    "cell-in",
    "charge",
    "equip",
    "door",
    "explosion",
    "ui-click",
    "ui-confirm",
    "ui-back",
    "ui-error",
    "ui-transition",
    "pickup-health",
    "pickup-armor",
    "pickup-ammo",
    "pickup-weapon",
    "hit-metal",
    "hit-flesh",
    "hit-stone",
    "break-glass",
    "break-metal",
    "break-wood",
    "travel-rocket",
    "travel-energy",
    "travel-acid",
    "travel-flame",
  ])
    assert.ok(ids.has(name), name);
  for (const family of ["human", "robot", "creature"])
    for (const state of ["pain", "death"])
      for (let n = 0; n < 3; n++) assert.ok(ids.has(`${state}-${family}${n}`));
  for (const clip of runtime.clips)
    for (const path of [clip.url, clip.fallback])
      assert.ok(statSync("public" + path).size > 500, path);
});
test("every source and derivative has verified CC0 provenance, hashes and reproducible recipes", () => {
  for (const [id, p] of Object.entries(art.packs)) {
    assert.equal(p.license, "CC0-1.0");
    assert.match(p.url, /^https:\/\//);
    assert.match(readFileSync(`art/audio/licenses/${id}/SOURCE.txt`, "utf8"), /CC0/);
  }
  for (const [path, source] of Object.entries(art.sources)) {
    assert.ok(art.packs[source.pack]);
    assert.equal(hash("art/audio/sources/" + path), source.sha256);
  }
  for (const clip of art.clips) {
    assert.ok(clip.layers.length > 0);
    for (const l of clip.layers) assert.ok(art.sources[l.source]);
    for (const ext of ["ogg", "mp3"])
      assert.equal(hash(`public/game/sfx/v2/${clip.id}.${ext}`), clip.measurements.files[ext]);
    assert.ok(clip.measurements.duration > 0 && clip.measurements.duration <= 1.8);
    assert.ok(clip.measurements.pcmPeakDb <= -6);
    assert.ok(truePeaks[clip.id].truePeakDb <= -3);
    assert.equal(truePeaks[clip.id].sha256, clip.measurements.files.ogg);
    assert.ok(clip.measurements.rmsDb > -55);
  }
  const fires = art.clips.filter((c) => /^fire\d+$/.test(c.id));
  assert.equal(new Set(fires.map((c) => c.measurements.files.ogg)).size, 33);
  const bytes = runtime.clips.reduce((sum, c) => sum + statSync("public" + c.url).size, 0);
  assert.ok(bytes < 2_000_000, `${bytes} bytes exceeds SFX primary budget`);
});
test("runtime manifest matches recipes, with no orphaned exports or incorrect vocal classification", () => {
  assert.deepEqual(
    runtime.clips.map((c) => c.id),
    art.clips.map((c) => c.id),
  );
  assert.equal(
    readdirSync("public/game/sfx/v2").filter((p) => /\.(ogg|mp3)$/.test(p)).length,
    runtime.clips.length * 2,
  );
  for (const c of runtime.clips) {
    assert.equal(c.vocal, !!art.clips.find((a) => a.id === c.id).vocal);
    if (c.vocal) assert.match(c.id, /^(pain|death)-(human|creature)/);
  }
});
