import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { readFileSync } from 'node:fs';

const base = process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/';
const readyOrError = (page,{allowError=true}={}) => page.waitForFunction(allowError => {
  const button=document.querySelector('.deploy-button');
  return (button && !button.disabled) || (allowError && document.querySelector('[role=alert]'));
},allowError,{timeout:180000});

async function menu(browser) {
  const page=await browser.newPage({viewport:{width:640,height:400}});
  await page.addInitScript(() => localStorage.setItem('blacksite-res','320'));
  return page;
}

test('manual retry readiness ignores the previous error until deployment is enabled', async () => {
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage();
    await page.setContent('<p role="alert">Previous attempt failed</p><button class="deploy-button" disabled>Deploy</button>');
    let settled=false;
    const waiting=readyOrError(page,{allowError:false}).then(()=>{settled=true;});
    void waiting.catch(()=>{});
    await page.waitForTimeout(100);
    assert.equal(settled,false,'a stale error must not complete the retry wait');
    await page.getByRole('alert').evaluate(element=>element.remove());
    await page.waitForTimeout(100);
    assert.equal(settled,false,'clearing an error does not mean loading finished');
    await page.locator('.deploy-button').evaluate(button=>{button.disabled=false;});
    await waiting;
  } finally {await browser.close();}
});

test('initial readiness waits for a real button and still reports a fresh loading error', async () => {
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage();
    await page.setContent('<body></body>');
    let settled=false;
    const waiting=readyOrError(page).then(()=>{settled=true;});
    void waiting.catch(()=>{});
    await page.waitForTimeout(100);
    assert.equal(settled,false,'an absent deployment button is not ready');
    await page.setContent('<button class="deploy-button" disabled>Deploy</button>');
    await page.waitForTimeout(100);
    assert.equal(settled,false);
    await page.evaluate(()=>{
      const error=document.createElement('p');error.setAttribute('role','alert');
      error.textContent='New loading failure';document.body.append(error);
    });
    await waiting;
    assert.equal(await page.locator('.deploy-button').isDisabled(),true);
  } finally {await browser.close();}
});

test('permanently missing art stops after two retries and allows a fresh manual retry', async () => {
  const browser=await chromium.launch({headless:true});
  try {
    const page=await menu(browser);let attempts=0, recover=false;
    await page.route('**/game/ui/weapon-thumbs/mr4.png*',async route=>{
      attempts++;
      if(recover)await route.continue();
      else await route.fulfill({status:404,body:'missing thumbnail'});
    });
    await page.goto(base);await readyOrError(page);
    assert.equal(attempts,3);
    await page.getByRole('alert').waitFor();
    assert.equal(await page.locator('.deploy-button').isDisabled(),true);
    recover=true;
    await page.getByRole('button',{name:'Retry loading',exact:true}).click();
    // The previous alert can outlive the click until React starts the new boot.
    // A successful retry requires the actual enabled deployment button.
    await readyOrError(page,{allowError:false});
    assert.equal(await page.getByRole('alert').count(),0);
    assert.equal(await page.locator('.deploy-button').isDisabled(),false);
    assert.equal(attempts,4,'manual boot must recover on its first healthy attempt');
  } finally {await browser.close();}
});

test('engine, world, themes and audio manifests recover from missing or corrupt responses', async () => {
  const browser=await chromium.launch({headless:true});
  try {
    const page=await menu(browser);const errors=[];
    page.on('pageerror',error=>errors.push(error.message));
    const cases=[
      ['blacksite.wasm','application/wasm',2],
      ['game/environment/props/datacenter_terminal_console.png','image/png',2],
      ['game/theme/door_hangar.png','image/png',1],
      ['game/voices/manifest.json','application/json',1],
      ['game/sfx/v2/manifest.json','application/json',1],
      [JSON.parse(readFileSync('public/game/voices/manifest.json')).enemies.find(p=>p.lines.length).lines[0].url.slice(1),'audio/wav',2],
      ['game/sfx/v2/fire0.ogg','audio/ogg',1],
      ['game/music/boss.mp3','audio/mpeg',1],
    ];
    const counts={};
    for(const [path,contentType,failures] of cases)await page.route(`**/${path}*`,async route=>{
      counts[path]=(counts[path]??0)+1;
      if(counts[path]<=failures)await route.fulfill({
        status: counts[path]===1 && failures===2 ? 503 : 200,
        contentType,body:'corrupt asset payload',
      });
      else await route.continue();
    });
    // Initial CSS image requests fail too: recovery must update the displayed
    // URLs, rather than merely mark the preload copy as successful.
    for (const name of ['menu-reactor','hud-panel']) await page.route(`**/game/ui/${name}.webp*`, async route => {
      if (new URL(route.request().url()).searchParams.has('blacksite_retry')) await route.continue();
      else await route.fulfill({status:404,body:'cached bad interface art'});
    });
    await page.goto(base);await readyOrError(page);
    assert.equal(await page.getByRole('alert').count(),0);
    assert.equal(await page.locator('.deploy-button').isDisabled(),false);
    for(const [path,,failures] of cases)assert.equal(counts[path],failures+1,path);
    const art = await page.evaluate(async () => {
      const background=getComputedStyle(document.querySelector('.operation-backdrop')).backgroundImage;
      const hud=document.documentElement.style.getPropertyValue('--hud-panel');
      const src=background.slice(5,-2);const image=new Image();image.src=src;await image.decode();
      return {background,hud,width:image.naturalWidth};
    });
    assert.match(art.background,/blob:/);
    assert.match(art.hud,/blob:/);
    assert.ok(art.width>500,'recovered menu background must decode');
    assert.deepEqual(errors,[]);
  } finally {await browser.close();}
});
