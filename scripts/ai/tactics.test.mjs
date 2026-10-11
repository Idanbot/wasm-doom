import { test } from "node:test";
import assert from "node:assert/strict";
import { combatTarget, needsAmmo, combatAim, availableWeapons } from "./tactics.mjs";

test("large aim corrections wait before firing and work in either direction", () => {
  for (const direction of [-1, 1]) {
    assert.deepEqual(combatAim(direction * 0.7), { delta: direction * 0.45, aligned: false });
    assert.deepEqual(combatAim(direction * 0.25), { delta: direction * 0.25, aligned: true });
    assert.equal(combatAim(direction * 0.52).aligned, true);
    assert.equal(combatAim(direction * 0.54).aligned, false);
  }
});

test("emergency weapon choices prefer loaded magazines over large reserve piles", () => {
  const ready = { id: 1, magazine: 12, reserve: 0 },
    empty = { id: 7, magazine: 0, reserve: 450 },
    exhausted = { id: 3, magazine: 0, reserve: 0 },
    inventory = [empty, ready, exhausted];
  assert.deepEqual(availableWeapons(inventory), [ready, empty]);
  assert.deepEqual(inventory, [empty, ready, exhausted], "sorting must not mutate the snapshot");
  const ties = [
    { id: 8, magazine: 4, reserve: 8 },
    { id: 6, magazine: 4, reserve: 8 },
  ];
  assert.equal(availableWeapons(ties)[0].id, 6);
});

test("combat lock resists small distance changes but yields to an immediate threat", () => {
  const nearest = { id: 1, distance: 5 },
    locked = { id: 2, distance: 5.5 };
  assert.equal(combatTarget([nearest, locked], 2), locked);
  assert.equal(combatTarget([{ ...nearest, distance: 2 }, locked], 2).id, 1);
  assert.equal(combatTarget([{ ...nearest, distance: 4 }, locked], 2).id, 1);
});

test("dead or hidden targets cannot preserve a lock and stable ties remain deterministic", () => {
  const visible = [
    { id: 1, distance: 5 },
    { id: 2, distance: 5 },
  ];
  assert.equal(combatTarget(visible, 99), visible[0]);
  assert.equal(combatTarget(visible, 2), visible[1]);
  assert.equal(combatTarget([], 2), undefined);
});

test("abundant launcher ammunition does not hide an empty rifle", () => {
  const inventory = [
    { capacity: 4, magazine: 4, reserve: 100 },
    { capacity: 36, magazine: 0, reserve: 0 },
  ];
  assert.equal(needsAmmo(inventory), true);
  inventory[1].reserve = 72;
  assert.equal(needsAmmo(inventory), false);
  assert.equal(needsAmmo([{ capacity: 4, magazine: 4, reserve: 4 }]), false);
  assert.equal(needsAmmo([]), false);
});
