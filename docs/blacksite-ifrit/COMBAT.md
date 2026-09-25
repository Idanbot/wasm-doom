# BLACKSITE combat pass

Enemy combat roles are defined in `engine/src/combat.rs`, separately from
their HP and presentation in `enemies.rs`. Riflemen hold medium range,
breachers close for a spread shot, marksmen keep distance, gunners fire
three-round spreads, and the boss fires a five-projectile fan. Subjects,
loaders, vat brutes and hounds use melee; spitters fire green acid bolts.
Shield guards carry a finite front plate. It soaks shots from a narrow facing
arc, turns slowly enough to flank, and breaks. After that, hits land on health.
Heavy gunners also have an armor layer. A small bar above a damaged enemy shows
the current layer — cyan plate, amber armor, red health — and fades two seconds
after the last hit.
Lamps shut off with USE or a shot. Explosive barrels detonate; fuel drums leave
fire. Military crates break into ammo, a medkit, or armor. Floor props sit on
the ground instead of hovering at eye line. The sector map sits under the FPS
readout and marks living hostiles.
Every attack has a 0.3–1.0 second windup, an amber sprite highlight, and
committed aim. Enemies stop moving while winding up. Damage interrupts the
windup; stepping out of melee reach or behind cover cancels its hit. Ranged
enemies retreat when crowded, and skirmishers alternate strafe direction.
Opening attacks are staggered so starting a game allows time to orient.

VLK-6 fires one guided missile every 0.78 seconds from a four-cell cassette.
Impacts cause splash damage; walls block the blast and nearby detonations can
hurt the player. Projectiles sweep their travel segment to detect hits between
simulation steps. AX-12 fires a precise electrical discharge, while M91 adds
a 90-round sustained rotary stream with growing spread.

Ammo crates replenish every weapon in the eight-gun arsenal. Each sector
also drops a fresh medkit, armor pickup, ammo crate, and an HX-8 case. The
field HUD warns when the active magazine runs low. A sector node must be
used before the override console will start the boss. Boss phase one seals
the arena doors and, in the foundry and bioforge, lights hazard pools. Phase
two exposes the boss: damage doubles until the window closes, and a later
attack reopens a shorter window.

All guns use their firing sheets, including KX-9. Weapon sheets preload
alongside world textures before play. Recoil weight varies by
weapon and respects reduced motion. The crosshair expands with recoil and
SMG spread, while a red KILL marker distinguishes kills from hits. Muzzle
flashes add local warm lighting. Plasma and acid cast blue and green light.

VFX sheets contain separate effects, so frame selection is explicit. Fire,
smoke, casings, muzzle flashes and impacts retain their identity as they age.
Smoke and impacts expand; fire pulses slightly. CPU rendering blends source
alpha; GPU rendering uses coverage dithering because target alpha stores
depth for post-processing. Dark effect details are preserved.

Regression checks cover windup/aim commitment, damage interruption, cover,
melee dodging, swept projectile damage, grenade cadence/splash, and stable
effect identity. Existing weapon, movement and map checks remain required.
