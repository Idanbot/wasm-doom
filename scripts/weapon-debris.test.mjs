import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { T_RELOAD_DEBRIS, T_WEAPON_FEED } from '../src/game/atlas-slots.ts';

test('reload debris has four verified RGBA sheets and dedicated atlas slots', () => {
  const manifest = JSON.parse(readFileSync('art/weapon-debris-v1/manifest.json'));
  assert.equal(manifest.assets.length, 4);
  assert.equal(T_WEAPON_FEED - T_RELOAD_DEBRIS, 4);
  for (const asset of manifest.assets) {
    const bytes = readFileSync(asset.file);
    assert.equal(createHash('sha256').update(bytes).digest('hex'), asset.sha256);
    assert.equal(bytes.readUInt32BE(16), 1024);
    assert.equal(bytes.readUInt32BE(20), 1024);
    assert.equal(bytes[25], 6, 'native RGBA transparency');
    assert.deepEqual(asset.grid, [2, 2]);
    assert.deepEqual(asset.cell, [512, 512]);
  }
});
