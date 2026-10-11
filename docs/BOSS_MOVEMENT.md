# Boss movement

Bosses move between attacks using deterministic radial and lateral steering.
Each profile has its own spacing, tempo, handedness and speed. Later phases
reduce preferred range by at most half a world unit; steering never exceeds
1.1 times the existing movement speed and still respects shields and slows.
The existing collision/slide rules apply. Without line of sight, bosses retain
flow-field navigation. Stun, attack warnings and exposed recovery remain stationary.
After recovery, bosses have 0.65–1.3 seconds to reposition before their next warning.

| Sector | Boss | Footwork |
| --- | --- | --- |
| 1 | Veyran | Rail sidesteps |
| 2 | Hecate | Clockwise arc |
| 3 | Chimera | Corrosive pressure surges |
| 4 | Oracle | Predictive figure-eight crossings |
| 5 | Gravemind | Slow reactor spiral |
| 6 | Archivist | Stop/start indexing |
| 7 | Halcyon | Guarded coolant march |
| 8 | Relay | Alternating signal flanks |
| 9 | Titan | Deliberate siege advance |
| 10 | Kest | Command feints |
| 11 | Mnemosyne | Reverse memory loop |
| 12 | Vulcan | Ember-lane zigzag |
| 13 | Leviathan | Tidal approach |
| 14 | Prism | Fast refracted figure eight |
| 15 | Magnus | Magnetic spiral |
| 16 | Broodmother | Wide swarm perimeter |
| 17 | Mycelium | Slow spore creep |
| 18 | Astra | Missile repositioning flanks |
| 19 | Scrapper | Rapid flechette sweep |
| 20 | Umbra | Stop/start phase footwork |
| 21 | Sol | Tight solar orbit |
| 22 | Nautilus | Reverse pressure surge |
| 23 | Chronos | Rapid reverse echo loops |
| 24 | Boreas | Glacial lateral sweep |
| 25 | Sovereign | Tightening crown spiral |

Sector 26 onward reuses these profiles in campaign order, alongside existing
endless difficulty scaling. This movement adds no new teleport or contact damage;
Umbra's existing warned ambush remains part of its separate attack system.

`engine/src/boss_movement.rs` owns profiles and steering. Its tests verify that
all 25 trajectories differ and remain bounded, and run each boss through real
simulation ticks to check movement, collision, stun, attack locks and exact replay.
