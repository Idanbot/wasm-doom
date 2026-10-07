#!/usr/bin/env node
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { createWriteStream } from 'node:fs';
import { performance } from 'node:perf_hooks';
import { join } from 'node:path';
import { startPlayer } from './model-client.mjs';
import { loadSimulation, snapshot, assertValid, observe, questions, validateDecision,
  translate, step, fingerprint, preflight, angle } from './simulation.mjs';

import { loadScenario, scriptedDecision } from './scenarios.mjs';
import { summarizeDecisions, choiceConfidence, reloadAvailability } from './metrics.mjs';

const modelKind = process.env.BLACKSITE_AI_MODEL ?? 'decider';
if (!['laya', 'decider', 'scripted'].includes(modelKind)) throw new Error('Unsupported QA model');
const sector = Number(process.env.BLACKSITE_AI_SECTOR ?? 1);
const config = JSON.parse(await readFile(new URL('./config.json', import.meta.url)));
if (modelKind === 'decider') Object.assign(config, JSON.parse(await readFile(new URL('./decider-config.json', import.meta.url))));
config.decisions = Number(process.env.BLACKSITE_AI_DECISIONS ?? config.decisions);
config.framesPerDecision = Number(process.env.BLACKSITE_AI_FRAMES ?? config.framesPerDecision);
config.inferenceBudgetSeconds = Number(process.env.BLACKSITE_AI_INFERENCE_SECONDS ?? 600);
if (!Number.isInteger(config.decisions) || config.decisions < 1 || config.decisions > 256 || ![30, 60].includes(config.framesPerDecision)) throw new Error('Invalid playtest budget');
if (!Number.isFinite(config.inferenceBudgetSeconds) || config.inferenceBudgetSeconds < 1 || config.inferenceBudgetSeconds > 1800) throw new Error('Invalid inference budget');
config.scenario = `independent-stock-sector-${sector}`;
const out = process.env.BLACKSITE_AI_OUTPUT ?? '.blacksite/ai-smoke';
const started = performance.now();
await mkdir(out, { recursive: true });
const log = createWriteStream(join(out, `${modelKind}.log`));
const trace = createWriteStream(join(out, 'trace.jsonl'));
const report = { schemaVersion: 2, tier: process.env.BLACKSITE_AI_TIER ?? 'smoke', status: 'FAIL', failures: [], playerWarnings: [],
  configuration: config, checks: {}, metrics: { decisions: 0, inferenceCount: 0, inferenceSeconds: 0, inferenceCpuSeconds: 0,
    simulationFrames: 0, distanceMoved: 0, shotsFired: 0, reloads: 0, hitSignals: 0, enemyDamageEvents: 0, kills: 0,
    damageTaken: 0, interactionAttempts: 0, interactionEffectsObserved: 0, visibleEnemyObservations: 0,
    objectiveActivations: 0, bossEncounters: 0, bossPhaseChanges: 0, turnRadians: 0,
    movingSeconds: 0, stationarySeconds: 0, emptyMagazineSeconds: 0, reloadingSeconds: 0,
    visibleCombatSeconds: 0, criticalHealthSeconds: 0, pathDistance: 0, observedEnemyHpLoss: 0,
    ammoConsumed: 0, armorAbsorbed: 0, shotsWithVisibleTarget: 0, weaponChanges: 0,
    emptyReloadOpportunities: 0, reloadChoicesOnEmpty: 0, reloadNotOfferedDecisions: 0, engageOpportunities: 0, engageChoices: 0,
    stuckIntervals: 0, invalidModelOutputs: 0 },
  limitations: ['Sector and boss completion are observations, not pass conditions; this is not a campaign.',
    'Interaction effect is checked only when a nearby door/object is actually encountered.',
    'Survival and combat skill are observations, not CI pass conditions.'] };
