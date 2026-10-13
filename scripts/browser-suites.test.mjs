import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';
import { BROWSER_SUITES, browserTestArgs } from './browser-suites.mjs';

test('CI browser partitions retain each regression once and bound hung tests', () => {
  const workflow = readFileSync('.github/workflows/ci.yml', 'utf8');
  const groups = workflow.match(/group: \[([^\]]+)\]/)[1].split(',').map(s => s.trim());
  assert.deepEqual(groups.filter(g => g !== 'smoke'), Object.keys(BROWSER_SUITES));
  const names = Object.values(BROWSER_SUITES).flat();
  assert.equal(new Set(names).size, names.length, 'a regression runs in multiple partitions');
  const files = names.map(name => `scripts/${name}.browser.mjs`);
  for (const file of files) assert.ok(existsSync(file), file);
  const previous = ['difficulty-runs', 'hud-insets', 'loading-ready', 'loading-recovery', 'loading-tiers', 'loading-fault-matrix', 'resolution-recovery', 'map-controls', 'weapon-viewmodel', 'asset-reload', 'arsenal'];
  for (const name of previous) assert.ok(names.includes(name), `lost interface regression: ${name}`);
  for (const group of Object.keys(BROWSER_SUITES)) {
    const args = browserTestArgs(group);
    assert.ok(args.includes('--test-concurrency=1'));
    assert.ok(args.includes('--test-timeout=600000'));
  }
  assert.throws(() => browserTestArgs('not-a-suite'), /Unknown browser suite/);
  assert.throws(() => browserTestArgs('constructor'), /Unknown browser suite/);
});
