import { test } from "node:test";
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);
const script = join(dirname(fileURLToPath(import.meta.url)), "check-perf.mjs");

function fixture(medianMs, p95Ms) {
  return {
    fixture: "test",
    results: [
      { w: 320, h: 200, yaw: 0, medianMs, p95Ms },
      { w: 640, h: 400, yaw: 0.7, medianMs, p95Ms },
    ],
  };
}

async function run(baseline, current) {
  const dir = mkdtempSync(join(tmpdir(), "perf-"));
  const base = join(dir, "base.json");
  const cur = join(dir, "cur.json");
  writeFileSync(base, JSON.stringify(baseline));
  writeFileSync(cur, JSON.stringify(current));
  try {
    const { stdout } = await execFileAsync(process.execPath, [script, base, cur]);
    return { ok: true, stdout };
  } catch (err) {
    return { ok: false, stdout: (err.stdout ?? "") + (err.stderr ?? "") };
  }
}

test("matching numbers pass", async () => {
  const r = await run(fixture(1, 2), fixture(1, 2));
  assert.equal(r.ok, true);
});

test("median past 1.5x fails", async () => {
  const r = await run(fixture(1, 2), fixture(1.6, 2));
  assert.equal(r.ok, false);
  assert.match(r.stdout, /median/);
});

test("p95 spikes alone do not fail", async () => {
  const r = await run(fixture(1, 2), fixture(1, 9));
  assert.equal(r.ok, true);
});

test("fixture mismatch fails", async () => {
  const base = fixture(1, 2);
  const cur = { fixture: "test", results: [{ w: 99, h: 99, yaw: 0, medianMs: 1, p95Ms: 2 }] };
  const r = await run(base, cur);
  assert.equal(r.ok, false);
});

test("real baseline is well-formed", async () => {
  const { readFileSync } = await import("node:fs");
  const baseline = JSON.parse(
    readFileSync(join(dirname(fileURLToPath(import.meta.url)), "..", "art", "perf-baseline.json"), "utf8"),
  );
  assert.ok(baseline.results.length >= 9);
  for (const r of baseline.results) {
    assert.ok(r.medianMs > 0 && r.p95Ms >= r.medianMs);
  }
});
