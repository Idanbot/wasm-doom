import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

test("WebGPU effect shader and 192-byte uniform layout validate", async (t) => {
  const browser = await chromium.launch({headless:true,args:["--enable-unsafe-webgpu","--use-angle=swiftshader"]});
  try {
    const page = await browser.newPage();
    await page.route("**/__worldfx_gpu.html",r=>r.fulfill({contentType:"text/html",body:"<body></body>"}));
    await page.goto(new URL("/__worldfx_gpu.html",process.env.BLACKSITE_TEST_URL??"http://127.0.0.1:8080/").href);
    const result = await page.evaluate(async()=>{
      const adapter = await navigator.gpu?.requestAdapter();
      if (!adapter) return {unavailable:true};
      const device = await adapter.requestDevice();
      const source = await (await fetch("/src/game/blit.ts")).text();
      const code = source.match(/const POST_WGSL = `([\s\S]*?)`;/)?.[1];
      if (!code) throw new Error("Post shader source missing");
      const shader = device.createShaderModule({code});
      const messages = (await shader.getCompilationInfo()).messages.filter(m=>m.type==="error").map(m=>m.message);
      device.pushErrorScope("validation");
      const layout = device.createBindGroupLayout({entries:[
        {binding:0,visibility:GPUShaderStage.FRAGMENT,texture:{sampleType:"float"}},
        {binding:1,visibility:GPUShaderStage.FRAGMENT,texture:{sampleType:"float"}},
        {binding:2,visibility:GPUShaderStage.FRAGMENT,buffer:{type:"uniform",minBindingSize:192}},
      ]});
      const pipeline = await device.createRenderPipelineAsync({layout:device.createPipelineLayout({bindGroupLayouts:[layout]}),
        vertex:{module:shader,entryPoint:"vs"},fragment:{module:shader,entryPoint:"fs",targets:[{format:"bgra8unorm"}]}});
      const error = await device.popErrorScope(); device.destroy();
      return {messages,error:error?.message??null,pipeline:!!pipeline};
    });
    if (result.unavailable) {t.skip("WebGPU adapter unavailable on this runner");return;}
    assert.deepEqual(result.messages,[]);assert.equal(result.error,null);assert.equal(result.pipeline,true);
  } finally {await browser.close();}
});

test("GPU refraction respects depth; lit dust responds to muzzle light; HD casing frames stay isolated", async () => {
  const browser = await chromium.launch({headless:true});
  try {
    const page = await browser.newPage({viewport:{width:960,height:600}});
    const errors=[]; page.on("pageerror",e=>errors.push(e.message));
    await page.addInitScript(()=>Object.defineProperty(navigator,"gpu",{value:undefined}));
    await page.route("**/__worldfx_test.html",r=>r.fulfill({contentType:"text/html",body:"<!doctype html><html><body></body></html>"}));
    await page.goto(new URL("/__worldfx_test.html",process.env.BLACKSITE_TEST_URL??"http://127.0.0.1:8080/").href);
    const result=await page.evaluate(async()=>{
      const {createBlitter}=await import("/src/game/blit.ts");
      const c=document.createElement("canvas"); c.style.cssText="width:960px;height:600px";document.body.append(c);
      const b=await createBlitter(c); b.setGfx({crt:false,bloom:false,fog:false});
      const w=320,h=200, data=new Uint8Array(w*h*4);
      for(let y=0;y<h;y++)for(let x=0;x<w;x++)data.set([25+(x%23)*6,30+(y%21)*6,60,128],(y*w+x)*4);
      const gl=c.getContext("webgl2"), read=fx=>{
        b.draw(data,w,h,fx);const p=new Uint8Array(c.width*c.height*4);gl.readPixels(0,0,c.width,c.height,gl.RGBA,gl.UNSIGNED_BYTE,p);return p;
      };
      const base={muzzle:0,hurt:0,time:2,boss:-1};
      const a=read(base), front=read({...base,shocks:[{x:.5,y:.5,radius:.24,depth:.2,strength:1}]}),behind=read({...base,shocks:[{x:.5,y:.5,radius:.24,depth:.9,strength:1}]});
      const diff=(x,y)=>x.reduce((n,v,i)=>n+Math.abs(v-y[i]),0);
      const dust=read({...base,sector:1}), flash=read({...base,muzzle:1}),lit=read({...base,muzzle:1,sector:1});
      const temp=document.createElement("canvas");temp.width=temp.height=1024;const ctx=temp.getContext("2d");
      const framePixels=[];let border=0;
      for (const name of ["casing", "shell"]) {
      const im=new Image();im.src=`/game/fx/${name}-hd.png`;await im.decode();ctx.clearRect(0,0,1024,1024);ctx.drawImage(im,0,0);
      for(let i=0;i<4;i++){
        const pixels=ctx.getImageData(i%2*512,Math.floor(i/2)*512,512,512).data;let visible=0;
        for(let y=0;y<512;y++)for(let x=0;x<512;x++) {const alpha=pixels[(y*512+x)*4+3];if(alpha>128)visible++;if((x<40||y<40||x>=472||y>=472)&&alpha>10)border++;}
        framePixels.push(visible);
      }
      }
      const entrance={x:.5,y:.5,phase:.5,style:3,depth:.2,color:[.2,.8,1]};
      const entryFront=read({...base,entrance});
      const entryBehind=read({...base,entrance:{...entrance,depth:.9}});
      // Leave the effect frame visible for inspection.
      read({...base,sector:15,muzzle:.4,shocks:[{x:.5,y:.5,radius:.24,depth:.2,strength:1}]});
      return {kind:b.kind,error:gl.getError(),entryFront:diff(a,entryFront),entryBehind:diff(a,entryBehind),front:diff(a,front),behind:diff(a,behind),dust:diff(a,dust),lit:diff(flash,lit),framePixels,border};
    });
    await page.screenshot({path:"screenshots/worldfx-renderer.png"});
    assert.equal(result.kind,"webgl2");assert.equal(result.error,0);
    assert.ok(result.front>1000,JSON.stringify(result));assert.equal(result.behind,0);
    assert.ok(result.entryFront>1000,JSON.stringify(result));assert.equal(result.entryBehind,0);
    assert.ok(result.dust>1000);assert.ok(result.lit>result.dust);
    assert.ok(result.framePixels.every(n=>n>6000));assert.equal(result.border,0);
    assert.deepEqual(errors,[]);
  } finally {await browser.close();}
});

