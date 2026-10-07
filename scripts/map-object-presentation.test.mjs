import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { ENEMY_TEX_BASE, T_ORDNANCE, SPR_FLOATS } from "../src/game/atlas-slots.ts";

test("published WASM keeps map objects out of enemy textures and cues in all 25 sectors", async () => {
  const { instance } = await WebAssembly.instantiate(readFileSync("public/blacksite.wasm"));
  const w = instance.exports;
  w.hs_init(160, 100);
  const hostileKinds = new Set([1, 2, 3, 23, 25]);
  let objects = 0;
  let enemies = 0;
  for (let sector = 1; sector <= 25; sector++) {
    w.hs_prepare_gpu();
    const sprites = new Float32Array(w.memory.buffer, w.hs_gpu_sprites(), w.hs_gpu_sprite_count() * SPR_FLOATS);
    const enemyPositions = new Set();
    for (let i = 0; i < sprites.length; i += SPR_FLOATS) {
      const [x, y, , , texture, , , kind] = sprites.subarray(i, i + SPR_FLOATS);
      if (hostileKinds.has(kind)) {
        enemyPositions.add(`${x},${y}`);
        enemies++;
      } else {
        assert.ok(texture < ENEMY_TEX_BASE || texture >= T_ORDNANCE,
          `sector ${sector}: object kind ${kind} selected enemy texture ${texture}`);
        objects++;
      }
    }
    const count = w.hs_prepare_enemies();
    const cues = new Float32Array(w.memory.buffer, w.hs_enemy_cues(), count * 10);
    for (let i = 0; i < cues.length; i += 10) {
      assert.ok(enemyPositions.has(`${cues[i + 4]},${cues[i + 5]}`),
        `sector ${sector}: map object entered enemy audio/minimap cues`);
    }
    assert.ok(count > 0, `sector ${sector}: genuine enemies must still produce cues`);
    if (sector < 25) w.hs_next_wave();
  }
  assert.ok(objects > 100);
  assert.ok(enemies > 100);
});
