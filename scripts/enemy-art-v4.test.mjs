import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
const root = new URL('../', import.meta.url);
const specs = JSON.parse(readFileSync(new URL('art/enemies-v4/specs.json', root)));
test('all 48 authored characters have reproducible transparent motion assets', () => {
  const generation = JSON.parse(readFileSync(new URL('art/enemies-v4/generation.json', root)));
  assert.equal(generation.enemies.length, 48);
  assert.equal(specs.enemies.filter(e => e.level).length, 25);
  assert.equal(new Set(specs.enemies.map(e => e.id)).size, 48);
  for (const character of generation.enemies) {
    assert.equal(character.frames, 28);
    assert.equal(createHash('sha256').update(readFileSync(new URL(character.source, root))).digest('hex'), character.source_sha256);
  }
  const check = spawnSync('python3', ['scripts/check-blacksite-motion.py'], { cwd: root, encoding: 'utf8' });
  assert.equal(check.status, 0, check.stdout + check.stderr);
});
