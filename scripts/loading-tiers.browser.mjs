import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const base=process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/';

test('menu tier and current-sector barrier precede campaign; repeat visits reuse validated persistent bytes',async()=>{
  const browser=await chromium.launch({headless:true});
  try {
    const context=await browser.newContext({viewport:{width:640,height:400}});
    await context.addInitScript(()=>{
      localStorage.setItem('blacksite-res','320');
      const NativeAudio=window.Audio;window.qaMusicElements=[];
      window.Audio=function(src){const element=new NativeAudio(src);window.qaMusicElements.push(element);return element;};
      window.Audio.prototype=NativeAudio.prototype;
    });
    const page=await context.newPage(),requests=[],errors=[];
    page.on('request',request=>requests.push(request.url()));
    page.on('pageerror',e=>errors.push(e.message));
    await page.goto(base);
    await page.waitForFunction(()=>!document.querySelector('.deploy-button')?.disabled || document.querySelector('[role=alert]'),null,{timeout:180000});
    assert.equal(await page.getByRole('alert').count(),0);
    const indexed=path=>requests.findIndex(url=>url.includes(path)&&url.includes('blacksite_version='));
    assert.ok(indexed('/game/ui/menu-reactor.webp')<indexed('/blacksite.wasm'));
    assert.ok(indexed('/game/sectors/upper-works/wall.png')<indexed('/game/sectors/cryogenic-foundry/wall.png'));
    const cached=await page.evaluate(async()=>{
      const cache=await caches.open('blacksite-validated-assets-v1');const keys=await cache.keys();
      return {count:keys.length,bytes:(await Promise.all(keys.map(async key=>Number((await cache.match(key)).headers.get('content-length'))))).reduce((sum,n)=>sum+n,0)};
    });
    assert.ok(cached.count>800);assert.ok(cached.bytes<=384*1024*1024);
    const beds=await page.evaluate(()=>window.qaMusicElements.filter(e=>e.loop).map(e=>e.src));
    assert.ok(beds.length>=1);assert.ok(beds.every(url=>url.startsWith('blob:')),'music playback must reuse validated boot/cache bytes');
    await page.screenshot({path:'screenshots/loading-tier-menu.png'});
    await page.locator('.deploy-button').click();
    await page.waitForFunction(()=>window.qaMusicElements.filter(e=>e.loop).length===3);
    assert.ok(await page.evaluate(()=>window.qaMusicElements.filter(e=>e.loop).every(e=>e.src.startsWith('blob:'))),'field and boss beds reuse the validated music too');
    await page.close();
    const second=await context.newPage(),warm=[];
    second.on('request',request=>{if(request.url().includes('blacksite_version='))warm.push(request.url());});
    second.on('pageerror',e=>errors.push(e.message));
    await second.goto(base);
    await second.waitForFunction(()=>!document.querySelector('.deploy-button')?.disabled || document.querySelector('[role=alert]'),null,{timeout:180000});
    assert.equal(await second.getByRole('alert').count(),0);
    await second.getByRole('button',{name:'Settings',exact:true}).click();
    await second.waitForTimeout(500);
    assert.equal(warm.length,0,'second page and menu playback must reuse validated engine, images and audio bytes');
    assert.deepEqual(errors,[]);
  } finally {await browser.close();}
});
