# BLACKSITE IFRIT — weapon mapping

The engine now carries seven weapon slots. Each has distinct magazine size,
cadence, recoil, reload animation, audio treatment and first-person silhouette.

| Slot | Weapon         | Combat identity                                        |
| ---- | -------------- | ------------------------------------------------------ |
| 0    | MK23-S         | suppressed 12-round precision sidearm                  |
| 1    | BR-12 BREAKER  | 8-shot close spread with heavy stagger                 |
| 2    | KX-9 VECTOR    | 36-round automatic PDW with controlled spread          |
| 3    | MR-4 LONGBOW   | 5-shot magnetic rifle that pierces three targets       |
| 4    | VLK-6 WARDEN   | 4-cell guided micro-missile launcher with splash       |
| 5    | AX-12 VOLT     | 10-charge electrical precision carbine                 |
| 6    | M91 CYCLONE    | 90-round rotary cannon for sustained suppression       |

Magazine sizes are `MAG_SZ = [12, 8, 36, 5, 4, 10, 90]`. Number keys 1–7,
mouse wheel and the mobile weapon button select owned weapons. Weapons 2–7
enter the arsenal through their map pickups.

Weapons 2–7 use transparent high-resolution masters in
`art/source_hd/weapons_v3/`. The deterministic processor produces one idle
view, four recoil/fire frames and eight distinct reload frames. Each model is
held on the center sightline and uses the same 800×480 frame as the game view.
VLK-6 has an explicit launch aperture, four-cell cassette and targeting optic.

Projectile and impact effects remain separate from weapon body art. VLK-6
launches the existing swept missile projectile and produces explosion effects
at collision; AX-12 layers short electrical beam segments over hitscan; M91
uses rapid hitscan, growing spread, casings and a heavy mechanical report.
