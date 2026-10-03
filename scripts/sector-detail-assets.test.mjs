import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { chromium } from 'playwright';
import { readPngInfo } from './check-assets.mjs';
const root=new URL('../',import.meta.url);
const sectors=JSON.parse(readFileSync(new URL('art/sector-detail/specs.json',root)));
const projectiles=JSON.parse(readFileSync(new URL('art/sector-detail/projectiles.json',root)));
const read=(path)=>readFileSync(new URL(path,root));
const hash=(bytes)=>createHash('sha256').update(bytes).digest('hex');

test('all 25 sectors have five exclusive seamless materials and three exclusive prop images',async()=>{
 assert.equal(sectors.length,25);
 const browser=await chromium.launch({headless:true});
 try{
  const page=await browser.newPage();const materials=new Set(),props=new Set();
  for(const sector of sectors){
   for(const name of ['wall','service','door','floor','ceiling','prop_1','prop_2','prop_3']){
    const bytes=read(`public/game/sectors/${sector.slug}/${name}.png`);const info=readPngInfo(bytes);
    assert.deepEqual([info.width,info.height],[256,256],sector.slug+' '+name);assert.ok(info.hasAlpha);
    const set=name.startsWith('prop')?props:materials;assert.ok(!set.has(hash(bytes)),`${sector.slug}/${name} repeats another sector's asset`);set.add(hash(bytes));
    const result=await page.evaluate(async({src,prop})=>{
     const im=new Image();im.src=src;await im.decode();const c=document.createElement('canvas');c.width=c.height=256;const ctx=c.getContext('2d',{willReadFrequently:true});ctx.drawImage(im,0,0);const a=ctx.getImageData(0,0,256,256).data;
     let opaque=0,seam=0;for(let i=3;i<a.length;i+=4)if(a[i]>128)opaque++;
     for(let i=0;i<256;i++)for(let ch=0;ch<3;ch++){seam=Math.max(seam,Math.abs(a[(i*256)*4+ch]-a[(i*256+255)*4+ch]),Math.abs(a[i*4+ch]-a[(255*256+i)*4+ch]));}
     return {coverage:opaque/65536,seam,corners:[a[3],a[255*4+3],a[255*256*4+3],a[a.length-1]]};
    },{src:`data:image/png;base64,${bytes.toString('base64')}`,prop:name.startsWith('prop')});
    if(name.startsWith('prop')){assert.ok(result.coverage>.08&&result.coverage<.78,`${sector.slug}/${name}: missing prop or opaque backdrop (${result.coverage})`);assert.deepEqual(result.corners,[0,0,0,0],`${sector.slug}/${name}: backdrop pixels`);}
    else{assert.equal(result.coverage,1,`${sector.slug}/${name}: material holes`);assert.equal(result.seam,0,`${sector.slug}/${name}: opposite edges do not join`);}
   }
  }
  assert.equal(materials.size,125);assert.equal(props.size,75);
 }finally{await browser.close();}
});

test('enemy and missile projectiles use complete isolated images and retained native sources',async()=>{
 assert.equal(projectiles.length,37);
 const records=JSON.parse(read('art/sector-detail/generation.json'));
 for(const record of records)assert.equal(hash(read(record.source)),record.sha256,record.id+' source hash');
 const manifest=JSON.parse(read('art/projectiles-v3/manifest.json'));
 assert.equal(manifest.assets.length,62);
 for(const source of manifest.sources)assert.equal(hash(read(source.path)),source.sha256);
 for(const entry of manifest.assets)assert.equal(hash(read(entry.file)),entry.sha256);
 const files=projectiles.map(p=>'enemy_'+p.slug+'.png');files.push(...Array.from({length:25},(_,i)=>`boss_weapon_${i+8}.png`));files.push(...['tactical','siege','naval'].flatMap(m=>['outgoing','incoming'].map(d=>`missile_${m}_${d}.png`)));
 const browser=await chromium.launch({headless:true});
 try{
  const page=await browser.newPage();const unique=new Set();
  for(const name of files){
   const bytes=read('public/game/projectiles/'+name),info=readPngInfo(bytes);assert.deepEqual([info.width,info.height],[256,256]);
   if(name.startsWith('enemy_')||name.startsWith('boss_weapon_')){assert.ok(!unique.has(hash(bytes)),name+' repeats another enemy projectile');unique.add(hash(bytes));}
   const result=await page.evaluate(async(src)=>{const im=new Image();im.src=src;await im.decode();const c=document.createElement('canvas');c.width=c.height=256;const ctx=c.getContext('2d',{willReadFrequently:true});ctx.drawImage(im,0,0);const a=ctx.getImageData(0,0,256,256).data;let visible=0,border=0;for(let y=0;y<256;y++)for(let x=0;x<256;x++){const alpha=a[(y*256+x)*4+3];if(alpha>128)visible++;if((x<8||x>247||y<8||y>247)&&alpha>10)border++;}return{visible,border};},`data:image/png;base64,${bytes.toString('base64')}`);
   assert.ok(result.visible>400,name+' blank');assert.equal(result.border,0,name+' crosses isolated frame boundary');
  }
 }finally{await browser.close();}
});
