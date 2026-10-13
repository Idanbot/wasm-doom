import { test } from "node:test";
import assert from "node:assert/strict";
import {
  SfxDirector,
  ENEMY_WEAPONS,
  enemyFamily,
  worldIntent,
  weaponMechanism,
  reloadLayers,
} from "./sfx-director.ts";
const base = { weapon: 0, ammo: 5, reloading: 0, weapFrame: 0, state: 0, wave: 1 };
const names = (a: { id: string }[]) => a.map((s) => s.id);
test("magazine stages follow crossed frames, never timer callbacks", () => {
  const d = new SfxDirector();
  d.weaponFrame(base);
  assert.deepEqual(names(d.weaponFrame({ ...base, reloading: 0.1, weapFrame: 10 })), ["mag-out", "reload-latch"]);
  assert.deepEqual(names(d.weaponFrame({ ...base, reloading: 0.5, weapFrame: 12 })), ["mag-in", "reload-seat"]);
  assert.deepEqual(names(d.weaponFrame({ ...base, reloading: 0.8, weapFrame: 14 })), ["chamber", "reload-bolt"]);
  assert.deepEqual(d.weaponFrame({ ...base, ammo: 12 }), []);
});
test("slow frames catch crossed reload stages once; firing and switching cancel future steps", () => {
  const d = new SfxDirector();
  d.weaponFrame(base);
  assert.deepEqual(names(d.weaponFrame({ ...base, reloading: 0.8, weapFrame: 14 })), [
    "mag-out", "reload-latch",
    "mag-in", "reload-seat",
    "chamber", "reload-bolt",
  ]);
  assert.deepEqual(d.weaponFrame({ ...base, reloading: 0.81, weapFrame: 14 }), []);
  assert.deepEqual(d.weaponFrame({ ...base, ammo: 4, weapFrame: 15 }), []);
  assert.deepEqual(names(d.weaponFrame({ ...base, weapon: 3, reloading: 0 })), ["equip"]);
  assert.deepEqual(d.weaponFrame({ ...base, weapon: 3 }), []);
});
test("shotgun inserts one cue per actual shell, including final shell; interrupted fire has no pump", () => {
  const d = new SfxDirector();
  const h = { ...base, weapon: 1, ammo: 0 };
  d.weaponFrame(h);
  assert.deepEqual(d.weaponFrame({ ...h, reloading: 1, weapFrame: 10 }), []);
  assert.deepEqual(d.weaponFrame({ ...h, reloading: 0.5, weapFrame: 12 }), []);
  assert.deepEqual(names(d.weaponFrame({ ...h, ammo: 1, reloading: 1, weapFrame: 10 })), ["shell"]);
  assert.deepEqual(names(d.weaponFrame({ ...h, ammo: 2, reloading: 0 })), ["shell", "pump"]);
  d.weaponFrame({ ...h, ammo: 2, reloading: 1, weapFrame: 10 });
  assert.deepEqual(d.weaponFrame({ ...h, ammo: 1, reloading: 0, weapFrame: 15 }), []);
});
test("pause, sector change and energy mechanisms don't leak reload cues", () => {
  const d = new SfxDirector();
  d.weaponFrame(base);
  assert.deepEqual(d.weaponFrame({ ...base, state: 1, reloading: 0.5, weapFrame: 12 }), []);
  d.reset();
  assert.deepEqual(d.weaponFrame({ ...base, reloading: 0.5, weapFrame: 12 }), []);
  assert.deepEqual(d.weaponFrame({ ...base, wave: 2, reloading: 0.1, weapFrame: 14 }), []);
  assert.deepEqual(weaponMechanism(10), ["cell-out", "cell-in", "charge"]);
});
const enemy = { id: 1, skin: 0, hp: 30, anim: 0, x: 5, y: 7, sight: true, distance: 3 };
test("spatial enemy shots, pain and death fire on transitions and once per death", () => {
  const d = new SfxDirector();
  d.enemyFrame([enemy], 0);
  const shot = d.enemyFrame([{ ...enemy, anim: 3 }], 1);
  assert.equal(shot[0]?.id, "fire2");
  assert.equal(shot[0]?.x, 5);
  assert.deepEqual(d.enemyFrame([{ ...enemy, anim: 3 }], 1.1), []);
  assert.deepEqual(names(d.enemyFrame([{ ...enemy, hp: 20, anim: 2 }], 1.2)), ["pain-human1"]);
  assert.deepEqual(d.enemyFrame([{ ...enemy, hp: 19, anim: 2 }], 1.3), []);
  assert.deepEqual(names(d.enemyFrame([{ ...enemy, hp: 0, anim: 5 }], 1.4)), [
    "body-thud",
    "death-human1",
  ]);
  assert.deepEqual(d.enemyFrame([{ ...enemy, hp: 0, anim: 5 }], 1.5), []);
  d.enemyFrame([], 2);
  assert.deepEqual(d.enemyFrame([{ ...enemy, hp: 0, anim: 5 }], 2.1), []);
});
test("entity-slot reuse and loading corpses cannot fabricate a death", () => {
  const d = new SfxDirector();
  d.enemyFrame([enemy], 0);
  assert.deepEqual(d.enemyFrame([{ ...enemy, skin: 13, hp: 0, anim: 5 }], 1), []);
  assert.equal(enemyFamily(13), "robot");
  assert.equal(enemyFamily(14), "creature");
});
test("all 37 enemy types and 25 bosses have valid intentional firing and special profiles", () => {
  assert.equal(ENEMY_WEAPONS.length, 37);
  for (let skin = 0; skin < 37; skin++) {
    const d = new SfxDirector();
    const cue = { ...enemy, skin };
    d.enemyFrame([cue], 0);
    const shot = d.enemyFrame([{ ...cue, anim: 3 }], 1);
    assert.match(shot[0]!.id, /^fire\d+$/);
    assert.ok(ENEMY_WEAPONS[skin]! >= 0 && ENEMY_WEAPONS[skin]! < 33);
    const special = d.enemyFrame([{ ...cue, anim: 6 }], 2);
    assert.match(special[0]!.id, /^hazard-/);
  }
});
test("boss phase cues occur once; material and pickup mappings are distinct", () => {
  const d = new SfxDirector();
  assert.deepEqual(names(d.bossPhase(1, 14)), ["hazard-acid"]);
  assert.deepEqual(d.bossPhase(1, 14), []);
  assert.equal(d.bossPhase(2, 14).length, 1);
  assert.equal(worldIntent(3)?.id, "hit-metal");
  assert.equal(worldIntent(4)?.id, "hit-flesh");
  assert.equal(worldIntent(8)?.id, "pickup-health");
  assert.equal(worldIntent(9)?.id, "pickup-armor");
  assert.equal(worldIntent(13, 3)?.id, "hazard-acid");
  assert.equal(worldIntent(99), null);
});

