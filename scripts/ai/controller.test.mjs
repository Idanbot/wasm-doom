import { test } from 'node:test';
import assert from 'node:assert/strict';
import { WEAPONS } from '../../src/components/game/data.ts';
import { loadSimulation, snapshot, observe, questions, translate, waypoint, step, fingerprint, INPUT } from './simulation.mjs';
import { actionDuration, createMemory, executeAction } from './controller.mjs';

const idle = { movement: 'hold', combat: 'hold', utility: 'nothing' };
const armory = async () => {
  const { w } = await loadSimulation();
  w.hs_qa(0, 1); w.hs_qa_armory(); w.hs_qa(0, 0);
  step(w, { bits: 0, mx: 0, my: 0 }, 1);
  return w;
};

test('all 33 guns expose accurate owned ammo and legal reload/switch choices; reload suppresses firing', async () => {
  const w = await armory();
  for (const gun of WEAPONS) {
    const s = snapshot(w);
    s.hud.weapon = gun.id; s.hud.ammo = 0; s.hud.reserve = 10;
    const o = observe(s), schema = questions(o);
    assert.equal(o.magazineCapacity, gun.magSize);
    assert.ok(schema.utility.criteria.reload, gun.name);
    const input = translate({ ...idle, combat: 'engage_nearest', utility: 'reload' }, { ...o, visibleEnemies: [{ bearing: 0 }] });
    assert.equal(input.bits & INPUT.fire, 0);
    assert.equal(input.bits & INPUT.reload, INPUT.reload);
    assert.equal('reload' in (questions({ ...o, magazine: o.magazineCapacity }).utility?.criteria ?? {}), false);
  }
  const o = observe(snapshot(w));
  assert.ok(questions(o).utility.criteria.equip_2);
  const input = translate({ ...idle, utility: 'equip_2' }, o);
  assert.equal(input.weaponSlot, 1);
  assert.equal(step(w, input, 1).snapshot.hud.weapon, 1);
  assert.throws(() => step(w, { bits: 0, mx: 0, my: 0, weaponSlot: 99 }, 1), /Unsafe/);
});

test('shotgun, precision rifle, missile launcher and expansion gun reload through real inputs', async () => {
  for (const slot of [1, 3, 4, 32]) {
    const w = await armory();
    step(w, { bits: 0, mx: 0, my: 0, weaponSlot: slot }, 120);
    const before = step(w, { bits: INPUT.fire, mx: 0, my: 0 }, 30).snapshot;
    const memory = createMemory(before), reserve = before.hud.reserve;
    const result = executeAction(w, { ...idle, combat: 'engage_nearest', utility: 'reload' }, memory);
    const after = result.snapshot;
    assert.ok(after.hud.ammo > before.hud.ammo, `slot ${slot}`);
    assert.ok(after.hud.reserve < reserve);
    assert.equal(result.events & 1, 0, 'Choosing reload must not trigger firing');
  }
});

test('exploration routes around a remembered failed edge and explores after objective completion without enemy locations', async () => {
  const s = snapshot((await loadSimulation()).w);
  s.width = 5; s.height = 5;
  s.map = [1,1,1,1,1, 1,0,0,0,1, 1,0,0,0,1, 1,0,0,0,1, 1,1,1,1,1];
  s.doors = Array(25).fill(0);
  Object.assign(s.hud, { x: 1.5, y: 1.5, nodeX: 3.5, nodeY: 1.5, objective: 0 });
  const memory = createMemory(s);
  assert.equal(waypoint(s, memory).cell, 7);
  memory.blocked.set('6:7', s.hud.elapsedMs + 10000);
  assert.equal(waypoint(s, memory).cell, 11);
  s.hud.objective = 1;
  assert.equal(waypoint(s, memory).mode, 'explore');
  const first = waypoint(s, memory);
  s.enemies = [{ id: 999, hp: 100, x: 3, y: 3, sight: false }];
  assert.deepEqual(waypoint(s, memory), first);
});

test('adaptive safe exploration lasts longer; all steering segments replay exactly', async () => {
  const a = await loadSimulation(), b = await loadSimulation();
  const s = snapshot(a.w), memory = createMemory(s);
  const action = { ...idle, movement: 'move_toward_objective' };
  assert.equal(actionDuration(action, { ...observe(s), nearbyDoor: false, takingDamage: false, visibleEnemies: [] }), 360);
  assert.equal(actionDuration(action, { ...observe(s), health: 15 }, 30), 30);
  const result = executeAction(a.w, action, memory);
  assert.ok(result.frames > 30);
  assert.ok(result.segments.length > 1);
  let replay;
  for (const segment of result.segments) replay = step(b.w, segment.input, segment.frames);
  assert.equal(fingerprint(replay.snapshot), fingerprint(result.snapshot));
  assert.ok(memory.visited.size > 1);
  assert.equal(memory.recent.length, 1);
});

test('combat stops at empty ammo and safe controller never invents a reload', async () => {
  const { w } = await loadSimulation();
  // Empty the real magazine with normal input, then ask to continue firing.
  step(w, { bits: INPUT.fire, mx: 0, my: 0 }, 180);
  step(w, { bits: INPUT.fire, mx: 0, my: 0 }, 180);
  const before = snapshot(w), memory = createMemory(before);
  const result = executeAction(w, { ...idle, combat: 'engage_nearest' }, memory);
  assert.equal(result.events & 4, 0);
  assert.equal(result.snapshot.hud.reserve, before.hud.reserve);
  assert.ok(result.segments.every(segment => !(segment.input.bits & INPUT.reload)));
});
