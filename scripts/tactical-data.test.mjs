import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
test('machinery effects match all 75 sector-specific assets and the Rust role table',()=>{
 const art=JSON.parse(readFileSync('art/sector-detail/specs.json'));
 const live=JSON.parse(readFileSync('src/game/tactical-machinery.json'));
 const rust=readFileSync('engine/src/tactical_roles.rs','utf8');
 const rows=[...rust.matchAll(/\[(\d+),(\d+),(\d+)\]/g)].map(m=>m.slice(1).map(Number));
 assert.equal(live.length,25);assert.equal(rows.length,25);
 for(const [i,sector]of live.entries()){
  assert.equal(sector.level,i+1);assert.deepEqual(sector.props.map(p=>p.name),art[i].props);
  assert.deepEqual(sector.props.map(p=>p.role),rows[i]);
  for(const prop of sector.props){assert.ok(prop.effect.includes('·'));assert.ok(prop.role>=0&&prop.role<=6);}
 }
});
