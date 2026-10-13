import assert from 'node:assert/strict';
import { test } from 'node:test';
import { chromium } from 'playwright';
const base = process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/';
const screenshotPrefix = new URL(base).port === '8082' ? 'difficulty-pages' : 'difficulty';
const record = { name: 'OPERATIVE', wave: 1, kills: 4, time: 10000, difficulty: 'operative', run: { difficulty: 'operative', shots: 10, hits: 4, ammoSpent: 10, damageTaken: 15, bosses: 1, assisted: false, sectors: [{ sector: 1, time: 10000, kills: 4, completed: true, map: '0:16o', route: [[6,16],[8,16],[12,16]] }] } };
for (const mobile of [false, true]) test(`difficulty menu and expandable local sector routes (${mobile ? 'mobile' : 'desktop'})`, { timeout: 240000 }, async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: mobile ? { width: 390, height: 844 } : { width: 1280, height: 800 } });
    const errors = []; page.on('pageerror', e => errors.push(e.message));
    await page.addInitScript(record => {
      localStorage.setItem('blacksite-res', '320');
      if (!localStorage.getItem('blacksite-board')) localStorage.setItem('blacksite-board', JSON.stringify([record, { ...record, name: 'VETERAN', difficulty: 'veteran', run: { ...record.run, difficulty: 'veteran' } }]));
    }, record);
    await page.goto(base, { waitUntil: 'domcontentloaded' });
    await page.locator('.deploy-button:enabled').waitFor({ timeout: 180000 });
    await page.locator('.difficulty-picker').getByRole('button', { name: 'Veteran', exact: true }).click();
    assert.equal(await page.locator('.difficulty-picker').getByRole('button', { name: 'Veteran', exact: true }).getAttribute('aria-pressed'), 'true');
    await page.locator('.menu-records > summary').click();
    assert.match(await page.locator('.run-card > summary').innerText(), /VETERAN/);
    assert.equal(await page.locator('.run-route').count(), 0, 'closed cards do not render route artwork');
    await page.locator('.run-card > summary').click();
    assert.match(await page.locator('.run-totals').innerText(), /40.0%/);
    await page.locator('.run-sector > summary').click();
    assert.equal(await page.locator('.run-route polyline').getAttribute('points'), '6,16 8,16 12,16');
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
    await page.locator('.run-card').screenshot({ path: `screenshots/${screenshotPrefix}-card-${mobile ? 'mobile' : 'desktop'}.png` });
    await page.screenshot({ path: `screenshots/${screenshotPrefix}-records-${mobile ? 'mobile' : 'desktop'}.png`, fullPage: true });
    if (!mobile && await page.evaluate(() => !!window.__blacksiteRuntimeTest)) {
      await page.locator('.deploy-button').click();
      await page.waitForFunction(() => window.__blacksiteRuntimeTest?.getHud().elapsedMs > 100);
      assert.equal(await page.evaluate(() => window.__blacksiteRuntimeTest.getRunRecord().difficulty), 'veteran');
      await page.keyboard.press('Escape');
      await page.getByRole('heading', { name: 'Operation paused' }).waitFor();
      const before = await page.evaluate(() => window.__blacksiteRuntimeTest.getHud().elapsedMs);
      await page.waitForTimeout(350);
      assert.equal(await page.evaluate(() => window.__blacksiteRuntimeTest.getHud().elapsedMs), before);
      await page.evaluate(() => {
        const rt = window.__blacksiteRuntimeTest;
        rt.wasm.hs_qa(0,1); rt.wasm.hs_qa_end(1); rt.setPlaying(true);
      });
      await page.getByRole('heading', { name: 'FLATLINED' }).waitFor();
      await page.locator('.end-name-input').fill('LOCAL');
      await page.getByRole('button', { name: 'Log run', exact: true }).click();
      const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('blacksite-board')).find(r => r.name === 'LOCAL'));
      assert.equal(saved.difficulty, 'veteran'); assert.ok(saved.run.sectors[0].route.length > 0);
    }
    await page.reload({ waitUntil: 'domcontentloaded' });
    await page.waitForFunction(() => document.querySelector('.difficulty-picker button[aria-pressed="true"]')?.textContent === 'Veteran');
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
