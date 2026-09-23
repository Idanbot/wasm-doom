# BLACKSITE IFRIT — weapon mapping

The engine now carries seven weapon slots. Each has distinct magazine size,
cadence, recoil, reload animation, audio treatment and first-person silhouette.

| Slot | Weapon | Combat identity                                        |
| ---- | ------ | ------------------------------------------------------ |
| 0    | MK23-S | suppressed 12-round precision sidearm                  |
| 1    | M870K  | six-shell close spread with stagger                    |
| 2    | VX-9   | 32-round automatic carbine with growing spread         |
| 3    | SHRIKE | four-shot rail rifle that pierces three targets        |
| 4    | RAVEN  | six-round shoulder missile launcher with splash damage |
| 5    | ARC-12 | eight-charge electrical precision rifle                |
| 6    | M56    | 80-round rotary cannon for sustained suppression       |

Magazine sizes are `MAG_SZ = [12, 6, 32, 4, 6, 8, 80]`. Number keys 1–7,
mouse wheel and the mobile weapon button select owned weapons. ARC-12 and M56
are experimental issue weapons available immediately; the five field weapons
retain their map pickup progression.

Every weapon uses a newly generated 1024px master in
`art/source_hd/weapons_v2/`. The deterministic processor keys the magenta
background and produces one idle view plus four recoil/fire and four reload
frames without changing weapon geometry between states. RAVEN's master is an
explicit rectangular launch tube with a wide missile muzzle and targeting
optic, rather than the old flamethrower-like silhouette.

Projectile and impact effects remain separate from weapon body art. RAVEN
launches the existing swept missile projectile and produces explosion effects
at collision; ARC-12 layers short electrical beam segments over hitscan; M56
uses rapid hitscan, growing spread, casings and a heavy mechanical report.
