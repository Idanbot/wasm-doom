import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { WEAPON_FEED_FILES, WEAPON_SLUGS } from '../src/game/weapon-assets.ts';
import { PROJECTILE_ART } from '../src/game/projectile-art.ts';
import { T_WEAPON_FEED, T_BEAM, TEX_N } from '../src/game/atlas-slots.ts';

const hash = p => createHash('sha256').update(readFileSync(p)).digest('hex');

test('every removable feed has unique, source-verified weapon-specific artwork', () => {
  const manifest = JSON.parse(readFileSync('art/weapon-feeds-v2/manifest.json'));
  assert.equal(manifest.assets.length, 28);
  assert.equal(WEAPON_FEED_FILES.length, WEAPON_SLUGS.length);
  assert.equal(WEAPON_FEED_FILES[1], null);
  const paths = WEAPON_FEED_FILES.filter(Boolean);
  assert.equal(new Set(paths).size, 32);
  for (const path of paths) assert.ok(existsSync(`public${path}`), path);
  for (const a of manifest.assets) {
    assert.equal(WEAPON_SLUGS[a.slot], a.id);
    assert.equal(WEAPON_FEED_FILES[a.slot], `/${a.file.replace(/^public\//, '')}`);
    assert.equal(hash(a.file), a.sha256);
    assert.equal(hash(a.source), a.source_sha256);
    const png = readFileSync(a.file);
    assert.equal(png.readUInt32BE(16), 1024);
    assert.equal(png.readUInt32BE(20), 1024);
    assert.equal(png[25], 6);
    assert.deepEqual(a.grid, [2, 2]);
    assert.equal(a.padding, 56);
  }
  assert.equal(T_BEAM - T_WEAPON_FEED, WEAPON_SLUGS.length);
  assert.equal(TEX_N - T_BEAM, 2);
});

test('all blue projectile replacements and both beam cores have reproducible native-alpha assets', () => {
  const manifest = JSON.parse(readFileSync('art/projectiles-v4/manifest.json'));
  assert.equal(manifest.assets.length, 18);
  assert.equal(Object.keys(PROJECTILE_ART).length, 16);
  for (const a of manifest.assets) {
    assert.equal(hash(a.file), a.sha256);
    assert.equal(hash(a.source), a.source_sha256);
    const png = readFileSync(a.file);
    assert.equal(png.readUInt32BE(16), 1024);
    assert.equal(png.readUInt32BE(20), 1024);
    assert.equal(png[25], 6);
    if (a.replaces) assert.equal(PROJECTILE_ART[a.replaces], `/${a.file.replace(/^public\//, '')}`);
  }
});
