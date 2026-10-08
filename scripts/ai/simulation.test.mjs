import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { loadSimulation, snapshot, assertValid, observe, questions, validateDecision, translate,
  step, fingerprint, preflight, INPUT } from './simulation.mjs';

test('headless committed WASM proves real movement, aim, firing and reload without QA helpers', async () => {
  const { w } = await loadSimulation();
  assert.deepEqual(preflight(w), { movement: true, turn: true, firingConsumesAmmo: true,
    reloadTransfersAmmo: true, interactInputAccepted: true, stateValid: true, simulationAdvances: true });
});

test('agent observations exclude enemies behind walls, behind the player and dead enemies', async () => {
  const { w } = await loadSimulation(), s = snapshot(w);
  const enemy = { id: 1, hp: 100, x: s.hud.x + 2, y: s.hud.y, sight: true, distance: 2, screenX: 0.5 };
  s.enemies = [enemy, { ...enemy, id: 2, sight: false }, { ...enemy, id: 3, screenX: -10 }, { ...enemy, id: 4, hp: 0 }];
  const o = observe(s);
  assert.deepEqual(o.visibleEnemies.map((e) => e.id), [1]);
  assert.equal('enemies' in o, false);
  assert.equal('map' in o, false);
  assert.equal('hp' in o.visibleEnemies[0], false);
  assert.equal('x' in o.visibleEnemies[0], false);
});

test('malformed choices are rejected and neutral input safely advances the actual simulation', async () => {
  const { w } = await loadSimulation();
  const s = snapshot(w), o = observe(s), schema = questions(o);
  for (const invalid of [null, {}, { movement: { type: 'choice', choice: '__proto__' } },
    { movement: { type: 'choice', choice: 'teleport' }, combat: { type: 'choice', choice: 'hold' } }]) {
    const v = validateDecision(invalid, schema);
    assert.ok(v.errors.length);
    assert.deepEqual(translate(v.action, o), { bits: 0, mx: 0, my: 0 });
  }
  const after = step(w, translate(validateDecision(null, schema).action, o), 5).snapshot;
  assert.ok(after.hud.elapsedMs > s.hud.elapsedMs);
  assertValid(after);
  assert.throws(() => step(w, { bits: INPUT.fire, mx: NaN, my: 0 }, 1), /Unsafe/);
  assert.throws(() => step(w, { bits: 0, mx: 0, my: 0 }, NaN), /Unsafe/);
});

test('only legal combat/reload decisions become input and navigation cannot fight scan steering', async () => {
  const { w } = await loadSimulation();
  const o = observe(snapshot(w));
  assert.equal('engage_nearest' in questions(o).combat.criteria, false);
  const routed = translate({ movement: 'move_toward_objective', combat: 'turn_left', utility: 'nothing' }, o);
  assert.equal(routed.bits & INPUT.turn_left, 0);
  assert.equal(routed.bits & INPUT.advance, INPUT.advance);
  const aiming = { ...o, magazine: 5, visibleEnemies: [{ bearing: 0.2 }] };
  assert.equal(translate({ movement: 'hold', combat: 'engage_nearest', utility: 'nothing' }, aiming).bits, INPUT.fire);
  assert.ok(questions(aiming).utility.criteria.reload);
  assert.equal(translate({ movement: 'hold', combat: 'engage_nearest', utility: 'nothing' }, { ...aiming, magazine: 0 }).bits, 0);
});

test('recorded normal inputs replay exactly; different input changes the state hash', async () => {
  const a = await loadSimulation(), b = await loadSimulation();
  const actions = [{ bits: 1, mx: 0, my: 0 }, { bits: 0, mx: 15, my: 0 },
    { bits: 16, mx: 0, my: 0 }, { bits: 4096, mx: 0, my: 0 }];
  for (const input of actions) {
    assert.equal(fingerprint(step(a.w, input, 30).snapshot), fingerprint(step(b.w, input, 30).snapshot));
  }
  assert.notEqual(fingerprint(step(a.w, { bits: 1, mx: 0, my: 0 }, 1).snapshot),
    fingerprint(step(b.w, { bits: 0, mx: 0, my: 0 }, 1).snapshot));
});

test('test input constants match the Rust ABI and AI code cannot call debug/wave-skipping APIs', () => {
  const rust = readFileSync('engine/src/consts.rs', 'utf8');
  for (const [key, name] of Object.entries({ advance: 'W', retreat: 'S', strafe_left: 'A', strafe_right: 'D',
    fire: 'FIRE', sprint: 'SPRINT', interact: 'USE', turn_left: 'TURNL', turn_right: 'TURNR', reload: 'RELOAD' })) {
    assert.match(rust, new RegExp(`IN_${name}: u32 = ${INPUT[key]};`));
  }
  for (const path of ['scripts/ai/simulation.mjs', 'scripts/ai/smoke.mjs', 'scripts/ai/controller.mjs']) {
    assert.doesNotMatch(readFileSync(path, 'utf8'), /\.hs_(?:qa\w*|next_wave|load_run|restart|render|prepare_gpu)\s*\(/);
  }
});

test('a validated typed reload choice transfers real ammo through the normal input ABI', async () => {
  const { w } = await loadSimulation();
  step(w, { bits: INPUT.advance, mx: 0, my: 0 }, 30);
  const before = step(w, { bits: INPUT.fire, mx: 0, my: 0 }, 10).snapshot;
  const o = observe(before), schema = questions(o);
  const validated = validateDecision({ movement: { type: 'choice', choice: 'hold' },
    combat: { type: 'choice', choice: 'hold' }, utility: { type: 'choice', choice: 'reload' } }, schema);
  assert.deepEqual(validated.errors, []);
  const after = step(w, translate(validated.action, o), 90).snapshot;
  assert.ok(after.hud.ammo > before.hud.ammo);
  assert.equal(before.hud.reserve - after.hud.reserve, after.hud.ammo - before.hud.ammo);
});


test('read-only item cues stay within view and malformed item telemetry fails clearly', async () => {
  const { w } = await loadSimulation(), s = snapshot(w);
  assertValid(s);
  assert.ok(s.items.length > 0);
  const item = { ...s.items[0] };
  for (const bad of [{ distance: NaN }, { screenX: -1 }, { category: 0 }, { x: s.width }]) {
    s.items = [{ ...item, ...bad }];
    assert.throws(() => assertValid(s), /Invalid visible item/);
  }
});
