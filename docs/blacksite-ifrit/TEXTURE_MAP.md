# BLACKSITE IFRIT — texture & sector mapping (spec → engine v1)

The engine has **8 wall kinds + 3 floor kinds** (`wall_tex` in lib.rs).
Same logic: sector identity comes from which kinds paint each zone,
plus props, lighting and signage — not new geometry code. Deliver final
textures at **256×256** (engine atlas slot size; keep 512/2K sources
archived per the asset spec pipeline).

## Wall slot remap

| Slot | Old identity | Blacksite identity | Spec family |
|---|---|---|---|
| `T_METAL` (1) | riveted steel | gunmetal bunker panel | Metal |
| `T_BRICK` (2) | brick chapel | poured concrete / blast-wall | Concrete |
| `T_FLESH` (3) | meat pit | JANUS tissue / contamination (late sectors) | Biotech |
| `T_PIPES` (4) | pipes | cable trays / coolant lines | Technical |
| `T_SKULL` (5) | bone | server panels / dark data-center cladding | Technical |
| `T_TECH` (6) | CRT lab | server racks / monitoring walls | Technical |
| `T_HAZARD` (7) | caution stripes | hazard-striped metal (unchanged role) | Nuclear |
| `T_DOOR` | door | blast door | — |
| `T_SECRET` | secret | maintenance hatch (misaligned panel) | — |

Hell-mode retexture (`hell = true`) already swaps the palette toward
flesh/skull on boss approach — reuse it for the MALIK-takeover shift
in late waves at zero extra cost.

## Sector palettes (wall kinds + props + light)

| Sector (map zone) | Walls | Floor | Accent props | Light mood |
|---|---|---|---|---|
| Hangar / start | concrete + gunmetal | concrete | crates, floodlight lamps | warm floodlights |
| Plaza hub | gunmetal + cable trays | grate | pipe block, banners | neutral industrial |
| Security wing | concrete + hazard | concrete | checkpoints, signs | fluorescent + red warnings |
| Data center | server cladding + tech | raised-floor tile | server racks, fiber | blue-white / cyan |
| Robotics foundry | gunmetal + hazard | diamond plate | assembly arms, drone racks | orange welding |
| Biotech (JANUS) | tissue + sterile panel | containment tile | specimen tanks | sickly green / red alarm |
| Nuclear | concrete + hazard | reactor flooring | coolant, warning signs | cyan glow + red alarms |
| MALIK core / vault | server black + crimson trim | dark | monoliths, cables | deep crimson / cold white |

## Signage (spec §24 workflow, engine-compatible)

Authored typography is post-processed onto plates, then uploaded as
wall textures or decals — never AI-rendered text. MALIK announcements,
memos and propaganda land in two places: **texture signage** (static,
per-sector) and a future **announcement toast system** (trigger-driven,
reuses ambush-zone infra — deferred, no code yet).

## Art asset IDs (must match `art/blacksite-manifest.json`)

Walls/floors (one file per atlas slot): `wall_concrete.png` (0),
`wall_gunmetal.png` (1), `wall_tissue.png` (2), `wall_serverclad.png` (3),
`wall_blastdoor.png` (4), `floor_grate_bi.png` (5), `floor_bunker.png` (6),
`wall_hatch.png` (17), `wall_server.png` (18), `wall_hazard_bi.png` (19),
`wall_cables.png` (26). Props: `spr_crate_bi.png` (21),
`spr_medkit_bi.png` (11); `spr_serverrack.png` and `spr_spectank.png`
need atlas-slot extension first. Decals (`decal_scorch_*`,
`decal_warning_*`, `decal_propaganda_*`) ship as separate transparent
PNGs when their sector lands.
