import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { BOSS_ATTACKS } from '../src/game/boss-attacks.ts';

test('boss warnings and exposed cores render inside their cards on desktop and mobile', async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    for (const viewport of [{ width: 1280, height: 800 }, { width: 390, height: 844 }]) {
      const page = await browser.newPage({ viewport });
      const errors = [];
      page.on('pageerror', e => errors.push(e.message));
      const url = new URL(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/');
      url.searchParams.set('qa', '1');
      await page.goto(url.href, { waitUntil: 'domcontentloaded' });
      await page.waitForFunction(() => window.__controlsTest?.isLive() && document.body.innerText.includes('HEALTH'), null, { timeout: 120000 });
      await page.evaluate(() => { window.__controlsTest.heal(); window.__controlsTest.visitObjective(); window.__controlsTest.triggerBoss(0); });
      await page.waitForFunction(() => window.__controlsTest.getBossAttack().state === 1, null, { timeout: 45000 });
      await page.locator('.hud-boss-warning').waitFor();
      assert.ok((await page.locator('.hud-boss-cue').innerText()).includes(BOSS_ATTACKS[0].name));
      await page.screenshot({ path: `screenshots/boss-counterplay-${viewport.width}-warning.png` });
      await page.waitForFunction(() => window.__controlsTest.getBossAttack().state === 2, null, { timeout: 45000 });
      await page.locator('.hud-boss-cue b').filter({ hasText: 'CORE EXPOSED' }).waitFor();
      const geometry = await page.locator('.hud-boss-cue').evaluate(el => {
        const card = el.closest('.hud-boss').getBoundingClientRect();
        return [...el.querySelectorAll('b,span,time')].map(e => {
          const r = e.getBoundingClientRect();
          return r.left >= card.left + 6 && r.right <= card.right - 6 && r.bottom <= card.bottom - 6;
        });
      });
      assert.ok(geometry.every(Boolean), 'boss instructions touch or cross the panel border');
      await page.screenshot({ path: `screenshots/boss-counterplay-${viewport.width}-recovery.png` });
      await page.evaluate(() => window.__controlsTest.dropBossReward());
      await page.waitForFunction(() => window.__controlsTest.getBossAttack().state === 0);
      assert.deepEqual(errors, []);
      await page.close();
    }
  } finally { await browser.close(); }
});
