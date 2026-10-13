import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

test('real WASM reloads eject feeds without port smoke or muzzle illumination', { timeout: 240000 }, async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 960, height: 600 } });
    const errors = []; page.on('pageerror', e => errors.push(e.message));
    await page.addInitScript(() => localStorage.setItem('blacksite-res', '320'));
    const url = new URL(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/'); url.searchParams.set('qa', '1');
    await page.goto(url.href, { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(() => window.__controlsTest?.isLive(), null, { timeout: 180000 });
    const results = await page.evaluate(() => {
      const rt = window.__blacksiteRuntimeTest; rt.setPlaying(false); rt.qaOn = false;
      const w = rt.wasm;
      const smoke = () => {
        w.hs_prepare_gpu();
        const data = new Float32Array(w.memory.buffer, w.hs_gpu_sprites(), w.hs_gpu_sprite_count() * 8);
        let count = 0;
        for (let i = 0; i < data.length; i += 8) {
          if (data[i + 7] === 17 && Math.hypot(data[i] - w.hs_x(), data[i + 1] - w.hs_y()) < 1.25) count++;
        }
        return count;
      };
      const results = [];
      const prepareWeapon = (slot, mag, reserve) => {
        // Loading a checkpoint preserves transient timers. Start each case
        // from a fresh engine, then restore its weapon and ammunition.
        w.hs_restart(); w.hs_qa(0, 1); w.hs_qa_armory(); w.hs_tick(0);
        const save = rt.exportSave(); save.weapon = slot; save.mag[slot] = mag; save.ammo[slot] = reserve;
        rt.loadSave(save);
        w.hs_tick(0);
        if (mag === 0) {
          // Sector entry deliberately refills expansion boss weapons. Empty
          // those magazines through real shots, then let shot effects expire.
          w.hs_qa(16, 1);
          for (let n = 0; rt.readHud().ammo > 0 && n < 500; n++) w.hs_tick(0.08);
          if (rt.readHud().ammo !== 0) throw new Error(`could not empty weapon ${slot}`);
          w.hs_qa(0, 1);
          for (let n = 0; n < 8; n++) w.hs_tick(0.08);
        }
      };
      for (const slot of [0, 1, 7, 10, 20, 24, 32]) {
        prepareWeapon(slot, 0, 100); w.hs_qa(4096, 1); w.hs_tick(1 / 60); w.hs_qa(0, 1);
        let maxSmoke = smoke(), maxMuzzle = rt.readHud().muzzle;
        const reloadFrame = rt.readHud().weapFrame, weapon = rt.readHud().weapon;
        for (let n = 0; n < 30; n++) {
          w.hs_tick(1 / 60); maxSmoke = Math.max(maxSmoke, smoke()); maxMuzzle = Math.max(maxMuzzle, rt.readHud().muzzle);
        }
        results.push({ slot, weapon, maxSmoke, maxMuzzle, reloadFrame });
      }
      prepareWeapon(0, 2, 50); w.hs_qa(16, 1); w.hs_tick(1 / 60);
      const shotMuzzle = rt.readHud().muzzle, shotSmoke = smoke();
      w.hs_qa(4096, 1); w.hs_tick(1 / 60);
      const reloadMuzzle = rt.readHud().muzzle;
      w.hs_qa(0, 1); rt.invalidate();
      return { results, shotMuzzle, shotSmoke, reloadMuzzle };
    });
    for (const r of results.results) {
      assert.equal(r.weapon, r.slot, 'the requested weapon is equipped');
      assert.equal(r.maxSmoke, 0, `port smoke on reload: ${JSON.stringify(r)}`);
      assert.equal(r.maxMuzzle, 0, `muzzle flash on reload: ${JSON.stringify(r)}`);
      assert.equal(r.reloadFrame, 10, `expected the first reload pose: ${JSON.stringify(r)}`);
    }
    assert.ok(results.shotMuzzle > 0); assert.ok(results.shotSmoke > 0);
    assert.equal(results.reloadMuzzle, 0, 'reload immediately clears the previous shot flash');
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
