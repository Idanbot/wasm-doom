import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { readFileSync } from 'node:fs';
const specs = JSON.parse(readFileSync(new URL('../art/enemies-v4/specs.json', import.meta.url)));
test('every sector presents its exclusive enemy after all new animation assets preload', async () => {
 const browser = await chromium.launch({ headless: true });
 try {
  const page = await browser.newPage({viewport:{width:1280,height:800}});
  await page.addInitScript(() => performance.setResourceTimingBufferSize(3000));
  const errors=[],failed=[];page.on('pageerror',e=>errors.push(e.message));
  page.on('response',r=>{if(r.status()>=400&&r.url().includes('/game/enemy_'))failed.push(r.url());});
  const url = new URL(process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/');url.searchParams.set('qa','1');
  await page.goto(url.href,{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>window.__controlsTest&&document.body.innerText.includes('HEALTH'),null,{timeout:180000});
  const loaded=await page.evaluate(()=>performance.getEntriesByType('resource').filter(e=>/\/game\/enemy_sector-.*\.png/.test(e.name)).map(e=>e.name));
  assert.equal(new Set(loaded).size,175,'every sector sheet must finish loading before play');
  for(const enemy of specs.enemies.filter(e=>e.level)) {
   await page.waitForFunction(skin=>window.__controlsTest.getEnemies().some(e=>e.skin===skin&&e.hp>0),enemy.id,{timeout:15000});
   const wrong=await page.evaluate(skin=>window.__controlsTest.getEnemies().filter(e=>e.skin>=37&&e.skin<62&&e.hp>0&&e.skin!==skin),enemy.id);
   assert.deepEqual(wrong,[],`sector ${enemy.level}: wrong themed enemy`);
   if([1,3,11,16,25].includes(enemy.level))await page.screenshot({path:`screenshots/enemy-sector-${enemy.level}.png`});
   if(enemy.level<25)await page.evaluate(()=>window.__controlsTest.nextWave());
  }
  assert.deepEqual(failed,[]);assert.deepEqual(errors,[]);
 } finally {await browser.close();}
});
