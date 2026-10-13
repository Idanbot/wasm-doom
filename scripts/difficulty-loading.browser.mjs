import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

test('difficulty controls stay locked during loading and unlock without restarting boot', { timeout: 240000 }, async () => {
  const browser = await chromium.launch({ headless: true });
  let release;
  const held = new Promise(resolve => { release = resolve; });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    const errors = []; page.on('pageerror', e => errors.push(e.message));
    let engineRequests = 0;
    // The document also preloads the unversioned WASM URL; count loader requests only.
    page.on('request', r => {
      const url = new URL(r.url());
      if (url.pathname.endsWith('/blacksite.wasm') && url.searchParams.has('blacksite_version')) engineRequests++;
    });
    await page.addInitScript(() => {
      localStorage.setItem('blacksite-res', '320');
      localStorage.setItem('blacksite-difficulty', 'veteran');
      window.loadingSamples = [];
      new MutationObserver(() => {
        const meter = document.querySelector('.load-meter');
        if (meter) window.loadingSamples.push(Number(meter.getAttribute('aria-valuenow')));
      }).observe(document, { subtree: true, attributes: true, attributeFilter: ['aria-valuenow'] });
    });
    await page.route('**/game/ui/menu-reactor.webp*', async route => {
      if (new URL(route.request().url()).searchParams.has('blacksite_version')) await held;
      await route.continue();
    });
    const menuRequest = page.waitForRequest(r => {
      const url = new URL(r.url());
      return url.pathname.endsWith('/game/ui/menu-reactor.webp') && url.searchParams.has('blacksite_version');
    }, { timeout: 60000 });
    await Promise.all([
      menuRequest,
      page.goto(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/', { waitUntil: 'domcontentloaded' }),
    ]);
    const picker = page.locator('.difficulty-picker');
    await page.waitForFunction(() => document.querySelector('.difficulty-picker button[aria-pressed=true]')?.textContent === 'Veteran');
    assert.equal(await page.locator('.deploy-button').isDisabled(), true);
    for (const name of ['Recruit', 'Blacksite', 'Operative']) {
      const button = picker.getByRole('button', { name, exact: true });
      assert.equal(await button.isDisabled(), true, `${name} was enabled during loading`);
      await button.evaluate(el => el.click());
    }
    assert.equal(await picker.getByRole('button', { name: 'Veteran', exact: true }).getAttribute('aria-pressed'), 'true');
    assert.equal(await page.evaluate(() => localStorage.getItem('blacksite-difficulty')), 'veteran');
    release();
    await page.locator('.deploy-button:enabled').waitFor({ timeout: 180000 });
    assert.equal(engineRequests, 1, 'difficulty interaction restarted or retried the engine download');
    assert.equal(await page.getByRole('alert').count(), 0);
    const samples = await page.evaluate(() => window.loadingSamples);
    assert.ok(samples.length > 2);
    assert.ok(samples.every((n, i) => i === 0 || n >= samples[i - 1]), 'loading restarted or progress moved backwards');
    const blacksite = picker.getByRole('button', { name: 'Blacksite', exact: true });
    assert.equal(await blacksite.isDisabled(), false);
    await blacksite.click();
    assert.equal(await blacksite.getAttribute('aria-pressed'), 'true');
    assert.equal(await page.evaluate(() => localStorage.getItem('blacksite-difficulty')), 'blacksite');
    await page.locator('.deploy-button').click();
    await page.getByRole('region', { name: 'Vitals' }).waitFor();
    if (await page.evaluate(() => !!window.__blacksiteRuntimeTest)) {
      assert.equal(await page.evaluate(() => window.__blacksiteRuntimeTest.getRunRecord().difficulty), 'blacksite');
    }
    assert.deepEqual(errors, []);
  } finally { release(); await browser.close(); }
});
