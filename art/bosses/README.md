# New sector boss art

The transparent source cutouts for ORACLE-7 and GRAVEMIND-4 were generated with the built-in image generator. Each boss has a base, firing, and special-charge pose. `python3 scripts/pack-new-bosses.py` packs those poses into seven 256×256 RGBA atlases, with four 128×128 cells per action.

- **ORACLE-7:** front-facing autonomous data-center defense chassis; charcoal ceramic and gunmetal, cyan fiber optics, antenna vanes, rail emitter; realistic tactical sprite, full body on transparent background.
- **GRAVEMIND-4:** front-facing nuclear-reactor security exosuit with human respirator, graphite shielding, amber reactor core, hydraulic legs and particle projector; realistic tactical sprite, full body on transparent background.

The source cutouts are kept here so future action frames can be refined while preserving each boss's visual identity. The shipped atlases are under `public/game/enemy_{oracle,gravemind}_*.png`.
