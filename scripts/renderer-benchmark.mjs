import { readFile, writeFile } from 'node:fs/promises';
import { performance } from 'node:perf_hooks';

// Measure only the Rust/WASM render call, excluding simulation, upload and presentation.
// Deterministic procedural textures and stationary scenes make revisions comparable.
const { instance } = await WebAssembly.instantiate(await readFile('public/hellscan.wasm'), {});
const e = instance.exports;
const results = [];
for (const [w, h] of [[320, 200], [640, 400], [1280, 800]]) {
  for (const yaw of [0, 0.7, 1.6]) {
    e.hs_init(w, h);
    e.hs_input(0, yaw / 0.0062, 0);
    e.hs_tick(1 / 60);
    for (let i = 0; i < 30; i++) e.hs_render();
    const times = [];
    for (let i = 0; i < 100; i++) {
      const start = performance.now();
      e.hs_render();
      times.push(performance.now() - start);
    }
    times.sort((a, b) => a - b);
    results.push({ w, h, yaw, medianMs: times[50], p95Ms: times[95] });
  }
}
const report = JSON.stringify({ fixture: 'spawn, procedural textures, stationary, 100 frames', results }, null, 2);
console.log(report);
if (process.argv[2]) await writeFile(process.argv[2], report + '\n');
