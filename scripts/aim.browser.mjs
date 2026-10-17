import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const base=process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/';

test('real WASM aim updates every display frame and camera movement interpolates physics', {timeout:120000}, async()=>{
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage();const errors=[];
    page.on('pageerror',error=>errors.push(error.message));
    await page.route('**/__aim.html',route=>route.fulfill({contentType:'text/html',body:'<!doctype html><body></body>'}));
    await page.goto(new URL('__aim.html',base).href);
    const results=await page.evaluate(async()=>{
      const {BlacksiteRuntime}=await import('/src/game/runtime.ts');
      const {instance}=await WebAssembly.instantiate(await (await fetch('/blacksite.wasm')).arrayBuffer(),{});
      const w=instance.exports;
      if(typeof w.hs_look!=='function'||typeof w.hs_camera!=='function')throw Error('Missing responsive camera ABI');
      w.hs_init(160,100);
      const create=()=>{
        w.hs_restart();w.hs_input(0,0,0);w.hs_tick(0);
        const samples=[];let ticks=0;
        const canvas=document.createElement('canvas');document.body.append(canvas);
        const rt=new BlacksiteRuntime(canvas,{onHud(){},onState(){}});
        rt.audio=new Proxy({},{get:(_,name)=>name==='updateEnemies'?()=>[]:()=>{}});
        rt.wasm={...w,hs_tick:dt=>{ticks++;w.hs_tick(dt);}};
        rt.blit={isReady:()=>true,revision:()=>0,drawWorld:frame=>{
          samples.push({x:frame.view[0],y:frame.view[1],yaw:Math.atan2(frame.view[3],frame.view[2]),horizon:frame.view[6]});
          return true;
        }};
        rt.assetsReady=true;rt.gpuReady=true;rt.atlasRevision=0;rt.playing=true;rt.last=0;
        return {rt,samples,ticks:()=>ticks};
      };
      const rates=[];
      for(const hz of [30,60,120,144,240]) {
        const {rt,samples,ticks}=create(),startYaw=w.hs_yaw();
        const frames=hz/3;
        for(let n=1;n<=frames;n++) {
          rt.lookX+=12/frames;rt.lookY-=6/frames;
          rt.frame(n*1000/hz);
        }
        rates.push({hz,frames,ticks:ticks(),yaw:w.hs_yaw()-startYaw,samples,
          queued:rt.lookX+rt.lookY+rt.touchLookX+rt.touchLookY});
      }
      const {rt,samples,ticks}=create();rt.keys.add('KeyW');
      for(let n=1;n<=16;n++)rt.frame(n*1000/240);
      const movement={samples:[...samples],ticks:ticks(),actual:w.hs_x()};
      const beforePause={x:w.hs_x(),yaw:w.hs_yaw()};
      rt.setPlaying(false);rt.lookX=100;rt.frame(100);
      const paused={x:w.hs_x(),yaw:w.hs_yaw(),view:samples.at(-1)};
      // Restore the real pointer-lock condition without requiring a user gesture.
      rt.playing=true;rt.ignoreMouse=0;rt.lookX=0;rt.lookY=0;
      const canvas=rt.canvas;
      Object.defineProperty(document,'pointerLockElement',{configurable:true,get:()=>canvas});
      rt.onMouse(new MouseEvent('mousemove',{movementX:1,movementY:-1}));
      rt.last=100;rt.accumulator=0;const beforeMouse=w.hs_yaw();rt.frame(101);
      const immediate={yaw:w.hs_yaw()-beforeMouse,ticks:ticks(),sample:samples.at(-1)};
      delete document.pointerLockElement;
      return {rates,movement,beforePause,paused,immediate};
    });
    for(const rate of results.rates) {
      assert.equal(rate.ticks,20,`${rate.hz} Hz changed simulation cadence`);
      assert.equal(rate.samples.length,rate.frames);
      assert.ok(Math.abs(rate.yaw-12*.5*.0062)<1e-5,`${rate.hz} Hz lost or doubled mouse input`);
      assert.equal(rate.queued,0);
      for(let i=1;i<rate.samples.length;i++)assert.ok(rate.samples[i].yaw>rate.samples[i-1].yaw,`${rate.hz} Hz repeated the previous aim`);
    }
    const positions=results.movement.samples.map(s=>s.x);
    const deltas=positions.slice(5).map((x,i)=>x-positions[i+4]);
    for(const delta of deltas)assert.ok(delta>.013 && delta<.015,`uneven walking camera step: ${delta}`);
    assert.equal(results.movement.ticks,4);
    assert.deepEqual({x:results.paused.x,yaw:results.paused.yaw},results.beforePause);
    assert.equal(results.paused.view.x,results.beforePause.x);
    assert.ok(Math.abs(results.immediate.yaw-.5*.0062)<1e-6,'mouse event must update aim before the next physics tick');
    assert.equal(results.immediate.ticks,4);
    assert.deepEqual(errors,[]);
  } finally {await browser.close();}
});

