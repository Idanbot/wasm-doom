# Enemy authoring guide

How to add a new enemy (or prop, pickup, effect) to HELLSCAN without
touching engine systems. The roster is data: one row per kind in
`engine/src/enemies.rs` (`ENEMY_DEFS`). Art spec lives in
`docs/monster-sprite-design.md`; this doc covers code, grids and sizes.

## 1. The grids and sizes you must know

### World grid (simulation)

| Fact | Value |
|---|---|
| Map size | **48 × 32 cells** (`MAP_W` × `MAP_H`) |
| Cell size | **1 world unit**; positions are `f32` unit coords |
| Cell codes | `0` open · `1–7` walls · `8` door · `9` secret door · `10` walkable trigger/gate |
| Player radius | **0.22** (`pr`); walk ≈ 3.4 u/s, enemies 1.7–2.7 u/s — you outrun everything, Doom-style |
| Entity cap | **192** live ents (`ENT_N`); `spawn()` returns `None` when full |

Wall kinds map to atlas slots in `wall_tex`: `1→METAL, 2→BRICK,
3→FLESH, 4→PIPES, 5→SKULL, 6→TECH, 7→HAZARD`, `8→DOOR`, `9→SECRET`.
Floor kinds: `0` normal, `1` alt, `2` seal-flag (under the vault seal).

### Texture atlas (renderer)

| Fact | Value |
|---|---|
| Atlas | **256×256 RGBA** per slot (`TEX`), **29 slots** (`TEX_N`, ids `0–28`) |
| Hostile sheets | **2×2 grid of 128×128 cells**, frame `(anim·4) mod 4` → cell `(fr&1, fr>>1)` |
| Single images | Whole 256×256 sampled (props, pickups, projectiles) |
| Upload | Stretched to 256×256 with smoothing **off**; procedural fallback generated if the PNG is missing |
| Transparency | Flood-filled from the atlas edge: alpha < 16, magenta (`R>220, B>220, G<40`), or near-black (`max(R,G,B) < 28`). **Never paint silhouettes pure black** |
| Billboard | Square (`sprite_w = sprite_h`), nearest sampling |

### The `EnemyDef` size fields

```
sprite_h = screen_h / depth * scale        // world-height multiplier
voff     = zoff / depth                    // vertical screen offset, px at depth 1
```

- `radius` — collision circle in world units. Player 0.22; Husk 0.28, Brute 0.38, Boss 0.62. Pickups 0.22 (generous), sparks 0.06.
- `scale` — world height. Husk 0.95 (human-ish), Brute 1.25, Wraith 0.7, Boss 2.45 (reuses the Brute sheet ×2.45 + red tint).
- `zoff` — `0.0` = feet planted on the floor. Floats go **negative** (Wraith −70); hanging/floor items go positive (Lamp −118 hangs from ceiling, Crate +86 sits up, pickups +34 float at hand height, Barrel +78).
- `hp` — Husk 28 / Brute 78 / Wraith 20 / Boss 520 (×1.5 per wave). Props: Barrel 14 (explodes), everything else 1.

### Combat behavior (built in, per kind)

| Kind | Speed | Hold range | Attack |
|---|---|---|---|
| Husk | 1.7 | 3.6 | Melee bite |
| Brute | 2.15 | 0.95 | Melee + spread shot |
| Wraith | 2.7 | 3.2 | Ranged bolt |
| Boss | 1.28 | 1.45 | Barrage + summons |

New hostiles automatically get chase (direct LOS ≤ 16u, else flow-field
pathing ≤ 22u), separation, hit-flash, stun and the wave lifecycle.
Custom attacks still need code in the AI section — the table covers
stats, art and lifecycle, not bespoke boss patterns.

## 2. Template: add a row

```rust
// engine/src/enemies.rs — copy a row, e.g. a fast charger:
EnemyDef {
    kind: EK_CHARGER,   // 1. new `pub(crate) const EK_CHARGER: u8 = 25;` in consts.rs
    name: "Charger",
    role: "skirmisher",
    hp: 34,
    radius: 0.30,
    zoff: 0.0,
    scale: 1.05,
    texture: T_CHARGER, // 2. new `T_CHARGER` id + PNG in TEX_FILES (runtime.ts)
    sheet4: true,       // animated → 4 frames; false → single image
    hostile: true,      // chases, counts as living, cleared per wave
    cleared_on_wave: true,
},
```

## 3. Checklist (nothing else to touch)

1. **Art** per `monster-sprite-design.md` → `public/game/spr_<name>.png`.
2. **`TEX_FILES`** in `src/game/runtime.ts`: `{ id: <T_*>, src: "/game/spr_<name>.png" }` — id must equal the `T_*` const.
3. **`EK_*`** const in `engine/src/consts.rs` (next free `u8`: 25+).
4. **`EnemyDef` row** in `engine/src/enemies.rs` (copy the TEMPLATE comment there).
5. **Placement** in `engine/src/map.rs`: `place_hub_spoke`, a spawn group, or an `AMBUSH_DEFS` entry.
6. **Tests**: `cargo test` pins roster uniqueness, hostile set, legacy tuning and map connectivity — extend them if you add zones.
7. **Rebuild**: `npm run build:wasm` (writes the binary + source hash), verify `npm run check:wasm`.

`spawn()`, the renderer, wave cleanup, the living count and the AI
filter all read `ENEMY_DEFS` — there is no other match on kinds to update.
