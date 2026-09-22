# BLACKSITE IFRIT — roster mapping (spec → engine v1)

The engine ships **4 hostile archetypes + danger props** (`engine/src/enemies.rs`).
Same logic, new cast: every spec unit below maps onto one archetype row
(stats, hitbox, behavior identical — only art, name and placement change).
Units needing new AI are marked DEFERRED with the missing system named.

## Archetypes (unchanged stats)

| Archetype | Engine rows | HP | Speed | Role |
|---|---|---|---|---|
| grunt | EK_HUSK | 28 | 1.7, melee | chaser |
| tank | EK_BRUTE | 78 | 2.15, melee + spread | bruiser |
| skirmisher | EK_WRAITH | 20 | 2.7, ranged bolt, floats | harasser |
| boss | EK_BOSS | 520 (×1.5/wave) | 1.28, barrage | seal ritual |
| volatile prop | EK_BARREL | 14, explodes | — | area denial |

## v1 cast (delivered art, shared AI)

| Spec unit | Archetype | Notes |
|---|---|---|
| Directorate Rifleman | grunt | mid-map ranged fantasy; engine chases to melee — place in lanes so it reads as patrol |
| Breacher | grunt | close-range fantasy matches the chase; shotgun role told through art + placement |
| Subject (failed augment) | grunt | JANUS flavor, ambush closets |
| Hazmat Security | tank | armored radiation guard; slow-bruiser stats fit |
| Heavy Gunner | tank | oversized weapon silhouette per spec §20 |
| Loader (industrial mech) | tank | heavy melee fantasy ✓ |
| Vat-grown Brute | tank | 1:1 with current Brute role |
| Marksman | skirmisher | long-range pressure via fast ranged bolt; narrow profile + optic per spec |
| Hornet (attack quadcopter) | skirmisher | flying attacker ✓ (float zoff) |
| Hound (quadruped robot) | skirmisher | fast melee-ish harasser |
| Spitter (bio ranged) | skirmisher | biological bolt flavor |
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
