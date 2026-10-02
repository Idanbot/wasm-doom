import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

test('Sovereign reward finishes sector 25 and carries into endless sector 26', async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    const url = new URL(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/');
    url.searchParams.set('qa','1'); url.searchParams.set('lvl','25');
    await page.goto(url.href, { waitUntil:'domcontentloaded' });
    await page.waitForFunction(() => window.__controlsTest && document.body.innerText.includes('APEX CONTROL') && document.body.innerText.includes('HEALTH'), null, {timeout:90000});
    await page.evaluate(() => { window.__controlsTest.heal(); window.__controlsTest.triggerBoss(2); });
    await page.getByText('THE SOVEREIGN', {exact:true}).first().waitFor();
    await page.screenshot({ path: 'screenshots/campaign25-sovereign.png' });
    await page.evaluate(() => window.__controlsTest.dropBossReward());
    await page.getByText(/CLAIM THE SV-25 DOMINION/).waitFor();
    await page.evaluate(() => window.__controlsTest.setKeys(['KeyW']));
    try { await page.waitForFunction(() => document.body.innerText.includes('APEX CONTROL CLEARED'), null, {timeout:30000}); }
    catch (error) { await page.screenshot({path:'screenshots/campaign25-transition-failure.png'}); console.log(await page.evaluate(() => ({x:window.__controlsTest?.getX(), y:window.__controlsTest?.getY(), gun:window.__controlsTest?.getWeapon(), text:document.body.innerText}))); throw error; }
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    assert.equal(await page.evaluate(() => window.__controlsTest.getWeapon()),32);
    await page.getByRole('button', {name:/Enter UPPER WORKS/i}).click();
    await page.waitForFunction(() => document.body.innerText.includes('NADIR–7A') && document.body.innerText.includes('HEALTH'));
    assert.equal(await page.evaluate(() => window.__controlsTest.getWeapon()),32);
    await page.screenshot({ path:'screenshots/campaign25-endless.png' });
    assert.deepEqual(errors,[]);
  } finally { await browser.close(); }
});
