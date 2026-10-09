import { test } from "node:test";
import assert from "node:assert/strict";
import { loadScenario } from "./scenarios.mjs";
import { snapshot, step, fingerprint } from "./simulation.mjs";
import {
  progressionMemory,
  remember,
  plan,
  view,
  compactRequest,
  chooseAnswers,
  referenceChoices,
  executeGoal,
  commanderUtility,
} from "./progression.mjs";
import { navigationTree } from "./navigation.mjs";

function navigationFixture() {
  return {
    width: 5,
    height: 3,
    map: [1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1],
    doors: [],
    items: [],
    enemies: [],
    hud: {
      x: 1.5,
      y: 1.5,
      yaw: Math.PI,
      elapsedMs: 100,
      health: 100,
      armor: 100,
      objective: 0,
      nodeX: 3.5,
      nodeY: 1.5,
      ownedWeapons: [0],
      inventory: [{ magazine: 12, reserve: 100 }],
    },
  };
}

test("routing rejects malformed coordinates and routes around walls with stable first steps", () => {
  const s = navigationFixture(),
    m = progressionMemory(),
    route = navigationTree(s, m);
  for (const [x, y] of [
    [-1, 1],
    [5, 1],
    [1, 3],
    [NaN, 1],
    [Infinity, 1],
  ])
    assert.equal(route({ x, y }), null);
  assert.equal(route({ x: 3.5, y: 1.5 }).cell, 11);
  m.blocked.set("6:11", 200);
  assert.equal(navigationTree(s, m)({ x: 3.5, y: 1.5 }), null);
  s.hud.elapsedMs = 200;
  assert.equal(navigationTree(s, m)({ x: 3.5, y: 1.5 }).cell, 11);
});

test("commander tries reachable supplies after an unreachable nearest pickup", () => {
  const s = navigationFixture(),
    m = progressionMemory();
  m.items.set(1, { id: 1, category: 1, slot: 1, x: 2.5, y: 1.5 });
  m.items.set(2, { id: 2, category: 1, slot: 2, x: 3.5, y: 1.5 });
  assert.equal(plan(s, m).target.id, 2);
});

test("memory preserves an unchecked pickup behind the player and expires route failures", () => {
  const s = navigationFixture(),
    m = progressionMemory();
  m.items.set(1, { id: 1, x: 2.3, y: 1.5 });
  m.blocked.set("6:11", 100);
  m.failed.set("item:1", 99);
  remember(s, m);
  assert.ok(m.items.has(1));
  assert.equal(m.blocked.size, 0);
  assert.equal(m.failed.size, 0);
  s.hud.x = 2.1;
  remember(s, m);
  assert.equal(m.items.size, 0);
});

test("commander finishes an ongoing reload before switching to a preferred weapon", () => {
  const m = progressionMemory(),
    h = {
      weapon: 0,
      ammo: 0,
      reserve: 20,
      reloading: true,
      elapsedMs: 2000,
      prompt: 0,
      inventory: [
        { id: 1, magazine: 0, reserve: 20 },
        { id: 3, magazine: 30, reserve: 60 },
      ],
    },
    v = { visible: [], nearbyDoor: false, magazineCapacity: 12 };
  assert.equal(commanderUtility(h, v, m), "nothing");
  assert.equal(m.equipUntil, 0);
  h.reloading = false;
  assert.equal(commanderUtility(h, v, m), "equip_3");
  assert.equal(m.equipUntil, 3500);
});

test("compact schemas accept only offered actions and commander is explicitly a high-level executor", async () => {
  const { w } = await loadScenario();
  const m = progressionMemory(),
    s = snapshot(w);
  remember(s, m);
  const v = view(s, m);
  const a = compactRequest(v, "compact"),
    b = compactRequest(v, "commander");
  assert.ok(JSON.stringify(a.state).length < 650);
  assert.deepEqual(Object.keys(b.questions), ["goal"]);
  assert.throws(
    () => chooseAnswers({ goal: { type: "choice", choice: "teleport" } }, b.questions),
    /Invalid/,
  );
  assert.ok(!JSON.stringify(b.state).includes("nodeX"));
});

