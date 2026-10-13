# Boss attack readability audit

All 25 bosses pass checks across three phases and both alternating attack cycles
(150 phase/cycle cases), plus four close-range distances per boss. Windups lock
aim and leave visible lane markers; no attack damage or projectile can occur
before release. Recovery opens the shield and does not emit another volley.

Close-range shots retain at least **350 ms** of travel toward a stationary player
after release; moving into a projectile can shorten it. Emissions
within 0.75 world units of the player are suppressed. Gravity capture resolves
before travel-speed limiting, so pulling the player cannot erase that window.
Target disks and teleport destinations remain locked to the warned position.
Normal longer-range speeds and each boss's distinct volley geometry remain intact.

The table measures the real engine release in an open arena; speed is in world
units/second, time in seconds. Environmental floor hazards retain their separate
warning/active stages and existing all-sector damage tests. This audit covers
mechanical readability; subjective clarity still benefits from human playtesting.

| Sector | Attack | Tell | Recovery | Shot speed | Max shots | Dodge cue |
|---|---|---|---|---|---|---|
| 1 | Override rails | 1.0 | 1.5 | 6.5–8.5 | 2 | Step between the paired rails |
| 2 | Forge cutter | 1.2 | 1.8 | 5.0–5.0 | 5 | Clear the parallel cutter lanes |
| 3 | Corrosive bloom | 1.1 | 1.6 | 3.4–4.8 | 5 | Strafe clear of the acid cluster |
| 4 | Predictor cross | 1.0 | 1.5 | 4.0–7.0 | 3 | Move away from the crossing lanes |
| 5 | Sink capture | 1.3 | 1.9 | 4.1–4.1 | 5 | Leave the marked gravity disk |
| 6 | Archive braid | 1.1 | 1.6 | 5.0–8.0 | 4 | Keep moving through the alternating rails |
| 7 | Rime crescent | 1.2 | 1.7 | 3.6–3.6 | 7 | Break sight to avoid the cryo fan |
| 8 | Relay cascade | 0.9 | 1.5 | 4.0–7.6 | 5 | Strafe across the shock lanes |
| 9 | Titan siege | 1.4 | 2.0 | 3.0–4.4 | 3 | Leave the converging ordnance lanes |
| 10 | Kest triplet | 1.0 | 1.5 | 5.6–8.0 | 3 | Keep strafing through all three pulses |
| 11 | Memory echoes | 1.3 | 1.9 | 3.3–6.8 | 6 | Wait for the slower trailing echoes |
| 12 | Cinder fork | 1.2 | 1.7 | 3.3–5.4 | 3 | Move between the split incendiary lanes |
| 13 | Undertow surge | 1.2 | 1.8 | 4.4–6.0 | 5 | Watch for the faster pressure harpoons |
| 14 | Spectrum lattice | 1.1 | 1.6 | 6.2–6.2 | 5 | Move out of the converging prism lanes |
| 15 | Polarity capture | 1.3 | 1.9 | 4.3–4.3 | 6 | Leave the magnetic target disk |
| 16 | Hatchery deployment | 1.4 | 2.0 | 3.8–3.8 | 4 | Dodge the spokes; clear the spawned drone |
| 17 | Spore drift | 1.3 | 1.8 | 2.6–3.5 | 7 | Clear the slow drifting spore cloud |
| 18 | Starfall array | 1.4 | 2.0 | 3.4–5.2 | 5 | Keep moving past the staggered missiles |
| 19 | Reclaimer comb | 1.1 | 1.6 | 7.2–7.2 | 9 | Clear the wide flechette wall |
| 20 | Veil ambush | 1.2 | 1.8 | 4.8–7.3 | 2 | Leave the marked position before the jump |
| 21 | Helios corona | 1.4 | 2.0 | 4.0–4.0 | 11 | Use the gap directly ahead of the boss |
| 22 | Torpedo pincer | 1.3 | 1.9 | 3.2–4.6 | 2 | Clear both converging torpedo paths |
| 23 | Epoch tripwire | 1.3 | 1.8 | 3.0–7.0 | 3 | Dodge the fast pulse and trailing echoes |
| 24 | Glacier chevron | 1.4 | 2.0 | 3.5–3.5 | 5 | Leave the ice chevron before impact |
| 25 | Dominion command | 1.5 | 2.1 | 4.2–6.2 | 6 | Dodge the alternating salvo; clear the escort |

Machine-readable measurements: [boss-attack-readability.json](boss-attack-readability.json).
Re-run the audit with:

```sh
BLACKSITE_ATTACK_AUDIT="$PWD/.blacksite/attack-readability.json" \
  cargo test --release --manifest-path engine/Cargo.toml audit_every_boss
```

Reachability coverage flood-fills at quarter-cell resolution with the actual
player collision radius. All authored sectors and representative endless waves
26, 50 and 100 are checked with three fixed seeds: initial entities, jittered
reinforcement copies, ambush groups, boss placements and escorts. Objective,
node, terminal and reward approach positions must have a route and line of sight.
Doors are treated as openable; existing tests separately verify USE, lockdown
release and door collision behavior. Synthetic disconnected/narrow maps verify
that the checker rejects unreachable placements.
