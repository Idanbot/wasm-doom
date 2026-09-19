import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const base = process.argv[2] || 'http://127.0.0.1:8080';
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
try {
  for (const mobile of [false, true]) {
    const context = await browser.newContext({
      viewport: mobile ? { width: 390, height: 844 } : { width: 1280, height: 800 },
      hasTouch: mobile, isMobile: mobile,
    });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
    await page.goto(`${base}/?qa=1`);
    await page.waitForFunction(() => document.body.innerText.includes('HEALTH') && window.__controlsTest?.grantWeapons);
    await page.evaluate(() => window.__controlsTest.grantWeapons());
    for (const weapon of [1, 2, 3, 4]) {
      await page.evaluate(async weapon => {
        const t = window.__controlsTest;
        t.setKeys([`Digit${weapon + 1}`]);
        for (let i = 0; i < 4; i++) await new Promise(requestAnimationFrame);
        t.setKeys([]);
      }, weapon);
      await page.waitForFunction(weapon => window.__controlsTest.getWeapon() === weapon, weapon);
      const ammo = await page.evaluate(() => window.__controlsTest.getAmmo());
      await page.evaluate(() => window.__controlsTest.setKeys(['Space']));
      await page.waitForFunction(ammo => window.__controlsTest.getAmmo() < ammo, ammo);
      if (weapon === 2) {
        await page.waitForFunction(() => window.__controlsTest.getSpread() > 0.12);
        const size = await page.locator('.crosshair').evaluate(el => el.getBoundingClientRect().width);
        assert.ok(size > 25);
      }
      await page.evaluate(() => window.__controlsTest.setKeys([]));
      if (weapon === 4) {
        await page.waitForFunction(() => window.__controlsTest.getFirePatches() > 0);
        assert.ok((await page.evaluate(() => window.__controlsTest.getFirePatches())) <= 12);
      }
      await page.screenshot({ path: `screenshots/arsenal-${mobile ? 'mobile' : 'desktop'}-${weapon}.png` });
      if (weapon === 2) await page.waitForFunction(() => window.__controlsTest.getSpread() === 0);
    }
    if (mobile) {
      await page.getByRole('button', { name: 'Next weapon' }).tap();
      await page.waitForFunction(() => window.__controlsTest.getWeapon() === 0);
    }
    assert.deepEqual(errors, []);
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
    console.log(`${mobile ? 'mobile' : 'desktop'}: four weapons, spread feedback/recovery, fire patches and switching passed`);
    await context.close();
  }
} finally { await browser.close(); }
