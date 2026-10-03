import {test} from 'node:test';
import assert from 'node:assert/strict';
import {chromium} from 'playwright';
import {readFileSync} from 'node:fs';
const projectiles=JSON.parse(readFileSync(new URL('../art/sector-detail/projectiles.json',import.meta.url)));

test('real WebGL renderer presents each complete enemy projectile and both missile directions',async()=>{
 const browser=await chromium.launch({headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader']});
 try{
  const page=await browser.newPage({viewport:{width:1280,height:900}});const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(()=>Object.defineProperty(navigator,'gpu',{value:undefined}));
  // Use a minimal same-origin fixture so React preload updates cannot replace the test canvas.
  await page.route('**/__projectile_test.html',route=>route.fulfill({contentType:'text/html',body:'<!doctype html><html><body></body></html>'}));
  await page.goto(new URL('/__projectile_test.html',process.env.BLACKSITE_TEST_URL??'http://127.0.0.1:8080/').href,{waitUntil:'domcontentloaded'});
  const result=await page.evaluate(async(entries)=>{
   const {createBlitter}=await import('/src/game/blit.ts');
   const {TEX,TEX_N,T_ENEMY_PROJECTILE,T_PLAYER_MISSILE,T_BOSS_PROJECTILE}=await import('/src/game/gpu-world.ts');
   document.body.replaceChildren();document.body.style.cssText='margin:0;background:#111;color:white';
   const c=document.createElement('canvas');c.style.cssText='width:960px;height:360px;display:block';document.body.append(c);
   const b=await createBlitter(c);b.setGfx({crt:false,bloom:false,fog:false,scanlines:false});
   const atlas=new Uint8Array(TEX_N*TEX*TEX*4),scratch=document.createElement('canvas');scratch.width=scratch.height=256;const ctx=scratch.getContext('2d',{willReadFrequently:true});
   const files=entries.map(e=>({name:e.slug,src:'/game/projectiles/enemy_'+e.slug+'.png',id:T_ENEMY_PROJECTILE+e.skin}));
   for(let i=0;i<25;i++)files.push({name:'boss weapon '+(i+8),src:'/game/projectiles/boss_weapon_'+(i+8)+'.png',id:T_BOSS_PROJECTILE+i});
   for(let i=0;i<3;i++)files.push({name:['tactical','siege','naval'][i]+' outgoing',src:'/game/projectiles/missile_'+['tactical','siege','naval'][i]+'_outgoing.png',id:T_PLAYER_MISSILE+i});
   // Incoming views use the same exact artwork as their named enemy missile layer.
   for(const [name,skin]of [['tactical',29],['siege',20],['naval',33]])files.push({name:name+' incoming',src:'/game/projectiles/missile_'+name+'_incoming.png',id:T_ENEMY_PROJECTILE+skin});
   for(const file of files){const im=new Image();im.src=file.src;await im.decode();ctx.clearRect(0,0,256,256);ctx.drawImage(im,0,0);atlas.set(ctx.getImageData(0,0,256,256).data,file.id*256*256*4);}
   b.uploadAtlas(atlas);const w=960,h=360,cols=new Float32Array(w*16),sprites=new Float32Array(files.length*8);
   for(let x=0;x<w;x++)cols.set([40,0,0,0,0,h,-1,1,0,0,0,0,0,0,40,0],x*16);
   for(let i=0;i<files.length;i++){const x=40+i%12*80,y=30+Math.floor(i/12)*60;sprites.set([3,3*.72*(x/(w*.5)-1),(y-h*.5)*3,.42,files[i].id,-1,0,4],i*8);}
   const frame={w,h,view:new Float32Array([0,0,1,0,0,.72,h*.5,0,0,0,w,h,0,files.length,0,0]),cols,sprites,spriteCount:files.length,floor:new Uint8Array(48*32),light:new Float32Array(48*32*3),smoke:new Float32Array(48*32)};
   b.drawWorld(frame);const gl=c.getContext('webgl2');const pixels=new Uint8Array(w*h*4);gl.readPixels(0,0,w,h,gl.RGBA,gl.UNSIGNED_BYTE,pixels);
   const visible=files.map((f,i)=>{const cx=40+i%12*80,cy=30+Math.floor(i/12)*60;let n=0;for(let y=cy-25;y<cy+25;y++)for(let x=cx-25;x<cx+25;x++){const k=((h-1-y)*w+x)*4;if(Math.max(pixels[k],pixels[k+1],pixels[k+2])>30)n++;}return{name:f.name,pixels:n};});
   return{renderer:b.kind,error:gl.getError(),visible};
  },projectiles);
  await page.screenshot({path:'screenshots/sector-projectile-renderer.png'});
  assert.equal(result.renderer,'webgl2');assert.equal(result.error,0);assert.ok(result.visible.every(r=>r.pixels>40),JSON.stringify(result.visible));assert.deepEqual(errors,[]);
 }finally{await browser.close();}
});
