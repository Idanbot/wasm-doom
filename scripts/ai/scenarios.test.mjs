import { test } from 'node:test';
import assert from 'node:assert/strict';
import { loadScenario, scriptedDecision, SECTORS, extendedScenarios, EXTENDED_SECTORS } from './scenarios.mjs';
import { snapshot, step, observe, questions, validateDecision, translate, fingerprint, INPUT, assertValid } from './simulation.mjs';

test('three distinct stock sector checkpoints initialize and replay deterministically', async () => {
  const maps = new Set();
  for (const sector of SECTORS) {
    const a = await loadScenario(sector), b = await loadScenario(sector);
    const before = snapshot(a.w);
    assert.equal(before.hud.wave, sector);
    assert.equal(before.hud.health, 100);
    assert.deepEqual(before.hud.ownedWeapons, [0]);
    assert.ok(before.hud.living > 0);
    maps.add(JSON.stringify(before.map));
    assert.equal(fingerprint(before), fingerprint(snapshot(b.w)));
    for (const input of [{ bits: INPUT.advance, mx: 0, my: 0 }, { bits: INPUT.fire, mx: 10, my: 0 },
      { bits: INPUT.reload, mx: 0, my: 0 }, { bits: INPUT.interact, mx: 0, my: 0 }]) {
      assert.equal(fingerprint(step(a.w, input, 60).snapshot), fingerprint(step(b.w, input, 60).snapshot));
    }
  }
  assert.equal(maps.size, 3, 'Sector layouts must be distinct');
  await assert.rejects(loadScenario(26), /Unsupported/);
});

test('held empty fire never consumes reserve or auto-reloads; explicit reload conserves ammo', async () => {
  const { w } = await loadScenario();
  let s = snapshot(w);
  for (let n = 0; n < 5 && s.hud.ammo > 0; n++) s = step(w, { bits: INPUT.fire, mx: 0, my: 0 }, 180).snapshot;
  assert.equal(s.hud.ammo, 0);
  const empty = step(w, { bits: INPUT.fire, mx: 0, my: 0 }, 120);
  assert.equal(empty.events & 1, 0, 'Empty magazine cannot fire');
  assert.equal(empty.snapshot.hud.ammo, 0);
  assert.equal(empty.snapshot.hud.reserve, s.hud.reserve);
  const reloaded = step(w, { bits: INPUT.reload, mx: 0, my: 0 }, 120).snapshot;
  assert.ok(reloaded.hud.ammo > 0);
  assert.equal(s.hud.reserve, reloaded.hud.reserve + reloaded.hud.ammo);
});

test('scripted baseline reloads empty ammo and produces only schema-valid choices', async () => {
  const { w } = await loadScenario();
  const o = { ...observe(snapshot(w)), magazine: 0, reserve: 24 };
  const schema = questions(o), decision = validateDecision(scriptedDecision(o, schema), schema);
  assert.deepEqual(decision.errors, []);
  assert.equal(translate(decision.action, o).bits & INPUT.reload, INPUT.reload);
});

test('each sector advances safely under a fixed multi-action stress sequence', async () => {
  for (const sector of SECTORS) {
    const { w } = await loadScenario(sector);
    let s = snapshot(w), frames = 0;
    for (let n = 0; n < 48 && s.hud.state === 0; n++) {
      const bits = [INPUT.advance | INPUT.sprint, INPUT.strafe_left | INPUT.fire, INPUT.retreat,
        INPUT.reload, INPUT.interact, INPUT.turn_right][n % 6];
      const interval = step(w, { bits, mx: 0, my: 0 }, 30);
      s = interval.snapshot; frames += interval.frames; assertValid(s);
    }
    assert.ok(frames > 0);
    assert.ok(s.hud.elapsedMs > 0);
  }
});

test('fixed observable-state baseline opens a real door and damages a real enemy', async () => {
  const { w } = await loadScenario();
  let s = snapshot(w), previous, doorOpened = false, damageDealt = false;
  for (let n = 0; n < 32 && s.hud.state === 0; n++) {
    const o = observe(s, previous), schema = questions(o);
    const decision = validateDecision(scriptedDecision(o, schema), schema);
    assert.deepEqual(decision.errors, []);
    previous = s;
    s = step(w, translate(decision.action, o), 30, (frame) => {
      doorOpened ||= frame.doors.some((v) => v > 0);
      damageDealt ||= frame.hud.kills > s.hud.kills || frame.enemies.some((e) =>
        s.enemies.some((old) => old.id === e.id && old.hp > e.hp));
      s = frame;
    }).snapshot;
  }
  assert.ok(doorOpened, 'Normal interact must open a door');
  assert.ok(damageDealt, 'Normal firing must damage an enemy');
});

test('state and input validation reject nonfinite values, negative ammo and unsafe inputs', async () => {
  const { w } = await loadScenario(), s = snapshot(w);
  for (const [key, value] of [['health', NaN], ['reserve', -1], ['ammo', -1], ['state', 3], ['x', -1]]) {
    assert.throws(() => assertValid({ ...s, hud: { ...s.hud, [key]: value } }));
  }
  for (const bits of [-1, 8192, 1.5, Infinity]) {
    assert.throws(() => step(w, { bits, mx: 0, my: 0 }, 1), /Unsafe/);
  }
  assert.throws(() => step(w, { bits: 0, mx: 0, my: Infinity }, 1), /Unsafe/);
  assert.throws(() => step(w, { bits: 0, mx: 0, my: 0 }, 181), /Unsafe/);
});


test('extended fixtures cover eight sectors and three distinct seeded real boss fights with normal health', async () => {
  const fixtures = extendedScenarios();
  assert.deepEqual(fixtures, extendedScenarios());
  assert.notDeepEqual(fixtures, extendedScenarios(1730));
  assert.deepEqual(fixtures.filter(f => f.kind === 'sector').map(f => f.sector), EXTENDED_SECTORS);
  const bosses = fixtures.filter(f => f.kind === 'boss');
  assert.equal(new Set(bosses.map(f => f.sector)).size, 3);
  for (const f of fixtures) {
    const a = await loadScenario(f.sector, f.kind), b = await loadScenario(f.sector, f.kind);
    const s = snapshot(a.w); assertValid(s);
    assert.equal(s.hud.health, 100);
    assert.equal(fingerprint(s), fingerprint(snapshot(b.w)));
    if (f.kind === 'boss') assert.ok(s.hud.bossHealth > 0);
    const input = { bits: INPUT.fire, mx: 0, my: 0 };
    assert.equal(fingerprint(step(a.w, input, 60).snapshot), fingerprint(step(b.w, input, 60).snapshot));
  }
});
