import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
const read = path => JSON.parse(readFileSync(path,'utf8'));
test('25 boss materials are opaque, seamless, unique and preserve HD sources',()=>{
 const specs=read('art/boss-arenas-v1/specs.json');
 const runtime=read('src/game/boss-arena-data.json');
 const generated=read('art/boss-arenas-v1/generation.json');
 assert.deepEqual(runtime,specs.bosses);
 assert.equal(generated.length,25);
 assert.equal(new Set(generated.map(r=>r.sourceHash)).size,25);
 execFileSync('python3',['-c',`import json
from PIL import Image
import numpy as np
from pathlib import Path
from hashlib import sha256
for entry in json.loads(Path('art/boss-arenas-v1/generation.json').read_text()):
 assert sha256(Path(entry['source']).read_bytes()).hexdigest()==entry['sourceHash']
 for key,size in [('runtime',256),('hd',1024)]:
  im=Image.open(entry[key]).convert('RGBA'); assert im.size==(size,size)
  a=np.array(im); assert (a[:,:,3]==255).all()
  assert (a[0]==a[-1]).all() and (a[:,0]==a[:,-1]).all(),entry['slug']
`]);
});
test('all bosses have clean Aura-2 combat and death performances with matching subtitles',()=>{
 const plan=read('art/boss-voices-v3/plan.json');const manifest=read('public/game/voices/manifest.json');
 const subtitles=read('src/game/boss-voice-lines.json'); const deaths=read('art/boss-voices.json');
 assert.equal(plan.bosses.length,25);assert.equal(deaths.bosses.length,50);
 for(const [i,boss] of plan.bosses.entries()){
  assert.equal(boss.effect,'clean');assert.equal(subtitles[i].name,boss.name);assert.deepEqual(subtitles[i].death,boss.death);
  const profile=manifest.enemies.find(e=>e.skin===boss.skin);assert.equal(profile.speaker,boss.speaker);
  assert.deepEqual(profile.lines.map(l=>[l.cue,l.text]),boss.lines);
  for(const [variant,text] of boss.death.entries()) assert.equal(deaths.bosses.find(b=>b.id===`boss-${boss.id}${variant?'-v2':''}`).text,text);
 }
});
