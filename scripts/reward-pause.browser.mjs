import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const url = new URL(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/');
url.searchParams.set('qa', '1');

test('Escape, P and actual pointer-lock loss pause; Obsidian reward says Echo and can be collected', async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto(url.href);
    await page.waitForFunction(() => window.__controlsTest && document.body.innerText.includes('HEALTH'), null, { timeout: 90000 });
    await page.waitForTimeout(200);
    for (const key of ['p', 'Escape']) {
      await page.keyboard.press(key);
      await page.getByRole('heading', { name: 'Operation paused' }).waitFor();
      const before = await page.evaluate(() => window.__controlsTest.getX());
      await page.keyboard.down('w');
      await page.waitForTimeout(180);
      await page.keyboard.up('w');
      assert.equal(await page.evaluate(() => window.__controlsTest.getX()), before, `${key} didn't freeze simulation`);
      await page.keyboard.press(key);
      await page.waitForFunction(() => !document.body.innerText.includes('Operation paused'));
      await page.waitForTimeout(200);
    }
    // Native Escape may never emit keydown. Exercise the capture-loss event.
    await page.locator('.game-canvas').evaluate((canvas) => {
      canvas.addEventListener('click', () => canvas.requestPointerLock(), { once: true });
    });
    await page.locator('.game-canvas').click();
    await page.waitForFunction(() => !!document.pointerLockElement);
    await page.evaluate(() => document.exitPointerLock());
    await page.getByRole('heading', { name: 'Operation paused' }).waitFor();
    await page.keyboard.press('p');
    await page.waitForFunction(() => !document.body.innerText.includes('Operation paused'));
    await page.evaluate(() => {
      const t = window.__controlsTest;
      for (let i = 1; i < 11; i++) t.nextWave();
      t.dropBossReward();
    });
    await page.getByText(/CLAIM THE MN-6 ECHO/).waitFor();
    assert.equal(await page.getByText(/CLAIM THE HC-9 FORGE/).count(), 0);
    await page.evaluate(() => window.__controlsTest.setKeys(['KeyW']));
    await page.waitForFunction(() => document.body.innerText.includes('OBSIDIAN VAULT CLEARED'), null, { timeout: 30000 });
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    assert.equal(await page.evaluate(() => window.__controlsTest.getWeapon()), 18);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
