import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

for (const mobile of [false, true]) test(`Settings update date fits its footer (${mobile ? 'mobile' : 'desktop'})`, { timeout: 60000 }, async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: mobile ? { width: 390, height: 844 } : { width: 1280, height: 800 } });
    const errors = []; page.on('pageerror', e => errors.push(e.message));
    await page.addInitScript(() => localStorage.setItem('blacksite-res', '320'));
    await page.goto(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/', { waitUntil: 'domcontentloaded' });
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    const dialog = page.getByRole('dialog');
    await dialog.waitFor();
    const date = dialog.locator('.settings-update time');
    assert.match(await date.getAttribute('datetime'), /^\d{4}-\d{2}-\d{2}$/);
    assert.match(await dialog.locator('.settings-update').innerText(), /^Latest update \d{1,2} [A-Z][a-z]{2} \d{4}$/);
    for (const tab of ['Audio', 'Display', 'Controls', 'Accessibility']) {
      await dialog.getByRole('button', { name: tab, exact: true }).click();
      const bounds = await date.evaluate(el => {
        const a = el.getBoundingClientRect(), b = el.closest('[role=dialog]').getBoundingClientRect();
        return { fits: a.left >= b.left + 8 && a.right <= b.right - 8 && a.top >= b.top && a.bottom <= b.bottom - 8, size: Number.parseFloat(getComputedStyle(el).fontSize) };
      });
      assert.equal(bounds.fits, true, `${tab}: update date clipped`);
      assert.ok(bounds.size <= 12, 'update date should stay small');
    }
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
    const prefix = new URL(page.url()).port === '8082' ? 'settings-pages' : 'settings';
    await dialog.screenshot({ path: `screenshots/${prefix}-update-${mobile ? 'mobile' : 'desktop'}.png` });
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
