import assert from "node:assert/strict";
import { test } from "node:test";
import { VoiceGate } from "./voice-gate.ts";

test("one voice owns the lane until its playback ends", () => {
  const gate = new VoiceGate();
  let finishFirst = () => {};
  let finishSecond = () => {};
  const played: string[] = [];
  assert.equal(gate.tryStart((done) => { played.push("enemy"); finishFirst = done; }), true);
  assert.equal(gate.tryStart(() => played.push("other enemy")), false);
  gate.enqueue((done) => { played.push("boss"); finishSecond = done; });
  assert.deepEqual(played, ["enemy"]);
  finishFirst();
  assert.deepEqual(played, ["enemy", "boss"]);
  assert.equal(gate.tryStart(() => played.push("third")), false);
  finishSecond();
  assert.equal(gate.tryStart((done) => { played.push("third"); done(); }), true);
  assert.deepEqual(played, ["enemy", "boss", "third"]);
});

test("duplicate completion cannot unlock a later voice", () => {
  const gate = new VoiceGate();
  let first = () => {};
  let second = () => {};
  gate.tryStart((done) => { first = done; });
  gate.enqueue((done) => { second = done; });
  first();
  first();
  assert.equal(gate.busy, true);
  second();
  assert.equal(gate.busy, false);
});
