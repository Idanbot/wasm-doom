import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

test('extended comparison combines six episodes and fails clearly for a missing sector artifact', async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'blacksite-ai-report-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const artifacts = join(root, 'artifacts'), out = join(root, 'report');
  for (const model of ['laya', 'decider']) for (const sector of [1, 2, 3]) {
    const directory = join(artifacts, `ai-game-${model}-extended-sector-${sector}-1`);
    await mkdir(directory, { recursive: true });
    await writeFile(join(directory, 'report.json'), JSON.stringify({ status: 'PASS',
      model: { kind: model, peakRssMiB: 200 }, finalHud: { wave: sector },
      terminationReason: 'decision_limit', metrics: { kills: sector, decisions: 1, inferenceCount: 1,
        inferenceSeconds: sector, simulationSeconds: 1, emptyReloadOpportunities: 999 }, nodePeakRssMiB: 100, checks: {}, playerWarnings: [] }));
    await writeFile(join(directory, 'trace.jsonl'), JSON.stringify({ inferenceSeconds: sector,
      observation: { magazine: 0, reserve: 10, reloading: false },
      questions: { utility: { criteria: { reload: 'Reload', nothing: 'Wait' } } },
      action: { utility: 'reload' }, answers: { utility: { choice: 'reload', probabilities: { reload: 0.8, nothing: 0.2 } } } }) + '\n');
  }
  const run = () => spawnSync(process.execPath, ['scripts/ai/collect.mjs', artifacts, out],
    { env: { ...process.env, QA_MODELS: 'comparison' }, encoding: 'utf8' });
  assert.equal(run().status, 0);
  let report = JSON.parse(await readFile(join(out, 'report.json')));
  assert.equal(report.aggregates.laya.totals.kills, 6);
  assert.equal(report.aggregates.laya.totals.emptyReloadOpportunities, 3);
  assert.equal(report.aggregates.laya.totals.reloadChoicesOnEmpty, 3);
  assert.equal(report.reloadAvailabilityComputedFromTrace, true);
  assert.equal(report.aggregates.decider.telemetry.confidence.utility.selectedProbability.count, 3);
  assert.equal(report.aggregates.decider.telemetry.inferenceLatencySeconds.p95, 3);
  await rm(join(artifacts, 'ai-game-decider-extended-sector-3-1'), { recursive: true });
  assert.equal(run().status, 1);
  report = JSON.parse(await readFile(join(out, 'report.json')));
  assert.equal(report.status, 'FAIL');
  assert.match(report.failures[0].message, /Expected exactly sectors/);
});
