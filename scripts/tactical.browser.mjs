import {test} from 'node:test';
import assert from 'node:assert/strict';
import {chromium} from 'playwright';
test('actual secret USE and machinery gunfire produce clues, rewards and tactical reports',async()=>{
 const browser=await chromium.launch({headless:true});
 try{
  const page=await browser.newPage({viewport:{width:1280,height:800}});const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  const url=new URL(process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/');url.searchParams.set('qa','1');url.searchParams.set('lvl','4');
  await page.goto(url.href,{waitUntil:'domcontentloaded'});
  await page.waitForFunction(()=>window.__controlsTest&&document.body.innerText.includes('HEALTH'),null,{timeout:120000});
  await page.waitForFunction(()=>window.__controlsTest.isLive());
  await page.evaluate(()=>window.__controlsTest.visitSecret());
  await page.getByText('SCUFFED SERVICE PANEL — USE E').waitFor();
  await page.screenshot({path:'screenshots/tactical-secret-clue.png'});
  const before=await page.evaluate(()=>window.__controlsTest.getSecrets());
  await page.evaluate(()=>window.__controlsTest.setKeys(['KeyE']));
  await page.waitForFunction(before=>window.__controlsTest.getSecrets()>before,before);
  await page.evaluate(()=>window.__controlsTest.setKeys([]));
  await page.getByText('RECOVERED LOG',{exact:true}).waitFor();
  await page.getByText('SCUFFED SERVICE PANEL — USE E').waitFor({state:'hidden'});
  await page.screenshot({path:'screenshots/tactical-secret-found.png'});
  await page.evaluate(()=>window.__controlsTest.visitMachinery());
  const hint=page.getByText(/CYAN DATA-SERVER TOWER —/);
  await hint.waitFor();
  const box=await hint.boundingBox();
  for(const panel of await page.locator('.hud-plate').all()){const card=await panel.boundingBox();assert.ok(box.y+box.height<=card.y-8,'tactical hint must sit above the HUD');}
  await page.screenshot({path:'screenshots/tactical-machinery-target.png'});
  await page.evaluate(()=>window.__controlsTest.setKeys(['Space']));
  await page.getByText('Network severed. Nearby reinforcement arrivals are blocked.',{exact:true}).waitFor();
  await page.evaluate(()=>window.__controlsTest.setKeys([]));
  await page.screenshot({path:'screenshots/tactical-machinery-disabled.png'});
  assert.deepEqual(errors,[]);
 }finally{await browser.close();}
});
