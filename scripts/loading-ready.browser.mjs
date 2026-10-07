import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

test("loading animates, retries failed assets and publishes a real HUD before deployment", async () => {
  const browser = await chromium.launch({headless:true});
  try {
    const page = await browser.newPage({viewport:{width:1280,height:800}});
    await page.addInitScript(() => localStorage.setItem("blacksite-res", "320"));
    let first = true;
    let release;
    let reached;
    const held = new Promise(resolve => { release = resolve; });
    const requested = new Promise(resolve => { reached = resolve; });
    await page.route("**/game/ui/weapon-thumbs/br12.png", async route => {
      if (first) {
        first = false;
        reached();
        await held;
        await route.fulfill({status:503,body:"temporary asset failure"});
      } else await route.continue();
    });
    const url = process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/";
    await page.goto(url);
    await requested;
    const button = page.locator(".deploy-button");
    assert.equal(await button.isDisabled(), true);
    const sprite = page.locator(".loading-operator img");
    await sprite.evaluate(img => img.decode());
    const transforms = [];
    for(let i=0;i<5;i++) {
      transforms.push(await sprite.evaluate(img => getComputedStyle(img).transform));
      await page.waitForTimeout(240);
    }
    assert.ok(new Set(transforms).size > 2, "loading sprite must advance through multiple poses");
    await page.screenshot({path:"screenshots/loading-operator.png"});
    release();
    await page.getByRole("button", {name:"Retry loading",exact:true}).waitFor({timeout:60000});
    assert.equal(await button.isDisabled(), true);
    await page.getByRole("button", {name:"Retry loading",exact:true}).click();
    await page.evaluate(() => {
      window.loadingSamples = [];
      const sample = () => {
        const meter = document.querySelector('.load-meter');
        if(meter)window.loadingSamples.push(Number(meter.getAttribute('aria-valuenow')));
      };
      new MutationObserver(sample).observe(document.body,{subtree:true,attributes:true,attributeFilter:['aria-valuenow']});
      sample();
    });
    await page.waitForFunction(() => !document.querySelector('.deploy-button')?.disabled, null, {timeout:180000});
    assert.equal(await page.locator('.loading-operator').count(), 0);
    const samples = await page.evaluate(() => window.loadingSamples);
    assert.ok(samples.length > 2);
    assert.ok(samples.every((value,i) => i===0 || value>=samples[i-1]), 'loading progress must not move backwards');
    await button.click();
    await page.getByRole("region",{name:"Vitals"}).waitFor();
    const live = await page.evaluate(() => ({
      enemies:window.__controlsTest.getEnemies().length,
      hostiles:Number(document.querySelector('.hud-threat b').textContent),
      canvas:document.querySelector('.game-canvas').dataset.resolution
    }));
    assert.ok(live.hostiles > 0, "initial HUD cannot show the placeholder zero-hostiles count");
    assert.ok(live.canvas, "renderer must be initialized before deployment");
    await page.screenshot({path:"screenshots/loading-ready.png"});
  } finally { await browser.close(); }
});
