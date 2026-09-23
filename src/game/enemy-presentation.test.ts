import test from "node:test";
import assert from "node:assert/strict";
import {
  readEnemyCues,
  subtitleFor,
  VoiceDirector,
  type EnemyCue,
  type VoiceProfile,
} from "./enemy-presentation.ts";

const enemy: EnemyCue = {
  id: 0,
  skin: 0,
  hp: 28,
  anim: 0,
  x: 8,
  y: 4,
  screenX: 0.3,
  screenY: 0.4,
  sight: true,
  distance: 4,
};
const profile: VoiceProfile = {
  id: "rifleman",
  skin: 0,
  name: "Rifleman",
  style: "radio",
  speaker: "orion",
  lines: [
    { id: "a", cue: "alert", text: "Contact!", url: "a.mp3", duration: 1.5 },
    { id: "b", cue: "alert", text: "Weapons free.", url: "b.mp3", duration: 2 },
    { id: "c", cue: "attack", text: "Fire!", url: "c.mp3", duration: 1 },
    { id: "d", cue: "taunt", text: "Hold still.", url: "d.mp3", duration: 1 },
  ],
};
const profiles = new Map([[0, profile]]);
test("captions use live projection and cannot reveal dead, occluded or offscreen enemies", () => {
  assert.deepEqual(subtitleFor(enemy, profile, profile.lines[0]!), {
    id: 0,
    name: "Rifleman",
    text: "Contact!",
    x: 30,
    y: 40,
  });
  for (const change of [
    { hp: 0 },
    { sight: false },
    { screenX: -1 },
    { screenX: 1.1 },
    { screenY: -2 },
    { distance: 17 },
  ])
    assert.equal(subtitleFor({ ...enemy, ...change }, profile, profile.lines[0]!), null);
});
test("speech director suppresses chatter overlap, rotates alerts and resets between runs", () => {
  const d = new VoiceDirector();
  assert.equal(d.choose([enemy], profiles, 0), null);
  assert.equal(d.choose([enemy], profiles, 1)?.line.id, "a");
  assert.equal(d.choose([{ ...enemy, id: 2 }], profiles, 1.1), null);
  assert.equal(d.choose([{ ...enemy, id: 2 }], profiles, 2.5), null);
  assert.equal(d.choose([{ ...enemy, id: 2 }], profiles, 4)?.line.id, "b");
  d.reset();
  assert.equal(d.choose([enemy], profiles, 0), null);
  assert.equal(d.choose([enemy], profiles, 1)?.line.id, "a");
});
test("silent profiles and enemies behind cover do not get dialogue", () => {
  const d = new VoiceDirector();
  const silent = new Map([[0, { ...profile, lines: [] }]]);
  assert.equal(d.choose([enemy], silent, 0), null);
  assert.equal(d.choose([enemy], silent, 100), null);
  assert.equal(d.choose([{ ...enemy, sight: false }], profiles, 0), null);
  assert.equal(d.choose([{ ...enemy, sight: false }], profiles, 100), null);
});
test("snapshot decoding follows ten-float ABI", () => {
  const data = new Float32Array([2, 7, 6, 20, 8, 4, 0.5, 0.25, 1, 4]);
  assert.deepEqual(readEnemyCues(data.buffer, 0, 1)[0], {
    id: 2,
    skin: 7,
    anim: 6,
    hp: 20,
    x: 8,
    y: 4,
    screenX: 0.5,
    screenY: 0.25,
    sight: true,
    distance: 4,
  });
});
