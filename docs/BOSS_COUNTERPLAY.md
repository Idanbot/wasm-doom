# Boss counterplay

All 25 sector bosses use locked, gun-inspired attacks. The charge animation,
world-space lane markers and HUD instruction give 0.9–1.5 seconds of warning.
The boss freezes its aim when the warning starts; lateral movement or cover
can defeat the attack. Markers and projectiles use the same authored geometry.
Origins inside walls or beyond sight from the boss are skipped.

After attacking, the boss stands still for 1.5–2.1 seconds with its shield open
and a raised core beacon. All weapons deal double damage during exposure. The
indicated weapon role deals another 25% during attack recovery (2.5× total,
after existing armor/combo calculations). This is a timed core vulnerability,
not a separate headshot hitbox. No weapon is mandatory. Ordinary hits preserve
the charge; stagger and shock interrupt it and require a fresh warning.

Phase transitions summon support and schedule a fresh warned attack instead
of an immediate projectile nova. Existing sector-specific floor hazards remain
independently telegraphed. Restarting, moving sectors, defeat or boss death clears
attack state. Levels above 25 cycle these profiles with existing endless tuning.

| Sector | Boss | Attack / ability | Counter | Recovery weapon role |
| --- | --- | --- | --- | --- |
| 1 | MALIK VEYRAN | Override rails | Step between the paired rails | Precision |
| 2 | HECATE–9 | Forge cutter | Clear the parallel cutter lanes | Shock |
| 3 | CHIMERA–9 | Corrosive bloom | Strafe clear of the acid cluster | Burn |
| 4 | ORACLE–7 | Predictor cross | Move away from the crossing lanes | Control |
| 5 | GRAVEMIND–4 | Sink capture | Leave the marked gravity disk | Blast |
| 6 | NULL ARCHIVIST | Archive braid | Keep moving through the alternating rails | Precision |
| 7 | HALCYON–3 | Rime crescent | Break sight to avoid the cryo fan | Burn |
| 8 | RELAY–0 | Relay cascade | Strafe across the shock lanes | Stagger |
| 9 | TITAN–12 | Titan siege | Leave the converging ordnance lanes | Precision |
| 10 | DIRECTOR KEST | Kest triplet | Keep strafing through all three pulses | Suppress |
| 11 | MNEMOSYNE–6 | Memory echoes | Wait for the slower trailing echoes | Shock |
| 12 | VULCAN–2 | Cinder fork | Move between the split incendiary lanes | Freeze |
| 13 | LEVIATHAN–8 | Undertow surge | Watch for the faster pressure harpoons | Shock |
| 14 | PRISM–5 | Spectrum lattice | Move out of the converging prism lanes | Stagger |
| 15 | MAGNUS–4 | Polarity capture | Leave the magnetic target disk | Acid |
| 16 | BROODMOTHER–3 | Hatchery deployment | Dodge the spokes; clear the spawned drone | Blast |
| 17 | MYCELIUM–9 | Spore drift | Clear the slow drifting spore cloud | Burn |
| 18 | ASTRA–7 | Starfall array | Keep moving past the staggered missiles | Precision |
| 19 | SCRAPPER–6 | Reclaimer comb | Clear the wide flechette wall | Shock |
| 20 | UMBRA–1 | Veil ambush | Leave the marked position before the jump | Control |
| 21 | SOL–12 | Helios corona | Use the gap directly ahead of the boss | Freeze |
| 22 | NAUTILUS–4 | Torpedo pincer | Clear both converging torpedo paths | Precision |
| 23 | CHRONOS–8 | Epoch tripwire | Dodge the fast pulse and trailing echoes | Suppress |
| 24 | BOREAS–11 | Glacier chevron | Leave the ice chevron before impact | Burn |
| 25 | THE SOVEREIGN | Dominion command | Dodge the alternating salvo; clear the escort | Shock |

Gravity capture affects only players who remain in the warned disk. Umbra jumps
to the warned position if it is clear, visible from the origin and away from the
player; it does not silently follow a dodge. Broodmother deploys one drone per
attack with a cap of three living drones. Sovereign deploys one shielded escort
per attack with a cap of two living escorts. Boreas raises its shield while
charging and drops it for recovery. These summons also respect the global
hostile limit. Different projectile speeds create staggered arrival rhythms;
they are emitted together, not delayed fire events.

The CPU smoke harness sees the same attack name, stage, countdown and counter
shown by the HUD. HUD fields are appended at offsets 236 and 240; the Rust/TS
HUD contract is now 244 bytes. No model or inference dependency ships in the game.

Verification: Rust tests cover all 25 patterns, warnings, aim locking, recovery,
weakness damage, interrupt rules, phase transitions, valid origins, gravity/ambush
dodging and summon caps. The browser check covers warning/recovery card layout on
desktop and mobile, plus state cleanup on death. Existing combat, arena, preload
and reward tests remain in place.
