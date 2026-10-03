import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { chromium } from 'playwright';
const layouts=JSON.parse(readFileSync(new URL('../art/maps/layouts-10-25.json',import.meta.url)));
test('all rebuilt sectors start in their authored room and render their own boss arena',async()=>{
 const browser=await chromium.launch({headless:true});
 try {
  const page=await browser.newPage({viewport:{width:1280,height:800}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
  const url=new URL(process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/');url.searchParams.set('qa','1');url.searchParams.set('lvl','10');
  await page.goto(url.href,{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>window.__controlsTest&&document.body.innerText.includes('HEALTH'),null,{timeout:90000});
  for(const layout of layouts){
   await page.waitForFunction(name=>document.body.innerText.includes(name.toUpperCase()),layout.name);
   const pose=await page.evaluate(()=>({x:window.__controlsTest.getX(),y:window.__controlsTest.getY()}));
   assert.ok(Math.abs(pose.x-layout.start[0])<.02&&Math.abs(pose.y-layout.start[1])<.02,`level ${layout.level} spawn: ${JSON.stringify(pose)}`);
   await page.evaluate(()=>{window.__controlsTest.heal();window.__controlsTest.visitObjective();window.__controlsTest.triggerBoss(1);});
   await page.locator('.hud-boss').waitFor({state:'visible'});
   await page.waitForFunction(skin=>window.__controlsTest.getEnemies().some(en=>en.skin===skin&&en.hp>0),layout.level+11);
   if([10,16,25].includes(layout.level))await page.screenshot({path:`screenshots/layout-rework-${layout.level}.png`});
   if(layout.level<25)await page.evaluate(()=>window.__controlsTest.nextWave());
  }
  assert.deepEqual(errors,[]);
 }finally{await browser.close();}
});
