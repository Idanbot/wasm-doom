import { spawn } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { SECTORS } from './scenarios.mjs';

const out = process.env.BLACKSITE_AI_OUTPUT ?? '.blacksite/ai-playtest';
const model = process.env.BLACKSITE_AI_MODEL ?? 'laya';
await mkdir(out, { recursive: true });
const results = [];
for (const sector of SECTORS) {
  // Sequential child processes release WASM/model memory between sectors.
  const directory = join(out, `sector-${sector}`);
  const child = spawn(process.execPath, ['--experimental-strip-types', 'scripts/ai/smoke.mjs'], {
    stdio: 'inherit', env: { ...process.env, BLACKSITE_AI_MODEL: model,
      BLACKSITE_AI_TIER: 'short', BLACKSITE_AI_SECTOR: String(sector), BLACKSITE_AI_DECISIONS: '48',
      BLACKSITE_AI_INFERENCE_SECONDS: '180',
      BLACKSITE_AI_FRAMES: '60', BLACKSITE_AI_OUTPUT: directory },
  });
  const exit = await new Promise((resolve) => {
    child.once('error', (error) => resolve(error.message));
    child.once('exit', (code, signal) => resolve(code ?? signal));
  });
  let report;
  try { report = JSON.parse(await readFile(join(directory, 'report.json'))); }
  catch { report = { status: 'FAIL', failures: [{ category: 'integration', message: `Harness exited: ${exit}` }] }; }
  if (exit !== 0) report.status = 'FAIL';
  results.push({ sector, ...report });
}
const status = results.every((r) => r.status === 'PASS') ? 'PASS' : 'FAIL';
const summary = `BLACKSITE ${model.toUpperCase()} SHORT PLAYTEST\nIndependent sector starts, not a completed campaign.\n` +
  results.map((r) => `Sector ${r.sector}: ${r.status}; decisions ${r.metrics?.decisions ?? 0}; inferences ${r.metrics?.inferenceCount ?? 0} / ${(r.metrics?.inferenceSeconds ?? 0).toFixed(1)}s; simulation ${(r.metrics?.simulationSeconds ?? 0).toFixed(1)}s; kills ${r.metrics?.kills ?? 0}; reloads ${r.metrics?.reloads ?? 0}; deaths ${r.metrics?.deaths ?? 0}; sector completed ${r.finalHud?.state === 2 ? 'YES' : 'NO'}; warnings ${r.playerWarnings?.join('; ') || 'none'}`).join('\n') + `\nRESULT: ${status}\n`;
await writeFile(join(out, 'report.json'), JSON.stringify({ schemaVersion: 1, tier: 'short', model, status, results }, null, 2) + '\n');
await writeFile(join(out, 'summary.txt'), summary);
console.log(summary);
process.exitCode = status === 'PASS' ? 0 : 1;
