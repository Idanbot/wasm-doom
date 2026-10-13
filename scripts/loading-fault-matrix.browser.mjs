import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { chromium } from 'playwright';

const base=process.env.BLACKSITE_TEST_URL ?? 'http://127.0.0.1:8080/';
test('loading fault matrix validates decoding, timeout recovery and cancellation at root and Pages paths', async()=>{
  const browser=await chromium.launch({headless:true});
  try {
    const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
    await page.route('**/__loading_matrix.html',r=>r.fulfill({contentType:'text/html',body:'<body></body>'}));
    await page.goto(new URL('__loading_matrix.html',base).href);
    const sources=[
      ['json','public/game/sfx/v2/manifest.json','application/json'],
      ['png','public/game/ui/weapon-thumbs/br12.png','image/png'],
      ['wasm','public/blacksite.wasm','application/wasm'],
      ['ogg','public/game/sfx/v2/fire0.ogg','audio/ogg'],
      ['mp3','public/game/sfx/v2/fire0.mp3','audio/mpeg'],
    ];
    const fixtures=sources.map(([kind,file,mime])=>{
      const bytes=readFileSync(file);return {kind,mime,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),body:bytes};
    });
    const counts={};
    await page.route('**/game/__fault_matrix/*',async r=>{
      const url=new URL(r.request().url()),name=url.pathname.split('/').at(-1);
      const f=fixtures.find(f=>name.startsWith(f.kind));const n=counts[name]=(counts[name]??0)+1;
      if(n===1) {await new Promise(resolve=>setTimeout(resolve,800));await r.fulfill({status:503,body:'stalled request'}).catch(()=>{});}
      else if(n===2)await r.fulfill({status:200,contentType:f.mime,body:'corrupt bytes'});
      else await r.fulfill({status:200,contentType:f.mime,body:f.body});
    });
    const rows=await page.evaluate(async fixtures=>{
      const {createAssetLoader}=await import('/src/game/asset-loading.ts');
      const {validatedAsset,setAssetManifest}=await import('/src/game/asset-cache.ts');
      const manifest={version:1,build:'a'.repeat(64),assets:{}};
      for(const f of fixtures)for(const prefix of ['', '/blacksite']) {
        const name=`${f.kind}-${prefix?'pages':'root'}`;
        manifest.assets[`/game/__fault_matrix/${name}`]={bytes:f.bytes,sha256:f.sha256};
      }
      setAssetManifest(manifest);const rows=[];
      for(const f of fixtures)for(const prefix of ['', '/blacksite']) {
        const name=`${f.kind}-${prefix?'pages':'root'}`,url=`${prefix}/game/__fault_matrix/${name}`;
        const samples=[],signals=[];const loader=createAssetLoader({concurrency:1,timeoutMs:600,retryDelays:[0,0]});
        let audio;
        const result=await loader.load(({attempt,signal})=>{
          signals.push(signal);
          return validatedAsset(url,attempt,signal,async response=>{
            if(f.kind==='json')return !!(await response.json()).clips;
            if(f.kind==='png'){const image=await createImageBitmap(await response.blob());const valid=image.width>0;image.close();return valid;}
            if(f.kind==='wasm')return (await WebAssembly.compile(await response.arrayBuffer())) instanceof WebAssembly.Module;
            audio??=new AudioContext();return (await audio.decodeAudioData(await response.arrayBuffer())).duration>0;
          });
        },{onAttempt:(n,total)=>samples.push(n/total)});
        await audio?.close();
        rows.push({name,result,attempts:signals.length,timedOut:signals[0].aborted,monotonic:samples.every((v,i)=>!i||v>=samples[i-1])});
      }
      // Cancellation must end an active download, without entering either retry.
      const loader=createAssetLoader({timeoutMs:1000,retryDelays:[0,0]});const controller=new AbortController();let requests=0;
      const pending=loader.load(({signal})=>{requests++;return new Promise((_,reject)=>signal.addEventListener('abort',()=>reject(signal.reason),{once:true}));},{signal:controller.signal});
      await new Promise(resolve=>setTimeout(resolve,10));controller.abort();
      let cancelled=false;try {await pending;}catch(e){cancelled=e.name==='AbortError';}
      return {rows,cancelled,requests};
    },fixtures.map(({body:_body,...f})=>f));
    for(const row of rows.rows) {assert.equal(row.result,true,JSON.stringify(row));assert.equal(row.attempts,3,row.name);assert.equal(row.timedOut,true);assert.equal(row.monotonic,true);assert.equal(counts[row.name],3);}
    assert.equal(rows.rows.length,10);assert.equal(rows.cancelled,true);assert.equal(rows.requests,1);assert.deepEqual(errors,[]);
    writeFileSync('.blacksite/loading-fault-matrix.json',JSON.stringify({result:'PASS',...rows},null,2)+'\n');
  } finally {await browser.close();}
});
