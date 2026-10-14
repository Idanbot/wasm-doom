import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

test('real WASM pressure controls vent, discharge, cool and preserve ammunition accounting', { timeout: 240000 }, async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 960, height: 600 } });
    const errors = []; page.on('pageerror', e => errors.push(e.message));
    await page.addInitScript(() => localStorage.setItem('blacksite-res','320'));
    const url = new URL(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/');
    url.searchParams.set('qa','1');
    await page.goto(url.href, { waitUntil:'domcontentloaded' });
    await page.waitForFunction(() => window.__controlsTest?.isLive(), null, { timeout:180000 });
    const cases = await page.evaluate(() => {
      const rt=window.__blacksiteRuntimeTest, w=rt.wasm;
      rt.setPlaying(false); rt.qaOn=false;
      const input=(bits,seconds=1/60)=>{w.hs_qa(bits,1);w.hs_tick(seconds);return rt.readHud();};
      w.hs_qa(0,1);w.hs_qa_armory();input(0);
      const baseline=rt.exportSave();
      const prepare=slot=>{
        // Restore the real checkpoint instead of rebuilding the entire atlas
        // seven times; this also verifies that transient pressure resets.
        rt.loadSave({...baseline,weapon:slot});w.hs_qa_heal();input(0);
        let h=rt.readHud();
        for(let n=0;n<150 && h.pressure<.65;n++)h=input(16,.08);
        return input(0);
      };
      const cases=[];
      for(const slot of [13,20,28]) {
        const warm=prepare(slot), vent=input(4096), release=input(16);
        const after=input(16);
        const cooling=prepare(slot);input(4096);
        let state;
        for(let n=0;n<15;n++)state=input(0,.08);
        const magazineStart=state;
        for(let n=0;n<40;n++)state=input(0,.08);
        cases.push({slot,warm,vent,release,after,cooling,magazineStart,complete:state});
      }
      // Publish a genuine vent state to the HUD for the presentation assertion.
      prepare(28);const visible=input(4096);rt.hud=visible;rt.hooks.onHud(visible,60,'320 × 200');
      return cases;
    });
    for(const c of cases) {
      assert.ok(c.warm.pressure>=.65 && c.warm.pressure<=1,`slot ${c.slot} failed to bank pressure`);
      assert.ok(c.vent.venting>0);
      assert.equal(c.vent.muzzle,0);
      assert.equal(c.vent.reloading,0,'vent must not start magazine animation');
      assert.ok(c.vent.weapFrame<5);
      assert.equal(c.release.ammo,c.vent.ammo-1);
      assert.equal(c.release.reserve,c.vent.reserve);
      assert.equal(c.release.pressure,0);
      assert.equal(c.release.venting,0);
      assert.ok(c.release.muzzle>0 && c.release.pressureRelease>0);
      assert.equal(c.after.ammo,c.release.ammo,'held fire cannot bypass release recovery');
      assert.equal(c.magazineStart.pressure,0);
      assert.equal(c.magazineStart.ammo,c.cooling.ammo,'cooling cannot refill ammunition');
      assert.equal(c.complete.ammo+c.complete.reserve,c.cooling.ammo+c.cooling.reserve);
      assert.equal(c.complete.reloading,0);
    }
    await page.getByRole('region',{name:'Weapon pressure',exact:true}).waitFor();
    assert.match(await page.locator('.hud-pressure').innerText(),/FIRE TO DISCHARGE/);
    await page.screenshot({path:'screenshots/boss-pressure-vent.png'});
    assert.deepEqual(errors,[]);
  } finally { await browser.close(); }
});
