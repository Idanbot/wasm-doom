import { readdir, readFile, mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { summarizeDecisions } from './metrics.mjs';

const root = process.argv[2] ?? '.blacksite/ai-artifacts';
const out = process.argv[3] ?? '.blacksite/ai-comparison';
const models = {}, failures = [];
for (const entry of await readdir(root, { withFileTypes: true })) {
  if (!entry.isDirectory() || !entry.name.startsWith('ai-game-')) continue;
  try {
    const directory = join(root, entry.name);
    const report = JSON.parse(await readFile(join(directory, 'report.json')));
    const records = (await readFile(join(directory, 'trace.jsonl'), 'utf8')).trim().split('\n').filter(Boolean).map((s) => JSON.parse(s));
    const name = report.model.kind;
    models[name] ??= { reports: [], records: [] };
    models[name].reports.push(report); models[name].records.push(...records);
    if (report.status !== 'PASS') failures.push({ artifact: entry.name, failures: report.failures });
  } catch (error) { failures.push({ artifact: entry.name, message: error.message }); }
}
if (!Object.keys(models).length) failures.push({ message: 'No model episode reports found' });
const expected = process.env.QA_MODELS === 'laya' ? ['laya'] : process.env.QA_MODELS === 'decider' ? ['decider'] : ['laya', 'decider'];
for (const name of expected) {
  const sectors = models[name]?.reports.map((r) => r.finalHud?.wave).sort() ?? [];
  if (JSON.stringify(sectors) !== '[1,2,3]') failures.push({ model: name, message: 'Expected exactly sectors 1, 2 and 3', sectors });
}
const aggregates = {};
for (const [name, { reports, records }] of Object.entries(models)) {
  const sum = (key) => reports.reduce((total, r) => total + (r.metrics[key] ?? 0), 0);
  const totals = Object.fromEntries(['decisions', 'inferenceCount', 'inferenceSeconds', 'inferenceCpuSeconds',
    'simulationSeconds', 'kills', 'deaths', 'sectorsCompleted', 'shotsFired', 'reloads', 'damageTaken',
    'observedEnemyHpLoss', 'armorAbsorbed', 'ammoConsumed', 'pathDistance', 'movingSeconds', 'stationarySeconds',
    'emptyMagazineSeconds', 'criticalHealthSeconds', 'reloadingSeconds', 'visibleCombatSeconds',
    'shotsWithVisibleTarget', 'emptyReloadOpportunities', 'reloadChoicesOnEmpty', 'engageOpportunities',
    'engageChoices', 'interactionAttempts', 'interactionEffectsObserved', 'objectiveActivations',
    'bossEncounters', 'bossPhaseChanges', 'stuckIntervals', 'invalidModelOutputs'].map((key) => [key, sum(key)]));
  aggregates[name] = { totals, telemetry: summarizeDecisions(records),
    modelPeakRssMiB: Math.max(...reports.map((r) => r.model.peakRssMiB)),
    nodePeakRssMiB: Math.max(...reports.map((r) => r.nodePeakRssMiB)),
    terminationReasons: reports.reduce((counts, r) => {
      counts[r.terminationReason] = (counts[r.terminationReason] ?? 0) + 1; return counts;
    }, {}), episodes: reports.map((r) => ({ sector: r.finalHud?.wave, status: r.status,
      terminationReason: r.terminationReason, finalHud: r.finalHud, metrics: r.metrics,
      checks: r.checks, warnings: r.playerWarnings })) };
}
const status = failures.length ? 'FAIL' : 'PASS';
const summary = `BLACKSITE EXTENDED CPU COMPARISON\nIndependent sector starts; confidence is diagnostic, not validated gameplay accuracy.\n\n` +
  Object.entries(aggregates).map(([name, r]) => {
    const t = r.totals, c = r.telemetry.confidence;
    return `${name}: ${t.decisions} decisions / ${t.inferenceSeconds.toFixed(1)}s inference / ${t.simulationSeconds.toFixed(1)}s gameplay\n` +
      `  Kills / deaths / completed sectors: ${t.kills} / ${t.deaths} / ${t.sectorsCompleted}\n` +
      `  Reloads / empty-mag time: ${t.reloads} / ${t.emptyMagazineSeconds.toFixed(1)}s\n` +
      `  Empty-mag reload choices / opportunities: ${t.reloadChoicesOnEmpty} / ${t.emptyReloadOpportunities}\n` +
      `  Latency median / p95: ${r.telemetry.inferenceLatencySeconds.p50?.toFixed(2) ?? 'n/a'} / ${r.telemetry.inferenceLatencySeconds.p95?.toFixed(2) ?? 'n/a'}s\n` +
      `  Mean chosen probability movement/combat/utility: ${['movement', 'combat', 'utility'].map((key) => c[key]?.selectedProbability.mean?.toFixed(3) ?? 'n/a').join(' / ')}\n` +
      `  Objective / boss activations: ${t.objectiveActivations} / ${t.bossEncounters}\n` +
      `  Terminations: ${JSON.stringify(r.terminationReasons)}\n`;
  }).join('\n') + `\nSystem/integration failures: ${JSON.stringify(failures)}\nRESULT: ${status}\n`;
await mkdir(out, { recursive: true });
await writeFile(join(out, 'report.json'), JSON.stringify({ schemaVersion: 2, status, aggregates, failures }, null, 2) + '\n');
await writeFile(join(out, 'summary.txt'), summary);
console.log(summary);
process.exitCode = status === 'PASS' ? 0 : 1;
