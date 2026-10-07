import { test } from 'node:test';
import assert from 'node:assert/strict';
import { choiceConfidence, distribution, summarizeDecisions, reloadAvailability } from './metrics.mjs';

test('confidence uses selected option probability rather than SDK entropy confidence', () => {
  const row = { choice: 'reload', probabilities: { nothing: 0.2, reload: 0.8 }, confidence: 0.01 };
  const value = choiceConfidence(row, { nothing: 'Wait', reload: 'Reload' });
  assert.equal(value.selectedProbability, 0.8);
  assert.ok(Math.abs(value.topTwoMargin - 0.6) < 1e-10);
  assert.ok(value.normalizedEntropy > 0 && value.normalizedEntropy < 1);
});

test('reload opportunities count only offered legal actions and expose controller gaps separately', () => {
  const observation = { magazine: 0, reserve: 20, reloading: false };
  const legal = { observation, questions: { utility: { criteria: { reload: 'Reload', nothing: 'Wait' } } }, action: { utility: 'reload' } };
  const unavailable = { observation, questions: {}, action: { utility: 'nothing' } };
  assert.deepEqual(reloadAvailability([legal, unavailable, { ...legal, observation: { ...observation, reloading: true } }]),
    { emptyReloadOpportunities: 1, reloadChoicesOnEmpty: 1, reloadNotOfferedDecisions: 1 });
});

test('rounded SDK probabilities normalize; malformed or absent telemetry is missing, never invented', () => {
  const criteria = { left: 'Left', right: 'Right', hold: 'Hold' };
  assert.ok(choiceConfidence({ choice: 'hold', probabilities: { left: 0.3333, right: 0.3333, hold: 0.3333 } }, criteria));
  for (const probabilities of [null, {}, { left: NaN, right: 0, hold: 1 },
    { left: -0.1, right: 0.1, hold: 1 }, { left: 0.1, right: 0.1, hold: 0.1 }]) {
    assert.equal(choiceConfidence({ choice: 'hold', probabilities }, criteria), null);
  }
});

test('latency percentiles, per-question confidence and action counts survive missing samples', () => {
  const questions = { utility: { criteria: { nothing: 'Wait', reload: 'Reload' } } };
  const records = [0.2, 0.8].map((p, n) => ({ questions, action: { utility: 'reload' }, inferenceSeconds: n + 1,
    answers: { utility: { choice: 'reload', probabilities: { nothing: 1 - p, reload: p } } } }));
  records.push({ questions, action: { utility: 'nothing' }, inferenceSeconds: 0, modelInference: false, answers: {} });
  const result = summarizeDecisions(records);
  assert.equal(result.confidence.utility.selectedProbability.mean, 0.5);
  assert.equal(result.confidence.utility.below50Percent, 1);
  assert.equal(result.confidence.utility.missing, 1);
  assert.equal(result.inferenceLatencySeconds.count, 2);
  assert.equal(result.inferenceLatencySeconds.p95, 2);
  assert.deepEqual(result.actionCounts.utility, { reload: 2, nothing: 1 });
  assert.equal(distribution([]).mean, null);
});