test("landmark memory prefers an ordinary unowned weapon and cannot acquire hidden items", async () => {
  const { w } = await loadScenario();
  const s = snapshot(w),
    m = progressionMemory();
  remember(s, m);
  assert.equal(m.items.size, s.items.length);
  assert.ok(
    ![...m.items.values()].some((i) => i.category === 6),
    "unseen remote boss console must not be invented",
  );
  s.items = [{ id: 77, category: 1, x: s.hud.x + 0.2, y: s.hud.y, slot: 1 }];
  remember(s, m);
  assert.equal(plan(s, m).target.id, 77);
  s.items = [];
  remember(s, m);
  assert.ok(!m.items.has(77), "collecting a nearby pickup clears its remembered target");
});

test("normal inputs can complete the sector and every tactical segment replays exactly", async () => {
  const { w } = await loadScenario(),
    m = progressionMemory(),
    actions = [];
  let s = snapshot(w);
  for (let n = 0; n < 30 && s.hud.state === 0; n++) {
    remember(s, m);
    const v = view(s, m),
      result = executeGoal(w, referenceChoices(v, "commander"), m, "commander");
    actions.push(result);
    s = result.snapshot;
  }
  assert.equal(s.hud.state, 2, "boss defeat and real reward pickup are required");
  assert.equal(s.hud.objective, 1);
  assert.ok(s.hud.ownedWeapons.includes(8));
  const replay = await loadScenario();
  for (const action of actions) {
    let played;
    for (const seg of action.segments) played = step(replay.w, seg.input, seg.frames);
    assert.equal(fingerprint(played.snapshot), action.stateHash);
  }
});

test("policy executor contains no debug, grant, checkpoint-write or god-mode actions", async () => {
  const { readFile } = await import("node:fs/promises");
  const code = await readFile("scripts/ai/progression.mjs", "utf8");
  assert.ok(!/hs_qa|hs_load|hs_next_wave|memory\.buffer/.test(code));
});

test("persistent controls request another decision when an enemy first appears", async () => {
  const { w } = await loadScenario(),
    m = progressionMemory();
  const s = snapshot(w);
  remember(s, m);
  assert.equal(view(s, m).visible.length, 0);
  const result = executeGoal(w, { movement: "navigate", combat: "scan" }, m, "persistent");
  assert.equal(result.reason, "enemy_appeared");
  assert.ok(result.frames < 720);
  assert.ok(
    compactRequest(view(result.snapshot, m), "persistent").questions.combat.criteria.engage,
  );
});

test("model sees named mission interactions and weapon roles instead of opaque HUD codes", async () => {
  const { w } = await loadScenario(),
    m = progressionMemory(),
    s = snapshot(w);
  remember(s, m);
  const v = view(s, m);
  v.interactPrompt = 13;
  v.inventory.push({ id: 2, magazine: 8, reserve: 48, capacity: 8 });
  v.weaponRoles[2] = "Shotgun";
  const request = compactRequest(v, "persistent");
  assert.equal(request.state.prompt, "activate node");
  assert.match(request.questions.utility.criteria.equip_2, /Shotgun/);
  assert.ok(request.questions.utility.criteria.interact);
});

for (const startTurn of [0, 0.35, -0.35, 0.7, -0.7]) {
  test(`commander completes sector from yaw ${startTurn} with deterministic replay`, async () => {
    const { w } = await loadScenario(),
      m = progressionMemory(),
      actions = [];
    step(w, { bits: 0, mx: startTurn / 0.0062, my: 0 }, 1);
    let s = snapshot(w);
    for (let n = 0; n < 30 && s.hud.state === 0; n++) {
      const goal = n === 2 ? "progress" : "aggressive";
      const result = executeGoal(w, { goal }, m, "commander");
      actions.push(result);
      s = result.snapshot;
    }
    assert.equal(s.hud.state, 2);
    assert.ok(s.hud.health > 0);
    assert.equal(s.hud.bossHealth, 0);
    assert.ok(s.hud.ownedWeapons.includes(8), "real boss reward must be collected");
    const replay = await loadScenario();
    step(replay.w, { bits: 0, mx: startTurn / 0.0062, my: 0 }, 1);
    for (const action of actions) {
      let played;
      for (const seg of action.segments) played = step(replay.w, seg.input, seg.frames);
      assert.equal(fingerprint(played.snapshot), action.stateHash);
    }
  });
}
