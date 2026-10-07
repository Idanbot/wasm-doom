import { spawn } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { extendedScenarios } from './scenarios.mjs';

const out = process.env.BLACKSITE_AI_OUTPUT ?? '.blacksite/ai-extended';
const seed = Number(process.env.BLACKSITE_AI_SCENARIO_SEED ?? 1729);
const scenarios = extendedScenarios(seed);
const seconds = Number(process.env.BLACKSITE_AI_TOTAL_SECONDS ?? 1800);
const limit = Number(process.env.BLACKSITE_AI_TOTAL_DECISIONS ?? 512);
if (!Number.isInteger(limit) || limit < scenarios.length || limit > 512 || !Number.isFinite(seconds) || seconds < 1 || seconds > 1800) throw new Error('Invalid shared QA budget');
const start = Date.now(), deadline = start + seconds * 1000;
await mkdir(out, { recursive: true });
const artifacts = join(out, 'episodes');
await mkdir(artifacts, { recursive: true });
let decisions = 0;
for (const [index, fixture] of scenarios.entries()) {
  const remaining = scenarios.length - index;
  const time = Math.max(1, (deadline - Date.now()) / 1000 / remaining);
  const allowance = Math.floor((limit - decisions) / remaining);
  const directory = join(artifacts, `ai-game-decider-extended-${fixture.id}-1`);
  const child = spawn(process.execPath, ['--experimental-strip-types', 'scripts/ai/smoke.mjs'], {
    stdio: 'inherit', env: { ...process.env, BLACKSITE_AI_MODEL: process.env.BLACKSITE_AI_MODEL ?? 'decider',
      BLACKSITE_AI_TIER: 'extended', BLACKSITE_AI_SECTOR: String(fixture.sector),
      BLACKSITE_AI_SCENARIO_KIND: fixture.kind, BLACKSITE_AI_DECISIONS: String(Math.max(1, allowance)),
      BLACKSITE_AI_INFERENCE_SECONDS: String(Math.min(1800, time)), BLACKSITE_AI_WALL_SECONDS: String(time),
      BLACKSITE_AI_FRAMES: '60', BLACKSITE_AI_OUTPUT: directory },
  });
  // Global hard stop includes model setup and replay; never grant another episode
  // a fresh 30-minute allowance. Ordinary deaths release time to remaining cases.
  const timer = setTimeout(() => child.kill('SIGTERM'), Math.max(1, deadline - Date.now()));
  const exit = await new Promise(resolve => {
    child.once('error', e => resolve(e.message));
    child.once('exit', (code, signal) => resolve(code ?? signal));
  });
  clearTimeout(timer);
  try {
    const report = JSON.parse(await readFile(join(directory, 'report.json')));
    decisions += report.metrics.decisions;
    if (exit !== 0) throw new Error(`Episode exited ${exit}`);
  } catch (error) {
    // Missing/failed episodes are diagnosed by the collector, never silently PASS.
    console.error(`${fixture.id}: ${error.message}`);
  }
  if (Date.now() >= deadline || decisions >= limit) break;
}
const collector = spawn(process.execPath, ['scripts/ai/collect.mjs', artifacts, out], {
  stdio: 'inherit', env: { ...process.env, QA_MODELS: process.env.BLACKSITE_AI_MODEL ?? 'decider',
    QA_SCENARIOS: JSON.stringify(scenarios) },
});
const status = await new Promise(resolve => collector.once('exit', resolve));
await writeFile(join(out, 'budget.json'), JSON.stringify({ seed, scenarios, totalDecisionLimit: limit,
  totalWallSecondsLimit: seconds, actualWallSeconds: (Date.now() - start) / 1000, decisions }, null, 2));
process.exitCode = status === 0 ? 0 : 1;
