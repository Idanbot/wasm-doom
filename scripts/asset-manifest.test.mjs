import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { generateAssetManifest } from './asset-manifest.ts';

test('manifest is reproducible, publishes exact sizes/hashes and changes identity when any shipped asset changes',async()=>{
  const root=await mkdtemp(join(tmpdir(),'blacksite-manifest-'));
  try {
    await mkdir(join(root,'game'));
    await writeFile(join(root,'blacksite.wasm'),'test engine');
    await writeFile(join(root,'game','wall.png'),'test texture');
    await writeFile(join(root,'game','.env'),'not a published asset');
    const first=await generateAssetManifest(root),second=await generateAssetManifest(root);
    assert.deepEqual(second,first);
    assert.equal(first.assets['/game/wall.png'].bytes,12);
    assert.equal(first.assets['/blacksite.wasm'].sha256,createHash('sha256').update('test engine').digest('hex'));
    assert.equal(first.assets['/game/.env'],undefined);
    assert.equal(first.assets['/game/asset-manifest.json'],undefined);
    await writeFile(join(root,'game','wall.png'),'changed');
    const changed=await generateAssetManifest(root);assert.notEqual(changed.build,first.build);
    assert.deepEqual(JSON.parse(await readFile(join(root,'game','asset-manifest.json'),'utf8')),changed);
  } finally {await rm(root,{recursive:true,force:true});}
});
