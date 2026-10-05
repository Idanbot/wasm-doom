import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { WorldEffects, sectorEffect, SECTOR_EFFECTS } from "../src/game/sector-effects.ts";

test("HD casing masters and padded runtime sheets retain recorded provenance", () => {
  const manifest = JSON.parse(readFileSync("art/world-fx-v1/manifest.json"));
  assert.equal(manifest.assets.length, 2);
  for (const asset of manifest.assets) {
    const hash = p => createHash("sha256").update(readFileSync(p)).digest("hex");
    assert.equal(hash(asset.source), asset.sourceSha256);
    assert.equal(hash(asset.file), asset.sha256);
    const png = readFileSync(asset.file);
    assert.equal(png.readUInt32BE(16), 1024); assert.equal(png.readUInt32BE(20), 1024);
    assert.equal(png[25], 6); assert.equal(asset.alpha, "native");
  }
});

test("all 25 sectors have distinct signatures and decoded non-vocal local audio", () => {
  const audio = JSON.parse(readFileSync("public/game/sfx/v2/manifest.json"));
  const recipes = JSON.parse(readFileSync("art/audio/manifest.json"));
  assert.equal(SECTOR_EFFECTS.length, 25);
  assert.equal(new Set(SECTOR_EFFECTS.map(p => p[0])).size, 25);
  const hashes = [];
  for (let wave = 1; wave <= 25; wave++) {
    const p = sectorEffect(wave);
    assert.equal(p.index, wave - 1);
    assert.ok(p.color.every(c => c >= 0 && c <= 1));
    const id = `sector-${wave}`;
    assert.equal(audio.clips.find(c => c.id === id).vocal, false);
    hashes.push(recipes.clips.find(c => c.id === id).measurements.files.ogg);
  }
  assert.equal(new Set(hashes).size, 25);
  assert.deepEqual(sectorEffect(26), sectorEffect(1));
});

test("explosion projection tracks camera orientation, expires and stays bounded", () => {
  const fx = new WorldEffects(); fx.tick(1/60, 1);
  for (let i = 0; i < 20; i++) fx.explosion(4, 0);
  const p = {x:0,y:0,yaw:0};
  const first = fx.project(p, 1.6);
  assert.equal(first.length, 4);
  assert.equal(first[0].x, .5);
  fx.tick(.2, 1);
  const later = fx.project(p, 1.6)[0];
  assert.ok(later.radius > first[0].radius);
  assert.ok(later.strength < first[0].strength);
  assert.deepEqual(fx.project({...p,yaw:Math.PI}, 1.6), []);
  fx.tick(.6, 1); assert.deepEqual(fx.project(p, 1.6), []);
});

test("map changes discard shockwaves and restart accents; no tick means no pause drift", () => {
  const fx = new WorldEffects();
  assert.equal(fx.tick(1, 1), false);
  assert.equal(fx.tick(2, 1), true);
  fx.explosion(4,0); const time = fx.time;
  assert.equal(fx.time, time);
  assert.equal(fx.tick(.016, 2), false);
  assert.deepEqual(fx.project({x:0,y:0,yaw:0}, 1.6), []);
  assert.ok(fx.time < .02);
});


test("boss entrance clocks match authored durations, reset and respect camera projection", () => {
  const profiles=JSON.parse(readFileSync("src/game/boss-arena-data.json"));
  for(let sector=0;sector<25;sector++) {
    const fx=new WorldEffects();fx.tick(.016,sector+1);
    fx.bossEntrance(4,0,sector*5);
    const player={x:0,y:0,yaw:0};
    const initial=fx.projectEntrance(player,1.6);
    assert.equal(initial.x,.5);assert.equal(initial.style,sector);assert.equal(initial.phase,0);
    fx.tick(profiles[sector].introSeconds*.5,sector+1);
    assert.ok(Math.abs(fx.projectEntrance(player,1.6).phase-.5)<.001);
    fx.bossEntrance(4,0,sector*5+2);
    assert.equal(fx.projectEntrance(player,1.6).phase,.5,"later stages never restart entrance");
    assert.equal(fx.projectEntrance({...player,yaw:Math.PI},1.6),undefined);
    fx.tick(profiles[sector].introSeconds*.51,sector+1);
    assert.equal(fx.projectEntrance(player,1.6),undefined);
    fx.bossEntrance(4,0,sector*5);fx.tick(.01,sector+2);
    assert.equal(fx.projectEntrance(player,1.6),undefined);
  }
});

test("eight native-alpha boss entrance motifs have valid hashes and texture slots", () => {
  const manifest=JSON.parse(readFileSync("art/boss-entry-v1/manifest.json"));
  const hash=p=>createHash("sha256").update(readFileSync(p)).digest("hex");
  assert.equal(hash(manifest.source),manifest.sourceSha256);
  assert.equal(manifest.assets.length,8);
  for(const [i,a] of manifest.assets.entries()) {
    assert.equal(a.textureSlot,806+i);assert.equal(hash(a.file),a.sha256);
    const png=readFileSync(a.file);assert.equal(png.readUInt32BE(16),1024);assert.equal(png.readUInt32BE(20),1024);
    assert.equal(png[25],6);assert.equal(a.alpha,"native");
  }
});
