import { test } from "node:test";
import assert from "node:assert/strict";
import { verifiedCompletion, completionStatus } from "./experiment-results.mjs";

const win = { status: "COMPLETE", completed: true, deterministicReplay: true };
test("only victories with verified replay count toward completion", () => {
  assert.equal(verifiedCompletion(win), true);
  assert.equal(verifiedCompletion({ ...win, deterministicReplay: false }), false);
  const divergence = { ...win, status: "SYSTEM_FAILURE" };
  assert.equal(verifiedCompletion(divergence), false);
  assert.equal(completionStatus([divergence]), "NOT_MET");
  assert.equal(completionStatus([win]), "MET");
});

test("budget exhaustion is inconclusive but cannot mask system failure or an empty suite", () => {
  const budget = { status: "INCONCLUSIVE" };
  assert.equal(completionStatus([win, budget]), "INCONCLUSIVE");
  assert.equal(completionStatus([budget, { status: "SYSTEM_FAILURE" }]), "NOT_MET");
  assert.equal(completionStatus([]), "NOT_MET");
  assert.equal(completionStatus([{ status: "PLAYER_FAILURE" }]), "NOT_MET");
});
