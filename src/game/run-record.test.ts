import assert from "node:assert/strict";
import { test } from "node:test";
import {
  accuracy,
  cleanRun,
  decodeRouteMap,
  difficulty,
  encodeRouteMap,
  RunRecorder,
  type RunTotals,
} from "./run-record.ts";
const totals = (time: number, kills = 0): RunTotals => ({
  time,
  kills,
  shots: 10,
  hits: 4,
  ammoSpent: 10,
  damageTaken: 15,
  bosses: 1,
});
const map = new Uint8Array(1536);
test("four presets normalize invalid and legacy selections to Operative", () => {
  for (const id of ["recruit", "operative", "veteran", "blacksite"])
    assert.equal(difficulty(id), id);
  assert.equal(difficulty("nightmare"), "operative");
});
test("accuracy counts successful shots, remains bounded and handles no fire", () => {
  assert.equal(accuracy({ shots: 10, hits: 4 }), 40);
  assert.equal(accuracy({ shots: 0, hits: 0 }), 0);
  assert.equal(accuracy({ shots: 1, hits: 8 }), 100);
});
test("route maps roundtrip walls and reject malformed or oversized records", () => {
  map[23] = 8;
  map[100] = 10;
  map[599] = 4;
  assert.deepEqual(decodeRouteMap(encodeRouteMap(map)), [...map]);
  for (const value of ["1:zzzz", "1:-1", "nan:abc", "0:1"])
    assert.deepEqual(decodeRouteMap(value), []);
});
test("sector splits preserve cumulative stats without double counting time or kills", () => {
  const recorder = new RunRecorder();
  recorder.reset("veteran");
  recorder.sample(1, totals(0), 6, 16, map, 0);
  recorder.sample(1, totals(10_000, 4), 20, 16, map, 2);
  recorder.sample(2, totals(10_000, 4), 7, 6, map, 0);
  recorder.sample(2, totals(15_000, 6), 12, 6, map, 1);
  const run = recorder.record(totals(15_000, 6));
  assert.deepEqual(
    run.sectors.map((s) => [s.time, s.kills, s.completed]),
    [
      [10000, 4, true],
      [5000, 2, false],
    ],
  );
  assert.equal(run.difficulty, "veteran");
  assert.equal(run.bosses, 1);
  const restored = new RunRecorder();
  restored.restore(run, totals(15000, 6), 2);
  restored.sample(2, totals(20000, 9), 16, 6, map, 2);
  assert.deepEqual(
    restored.record(totals(20000, 9)).sectors.map((s) => [s.time, s.kills]),
    [
      [10000, 4],
      [10000, 5],
    ],
  );
});
test("long routes remain bounded, preserve their start and capture the final endpoint", () => {
  const recorder = new RunRecorder();
  for (let i = 0; i < 2000; i++)
    recorder.sample(1, totals(i * 1000), 1 + (i % 46), 1 + (i % 30), map, 0);
  recorder.sample(1, totals(2000000), 40, 25, map, 2);
  const route = recorder.record(totals(2000000)).sectors[0]!.route;
  assert.ok(route.length <= 128);
  assert.deepEqual(route[0], [1, 1]);
  assert.deepEqual(route.at(-1), [40, 25]);
});
test("corrupt stored runs cannot inject nonfinite values or oversized routes", () => {
  const run = cleanRun({
    shots: NaN,
    hits: 99,
    sectors: [
      {
        sector: 1,
        time: Infinity,
        route: [
          [4, 5],
          [NaN, 0],
          [50, 0],
        ],
        map: "1:zzz",
      },
    ],
  })!;
  assert.equal(run.shots, 0);
  assert.equal(run.hits, 0);
  assert.equal(run.sectors[0]!.time, 0);
  assert.deepEqual(run.sectors[0]!.route, [[4, 5]]);
  assert.equal(run.sectors[0]!.map, "");
});
