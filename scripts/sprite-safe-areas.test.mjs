import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {RELOAD_EJECT_PROGRESS} from '../src/game/sfx-director.ts';

test('published sprite frames match reviewed bounds, gutters, pixels and approved aim poses',()=>{
  const result=execFileSync('python3',['scripts/sprite-safe-areas.py','--check'],{encoding:'utf8',timeout:180000});
  assert.match(result,/frames verified/);
  const contract=JSON.parse(readFileSync('art/sprite-safe-areas.json'));
  assert.equal(contract.assets.filter(a=>a.approved_aim_rgba_sha256).length,33);
  assert.ok(contract.assets.filter(a=>a.file.includes('/enemy_')).length>=434);
});
test('safe-area measurement rejects boundary bleed, blank cells and nonintegral packing',()=>{
  execFileSync('python3',['-c',`
import importlib.util
from PIL import Image
spec=importlib.util.spec_from_file_location('contracts','scripts/sprite-safe-areas.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
for image,grid in [(Image.new('RGBA',(8,8),(255,0,0,255)),[2,2]),(Image.new('RGBA',(8,8)),[2,2]),(Image.new('RGBA',(9,8)),[2,2])]:
    try:m.measure(image,grid)
    except AssertionError:continue
    raise AssertionError('unsafe sprite was accepted')
image=Image.new('RGBA',(16,16));image.paste((255,0,0,255),(2,2,14,14))
assert m.measure(image,[1,1])[1][0]['gutters']==dict(left=2,top=2,right=2,bottom=2)
`],{timeout:10000});
});
test('feed removal sound and simulation use the same first-pose milestone',()=>{
  const rust=readFileSync('engine/src/consts.rs','utf8');
  assert.equal(Number(rust.match(/RELOAD_EJECT_PROGRESS: f32 = ([\d.]+)/)[1]),RELOAD_EJECT_PROGRESS);
  assert.ok(RELOAD_EJECT_PROGRESS>0&&RELOAD_EJECT_PROGRESS<0.2);
});
