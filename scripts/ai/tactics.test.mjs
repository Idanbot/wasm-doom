import { test } from "node:test";
import assert from "node:assert/strict";
import { combatTarget, needsAmmo } from "./tactics.mjs";

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
