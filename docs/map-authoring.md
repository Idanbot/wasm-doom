# Map authoring guide

The level is code, not data: `engine/src/map.rs` builds the hub-and-spoke
map with generic carvers (`room`, `hall_h`, `hall_v`, `pillar`,
`mix_edge` in `lib.rs`). A second map is a new `build_*`/`place_*` pair.

## 1. Current plan: hub-and-spoke (48×32)

```
y  1 ┌─────────────┐
     │  TECH LAB   │  BR-12 · brute+wraith · A-north closet
     │  x16-28     │
y  6 └──────╳──────┘  north lane x22-23 (door y8)
y  7       ║
y 10 ┌─────╨──────────────┐  ┌─────────┐
     │                    │  │ CHAPEL  │  MR-4 · brute line
     │  CENTRAL PLAZA     ├──┤ x33-41  │
y 12 │  x16-29 · med/ammo │S2│    ║    │  S2 secret vent (KX-9)
     │  bait, pipe block  │  └────╨────┘  east lane y15-16 (door x31)
y 15 ═╬═══════╬═══════════╬════╬══  ═════
y 19 │HANGAR   ║  VAULT DOORS (36-37,18-19, seal texture)
     │x1-11    ║  ┌─────────────┐
y 21 └─────────╨──╨─────────────┘  south lane x22-23 (door y22)
y 23 ┌─────────╨────────┐ ┌──────────────┐
     │   FLESH PIT      │ │    VAULT     │  seal x38-45/y19-27
     │   x16-28 · VLK-6 │ │    x33-45    │  boss arena · crypt S3
y 29 └──────────────────┘ └──────────────┘
     S1 cache (armor) under hangar · S3 crypt west of vault
```

Zones and their jobs: **hangar** (safe start, 1 medkit), **plaza**
(vista + bait that commits the player mid-arena), **lab/chapel/pit**
(one weapon each behind a fight), **vault** (seal ritual → boss),
**lanes** (warm-up + ambush thresholds), **secrets** (armor / KX-9 /
crypt stock).

## 2. Building blocks

```rust
e.room(x, y, w, h, wall, floor);   // walled rect, edge=wall kind, inside open
e.hall_h(x0, x1, y, door_x);       // 2-tall horizontal carve + door pair
e.hall_v(x, y0, y1, door_y);       // 2-wide vertical carve + door pair
e.pillar(x, y, kind);              // 2×2 solid block (cover, landmarks)
e.mix_edge(x, y, w, h, &[kinds]);  // cycle edge cells through wall kinds
e.set_cell(x, y, v);               // raw codes: 0 open, 1-7 walls, 8 door, 9 secret, 10 gate
```

- **Doors (`8`)** auto-open on player proximity; render with `T_DOOR`
  (the vault pair renders `T_SEAL` via `is_boss_door`).
- **Secret doors (`9`)** need USE (`E`); each one found increments
  `secrets` — the HUD/leaderboard side reads it, so keep pockets stocked.
- **Ambushes**: add an `AmbushDef { zone, group }` — monsters spawn once
  when the player enters the rect, with a wall-rumble (`EV_DOOR`) cue.
  Two patterns in use: **alcove closets** (carved pockets beside lanes)
  and **arena-rise** (spawns at the far side, no geometry).
- **Waves**: `HOSTILES` (14 entries) spawns at open, `spawn_hostiles(mult)`
  repeats with jitter per wave (mult doubles to 2^8, cap wave 12).
  Every coordinate must be an open cell.
- **Boss**: stepping on the seal with zero hostiles alive starts the
  6s intro → arena goes hell → boss spawns at `BOSS_SPOTS`.

## 3. Rules (enforced by tests where noted)

1. **Connectivity**: every weapon, the seal, all secrets and all ambush
   pockets must flood-fill from spawn (test:
   `hub_spoke_zones_are_all_connected`; doors/secrets count as passable).
2. **Spawn cells must be open** — `spawn()` does not check walls; a
   coordinate inside a pillar traps the monster forever.
3. **Never spawn on a door cell** — the monster would sit inside a closed
   door until the player opens it.
4. **One weapon per zone**, escalating: MK23-S (start) → BR-12
   (lab) → MR-4 (chapel) → VLK-6 (pit); KX-9 hides in secret S2.
5. **Tough fight ⇒ better gun or escape route nearby** (Romero's rule);
   lanes are always the retreat path.
6. **Secrets ≥ 2, stocked** with something worth the detour.
7. After editing: `cargo test`, `npm run build:wasm`, `npm run check:wasm`.
