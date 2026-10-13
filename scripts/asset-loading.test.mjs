import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createAssetLoader, fetchAsset, preloadAssets, retryImageUrl } from '../src/game/asset-loading.ts';

test('transient failure and corrupt decode recover on the third attempt', async () => {
  const loader = createAssetLoader({retryDelays:[0,0]});
  const attempts=[], progress=[];
  const value=await loader.load(async ({attempt}) => {
    attempts.push(attempt);
    if(attempt===0)throw new Error('HTTP 503');
    if(attempt===1)throw new Error('corrupt PNG');
    return 'decoded';
  },{onAttempt:(done,total)=>progress.push(done/total)});
  assert.equal(value,'decoded');
  assert.deepEqual(attempts,[0,1,2]);
  assert.deepEqual(progress,[1/3,2/3,1]);
});

test('persistent missing assets stop at exactly two retries', async () => {
  const loader=createAssetLoader({retryDelays:[0,0]});let count=0;
  await assert.rejects(loader.load(async()=>{count++;throw new Error('404');}),/404/);
  assert.equal(count,3);
});

test('only failed items retry; progress advances during recovery and stays monotonic', async () => {
  const loader=createAssetLoader({retryDelays:[0,0]});const counts={}, progress=[];
  const out=await preloadAssets(['good','temporary','missing'],(url,onAttempt)=>loader.load(async({attempt})=>{
    counts[url]=(counts[url]??0)+1;
    if(url==='missing'||(url==='temporary'&&attempt<2))throw new Error(url);
    return url;
  },{onAttempt}),2,(n,total)=>progress.push(n/total));
  assert.deepEqual(out,['good','temporary',null]);
  assert.deepEqual(counts,{good:1,temporary:3,missing:3});
  assert.equal(progress.at(-1),1);
  assert.ok(progress.some(p=>p>1/3&&p<2/3),'retry attempts must contribute to progress');
  assert.ok(progress.every((p,i)=>i===0||p>=progress[i-1]));
});

test('bounded concurrency queues downloads without using up their timeout', async () => {
  const loader=createAssetLoader({concurrency:1,timeoutMs:30,retryDelays:[0,0]});
  let release;const held=new Promise(resolve=>release=resolve);const starts=[];
  const first=loader.load(async()=>{starts.push('first');await held;return 1;},{timeoutMs:1000});
  const second=loader.load(async()=>{starts.push('second');return 2;});
  await new Promise(resolve=>setTimeout(resolve,50));
  assert.deepEqual(starts,['first']);release();
  assert.deepEqual(await Promise.all([first,second]),[1,2]);
  assert.deepEqual(starts,['first','second']);
});

test('stalled attempts time out, abort and release the shared download slot', async () => {
  const loader=createAssetLoader({concurrency:1,timeoutMs:10,retryDelays:[0,0]});
  const signals=[];
  await assert.rejects(loader.load(({signal})=>{signals.push(signal);return new Promise(()=>{});}),{name:'TimeoutError'});
  assert.equal(signals.length,3);assert.ok(signals.every(s=>s.aborted));
  assert.equal(await loader.load(async()=>42),42);
});

test('cancelled queued assets never start a request or retry', async () => {
  const loader=createAssetLoader({concurrency:1});let release;
  const first=loader.load(()=>new Promise(resolve=>release=resolve));
  await new Promise(resolve=>setImmediate(resolve));
  const controller=new AbortController();let requests=0;
  const queued=loader.load(async()=>{requests++;},{signal:controller.signal});
  controller.abort();await assert.rejects(queued,{name:'AbortError'});
  assert.equal(requests,0);release();await first;
});

test('HTTP success with corrupt bytes retries with fresh-cache fetches', async () => {
  const previous=globalThis.fetch;const cache=[];
  globalThis.fetch=async(_url,options)=>{
    cache.push(options.cache);
    return new Response(cache.length<3?'broken json':'{"ready":true}');
  };
  try {
    assert.deepEqual(await fetchAsset('/manifest.json',r=>r.json()),{ready:true});
    assert.deepEqual(cache,['default','reload','reload']);
  } finally {globalThis.fetch=previous;}
});

test('image retry URLs preserve Pages paths, existing query and fragments', () => {
  const url='/blacksite/game/ui/menu.webp?version=7#preview';
  assert.equal(retryImageUrl(url,0),url);
  const first=retryImageUrl(url,1),second=retryImageUrl(url,2);
  assert.match(first,/^\/blacksite\/game\/ui\/menu\.webp\?version=7&blacksite_retry=\d+-\d+#preview$/);
  assert.notEqual(first,second);
});

test('cancellation between slot acquisition and request startup prevents the request', async () => {
  const loader=createAssetLoader();const controller=new AbortController();let requests=0;
  const pending=loader.load(async()=>{requests++;return 1;},{signal:controller.signal});
  queueMicrotask(()=>controller.abort());
  await assert.rejects(pending,{name:'AbortError'});
  assert.equal(requests,0);
});
