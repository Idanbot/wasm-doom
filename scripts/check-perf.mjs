#!/usr/bin/env node
// Fail when the WASM renderer regresses past the committed baseline.
// Usage: node scripts/check-perf.mjs <baseline.json> <current.json>
// Medians gate (stable across runs); p95 is reported but never gates
// because a single GC pause on a shared runner can spike it 1.6x.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const [baselinePath, currentPath] = process.argv.slice(2);
if (!baselinePath || !currentPath) {
  console.error("usage: node scripts/check-perf.mjs <baseline.json> <current.json>");
  process.exit(2);
}
const baseline = JSON.parse(readFileSync(baselinePath, "utf8"));
const current = JSON.parse(readFileSync(currentPath, "utf8"));
assert.equal(
  JSON.stringify(current.results.map((r) => [r.w, r.h, r.yaw])),
  JSON.stringify(baseline.results.map((r) => [r.w, r.h, r.yaw])),
  "benchmark fixtures must match the baseline",
);
const errors = [];
const warnings = [];
for (const [i, base] of baseline.results.entries()) {
  const cur = current.results[i];
  const tag = `${cur.w}x${cur.h} yaw=${cur.yaw}`;
  if (cur.medianMs > base.medianMs * 1.5) {
    errors.push(
      `${tag}: median ${cur.medianMs.toFixed(2)}ms > 1.5x baseline ${base.medianMs.toFixed(2)}ms`,
    );
  } else if (cur.medianMs > base.medianMs * 1.25) {
    warnings.push(
      `${tag}: median ${cur.medianMs.toFixed(2)}ms > 1.25x baseline ${base.medianMs.toFixed(2)}ms`,
    );
  }
}
for (const w of warnings) console.warn(`[check:perf] warn: ${w}`);
if (errors.length) {
  for (const e of errors) console.error(`[check:perf] ${e}`);
  process.exit(1);
}
console.log(`[check:perf] no render regression (${current.results.length} fixtures).`);
