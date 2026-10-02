import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const url = new URL(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/');
url.searchParams.set('qa', '1');
url.searchParams.set('lvl', '11');

test('local qa=1&lvl=11 starts Obsidian Vault with all nineteen guns selectable', async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(url.href, { waitUntil: 'domcontentloaded', timeout: 90000 });
    await page.waitForFunction(() => window.__controlsTest && document.body.innerText.includes('NADIR–7K') && document.body.innerText.includes('HEALTH'), null, { timeout: 90000 });
    assert.equal(await page.evaluate(() => window.__controlsTest.getWeapon()), 0);
    for (let slot = 1; slot <= 19; slot++) {
      await page.locator('.game-canvas').dispatchEvent('wheel', { deltaY: 100 });
      await page.waitForFunction(expected => window.__controlsTest.getWeapon() === expected, slot % 19);
      await page.waitForTimeout(200);
    }
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});

const productionUrl = process.env.BLACKSITE_PRODUCTION_URL;
test('production ignores qa and lvl and starts with the normal first-sector arsenal', { skip: !productionUrl }, async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const url = new URL(productionUrl);
    url.searchParams.set('qa', '1');
    url.searchParams.set('lvl', '11');
    await page.goto(url.href, { waitUntil: 'domcontentloaded', timeout: 90000 });
    await page.waitForFunction(() => !document.querySelector('.deploy-button')?.hasAttribute('disabled'), null, { timeout: 90000 });
    assert.equal(await page.evaluate(() => typeof window.__controlsTest), 'undefined');
    assert.equal(await page.getByText('HEALTH', { exact: true }).count(), 0);
    await page.locator('.deploy-button').click();
    await page.waitForFunction(() => document.body.innerText.includes('NADIR–7A') && document.body.innerText.includes('HEALTH'));
    await page.locator('.game-canvas').dispatchEvent('wheel', { deltaY: 100 });
    await page.locator('.weapon-spiral.visible').waitFor();
    assert.equal(await page.locator('.weapon-spiral-slot').count(), 1);
  } finally { await browser.close(); }
});
