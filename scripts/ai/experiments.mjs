#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { performance } from "node:perf_hooks";
import { join } from "node:path";
import { loadScenario } from "./scenarios.mjs";
import { snapshot, fingerprint, step } from "./simulation.mjs";
import {
  progressionMemory,
  remember,
  view,
  compactRequest,
  chooseAnswers,
  referenceChoices,
  executeGoal,
  APPROACHES,
} from "./progression.mjs";
import { startPlayer } from "./model-client.mjs";
import { verifiedCompletion, completionStatus } from "./experiment-results.mjs";
import { summarizeDecisions } from "./metrics.mjs";

const out = process.env.BLACKSITE_AI_OUTPUT ?? ".blacksite/decider-experiments";
const approaches = (process.env.BLACKSITE_AI_APPROACHES ?? APPROACHES.join(",")).split(",");
const repeats = Number(process.env.BLACKSITE_AI_REPEATS ?? 2),
  limit = Number(process.env.BLACKSITE_AI_DECISIONS ?? 96);
const wall = Number(process.env.BLACKSITE_AI_TOTAL_SECONDS ?? 1800),
  sector = Number(process.env.BLACKSITE_AI_SECTOR ?? 1);
const startTurns = (process.env.BLACKSITE_AI_START_TURNS ?? "0").split(",").map(Number);
if (startTurns.some((v) => !Number.isFinite(v) || Math.abs(v) > Math.PI))
  throw new Error("Invalid start turn");
const inferenceTimeoutMs = 120000;
const scripted = process.env.BLACKSITE_AI_MODEL === "scripted";
if (
  !approaches.every((a) => APPROACHES.includes(a)) ||
  new Set(approaches).size !== approaches.length ||
  !Number.isInteger(repeats) ||
  repeats < 1 ||
  repeats > 5 ||
  !Number.isInteger(limit) ||
  limit < 1 ||
  limit > 512 ||
  !Number.isFinite(wall) ||
  wall < 60 ||
  wall > 3600
)
  throw new Error("Invalid experiment configuration");
await mkdir(out, { recursive: true });
const started = performance.now(),
  deadline = started + wall * 1000;
const log = createWriteStream(join(out, "model.log"));
let worker, modelInfo;
const totalDecisions = Number(process.env.BLACKSITE_AI_TOTAL_DECISIONS ?? 512);
if (!Number.isInteger(totalDecisions) || totalDecisions < 1 || totalDecisions > 512)
  throw new Error("Invalid total decisions");
const episodes = [];
let failed = false,
  decisionsUsed = 0;
