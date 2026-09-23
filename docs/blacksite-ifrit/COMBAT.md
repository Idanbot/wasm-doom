# BLACKSITE combat pass

Enemy combat roles are defined in `engine/src/combat.rs`, separately from
their HP and presentation in `enemies.rs`. Riflemen hold medium range,
breachers close for a spread shot, marksmen keep distance, gunners fire
three-round spreads, and the boss fires a five-projectile fan. Subjects,
loaders, vat brutes and hounds use melee; spitters fire green acid bolts.

Every attack has a 0.3–1.0 second windup, an amber sprite highlight, and
committed aim. Enemies stop moving while winding up. Damage interrupts the
windup; stepping out of melee reach or behind cover cancels its hit. Ranged
enemies retreat when crowded, and skirmishers alternate strafe direction.
Opening attacks are staggered so starting a game allows time to orient.

RAVEN fires one incendiary grenade every 0.65 seconds from a six-round
magazine. Impacts cause splash damage and bounded fire patches; walls block
blast damage and nearby blasts can hurt the player. Projectiles sweep their
travel segment to detect hits between simulation steps.

All guns use their firing sheets, including VX-9. Weapon sheets preload
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
