import { test } from 'node:test';
import assert from 'node:assert/strict';
import { hasPressureBank, pressureHint } from './weapon-mechanics.ts';
import { worldIntent, SfxDirector } from './sfx-director.ts';

test('vent hints only offer discharge while pressure and magazine ammunition remain', () => {
  assert.match(pressureHint(0.8, 0.8, 2), /FIRE TO DISCHARGE/);
  assert.equal(pressureHint(0.8, 0.8, 0), 'VENTING SAFELY');
  assert.equal(pressureHint(0.1, 0.1, 2), 'VENTING SAFELY');
  assert.match(pressureHint(1, 0, 2), /PRESSURE LIMIT/);
  assert.match(pressureHint(0.4, 0, 2), /R TO VENT/);
  assert.deepEqual(Array.from({length:33},(_,slot)=>slot).filter(hasPressureBank), [13,20,28]);
});

test('pressure cues use local handling audio without starting magazine reload layers', () => {
  assert.equal(worldIntent(25)?.id, 'reload-valve');
  assert.equal(worldIntent(26)?.id, 'hazard-pressure');
  const director = new SfxDirector();
  const state = { weapon:20,ammo:3,reloading:0,weapFrame:1,state:0,wave:1 };
  director.weaponFrame(state);
  assert.deepEqual(director.weaponFrame({...state,weapFrame:2}), []);
});
