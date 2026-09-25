# Map authoring guide

BLACKSITE uses three code-built 48×32 sectors in `engine/src/map.rs`. Generic carvers (`room`, `hall_h`, `hall_v`, `pillar`, and `mix_edge`) live in `lib.rs`; sector geometry, props, ambushes, hostile casts, objectives, and boss spawn points live in the map module.

## Campaign sectors

| Sector | Layout | Boss | Core ability |
|---|---|---|---|
| Nadir–7A: Upper Works | Hub and four spokes through hangar, lab, chapel, pit, and vault | Malik Veyran | Five-shot command fan and mixed tactical reinforcements |
| Nadir–7B: Cryogenic Foundry | Long production lanes with north/south cross-connections | HECATE–9 | Sustained heavy bursts, eight-way electrical barrage, and drone support |
| Nadir–7C: Bioforge Depths | Central containment loop with two specimen wings | CHIMERA–9 | Fast melee pursuit, corrosive nova, and hound/spitter support |

`level_index(wave)` cycles these sectors. `next_wave()` rebuilds the map, doors, props, ambushes, and hostile roster while preserving acquired weapons. A sector repeats only after all three layouts have been played.

## Building blocks

```rust
e.room(x, y, w, h, wall, floor);   // walled rectangle
e.hall_h(x0, x1, y, door_x);       // two-cell horizontal hall + door pair
e.hall_v(x, y0, y1, door_y);       // two-cell vertical hall + door pair
e.pillar(x, y, kind);              // 2×2 cover block
e.mix_edge(x, y, w, h, &[kinds]);  // themed wall cycle
e.set_cell(x, y, value);            // 0 open, 1–7 wall, 8 door, 9 secret
```

Doors (`8`) auto-open on approach. Secret doors (`9`) require USE and increment the secret count. `AmbushDef` entries spawn once when their rectangular trigger is crossed. Hostile casts are sector-specific `HostileSpawn` tables with explicit skins and positions.

## Boss override

Each sector ends at `override_point(wave)`, marked by floor style `2` and the industrial `floor_override.png` beacon. Clearing ordinary hostiles reveals the objective. The player must stand near it and press USE; simply walking across it cannot start the encounter. The six-second lockdown then changes arena lighting and spawns the sector-specific boss at `boss_spots(wave)`.

## Validation rules

1. Every override and both boss spawn points must flood-fill from that sector's player start.
2. Each hostile and prop coordinate must be on an open cell and outside door cells.
3. Keep combat cover between long sightlines and supplies beside the hardest rooms.
4. Preserve the three distinct traversal shapes when adding rooms.
5. Run `cargo test`, `npm run build:wasm`, and `npm run check:wasm` after engine changes.
