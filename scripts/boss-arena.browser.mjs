import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
test('boss arenas preload, warn, activate and clear when their boss dies',async()=>{
 const browser=await chromium.launch({headless:true});
 try{
  const page=await browser.newPage({viewport:{width:1280,height:800}});const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  const url=new URL(process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/');url.searchParams.set('qa','1');
  await page.goto(url.href,{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>window.__controlsTest&&document.body.innerText.includes('HEALTH'),null,{timeout:120000});
  for(let level=1;level<=25;level++){
   await page.evaluate(()=>{window.__controlsTest.heal();window.__controlsTest.visitObjective();window.__controlsTest.triggerBoss(2);});
   // A targeted strike may miss the console refuge; other patterns become visible immediately.
   await page.waitForFunction(()=>window.__controlsTest.getEnemies().some(e=>e.skin>=12),null,{timeout:10000});
   if([1,3,11,16,25].includes(level)){
    await page.waitForFunction(()=>window.__controlsTest.getArena().warning>0,null,{timeout:15000});
    await page.screenshot({path:`screenshots/boss-arena-${level}-warning.png`});
    await page.waitForFunction(()=>window.__controlsTest.getArena().active>0,null,{timeout:15000});
    await page.screenshot({path:`screenshots/boss-arena-${level}-active.png`});
   }
   await page.evaluate(()=>window.__controlsTest.dropBossReward());
   await page.waitForFunction(()=>{const a=window.__controlsTest.getArena();return a.warning===0&&a.active===0;});
   if(level<25)await page.evaluate(()=>window.__controlsTest.nextWave());
  }
  assert.deepEqual(errors,[]);
 }finally{await browser.close();}
});
