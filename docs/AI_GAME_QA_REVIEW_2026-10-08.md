# Review of three completed gameplay runs

These are the last three actual completed gameplay executions, newest first.
Cancelled attempts and the report-only rescoring job 37675690334 are excluded.
The numbers below describe Decider; the oldest run also compared Laya.

| Run | Fixtures | Decisions | Simulated seconds | Median inference | Kills / deaths | Reloads | Objectives / sector wins |
| --- | --- | ---: | ---: | ---: | --- | ---: | --- |
| [37690762591](https://github.com/Idanbot/wasm-doom/actions/runs/37690762591) | 8 sectors + 3 bosses | 76 | 75.2 | 22.19s | 16 / 0 | 16 | 0 / 0 |
| [37681313235](https://github.com/Idanbot/wasm-doom/actions/runs/37681313235) | 8 sectors + 3 bosses | 155 | 118.3 | 10.65s | 16 / 0 | 13 | 0 / 0 |
| [37671589589](https://github.com/Idanbot/wasm-doom/actions/runs/37671589589) | 3 sectors | 93 | 63.7 | 17.63s | 16 / 1 | 6 | 0 / 0 |

All passed their system/integration checks, including state validity and
actual-input deterministic replay. None completed a sector. Neither of the two
runs with explicit boss fixtures reached a boss phase transition. Their PASS
therefore validates integration, not campaign progression or encounter balance.
The eleven-fixture runs hit their wall-time allowance in every episode.

The newest run finished in 1,757.89 seconds (29m18s) under the shared 1,800-second
budget, with about 4.94 GiB model peak RSS. No invalid model output was observed.
Empty-magazine time fell from 2.0 seconds to zero; all 40 visible combat decisions
chose engagement. Reloads improved, with no recorded stuck interval. However,
76 expensive inferences produced only 75.2 seconds of gameplay in total, spread
across eleven independent starts. Nine interaction attempts produced no recorded
interaction effect or objective activation. That metric detects objective/radio/
interaction event changes; it is not a complete measure of opened doors.

Movement/combat/utility mean selected probabilities in the newest run were
0.681 / 0.659 / 0.732. These are uncalibrated choice probabilities, not accuracy.
The previous run's values were 0.755 / 0.615 / 0.711. They do not establish that
one policy is better, particularly with different durations and sample counts.

## Recommended next changes

1. **Compact the model context first.** Mean observation JSON grew from about
   544 to 1,234 characters and question JSON from 737 to 1,034. The first logged
   state grew from 140 to 214 tokens; neither was truncated. Median inference
   rose from 10.65 to 22.19 seconds. The added context plausibly contributes,
   but these runs do not isolate prompt cost from runner performance. Use short
   weapon-role codes, bounded recent history and omit derivable/repeated fields;
   benchmark identical observations before changing models.
2. **Keep useful goals running between inferences.** In the newest run, 16 new
   enemies, 16 kills, 14 completed reloads and 9 damage events interrupted actions.
   Only seven reached the planned duration. Batch minor combat events and allow
   the existing local steering/aim controller to react while retaining a goal.
   Continue to record all resulting inputs and attribute controller interventions
   separately. Do not mask real stuck states or silently force wins.
3. **Give focused progression fixtures enough time.** Eleven independent starts
   divide CPU time too thinly for boss health totals and stock pistol loadouts.
   Keep broad engine smoke coverage, but use a separate manual objective/boss tier
   with fewer fixtures and longer per-fixture time. Test stock equipment and a
   documented legitimate campaign inventory as separate scenarios; do not grant
   invulnerability or silently skip gameplay.
4. **Assert interactions and boss counterplay directly.** Add dedicated system
   scenarios for console activation, reward collection, warning-before-damage,
   recovery, and phase progression. Record opened-door effects separately from
   objective effects and distinguish budget exhaustion from a broken objective.
   The boss-counterplay patch adds engine tests and observable attack HUD state;
   it does not claim the AI has completed a new encounter.
5. **Use controlled comparisons.** Reuse a fixed fixture/seed/input inventory,
   repeat runs, and compare latency, simulated seconds, objective activations,
   damage and wins. Keep system failures distinct from player failure and
   budget-inconclusive progression. Confidence remains diagnostic.

Keep pinned Decider on CPU for now. Improving token cost and useful simulation
per call is the strongest next experiment. In the older comparison Laya had a
2.64-second median latency but died in all three episodes and completed no
sector; it activated two objectives. That is useful evidence for a future
controlled objective-navigation comparison, not enough to justify switching the
entire suite or to claim Decider is more accurate.