let worker, currentFailure = 'integration';
try {
  currentFailure = 'game/system';
  const probe = await loadSimulation();
  report.checks.engineInputs = preflight(probe.w);
  const { w, wasmSha256 } = await loadScenario(sector);
  report.wasmSha256 = wasmSha256;
  let s = snapshot(w); assertValid(s);
  const initialHash = fingerprint(s);
  const records = [];
  currentFailure = 'model/integration';
  worker = modelKind === 'scripted' ? { ready: async () => ({ device: 'cpu', revision: config.revision, sdk: config.sdk, peakRssMiB: 0 }),
    decide: async (o, schema) => ({ answers: scriptedDecision(o, schema), seconds: 0, peakRssMiB: 0 }), close: async () => {} }
    : startPlayer({ stderr: log });
  report.model = { ...(await worker.ready()), kind: modelKind };
  if (report.model.revision !== config.revision || report.model.sdk !== config.sdk) throw new Error('Model pin mismatch');
  report.checks.playerInitializedOnCpu = true;
  report.checks.modelLoadedOnCpu = modelKind !== 'scripted';
  currentFailure = 'game/system';
  let previous;
  for (let n = 0; n < config.decisions && s.hud.state === 0 && report.metrics.inferenceSeconds < config.inferenceBudgetSeconds; n++) {
    const observation = observe(s, previous);
    const schema = questions(observation);
    currentFailure = 'model/integration';
    const result = await worker.decide(observation, schema);
    if (result.usage?.truncated) throw new Error('Model context truncated the observation/questions');
    const validated = validateDecision(result.answers, schema);
    if (!Number.isFinite(result.seconds) || result.seconds < 0) throw new Error('Invalid inference timing');
    if (modelKind !== 'scripted') report.metrics.inferenceCount++;
    report.metrics.inferenceSeconds += result.seconds;
    report.metrics.inferenceCpuSeconds += result.cpuSeconds ?? 0;
    report.model.peakRssMiB = Math.max(report.model.peakRssMiB, result.peakRssMiB);
    if (validated.errors.length) {
      report.metrics.invalidModelOutputs++;
      report.failures.push({ category: 'model/integration', decision: n, errors: validated.errors });
    }
    const input = translate(validated.action, observation);
    if (Object.hasOwn(schema.combat.criteria, 'engage_nearest')) {
      report.metrics.engageOpportunities++;
      if (validated.action.combat === 'engage_nearest') report.metrics.engageChoices++;
    }
    previous = s;
    currentFailure = 'game/system';
    const interval = step(w, input, config.framesPerDecision, (frame) => {
      const event = frame.hud.events;
      const visible = s.enemies.some((e) => e.hp > 0 && e.sight && e.screenX >= 0 && e.screenX <= 1);
      const travel = Math.hypot(frame.hud.x - s.hud.x, frame.hud.y - s.hud.y);
      report.metrics.pathDistance += travel;
      report.metrics[travel > 0.0001 ? 'movingSeconds' : 'stationarySeconds'] += config.tickSeconds;
      if (s.hud.ammo === 0) report.metrics.emptyMagazineSeconds += config.tickSeconds;
      if (s.hud.reloading > 0) report.metrics.reloadingSeconds += config.tickSeconds;
      if (visible) report.metrics.visibleCombatSeconds += config.tickSeconds;
      if (s.hud.health <= 25) report.metrics.criticalHealthSeconds += config.tickSeconds;
      if (frame.hud.weapon !== s.hud.weapon) report.metrics.weaponChanges++;
      if ((event & 1) && visible) report.metrics.shotsWithVisibleTarget++;
      if ((event & 1) && frame.hud.weapon === s.hud.weapon) report.metrics.ammoConsumed += Math.max(0, s.hud.ammo - frame.hud.ammo);
      for (const e of frame.enemies) {
        const old = s.enemies.find((previous) => previous.id === e.id && previous.skin === e.skin);
        if (old) report.metrics.observedEnemyHpLoss += Math.max(0, Math.max(0, old.hp) - Math.max(0, e.hp));
      }
      if (event & 1) report.metrics.shotsFired++;
      if (event & 4) report.metrics.reloads++;
      if (event & 8) report.metrics.hitSignals++;
      // EV_HIT is also raised for environmental interactions. Count enemy damage
      // only when a pistol shot changes an enemy's health or the kill counter.
      if ((event & 1) && (frame.hud.kills > s.hud.kills || frame.enemies.some((e) =>
        s.enemies.some((old) => old.id === e.id && old.skin === e.skin && old.hp > e.hp)))) {
        report.metrics.enemyDamageEvents++;
      }
      report.metrics.damageTaken += Math.max(0, s.hud.health - frame.hud.health);
      report.metrics.armorAbsorbed += Math.max(0, s.hud.armor - frame.hud.armor);
      if (!s.hud.objective && frame.hud.objective) report.metrics.objectiveActivations++;
      if (!s.hud.bossHealth && frame.hud.bossHealth > 0) report.metrics.bossEncounters++;
      if (s.hud.bossPhase !== frame.hud.bossPhase) report.metrics.bossPhaseChanges++;
      report.metrics.turnRadians += Math.abs(angle(frame.hud.yaw - s.hud.yaw));
      s = frame;
      // Interrupt held actions for changes a player can observe.
      return !(frame.hud.health <= previous.hud.health - 8 ||
        frame.hud.kills > previous.hud.kills ||
        (previous.hud.ammo > 0 && frame.hud.ammo === 0) ||
        (previous.hud.reloading > 0 && frame.hud.reloading === 0) ||
        frame.hud.bossPhase !== previous.hud.bossPhase ||
        (observation.visibleEnemies.length === 0 && observe(frame).visibleEnemies.length > 0));
    });
    s = interval.snapshot;
    if (s.hud.state === 0 && s.hud.elapsedMs <= previous.hud.elapsedMs) throw new Error('Simulation clock stopped');
    const moved = Math.hypot(s.hud.x - previous.hud.x, s.hud.y - previous.hud.y);
    report.metrics.distanceMoved += moved;
    if ((input.bits & 15) && moved < 0.01) report.metrics.stuckIntervals++;
    if (input.bits & 64) {
      report.metrics.interactionAttempts++;
      if ((interval.events & 1024) || s.hud.objective !== previous.hud.objective ||
          s.hud.radioSeq !== previous.hud.radioSeq) report.metrics.interactionEffectsObserved++;
    }
    report.metrics.visibleEnemyObservations += observation.visibleEnemies.length;
    report.metrics.decisions++;
    report.metrics.simulationFrames += interval.frames;
    const record = { decision: n, observation, questions: schema, answers: result.answers,
      action: validated.action, input, frames: interval.frames, stateHash: fingerprint(s),
      hud: s.hud, inferenceSeconds: result.seconds, inferenceCpuSeconds: result.cpuSeconds ?? null,
      modelInference: modelKind !== 'scripted',
      confidence: Object.fromEntries(Object.entries(schema).map(([name, question]) =>
        [name, choiceConfidence(result.answers?.[name], question.criteria)])), usage: result.usage };
    records.push(record); trace.write(JSON.stringify(record) + '\n');
  }
  if (report.metrics.decisions === 0) throw new Error('No model decisions executed');
  report.finalHud = s.hud;
  report.metrics.kills = s.hud.kills;
  report.metrics.deaths = s.hud.state === 1 ? 1 : 0;
  report.metrics.sectorsCompleted = s.hud.state === 2 ? 1 : 0;
  report.terminationReason = s.hud.state === 1 ? 'player_death' : s.hud.state === 2 ? 'sector_won'
    : report.metrics.inferenceSeconds >= config.inferenceBudgetSeconds ? 'inference_time_limit' : 'decision_limit';
  report.decisionTelemetry = summarizeDecisions(records);
  Object.assign(report.metrics, reloadAvailability(records));
  report.metrics.simulationSeconds = report.metrics.simulationFrames * config.tickSeconds;
  report.metrics.simulationSecondsPerInferenceSecond = report.metrics.inferenceSeconds > 0
    ? report.metrics.simulationSeconds / report.metrics.inferenceSeconds : null;
  report.metrics.inferenceAverageCpuCores = report.metrics.inferenceSeconds > 0
    ? report.metrics.inferenceCpuSeconds / report.metrics.inferenceSeconds : null;
  report.checks.agentMoved = report.metrics.distanceMoved > 0.1;
  report.checks.agentFired = report.metrics.shotsFired > 0;
  report.checks.agentReloaded = report.metrics.reloads > 0;
  report.checks.enemyEncountered = report.metrics.visibleEnemyObservations > 0;
  report.checks.damageDealt = report.metrics.enemyDamageEvents > 0;
  report.checks.stateRemainedValid = true;
  if (s.hud.state === 1) report.playerWarnings.push('Player died during normal gameplay.');
  if (report.metrics.inferenceSeconds >= config.inferenceBudgetSeconds) report.playerWarnings.push('Inference time budget exhausted; episode ended at a valid decision boundary.');
  if (!report.checks.agentMoved) report.playerWarnings.push('Model did not choose effective movement.');
  if (s.hud.ammo === 0 && s.hud.reserve > 0 && !report.checks.agentReloaded) report.playerWarnings.push('Model did not reload its empty magazine.');
  if (report.metrics.stuckIntervals > 3) report.playerWarnings.push('Model attempted movement into geometry repeatedly.');
  if (report.metrics.turnRadians > 6 * Math.PI && report.metrics.distanceMoved < 1) report.playerWarnings.push('Repeated spinning without effective movement; inspect trace.');
  if (Math.abs(angle(s.hud.yaw - previous?.hud.yaw)) > 1) report.playerWarnings.push('Large final turn; inspect trace.');
  // Replay recorded inputs, not model predictions: CPU inference timing cannot affect the clock.
  const replay = await loadScenario(sector);
  if (fingerprint(snapshot(replay.w)) !== initialHash) throw new Error('Initial conditions diverged');
  for (const record of records) {
    const result = step(replay.w, record.input, record.frames);
    if (fingerprint(result.snapshot) !== record.stateHash) throw new Error(`Deterministic replay diverged at decision ${record.decision}`);
  }
  report.checks.deterministicReplay = true;
  report.status = report.failures.length ? 'FAIL' : 'PASS';
} catch (error) {
  report.failures.push({ category: currentFailure, message: error.message, stack: error.stack });
} finally {
  await worker?.close();
  await Promise.all([new Promise((resolve) => log.end(resolve)), new Promise((resolve) => trace.end(resolve))]);
  report.wallSeconds = (performance.now() - started) / 1000;
  report.nodePeakRssMiB = process.resourceUsage().maxRSS / 1024;
  report.terminationReason ??= 'system_failure';
  const observed = (v) => v ? 'YES' : 'NO (not required in smoke)';
  const summary = `BLACKSITE ${modelKind.toUpperCase()} SECTOR ${sector} TEST\n\n` +
    `CPU player initialized: ${report.checks.playerInitializedOnCpu ? 'PASS' : 'FAIL'}\n` +
    `Engine input preflight: ${report.checks.engineInputs ? 'PASS' : 'FAIL'}\n` +
    `Decisions / inference calls: ${report.metrics.decisions} / ${report.metrics.inferenceCount}\n` +
    `Simulation time: ${(report.metrics.simulationSeconds ?? 0).toFixed(2)} sec\n` +
    `Agent moved / fired / reloaded: ${observed(report.checks.agentMoved)} / ${observed(report.checks.agentFired)} / ${observed(report.checks.agentReloaded)}\n` +
    `Enemy encountered / damage dealt: ${observed(report.checks.enemyEncountered)} / ${observed(report.checks.damageDealt)}\n` +
    `Deterministic input replay: ${report.checks.deterministicReplay ? 'PASS' : 'FAIL'}\n` +
    `Wall time: ${report.wallSeconds.toFixed(2)} sec\n` +
    `Termination: ${report.terminationReason}\n` +
    `Inference median / p95: ${report.decisionTelemetry?.inferenceLatencySeconds.p50?.toFixed(2) ?? 'n/a'} / ${report.decisionTelemetry?.inferenceLatencySeconds.p95?.toFixed(2) ?? 'n/a'} sec\n` +
    `Mean chosen probability (movement/combat/utility): ${['movement', 'combat', 'utility'].map((name) => report.decisionTelemetry?.confidence[name]?.selectedProbability.mean?.toFixed(3) ?? 'n/a').join(' / ')} (diagnostic only)\n` +
    `Moving / empty magazine / critical health: ${report.metrics.movingSeconds.toFixed(2)} / ${report.metrics.emptyMagazineSeconds.toFixed(2)} / ${report.metrics.criticalHealthSeconds.toFixed(2)} sec\n` +
    `Player warnings: ${report.playerWarnings.join('; ') || 'none'}\n` +
    `System/integration failures: ${JSON.stringify(report.failures)}\n\nRESULT: ${report.status}\n`;
  await writeFile(join(out, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await writeFile(join(out, 'summary.txt'), summary);
  console.log(summary);
}
process.exitCode = report.status === 'PASS' ? 0 : 1;
