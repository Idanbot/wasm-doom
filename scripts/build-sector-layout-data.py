#!/usr/bin/env python3
"""Regenerate typed layout tables from the authored JSON without replacing runtime logic."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
import os
os.chdir(ROOT)
records=json.loads(Path("art/maps/layouts-10-25.json").read_text())
def point(p):return '('+','.join(f'{v:.1f}' for v in p)+')'
lines=['//! Authored sector graphs for levels 10–25. Source: art/maps/layouts-10-25.json.','use crate::{Engine, consts::*, map::HostileSpawn};','pub(crate) struct Layout { pub rooms: &\'static [(i32,i32,i32,i32)], pub edges: &\'static [(usize,usize,bool)], pub doors: &\'static [(i32,i32)], pub wall: u8, pub start: (f32,f32,f32), pub node: (f32,f32), pub objective: (f32,f32), pub bosses: [(f32,f32);2], pub terminals: &\'static [(f32,f32)], pub supplies: &\'static [(u8,f32,f32)], pub power: (f32,f32), pub hostiles: &\'static [HostileSpawn] }','pub(crate) const LAYOUTS: [Layout;16] = [']
for r in records:
 lines += ['Layout { rooms: &['+','.join(str(tuple(p)) for p in r['rooms'])+'], edges: &['+','.join(f'({a},{b},{str(bool(turn)).lower()})' for a,b,turn in r['edges'])+'], doors: &['+','.join(str(tuple(p)) for p in r['doors'])+f"], wall: {r['wall']}, start: {point(r['start'])}, node: {point(r['node'])}, objective: {point(r['override'])}, bosses: ["+','.join(map(point,r['bosses']))+'], terminals: &['+','.join(map(point,r['terminals']))+'], supplies: &['+','.join(f'({k},{x:.1f},{y:.1f})' for k,x,y in r['supplies'])+f"], power: {point(r['power'])}, hostiles: &["+','.join(f'({k},{s},{x:.1f},{y:.1f})' for k,s,x,y in r['hostiles'])+'] },']
lines+= ['];','pub(crate) fn layout(wave: i32) -> Option<&\'static Layout> { let i=crate::map::level_index(wave); if i>=9 {Some(&LAYOUTS[i-9])}else{None} }']
runtime=Path('engine/src/layouts.rs').read_text().split('\npub(crate) fn build(e:')[1]
output='\n'.join(lines)+'\n\npub(crate) fn build(e:'+runtime
if '--check' in sys.argv:
    if Path('engine/src/layouts.rs').read_text()!=output:raise SystemExit('Sector layout tables are stale; regenerate from JSON.')
    print('All 16 layout contracts match the authored JSON.')
else:Path('engine/src/layouts.rs').write_text(output)
