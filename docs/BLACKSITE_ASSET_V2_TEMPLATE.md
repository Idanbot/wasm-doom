# BLACKSITE asset v2 visual template

Status: live assets. The dev-only catalog previews the same v2 weapon sheets used in gameplay.

## Shared camera and material language

- First-person, right-handed camera at eye height. The player's black tactical glove enters from the lower right; barrel travels diagonally toward the center of a 4:3 frame. Reserve the exact center pixel for the reticle.
- Standard weapons (slots 1–8): plausible contemporary military or near-future tactical hardware. Graphite or parkerized steel, restrained olive polymer, functional optics, legible wear. No glowing sci-fi housings or toy colors.
- Boss drops (slots 9–19): physically grounded futuristic mechanisms with one restrained accent: amber rail, cyan induction, green chemical containment, coolant, pressure, signal, siege, tactical red, or violet echo energy.
- Neutral cool studio lighting from upper left, detailed PBR-like metal and polymer, clean transparent alpha, consistent hand scale. No environment, typography, baked-in crosshair, heavy bloom, or floating components.
- Viewmodel source master: transparent PNG. Review cell: 512×384. Master sheet: 5×5, 2560×1920. Sheet layout follows `.agents/skills/weapon-rework/SKILL.md`.

## Reusable generation prompt

> Standalone transparent BLACKSITE game viewmodel, photoreal modern tactical FPS render. First-person right-handed operator in black tactical gloves, weapon fills lower-right foreground and extends diagonally toward the center-left. Camera at player's eye. Barrel muzzle aimed toward screen center with the center reticle point unobstructed. Full weapon silhouette visible, cool neutral studio light, black gunmetal, practical polymer, realistic surface wear and fine mechanical detail. No background, scenery, labels, crosshair, floating parts, or side-profile product shot. [Insert exact weapon role and mechanical features here.]

For boss weapons only, replace “modern tactical” with “restrained physically plausible sci-fi” and add one of the accent systems above.

## Sheet and supporting asset contract

| Asset | Draft format | Rule |
| --- | --- | --- |
| Weapon viewmodel | 5×5 PNG, 512×384 per cell | Row 0 ammo states; row 1 reserved empty pose + equip; row 2 reload; row 3 fire; row 4 reserved special artwork. |
| Gun case | 1024×768 transparent PNG | Rugged foam case sized for the matching weapon; accent color identifies slot without overpowering the gun. |
| World texture | 1920×1080 PNG | Seamless both axes; each sector has a distinct panel palette. |
| Sector door | 1920×1080 PNG | Front orthographic architecture for one sector; displayed once, not tiled. |
| Projectile/particle/FX | 512×512 transparent PNG | Bright core, restrained halo; keep alpha clean and silhouette readable against dark rooms. |
| Item/light | 768×1024 transparent PNG | Industrial fixture, practical stand/handle and believable emission source. |

The v2 sheets retain approved full-aim renders. Boss animation revision 3 uses individually generated pickup, feed removal, fresh feed insertion and charging/locking poses, packed from 3×3 source boards into the runtime 5×5 layout. Sources, prompts and provenance live in `art/weapon-animations-v3/`; reproduce packing with `python3 scripts/rework-boss-animations.py`. Cell 5 repeats empty aim because dry fire is disabled. Row 4 is reserved artwork, not an implemented alternate-fire mechanic.

See [boss animation rework](BOSS_WEAPON_ANIMATIONS.md) for source mapping and verification. Artwork includes live weapon sheets, gun cases, forward projectiles, particles, doors, repeatable wall panels and machinery props. High-resolution source boards are kept outside `public/` to avoid deployment and loading costs.
