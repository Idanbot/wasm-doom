import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { BOSS_ATTACKS, bossAttackForWave } from '../src/game/boss-attacks.ts';

test('every boss has a unique named counter and UI matches engine combat timings and weakness', async () => {
  assert.equal(BOSS_ATTACKS.length, 25);
  assert.equal(new Set(BOSS_ATTACKS.map(a => a.name)).size, 25);
  const rust = await readFile('engine/src/boss_attacks.rs', 'utf8');
  const profiles = [...rust.matchAll(/Profile\s*\{\s*windup:\s*([\d.]+),\s*recovery:\s*([\d.]+),\s*weak:\s*(\w+)\s*,?\s*\}/g)];
  assert.equal(profiles.length, 25);
  for (const [i, attack] of BOSS_ATTACKS.entries()) {
    assert.deepEqual(profiles[i].slice(1).map((v,n) => n < 2 ? Number(v) : v), [attack.windup, attack.recovery, attack.weak]);
    assert.ok(attack.dodge.length > 12);
    assert.equal(bossAttackForWave(i + 1), attack);
    assert.equal(bossAttackForWave(i + 26), attack);
  }
});