test("actual gameplay preloads new clips, plays casing landings and sector ambience, and pauses them",async()=>{
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage({viewport:{width:1280,height:800}}),errors=[];
    page.on("pageerror",e=>errors.push(e.message));
    await page.addInitScript(()=>localStorage.setItem("blacksite-res","320"));
    const url=new URL(process.env.BLACKSITE_TEST_URL??"http://127.0.0.1:8080/");url.searchParams.set("qa","1");
    await page.goto(url.href);
    await page.waitForFunction(()=>window.__controlsTest?.getReserve()===72,null,{timeout:120000});
    assert.equal(await page.evaluate(()=>window.__controlsTest.getSfxAudio().loaded),130);
    await page.locator(".game-canvas").click();
    await page.evaluate(()=>{window.__controlsTest.heal();window.__controlsTest.setKeys(["Space"]);});
    await page.waitForFunction(()=>window.__controlsTest.getSfxAudio().recent.includes("casing-brass"),null,{timeout:15000});
    await page.waitForFunction(()=>window.__controlsTest.getSfxAudio().recent.includes("sector-1"),null,{timeout:15000});
    await page.evaluate(()=>{window.__controlsTest.setKeys([]);window.__controlsTest.nextWave();window.__controlsTest.heal();});
    await page.waitForFunction(()=>window.__controlsTest.getWorldEffects().wave===2 && window.__controlsTest.getWorldEffects().time>=3.2,null,{timeout:60000});
    assert.ok(await page.evaluate(()=>window.__controlsTest.getSfxAudio().recent.includes("sector-2")));
    await page.screenshot({path:"screenshots/worldfx-gameplay.png"});
    await page.evaluate(()=>window.__controlsTest.setKeys([]));
    await page.keyboard.press("p");
    await page.getByRole("heading",{name:"Operation paused"}).waitFor();
    const frozen = await page.evaluate(()=>window.__controlsTest.getWorldEffects().time);
    await page.waitForTimeout(200);
    assert.equal(await page.evaluate(()=>window.__controlsTest.getWorldEffects().time),frozen);
    assert.equal(await page.evaluate(()=>window.__controlsTest.getSfxAudio().active),0);
    assert.deepEqual(errors,[]);
  } finally {await browser.close();}
});


test("boss entrance preloads eight effects, advances five stages and freezes when paused",async()=>{
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage({viewport:{width:1280,height:800}}),errors=[];
    page.on("pageerror",e=>errors.push(e.message));
    await page.addInitScript(()=>{localStorage.setItem("blacksite-res","320");performance.setResourceTimingBufferSize(5000);});
    const url=new URL(process.env.BLACKSITE_TEST_URL??"http://127.0.0.1:8080/");url.searchParams.set("qa","1");
    await page.goto(url.href,{waitUntil:"domcontentloaded"});
    await page.waitForFunction(()=>window.__controlsTest&&document.body.innerText.includes("HEALTH"),null,{timeout:120000});
    const loaded=await page.evaluate(()=>performance.getEntriesByType("resource").filter(e=>/boss-entry-\d\.png/.test(e.name)).map(e=>e.name));
    assert.equal(new Set(loaded).size,8);
    await page.evaluate(()=>{window.__controlsTest.heal();window.__controlsTest.visitObjective();window.__controlsTest.triggerBoss(-1);});
    await page.waitForFunction(()=>window.__controlsTest.getWorldEffects().entrance?.phase>.1,null,{timeout:15000});
    await page.screenshot({path:"screenshots/boss-entrance-1.png"});
    await page.evaluate(()=>window.__controlsTest.setKeys([]));await page.keyboard.press("p");
    await page.getByRole("heading",{name:"Operation paused"}).waitFor();
    const frozen=await page.evaluate(()=>window.__controlsTest.getWorldEffects().entrance?.phase);
    await page.waitForTimeout(300);assert.equal(await page.evaluate(()=>window.__controlsTest.getWorldEffects().entrance?.phase),frozen);
    await page.keyboard.press("p");
    await page.waitForFunction(()=>window.__controlsTest.getEnemies().some(e=>e.skin===12&&e.hp>0),null,{timeout:30000});
    await page.waitForFunction(()=>!window.__controlsTest.getWorldEffects().entrance,null,{timeout:15000});
    assert.ok(await page.evaluate(()=>window.__controlsTest.getEnemies().filter(e=>e.skin===37&&e.hp>0).length>=2));
    assert.deepEqual(errors,[]);
  }finally{await browser.close();}
});
