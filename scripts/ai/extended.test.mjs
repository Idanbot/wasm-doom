import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawn } from 'node:child_process';

test('shared-budget runner executes all eleven real fixtures and never grants each a new decision cap', async t => {
  const out = await mkdtemp(join(tmpdir(), 'blacksite-extended-'));
  t.after(() => rm(out, { recursive: true, force: true }));
  const child = spawn(process.execPath, ['--experimental-strip-types', 'scripts/ai/extended.mjs'], {
    stdio: ['ignore', 'ignore', 'pipe'], env: { ...process.env, BLACKSITE_AI_MODEL: 'scripted',
      BLACKSITE_AI_TOTAL_DECISIONS: '11', BLACKSITE_AI_TOTAL_SECONDS: '1800', BLACKSITE_AI_OUTPUT: out },
  });
  let errors = ''; child.stderr.on('data', d => { errors += d; });
  const code = await new Promise(resolve => child.once('exit', resolve));
  assert.equal(code, 0, errors);
  const report = JSON.parse(await readFile(join(out, 'report.json')));
  const budget = JSON.parse(await readFile(join(out, 'budget.json')));
  assert.equal(report.status, 'PASS');
  assert.equal(report.aggregates.scripted.episodes.length, 11);
  assert.equal(report.aggregates.scripted.totals.decisions, 11);
  assert.equal(budget.decisions, 11);
  assert.ok(budget.actualWallSeconds < budget.totalWallSecondsLimit);
  assert.equal(report.aggregates.scripted.episodes.filter(e => e.fixture.kind === 'boss').length, 3);
});