test('clicking an already captured canvas keeps pending mouse input', async()=>{
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage();
    await page.route('**/__aim_click.html',route=>route.fulfill({contentType:'text/html',body:'<!doctype html><body></body>'}));
    await page.goto(new URL('__aim_click.html',base).href);
    const result=await page.evaluate(async()=>{
      const {BlacksiteRuntime}=await import('/src/game/runtime.ts');
      const canvas=document.createElement('canvas');document.body.append(canvas);
      const rt=new BlacksiteRuntime(canvas,{onHud(){},onState(){}});
      rt.playing=true;rt.lookX=4;rt.lookY=-2;rt.ignoreMouse=0;
      let requests=0;canvas.requestPointerLock=()=>{requests++;};
      Object.defineProperty(document,'pointerLockElement',{configurable:true,get:()=>canvas});
      rt.requestLock();rt.onMouse(new MouseEvent('mousemove',{movementX:3,movementY:-1}));
      delete document.pointerLockElement;
      return {requests,x:rt.lookX,y:rt.lookY,ignored:rt.ignoreMouse};
    });
    assert.deepEqual(result,{requests:0,x:7,y:-3,ignored:0});
  } finally {await browser.close();}
});

for(const mode of ['webgl2','canvas2d']) {
  test(`wide vertical look renders loaded artwork in ${mode}`, {timeout:210000}, async()=>{
    const browser=await chromium.launch({headless:true});
    try {
      const page=await browser.newPage({viewport:{width:1280,height:800}});
      const errors=[];page.on('pageerror',error=>errors.push(error.message));
      await page.addInitScript(mode=>{
        localStorage.setItem('blacksite-res','320');
        Object.defineProperty(navigator,'gpu',{value:undefined});
        if(mode==='canvas2d') {
          const original=HTMLCanvasElement.prototype.getContext;
          HTMLCanvasElement.prototype.getContext=function(type,...args) {
            return type==='webgl2'||type==='webgpu'?null:original.call(this,type,...args);
          };
        }
      },mode);
      const url=new URL(base);url.searchParams.set('qa','1');
      await page.goto(url.href);
      try {
        await page.waitForFunction(()=>window.__controlsTest?.isLive(),null,{timeout:180000});
      } catch(error) {
        const state=await page.evaluate(()=>({body:document.body.innerText,live:window.__controlsTest?.isLive()}));
        throw new Error(`${mode} failed to start: ${JSON.stringify({state,errors})}`,{cause:error});
      }
      const images=[];
      for(const [name,my] of [['up',-1000],['down',1000]]) {
        await page.evaluate(my=>window.__controlsTest.look(0,my),my);
        await page.waitForTimeout(200);
        const png=await page.locator('.game-canvas').screenshot({
          path:`screenshots/aim-${mode}-${name}.png`,
          style:'.game-canvas ~ * {visibility:hidden !important;}',
        });
        const litFraction=await page.evaluate(async data=>{
          const img=new Image();img.src=`data:image/png;base64,${data}`;await img.decode();
          const canvas=document.createElement('canvas');canvas.width=img.width;canvas.height=img.height;
          const context=canvas.getContext('2d');context.drawImage(img,0,0);
          const pixels=context.getImageData(0,0,canvas.width,canvas.height).data;
          let lit=0;
          for(let i=0;i<pixels.length;i+=4)if(pixels[i]+pixels[i+1]+pixels[i+2]>15)lit++;
          return lit/(pixels.length/4);
        },png.toString('base64'));
        assert.ok(litFraction>.8,`${mode} ${name}: unrendered world (${litFraction})`);
        images.push(png);
      }
      assert.notDeepEqual(images[0],images[1],'up and down must show different world views');
      assert.deepEqual(errors,[]);
    } finally {await browser.close();}
  });
}
