# World effects and sector atmosphere

Native-alpha imagegen masters are retained here with SHA-256 provenance in `manifest.json`. `python3 scripts/pack-world-fx.py` rebuilds both 1024×1024 sheets: four padded 512×512 frames in a 2×2 grid. The engine uses its standard 256×256 world-atlas layer, keeping authored HD sheets available in the local catalog. Brass casings and red BR-12 shells have separate textures and landing recordings.

Casings eject in front of the player, tumble, collide with walls, bounce up to three times, settle, and expire after six seconds. At most 24 live casings share the existing bounded entity pool. A short smoke puff follows ejection. Landing sounds originate at physical floor contacts, using the existing spatial SFX mixer.

Explosion world cues drive four bounded camera-projected shockwaves. WebGPU and WebGL2 distort the world texture radially with packed-depth occlusion; Canvas2D uses source-image tile refraction. Effects freeze with simulation pause and reset on sector transitions. The HUD and weapon viewmodels remain sharp.

Dust and haze use scene luminance plus muzzle lighting. Sector profiles vary colors, particle shapes, drift, glints, scan pulses and phase motion. Fine procedural details scale with display resolution; no opaque particle backdrop is used. Canvas2D retains a lighter dust fallback.

The 25 distinct sound accents are quiet finite CC0 mixes, predecoded during loading alongside combat SFX. They occur every 6–11 seconds during active gameplay, stop on pause/mute/transition/disposal, and duck during dialogue. They use the existing master/SFX sliders. Recipes and source credits remain in `art/audio/`. Rebuild all sounds with `npm run assets:sfx`, or selected IDs with `python3 scripts/process-blacksite-sfx.py sector-1 casing-brass`.

| Level | Sector | Visual signature | Sound ID | Interval |
|---|---|---|---|---|
| 1 | upper-works | Welding dust | sector-1 | 7s |
| 2 | cryogenic-foundry | Foundry ice crystals | sector-2 | 8s |
| 3 | bioforge-depths | Bioforge spores | sector-3 | 9s |
| 4 | data-spine | Data scan motes | sector-4 | 6s |
| 5 | reactor-sink | Reactor ion haze | sector-5 | 8s |
| 6 | null-archive | Archive phase wisps | sector-6 | 10s |
| 7 | cryo-reserve | Reserve snow drift | sector-7 | 9s |
| 8 | signal-crypt | Signal interference | sector-8 | 7s |
| 9 | siege-yard | Siege grit | sector-9 | 6s |
| 10 | command-bunker | Bunker vent mist | sector-10 | 9s |
| 11 | obsidian-vault | Vault obsidian glints | sector-11 | 10s |
| 12 | ashen-transit | Transit ash | sector-12 | 8s |
| 13 | tidal-pumpstation | Pumpstation spray | sector-13 | 7s |
| 14 | optics-array | Optics diffraction | sector-14 | 9s |
| 15 | magnetic-forge | Forge magnetic filings | sector-15 | 6s |
| 16 | drone-hatchery | Hatchery tracking lights | sector-16 | 7s |
| 17 | fungal-research | Fungal pollen | sector-17 | 10s |
| 18 | orbital-uplink | Uplink charged dust | sector-18 | 8s |
| 19 | waste-reclaimer | Reclaimer corrosive vapor | sector-19 | 9s |
| 20 | shadow-lab | Shadow lab phase shimmer | sector-20 | 11s |
| 21 | fusion-chamber | Fusion plasma wisps | sector-21 | 7s |
| 22 | flooded-silo | Silo suspended droplets | sector-22 | 8s |
| 23 | chrono-archive | Chrono drifting echoes | sector-23 | 10s |
| 24 | black-ice-core | Black ice diamond frost | sector-24 | 9s |
| 25 | apex-control | Apex command lattice | sector-25 | 6s |

Level 26 onward follows the repeating sector theme. Tests: `scripts/world-effects.test.mjs`, `scripts/world-effects.browser.mjs`, the Rust casing physics tests, and existing SFX provenance/mixer checks.
