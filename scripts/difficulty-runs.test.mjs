import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
const binary = readFileSync(new URL('../public/blacksite.wasm', import.meta.url));
test('WASM difficulty is bounded, locked after play begins, and preserved across restart and sectors', async () => {
  for (let preset = 0; preset < 4; preset++) {
    const { instance } = await WebAssembly.instantiate(binary, {});
    const w = instance.exports; w.hs_init(160, 100);
    assert.equal(w.hs_run_stat(5), 1);
    assert.equal(w.hs_set_difficulty(-1), 0); assert.equal(w.hs_set_difficulty(4), 0);
    assert.equal(w.hs_set_difficulty(preset), 1);
    w.hs_tick(1 / 60);
    assert.equal(w.hs_set_difficulty((preset + 1) % 4), 0);
    assert.equal(w.hs_run_stat(5), preset);
    w.hs_next_wave(); assert.equal(w.hs_run_stat(5), preset);
    w.hs_restart(); assert.equal(w.hs_run_stat(5), preset);
    for (let i = 0; i < 5; i++) assert.equal(w.hs_run_stat(i), 0);
    w.hs_input(16, 0, 0); w.hs_tick(1 / 60);
    assert.equal(w.hs_run_stat(0), 1); assert.equal(w.hs_run_stat(2), 1);
    w.hs_restore_run_stats(20, 8, 19, 17, 2, preset);
    assert.deepEqual(Array.from({ length: 6 }, (_, i) => w.hs_run_stat(i)), [20, 8, 19, 17, 2, preset]);
    assert.equal(w.hs_run_stat(-1), 0);
    w.hs_restore_run_stats(1, 99, 0, 0, 0, 99);
    assert.equal(w.hs_run_stat(1), 1); assert.equal(w.hs_run_stat(5), 3);
  }
});
