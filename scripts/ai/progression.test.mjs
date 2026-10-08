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
} from "./progression.mjs";

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
