# Decider sector-completion experiments

The experiment tier runs the pinned Decider CPU model against an ordinary sector
start. Completion means the engine actually enters the won state after defeating
the boss and collecting its reward. Activating the node or damaging a boss does
not count as completion. Stock health/ammo, normal weapon cases and normal
simulation inputs are used throughout gameplay.

Three approaches share the same observation sources and initial conditions:

| Approach | Model decisions | Local execution | Max interval |
| --- | --- | --- | --- |
| compact | Movement, combat, utility | Route/aim refinement; edge-triggered interaction | 2 seconds |
| persistent | Same compact typed choices | Same executor, batched minor combat events | 12 seconds |
| commander | Mission policy: progress, cautious, aggressive | Explicit local aiming, weapon selection, reload, interaction, wall/hazard avoidance and movement | 24 seconds |

The commander is a hybrid QA player. Its results must not be presented as pure
model tactical skill: Decider chooses a policy, and the executor handles ordinary
FPS controls. Every intervention is counted and every actual input segment is
recorded and replayed. The other approaches retain model-selected utility/combat
choices. Poor play, system failure and budget-inconclusive episodes are reported
separately. A repeated deterministic success with identical inputs is not a
statistical win rate; varied starting turns test robustness to different inputs.

The read-only `hs_prepare_agent_items` / `hs_agent_items` presentation exports
provide visible weapon/ammo/health/armor pickups, the node, boss console and
reward. Cues are bounded, restricted to line of sight and horizontal field of
view, and never expose hidden inventory, spawn scripts or enemy internals. The
agent remembers only items it has actually observed. Enemy contacts used for
navigation obey the same near/seen rules as the shipped minimap. As in the
original prototype, pathfinding uses the available full map geometry; the
minimap's explored cells constrain contacts, not the BFS geometry. Floor hazards
are used only to avoid the local next movement position.

The executor steers movement in world space while aiming, pulses and releases
USE, pursues remembered supplies, and handles the mission sequence: node → clear
hostiles → discovered console → boss → visible reward. It neither teleports nor
patches health/ammo, grants guns, clears enemies, skips sectors or calls debug
helpers during gameplay. AI is never a production dependency.

## Local commands

Use the existing pinned CPU environment/checkpoint cache:

```sh
BLACKSITE_AI_MODEL=decider \
BLACKSITE_AI_PYTHON=.blacksite/decider-venv/bin/python \
BLACKSITE_AI_APPROACHES=compact,persistent,commander \
BLACKSITE_AI_REPEATS=1 BLACKSITE_AI_TOTAL_SECONDS=1800 \
BLACKSITE_AI_OUTPUT=.blacksite/decider-approaches \
node --experimental-strip-types scripts/ai/experiments.mjs
```

After choosing an approach, test five different ordinary starting orientations:

```sh
BLACKSITE_AI_MODEL=decider \
BLACKSITE_AI_PYTHON=.blacksite/decider-venv/bin/python \
BLACKSITE_AI_APPROACHES=commander BLACKSITE_AI_REPEATS=5 \
BLACKSITE_AI_START_TURNS=0,0.35,-0.35,0.7,-0.7 \
BLACKSITE_AI_TOTAL_SECONDS=1800 \
BLACKSITE_AI_OUTPUT=.blacksite/decider-completion \
node --experimental-strip-types scripts/ai/experiments.mjs
```

The model loads once and is reused sequentially across episodes. Existing GGUF
revision/checksum and CPU settings remain pinned. The total wall budget is shared
fairly; unused allowance rolls forward. The suite is capped at 512 decisions as well as its wall budget. Maximum decisions are bounded per episode
and initial starting turns are ordinary mouse inputs, replayed identically.
Reports include completion, death, budget exhaustion, model confidence,
inference latency, damage, shots, reloads, objective/boss progression and local
executor intervention counts. Raw traces and reports remain ignored local QA
artifacts; measured conclusions are recorded below after execution.

## First measured CPU pilot

The first completed real-model commander episode (local i7-10510U, four CPU
inference threads, pinned v2.1 Q4_K_M) won sector 1 in **eight decisions**:
83.3 simulation seconds, 31 kills, node activated, boss defeated and reward
collected. It finished with 97 health. Inference totaled 136.9 seconds, median
17.7 seconds per call; model process peak RSS was 4,966 MiB. Chosen-goal
probability averaged 58%, which is a diagnostic rather than a success score.
All eight selected goals were aggressive. Every normal input replayed exactly.

The initial twelve-second direct-control pilot died after eight decisions and
46.9 simulation seconds, with six kills and no objective activation. This
exposed stale hold-fire controls after enemies appeared; the executor now
interrupts on enemy appearance, reload completion and boss phase transitions.
A short-control pilot also exposed opaque numeric interaction prompts. Requests
now use named node/console/door prompts and role descriptions for weapon-switch
choices. Those fixes preserve free model choices and do not grant inventory.

A preceding diagnostic pilot failed on a 60-second model timeout and consumed a
late response in the next episode. That pilot is excluded from gameplay
comparisons. Sessions now reject all later decisions after a timeout, discard
late replies and restart before another episode; experiment calls allow 120
seconds. Regression tests cover the protocol, bounded reports, view validation,
real sector victory, fresh-enemy interruptions and exact replay.

These are local measurements. Repeated starting-orientation tests on the actual
free GitHub CPU runner are required before claiming hosted completion
reliability. The dated original three-run review remains an integration baseline,
not evidence of a completed campaign.

## Initial comparison and hosted robustness test

The complete initial local comparison used the same sector start and stock
inventory. All input traces replayed exactly; no system failures occurred.

| Initial approach | Completion | Decisions | Simulation seconds | Inference seconds |
| --- | --- | ---: | ---: | ---: |
| Compact controls | Budget-inconclusive, node not activated | 64 | 102.7 | 1,260.9 |
| Persistent controls | Player death, node not activated | 8 | 46.9 | 270.3 |
| Hybrid commander | Complete, 97 HP | 8 | 83.3 | 136.9 |

The revised commander was then tested on the free GitHub runner in
[run 37843896931](https://github.com/Idanbot/wasm-doom/actions/runs/37843896931).
It completed four of five orientations in 344.2 seconds of harness time:
43 decisions, median inference 6.8–7.9 seconds per episode, approximately
4.8 GiB model RSS, no system failures and exact replay for all five traces.
The one normal death was at +0.35 radians during the boss's last phase.
**Integration PASS was not treated as meeting the completion target.**

Replaying that failure exposed a local-controller problem: it stopped before
some doors were reachable, then repeatedly tried a boss-sealed door to collect
supplies outside the arena. Stalled USE attempts were excluded from the movement
stuck detector. The executor now approaches doors while pulsing USE and counts
refused interactions toward blocked-route memory. It reroutes using its observed
failure, without inspecting hidden lock rules or changing the game. A regression
scenario replays the formerly failing tactical choices and requires a genuine
sector win. The corrected five-orientation hosted test is evaluated separately.

The corrected direct persistent controller was also tested with the real local
model: 17 decisions, 51.5 simulation seconds, node activation on decision three,
14 kills, then a normal player death. Median inference was 22.1 seconds, total
harness time 395.6 seconds, and replay passed. Semantic prompts improved objective
use, but direct movement/combat choices remain less robust than the commander.
