export function distribution(values) {
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b);
  if (!sorted.length) return { count: 0, mean: null, min: null, p10: null, p50: null, p95: null, max: null };
  const percentile = (p) => sorted[Math.ceil(p * sorted.length) - 1];
  return { count: sorted.length, mean: sorted.reduce((a, b) => a + b, 0) / sorted.length,
    min: sorted[0], p10: percentile(0.1), p50: percentile(0.5), p95: percentile(0.95), max: sorted.at(-1) };
}

// Both SDKs expose option probabilities; their fields named "confidence" have
// different meanings. Use chosen probability and compute entropy consistently.
export function choiceConfidence(answer, criteria) {
  const keys = Object.keys(criteria), raw = answer?.probabilities;
  if (keys.length < 2 || !raw || !Object.hasOwn(criteria, answer?.choice)) return null;
  const values = keys.map((key) => raw[key]);
  if (values.some((v) => !Number.isFinite(v) || v < 0 || v > 1)) return null;
  const sum = values.reduce((a, b) => a + b, 0);
  // Laya rounds probabilities to four places. Reject invalid telemetry, not
  // valid actions: missing/bad confidence never affects simulation inputs.
  if (Math.abs(sum - 1) > 0.002) return null;
  const probabilities = values.map((v) => v / sum), ranked = [...probabilities].sort((a, b) => b - a);
  return { selectedProbability: probabilities[keys.indexOf(answer.choice)],
    normalizedEntropy: -probabilities.reduce((h, p) => h + (p ? p * Math.log(p) : 0), 0) / Math.log(keys.length),
    topTwoMargin: ranked[0] - ranked[1], optionCount: keys.length };
}

export function summarizeDecisions(records) {
  const confidence = {}, actionCounts = {};
  for (const record of records) {
    for (const [name, question] of Object.entries(record.questions)) {
      actionCounts[name] ??= {};
      const action = record.action[name];
      actionCounts[name][action] = (actionCounts[name][action] ?? 0) + 1;
      confidence[name] ??= { valid: [], missing: 0 };
      const value = choiceConfidence(record.answers?.[name], question.criteria);
      if (value) confidence[name].valid.push(value); else confidence[name].missing++;
    }
  }
  return { inferenceLatencySeconds: distribution(records.filter((r) => r.modelInference !== false).map((r) => r.inferenceSeconds)),
    confidence: Object.fromEntries(Object.entries(confidence).map(([name, { valid, missing }]) => [name, {
      selectedProbability: distribution(valid.map((v) => v.selectedProbability)),
      normalizedEntropy: distribution(valid.map((v) => v.normalizedEntropy)),
      topTwoMargin: distribution(valid.map((v) => v.topTwoMargin)),
      below50Percent: valid.filter((v) => v.selectedProbability < 0.5).length,
      missing, diagnosticOnly: true,
    }])), actionCounts };
}

export function reloadAvailability(records) {
  const counts = { emptyReloadOpportunities: 0, reloadChoicesOnEmpty: 0, reloadNotOfferedDecisions: 0 };
  for (const r of records) {
    const o = r.observation;
    if (!o || o.magazine !== 0 || o.reserve <= 0 || o.reloading) continue;
    if (Object.hasOwn(r.questions.utility?.criteria ?? {}, 'reload')) {
      counts.emptyReloadOpportunities++;
      if (r.action.utility === 'reload') counts.reloadChoicesOnEmpty++;
    } else counts.reloadNotOfferedDecisions++;
  }
  return counts;
}
