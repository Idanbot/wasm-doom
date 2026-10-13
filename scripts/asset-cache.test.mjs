import { softwareAssetDigest } from "../src/game/asset-digest.ts";
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { assetBytes, setAssetManifest, validatedAsset, versionedAssetUrl, CACHE_BUDGET } from '../src/game/asset-cache.ts';
import { preloadAssets, fetchAsset } from '../src/game/asset-loading.ts';
const hash = value => createHash('sha256').update(value).digest('hex');
const a='{"valid":true}', b='large asset';
const manifest={version:1,build:hash('build'),assets:{
  '/game/a.json':{bytes:Buffer.byteLength(a),sha256:hash(a)},
  '/game/b.json':{bytes:Buffer.byteLength(b),sha256:hash(b)},
}};

test('manifest rejects malformed identities and applies content versions under Pages paths',()=>{
  assert.throws(()=>setAssetManifest({...manifest,build:'latest'}),/Invalid/);
  assert.throws(()=>setAssetManifest({...manifest,assets:{'/game/a.json':{bytes:-1,sha256:hash(a)}}}),/Invalid/);
  setAssetManifest(manifest);
  assert.equal(assetBytes('/blacksite/game/a.json'),a.length);
  assert.equal(versionedAssetUrl('/blacksite/game/a.json'),`/blacksite/game/a.json?blacksite_version=${hash(a)}`);
});

test('progress weights real asset bytes rather than counting equally sized files',async()=>{
  setAssetManifest(manifest);let release;
  const held=new Promise(resolve=>release=resolve), samples=[];
  const loading=preloadAssets(['/game/a.json','/game/b.json'],async url=>{
    if(url.endsWith('b.json'))await held;
    return url;
  },2,(n,total)=>samples.push(n/total));
  await new Promise(resolve=>setImmediate(resolve));
  assert.equal(samples.at(-1),a.length/(a.length+b.length));release();await loading;
  assert.equal(samples.at(-1),1);
});

test('persistent cache validates hashes, caches only decoded success, evicts by budget and tolerates quota errors',async()=>{
  const previous={fetch:globalThis.fetch,caches:globalThis.caches,location:globalThis.location};
  const values=new Map([['http://local/game/old',new Response('old',{headers:{'content-length':String(CACHE_BUDGET+1)}})]]);
  let requests=0,quota=false;
  const absolute=key=>new URL(typeof key==='string'?key:key.url,'http://local/').href;
  const cache={keys:async()=>Array.from(values.keys(),key=>new Request(key)),
    match:async key=>values.get(absolute(key))?.clone(),delete:async key=>values.delete(absolute(key)),
    put:async(key,response)=>{if(quota)throw new Error('quota');values.set(absolute(key),response.clone());}};
  globalThis.location={href:'http://local/'};
  globalThis.caches={open:async()=>cache};
  globalThis.fetch=async()=>{requests++;return new Response(a);};
  const signal=new AbortController().signal;
  try {
    setAssetManifest(manifest);
    assert.deepEqual(await validatedAsset('/game/a.json',0,signal,r=>r.json()),{valid:true});
    assert.equal(requests,1);assert.equal(values.has('http://local/game/old'),false);
    assert.deepEqual(await validatedAsset('/game/a.json',0,signal,r=>r.json()),{valid:true});
    assert.equal(requests,1,'second boot reuses validated bytes');
    const key=absolute(versionedAssetUrl('/game/a.json'));
    values.set(key,new Response('corrupt'));
    assert.deepEqual(await fetchAsset('/game/a.json',r=>r.json()),{valid:true});
    assert.equal(requests,2,'corrupt cached bytes are deleted before network retry');
    await cache.delete(key);
    await assert.rejects(validatedAsset('/game/a.json',0,signal,async()=>{throw new Error('decode failed');}),/decode/);
    assert.equal(values.has(key),false,'hash-valid but undecodable bytes must not persist');
    quota=true;
    assert.deepEqual(await validatedAsset('/game/a.json',0,signal,r=>r.json()),{valid:true});
    quota=false;
    const changed='{"valid":false}';
    setAssetManifest({version:1,build:hash('changed'),assets:{'/game/a.json':{bytes:changed.length,sha256:hash(changed)}}});
    globalThis.fetch=async()=>{requests++;return new Response(changed);};
    assert.deepEqual(await validatedAsset('/game/a.json',0,signal,r=>r.json()),{valid:false});
    assert.ok(values.has(absolute(versionedAssetUrl('/game/a.json'))));
  } finally {Object.assign(globalThis,previous);}
});

test('LAN HTTP checksum fallback matches native SHA-256 at padding/block boundaries',()=>{
  for(const size of [0,1,55,56,63,64,65,1024,65537]) {
    const bytes=Uint8Array.from({length:size},(_,i)=>(i*37+13)%256);
    assert.equal(softwareAssetDigest(bytes.buffer),hash(bytes));
  }
});
