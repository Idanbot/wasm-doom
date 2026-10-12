import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { chromium } from 'playwright';

const base = process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/';

test('new magazine poses and projectiles decode with isolated transparent borders', async () => {
  const browser = await chromium.launch({headless:true});
  try {
    const page = await browser.newPage();
    await page.route('**/__feed_art.html', r => r.fulfill({contentType:'text/html',body:'<body></body>'}));
    await page.goto(new URL('__feed_art.html', base).href);
    const records = ['weapon-feeds-v2', 'projectiles-v4'].flatMap(folder =>
      JSON.parse(readFileSync(`art/${folder}/manifest.json`)).assets);
    const results = await page.evaluate(async ({records,base}) => {
      const canvas=document.createElement('canvas');canvas.width=canvas.height=1024;
      const ctx=canvas.getContext('2d');const results=[];
      for(const a of records) {
        const image=new Image();image.src=new URL(a.file.replace(/^public\//,''),base).href;await image.decode();
        ctx.clearRect(0,0,1024,1024);ctx.drawImage(image,0,0);
        const count=a.grid[0]*a.grid[1];const cell=1024/a.grid[0];
        for(let frame=0;frame<count;frame++) {
          const pixels=ctx.getImageData(frame%a.grid[0]*cell,Math.floor(frame/a.grid[0])*cell,cell,cell).data;
          let border=0,visible=0;
          for(let y=0;y<cell;y++)for(let x=0;x<cell;x++) {
            const alpha=pixels[(y*cell+x)*4+3];if(alpha>64)visible++;
            if((x<40||y<40||x>=cell-40||y>=cell-40)&&alpha>0)border++;
          }
          results.push({id:a.id,frame,border,visible});
        }
      }
      return results;
    },{records,base});
    for(const a of results) {assert.equal(a.border,0,JSON.stringify(a));assert.ok(a.visible>1000,JSON.stringify(a));}
    assert.equal(results.length,28*4+18);
  } finally {await browser.close();}
});

test('WebGL beam uses its texture instead of the old procedural blue circle', async () => {
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage();
    await page.route('**/__feed_renderer.html',r=>r.fulfill({contentType:'text/html',body:'<body></body>'}));
    await page.goto(new URL('__feed_renderer.html',base).href);
    const result=await page.evaluate(async()=>{
      const {createWebGlWorld,TEX_N,TEX,T_BEAM,MAP_W,MAP_H}=await import('/src/game/gpu-world.ts');
      const canvas=document.createElement('canvas');canvas.width=320;canvas.height=200;
      const gl=canvas.getContext('webgl2');const world=createWebGlWorld(gl);
      const layers=new Uint8Array(TEX_N*TEX*TEX*4);
      for(let p=T_BEAM*TEX*TEX*4;p<(T_BEAM+1)*TEX*TEX*4;p+=4)layers.set([7,231,19,255],p);
      world.uploadAtlas(layers);
      const target=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,target);
      gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA8,320,200,0,gl.RGBA,gl.UNSIGNED_BYTE,null);
      gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.NEAREST);
      const cols=new Float32Array(320*16);
      for(let x=0;x<320;x++){cols[x*16+4]=0;cols[x*16+5]=200;cols[x*16+6]=200;cols[x*16+14]=20;}
      const ok=world.draw(target,{w:320,h:200,view:new Float32Array([10,10,1,0,0,.66,100,0,0,0,320,200,.66,0,0,0]),
        cols,sprites:new Float32Array([12,10,0,.5,T_BEAM,-1,0,21]),spriteCount:1,
        floor:new Uint8Array(MAP_W*MAP_H),light:new Float32Array(MAP_W*MAP_H*3),smoke:new Float32Array(MAP_W*MAP_H)});
      const framebuffer=gl.createFramebuffer();gl.bindFramebuffer(gl.FRAMEBUFFER,framebuffer);
      gl.framebufferTexture2D(gl.FRAMEBUFFER,gl.COLOR_ATTACHMENT0,gl.TEXTURE_2D,target,0);
      const pixel=new Uint8Array(4);gl.readPixels(160,100,1,1,gl.RGBA,gl.UNSIGNED_BYTE,pixel);
      const error=gl.getError();world.dispose();return {ok,pixel:Array.from(pixel),error};
    });
    assert.equal(result.ok,true);assert.equal(result.error,0);assert.deepEqual(result.pixel.slice(0,3),[7,231,19]);
  } finally {await browser.close();}
});

test('WebGPU world shaders validate after switching beams to atlas artwork', async t => {
  const browser=await chromium.launch({headless:true,args:['--enable-unsafe-webgpu','--use-angle=swiftshader']});
  try {
    const page=await browser.newPage();
    await page.route('**/__feed_shader.html',r=>r.fulfill({contentType:'text/html',body:'<body></body>'}));
    await page.goto(new URL('__feed_shader.html',base).href);
    const result=await page.evaluate(async()=>{
      const adapter=await navigator.gpu?.requestAdapter();
      if(!adapter)return {unavailable:true};
      const device=await adapter.requestDevice();
      const {WORLD_SHADER_SOURCES}=await import('/src/game/gpu-world.ts');
      const errors=[];
      for(const [name,code] of Object.entries(WORLD_SHADER_SOURCES)) {
        const info=await device.createShaderModule({code}).getCompilationInfo();
        errors.push(...info.messages.filter(m=>m.type==='error').map(m=>`${name}: ${m.message}`));
      }
      device.destroy();return {errors};
    });
    if(result.unavailable){t.skip('WebGPU adapter unavailable');return;}
    assert.deepEqual(result.errors,[]);
  } finally {await browser.close();}
});
