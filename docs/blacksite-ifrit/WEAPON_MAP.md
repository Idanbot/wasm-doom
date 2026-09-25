# BLACKSITE IFRIT — weapon mapping

The engine carries eleven weapon slots. Eight are found in the sectors.
The last three exist only as the case a specific boss drops.

| Slot | Weapon         | Combat identity                                        |
| ---- | -------------- | ------------------------------------------------------ |
| 0    | MK23-S         | suppressed 12-round precision sidearm                  |
| 1    | BR-12 BREAKER  | 8-shot close spread with heavy stagger                 |
| 2    | KX-9 VECTOR    | 36-round automatic PDW with controlled spread          |
| 3    | MR-4 LONGBOW   | 5-shot magnetic rifle that pierces three targets       |
| 4    | VLK-6 WARDEN   | 4-cell guided micro-missile launcher with splash       |
| 5    | AX-12 VOLT     | 10-charge electrical precision carbine                 |
| 6    | M91 CYCLONE    | 90-round rotary cannon for sustained suppression       |
| 7    | HX-8 PYRE      | ground burn lasts 10s; the shot trail fades in a few seconds and damages until then |
| 8    | VR-9 OVERRIDE  | Veyran only. Pierces the lane for 120, then a 2.8 burst |
| 9    | HC-9 FORGE     | HECATE-9 only. Wide cutter plus impact splash, no ground fire |
| 10   | CM-9 CHIMERA   | CHIMERA-9 only. Acid bolts burst and leave a 4s pool |

Magazine sizes are `MAG_SZ = [12, 8, 36, 5, 4, 10, 90, 6, 4, 14, 5]`.
Number keys 1–8, `9`, `0` and `-` select owned weapons, as does the wheel.
Weapons 2–8 enter through map pickups. VR-9, HC-9 and CM-9 are never placed
on the map; picking the dropped case ends the sector.

Weapons 2–7 use transparent high-resolution masters in
`art/source_hd/weapons_v3/`. The deterministic processor produces one idle
view, four recoil/fire frames and eight distinct reload frames. Each model is
held on the center sightline and uses the same 800×480 frame as the game view.
VLK-6 has an explicit launch aperture, four-cell cassette and targeting optic.

Projectile and impact effects remain separate from weapon body art. VLK-6
launches the existing swept missile projectile and produces explosion effects
at collision; AX-12 layers short electrical beam segments over hitscan; M91
uses rapid hitscan, growing spread, casings and a heavy mechanical report.
