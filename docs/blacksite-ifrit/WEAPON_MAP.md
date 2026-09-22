# BLACKSITE IFRIT — weapon mapping (spec → engine v1)

The engine has **5 weapon slots** (`MAG_SZ`, `ammo`/`mag`, input bits
W1–W5, HUD index 0–4). Same logic: each spec weapon below inherits a
slot's magazine, spread, pellets, reload and place in the progression.
Five spec weapons ship v1; five need new mechanics (named, deferred).

## v1 (delivered)

| Slot | Engine behavior | Spec weapon | Why it fits |
|---|---|---|---|
| 0 | Sidearm · 12 mag, precision | **MK23-S Suppressed Pistol** | starter precision, low recoil ✓ |
| 1 | Scattergun · 6 mag, close spread + stagger | **M870K Combat Shotgun** | primary close-range, pump fantasy ✓ |
| 2 | Ripper · 32 mag, short bursts, spread control | **VX-9 Compact SMG** | close automatic bullet hose ✓ |
| 3 | Lance · 4 mag, pierces 3, line them up | **SHRIKE Anti-Materiel Rifle** | slow armor-penetrating precision ✓ |
| 4 | Pyre · 40 mag, lingering fire patches | **RAVEN Grenade Launcher** | area denial, hold doorways ✓ |

Placement mirrors the current progression: MK23-S (start) → VX-9 (lab) →
SHRIKE (chapel) → RAVEN (pit); Ripper-slot secret becomes a VX-9 cache.
Magazine sizes stay engine-side (`MAG_SZ = [12, 6, 32, 4, 40]`) until
live balance passes say otherwise. The five delivered art sets now use the
spec IDs in the runtime HUD (`weap_mk23s`, `weap_m870k`, `weap_vx9`,
`weap_shrike`, `weap_raven`) with idle, four-cell fire, and four-cell reload
renders. The old short filenames remain only as source aliases for rollback.

Projectile and weapon VFX are sourced from the 1024px master at
`art/source_hd/projectiles/projectile_effects_4x4.png`. The runtime keeps the
engine's existing slots: hostile plasma (`T_BALL`), impact (`T_IMPACT`),
muzzle (`T_MUZZLEFX`), and the 2x2 fire/smoke sheet (`T_FLAME`).

## DEFERRED (needs new mechanics)

| Spec weapon | Missing system |
|---|---|
| CAR-11 Assault Rifle | 6th slot (arrays, input bits, HUD are all length-5) |
| KSDD M91 Rotary Cannon | spin-up + overheat loop |
| ARC-4 Discharge Rifle | chain-lightning arcs + machine-stun status |
| JANUS Bio-Disruptor | damage-over-time tissue reaction |
| SUNHAMMER | charge-up + BFG-class projectile budget |

## Art asset IDs

`weapon_mk23s`, `weapon_m870k`, `weapon_vx9`, `weapon_shrike`,
`weapon_raven` — each ships idle + fire + reload 2×2 sheets
(640px idle / 1024px sheets, matching the current DOM weapon view).
Deferred five get IDs reserved in the manifest when specced.
