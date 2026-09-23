# BLACKSITE IFRIT — roster mapping (spec → engine v1)

The engine ships **4 hostile archetypes + danger props** (`engine/src/enemies.rs`).
Each spec unit maps onto an archetype for HP and hitbox. Skin-specific
movement, range, windup, damage and attacks live in `engine/src/combat.rs`.
See [COMBAT.md](COMBAT.md) for the attack and counterplay rules.
Units needing new AI are marked DEFERRED with the missing system named.

## Archetypes (unchanged stats)

| Archetype | Engine rows | Base HP |
|---|---|---|
| grunt | EK_HUSK | 28 |
| tank | EK_BRUTE | 78 |
| skirmisher | EK_WRAITH | 20 |
| boss | EK_BOSS | 520 (scaled by wave) |
| volatile prop | EK_BARREL | 14 |

## v1 cast (delivered art and combat roles)

| Spec unit | Archetype | Notes |
|---|---|---|
| Directorate Rifleman | grunt | medium-range single shots, 0.45s windup |
| Breacher | grunt | closes to 2.8 units, three-projectile spread |
| Subject (failed augment) | grunt | fast melee chaser |
| Hazmat Security | tank | slow ranged guard, 0.65s windup |
| Heavy Gunner | tank | three-projectile spread, 0.8s windup |
| Loader (industrial mech) | tank | heavy melee fantasy ✓ |
| Vat-grown Brute | tank | 1:1 with current Brute role |
| Marksman | skirmisher | long-range pressure via fast ranged bolt; narrow profile + optic per spec |
| Hornet (attack quadcopter) | skirmisher | flying attacker ✓ (float zoff) |
| Hound (quadruped robot) | skirmisher | fast melee with a dodgeable 0.3s windup |
| Spitter (bio ranged) | skirmisher | green acid bolts, medium-range strafing |
| Martyr Drone | volatile prop | explosive one-way drone = barrel behavior exactly |
| VEYRAN // MALIK (final) | boss | single-phase v1; 3-phase design (§bosses) needs phase system (deferred) |

## DEFERRED (needs engine work — named for planning)

| Spec unit | Missing system |
|---|---|
| Sentinel (turret) | stationary attacker AI (current AI always chases) |
| Shield Guard | directional armor (front-shield damage gate) |
| Watcher (recon drone) | non-combat observer behavior; ship as decor prop first |
| Crawler / Crawler-B (wall-crawler) | wall-walking movement + ceiling sprites |
| Seraph Unit, Command Cyborg, Neural Husk, Execution Platform | endgame roster; map onto tank/skirmisher rows once art lands, or extend table |
| BULWARK / MOTHER / JANISSARY PRIME | multi-boss campaign flow (engine has one seal-ritual boss) |

## Art asset IDs (must match `art/blacksite-manifest.json`)

`enemy_rifleman`, `enemy_breacher`, `enemy_subject`, `enemy_hazmat`,
`enemy_gunner`, `enemy_loader`, `enemy_vatbrute`, `enemy_marksman`,
`enemy_hornet`, `enemy_hound`, `enemy_spitter`, `enemy_martyr`,
`enemy_veyran`. Prop/decor conversions (`prop_turret_broken`,
`prop_watcher_perch`) ship as non-hostile art when ready.