test("all sector enemies inherit their intentional weapon and death family", () => {
  const bases = [0, 3, 10, 7, 3, 0, 3, 7, 4, 1, 7, 1, 3, 7, 4, 8, 10, 7, 5, 2, 3, 3, 7, 8, 4];
  for (const [sector, base] of bases.entries()) {
    const skin = 37 + sector;
    assert.equal(enemyFamily(skin), enemyFamily(base));
    const director = new SfxDirector();
    director.enemyFrame([{ ...enemy, skin }], 0);
    assert.equal(
      director.enemyFrame([{ ...enemy, skin, anim: 3 }], 1)[0]?.id,
      `fire${ENEMY_WEAPONS[base]}`,
    );
  }
});

test("reload layers match ballistic, powered and pressure feeds, and never duplicate a stage", () => {
  assert.deepEqual(reloadLayers(0), ["reload-latch", "reload-seat", "reload-bolt"]);
  assert.deepEqual(reloadLayers(7), ["reload-valve", "reload-seat", "reload-seal"]);
  assert.deepEqual(reloadLayers(9), ["reload-lock", "reload-seat", "reload-charge"]);
  for(let weapon=0;weapon<33;weapon++) {
    if(weapon===1)continue;
    const d=new SfxDirector(); d.weaponFrame({...base,weapon});
    const h={...base,weapon,reloading:.5,weapFrame:12};
    assert.equal(d.weaponFrame(h).length,4);
    assert.deepEqual(d.weaponFrame(h),[]);
  }
});

test("feed removal audio waits for the authored crossing and differentiates shield contact", () => {
  const d=new SfxDirector();d.weaponFrame(base);
  assert.deepEqual(d.weaponFrame({...base,reloading:.05,weapFrame:10}),[]);
  assert.deepEqual(d.weaponFrame({...base,reloading:.099,weapFrame:10}),[]);
  assert.deepEqual(names(d.weaponFrame({...base,reloading:.1,weapFrame:10})),["mag-out","reload-latch"]);
  assert.deepEqual(d.weaponFrame({...base,reloading:.15,weapFrame:10}),[]);
  assert.equal(worldIntent(3,1)?.id,"hazard-electric");
  assert.equal(worldIntent(3,0)?.id,"hit-metal");
  assert.equal(worldIntent(4,0)?.id,"hit-flesh");
});
