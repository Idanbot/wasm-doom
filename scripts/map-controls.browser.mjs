import {test} from 'node:test';
import assert from 'node:assert/strict';
import {chromium} from 'playwright';
test('sensitivity endpoints persist and minimap pixels match the live row-major layout',async()=>{
 const browser=await chromium.launch({headless:true});
 try {
  const page=await browser.newPage({viewport:{width:1280,height:800}});const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  const base=process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/';
  await page.goto(base);
  await page.getByRole("button",{name:/Enter blacksite/}).waitFor({timeout:120000});
  await page.getByRole('button',{name:'Settings',exact:true}).first().click();
  await page.getByRole('button',{name:'Controls',exact:true}).click();
  const slider=page.getByRole('slider',{name:'Look sensitivity'});
  assert.equal(await slider.getAttribute('min'),'0.1');assert.equal(await slider.getAttribute('max'),'3');
  await slider.focus();await slider.press('Home');assert.equal(await slider.inputValue(),'0.1');
  await slider.press('End');assert.equal(await slider.inputValue(),'3');
  await page.waitForFunction(()=>localStorage.getItem('blacksite-sensitivity')==='3');
  await page.reload();
  await page.getByRole("button",{name:/Enter blacksite/}).waitFor({timeout:120000});
  await page.getByRole('button',{name:'Settings',exact:true}).first().click();
  await page.getByRole('button',{name:'Controls',exact:true}).click();
  assert.equal(await slider.inputValue(),'3');
  await page.screenshot({path:'screenshots/map-controls-settings.png'});
  const url=new URL(base);url.searchParams.set('qa','1');url.searchParams.set('lvl','11');
  await page.goto(url.href);
  await page.waitForFunction(()=>window.__controlsTest?.getReserve()===72,null,{timeout:120000});
  const result=await page.evaluate(()=>{
   const state=window.__controlsTest.getMapState(),canvas=document.querySelector('.automap');
   const ctx=canvas.getContext('2d');const pixels=ctx.getImageData(0,0,canvas.width,canvas.height).data;
   const colors=getComputedStyle(canvas),sample=document.createElement('canvas').getContext('2d');
   const rgb=token=>{sample.fillStyle=colors.getPropertyValue(token).trim();sample.fillRect(0,0,1,1);return [...sample.getImageData(0,0,1,1).data].slice(0,3);};
   const palette={open:rgb('--color-bg'),wall:rgb('--color-elevated'),door:rgb('--color-steel'),secret:rgb('--color-danger')};
   const failures=[];let checked=0;const px=Math.floor(state.x),py=Math.floor(state.y);
   for(let y=py-5;y<=py+5;y++)for(let x=px-5;x<=px+5;x++){
    if(x<0||y<0||x>=state.width||y>=state.height)continue;
    if(Math.abs(x-state.x)<2&&Math.abs(y-state.y)<2)continue;
    if(x===Math.floor(state.nodeX)&&y===Math.floor(state.nodeY))continue;
    if(state.enemies.some(e=>e.hp>0&&Math.abs(x-e.x)<1.5&&Math.abs(y-e.y)<1.5))continue;
    const i=y*state.width+x,cell=state.map[i],opened=state.doors[i]>=.98;
    const expected=cell===0||cell===10||((cell===8||cell===9)&&opened)?palette.open:cell===8?palette.door:cell===9?palette.secret:palette.wall;
    const offset=((y*5+2)*canvas.width+x*5+2)*4,actual=[...pixels.slice(offset,offset+3)];
    if(actual.some((v,i)=>v!==expected[i]))failures.push({x,y,cell,expected,actual});checked++;
   }
   return {checked,failures,width:canvas.width,height:canvas.height};
  });
  assert.ok(result.checked>25);assert.deepEqual(result.failures,[]);assert.equal(result.width,240);assert.equal(result.height,160);
  await page.screenshot({path:'screenshots/map-controls-layout.png'});
  assert.deepEqual(errors,[]);
 } finally {await browser.close();}
});
