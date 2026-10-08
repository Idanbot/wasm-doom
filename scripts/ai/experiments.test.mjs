import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

test("bounded reference experiment reports unfinished gameplay separately from system failure", () => {
  const out = mkdtempSync(join(tmpdir(), "blacksite-experiment-"));
  try {
    const run = spawnSync(
      process.execPath,
      ["--experimental-strip-types", "scripts/ai/experiments.mjs"],
      {
        env: {
          ...process.env,
          BLACKSITE_AI_MODEL: "scripted",
          BLACKSITE_AI_APPROACHES: "commander",
          BLACKSITE_AI_DECISIONS: "1",
          BLACKSITE_AI_TOTAL_DECISIONS: "1",
          BLACKSITE_AI_REPEATS: "1",
          BLACKSITE_AI_TOTAL_SECONDS: "60",
          BLACKSITE_AI_OUTPUT: out,
        },
        encoding: "utf8",
        timeout: 60_000,
      },
    );
    assert.equal(run.status, 0, run.stderr);
    const report = JSON.parse(readFileSync(join(out, "report.json"), "utf8"));
    assert.equal(report.systemStatus, "PASS");
    assert.equal(report.completionStatus, "INCONCLUSIVE");
    assert.equal(report.decisionsUsed, 1);
    assert.equal(report.episodes[0].reason, "decision_limit");
    assert.equal(report.episodes[0].deterministicReplay, true);
    assert.equal(report.episodes[0].model, "scripted-reference");
    assert.equal(report.episodes[0].telemetry.inferenceLatencySeconds.count, 0);
    assert.match(readFileSync(join(out, "summary.txt"), "utf8"), /decisions: 1\/1/);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
});

test("duplicate policy names cannot overwrite experiment traces or inflate win rates", () => {
  const run = spawnSync(
    process.execPath,
    ["--experimental-strip-types", "scripts/ai/experiments.mjs"],
    {
      env: {
        ...process.env,
        BLACKSITE_AI_MODEL: "scripted",
        BLACKSITE_AI_APPROACHES: "commander,commander",
      },
      encoding: "utf8",
      timeout: 10_000,
    },
  );
  assert.notEqual(run.status, 0);
  assert.match(run.stderr, /Invalid experiment configuration/);
});