try {
  if (!scripted) {
    worker = startPlayer({ stderr: log, decisionTimeoutMs: inferenceTimeoutMs });
    modelInfo = await worker.ready();
  }
  // Rotate execution order to avoid giving one approach systematically more wall budget.
  const cases = Array.from({ length: repeats }, (_, r) =>
    approaches.map((_, n) => ({ approach: approaches[(n + r) % approaches.length], repeat: r })),
  ).flat();
  for (const [index, c] of cases.entries()) {
    const remaining = (deadline - performance.now()) / 1000;
    const minimum = scripted ? 1 : worker ? inferenceTimeoutMs / 1000 + 1 : 360;
    if (remaining < minimum || decisionsUsed >= totalDecisions) {
      episodes.push({ ...c, status: "INCONCLUSIVE", reason: "suite_budget" });
      continue;
    }
    if (!scripted && !worker) {
      worker = startPlayer({ stderr: log, decisionTimeoutMs: inferenceTimeoutMs });
      modelInfo = await worker.ready();
    }
    const decisionAllowance = Math.min(
      limit,
      Math.floor((totalDecisions - decisionsUsed) / (cases.length - index)),
    );
    const allowance = remaining / (cases.length - index),
      episodeDeadline = performance.now() + allowance * 1000;
    const startTurn = startTurns[c.repeat % startTurns.length];
    const { w, wasmSha256 } = await loadScenario(sector);
    if (startTurn) step(w, { bits: 0, mx: startTurn / 0.0062, my: 0 }, 1);
    let s = snapshot(w);
    const initialHash = fingerprint(s);
    const memory = progressionMemory(),
      records = [],
      metrics = {
        decisions: 0,
        inferenceSeconds: 0,
        simulationSeconds: 0,
        kills: 0,
        shots: 0,
        reloads: 0,
        objectives: 0,
        bossPhases: 0,
        damage: 0,
        interventions: {},
      };
    const episode = {
      ...c,
      sector,
      startTurn,
      status: "INCONCLUSIVE",
      wasmSha256,
      metrics,
      inventory: "normal fresh game; ordinary pickups only",
      policy:
        c.approach === "commander"
          ? "Decider high-level goals with explicit local tactical executor"
          : "Decider movement/combat/utility decisions",
      model: scripted ? "scripted-reference" : "decider",
    };
    const trace = createWriteStream(join(out, `${c.approach}-${c.repeat}.jsonl`));
    try {
      for (
        let n = 0;
        n < decisionAllowance &&
        s.hud.state === 0 &&
        performance.now() + (!scripted ? inferenceTimeoutMs : 1000) < episodeDeadline;
        n++
      ) {
        remember(s, memory);
        const v = view(s, memory),
          request = compactRequest(v, c.approach);
        decisionsUsed++;
        const result = scripted
          ? { answers: null, seconds: 0 }
          : await worker.decide(request.state, request.questions);
        if (!Number.isFinite(result.seconds) || result.seconds < 0)
          throw new Error("Invalid inference timing");
        if (result.usage?.truncated) throw new Error("Model context truncated");
        const choices = scripted
          ? referenceChoices(v, c.approach)
          : chooseAnswers(result.answers, request.questions);
        const action = executeGoal(w, choices, memory, c.approach, (f, p) => {
          metrics.damage += Math.max(0, p.hud.health - f.hud.health);
          if (f.hud.events & 1) metrics.shots++;
          if (f.hud.events & 4) metrics.reloads++;
          if (f.hud.objective !== p.hud.objective) metrics.objectives++;
          if (f.hud.bossPhase !== p.hud.bossPhase) metrics.bossPhases++;
        });
        s = action.snapshot;
        metrics.modelPeakRssMiB = Math.max(metrics.modelPeakRssMiB ?? 0, result.peakRssMiB ?? 0);
        metrics.decisions++;
        metrics.inferenceSeconds += result.seconds;
        metrics.simulationSeconds += action.frames / 60;
        metrics.kills = s.hud.kills;
        for (const [k, v] of Object.entries(action.interventions))
          metrics.interventions[k] = (metrics.interventions[k] ?? 0) + v;
        const record = {
          decision: n,
          observation: request.state,
          questions: request.questions,
          answers: result.answers,
          choices,
          action: choices,
          modelInference: !scripted,
          frames: action.frames,
          segments: action.segments,
          stateHash: action.stateHash,
          hud: s.hud,
          reason: action.reason,
          inferenceSeconds: result.seconds,
          confidence: result.answers,
          usage: result.usage,
        };
        records.push(record);
        trace.write(JSON.stringify(record) + "\n");
        console.log(
          `${c.approach}/${c.repeat} ${n + 1}: hp ${s.hud.health}, kills ${s.hud.kills}, living ${s.hud.living}, objective ${s.hud.objective}, boss ${s.hud.bossHealth}, gun ${s.hud.weapon + 1}, ammo ${s.hud.ammo}/${s.hud.reserve}, sim ${metrics.simulationSeconds.toFixed(1)}s`,
        );
      }
      episode.finalHud = s.hud;
      episode.completed = s.hud.state === 2;
      episode.status = episode.completed
        ? "COMPLETE"
        : s.hud.state === 1
          ? "PLAYER_FAILURE"
          : "INCONCLUSIVE";
      episode.reason = episode.completed
        ? "sector_won"
        : s.hud.state === 1
          ? "player_death"
          : records.length >= decisionAllowance
            ? "decision_limit"
            : "wall_time_limit";
      // Exact normal-input replay, including all executor interventions.
      const replay = await loadScenario(sector);
      if (startTurn) step(replay.w, { bits: 0, mx: startTurn / 0.0062, my: 0 }, 1);
      if (fingerprint(snapshot(replay.w)) !== initialHash)
        throw new Error("Initial replay divergence");
      for (const r of records) {
        let played;
        for (const seg of r.segments) played = step(replay.w, seg.input, seg.frames);
        if (fingerprint(played.snapshot) !== r.stateHash)
          throw new Error(`Replay divergence at ${r.decision}`);
      }
      episode.deterministicReplay = true;
      episode.telemetry = summarizeDecisions(records);
    } catch (e) {
      episode.status = "SYSTEM_FAILURE";
      episode.reason = e.message;
      failed = true;
      if (worker) {
        await worker.close();
        worker = undefined;
      }
    } finally {
      await new Promise((resolve) => trace.end(resolve));
      episode.finalHud = s.hud;
      episode.telemetry ??= summarizeDecisions(records);
      episodes.push(episode);
      await writeFile(
        join(out, "report.json"),
        JSON.stringify({ schemaVersion: 1, sector, scripted, episodes }, null, 2) + "\n",
      );
    }
  }
} finally {
  await worker?.close();
  await new Promise((resolve) => log.end(resolve));
}
const aggregates = Object.fromEntries(
  approaches.map((a) => {
    const runs = episodes.filter((e) => e.approach === a);
    return [
      a,
      {
        runs: runs.length,
        completed: runs.filter(verifiedCompletion).length,
        deaths: runs.filter((e) => e.status === "PLAYER_FAILURE").length,
        inconclusive: runs.filter((e) => e.status === "INCONCLUSIVE").length,
        systemFailures: runs.filter((e) => e.status === "SYSTEM_FAILURE").length,
      },
    ];
  }),
);
const report = {
  schemaVersion: 1,
  sector,
  scripted,
  modelInfo,
  totalDecisions,
  decisionsUsed,
  systemStatus: failed ? "FAIL" : "PASS",
  completionStatus: completionStatus(episodes),
  wallSeconds: (performance.now() - started) / 1000,
  aggregates,
  episodes,
};
const summary =
  "BLACKSITE DECIDER APPROACH COMPARISON\nCompletion means boss defeated AND reward collected. No debug actions during play.\n" +
  Object.entries(aggregates)
    .map(
      ([a, r]) =>
        `${a}: ${r.completed}/${r.runs} complete, ${r.deaths} deaths, ${r.inconclusive} budget-inconclusive, ${r.systemFailures} system failures`,
    )
    .join("\n") +
  "\n" +
  episodes
    .map(
      (e) =>
        `${e.approach}/${e.repeat}: ${e.status}, ${e.metrics?.decisions ?? 0} decisions, ${(e.metrics?.simulationSeconds ?? 0).toFixed(1)} simulation seconds, HP ${e.finalHud?.health ?? "n/a"}, median inference ${e.telemetry?.inferenceLatencySeconds?.p50?.toFixed(1) ?? "n/a"}s, replay ${e.deterministicReplay ? "PASS" : "unverified"}`,
    )
    .join("\n") +
  `\nSystem: ${report.systemStatus}; completion target: ${report.completionStatus}; wall time: ${report.wallSeconds.toFixed(1)}s; decisions: ${decisionsUsed}/${totalDecisions}\n`;
await writeFile(join(out, "report.json"), JSON.stringify(report, null, 2) + "\n");
await writeFile(join(out, "summary.txt"), summary);
console.log(summary);
process.exitCode = failed ? 1 : 0;
