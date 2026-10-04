# BLACKSITE enemy art, revision 4

48 freshly generated character pose boards: twelve regular enemies, eleven
bosses (sectors 1–11), and one exclusive enemy for each of the 25 sectors.
The remaining fourteen bosses retain their current sheets.

`specs.json` records character designs; `prompts.json` records the generation
prompt template and character-specific overrides. `*-source.png` are native
transparent image_gen masters. `generation.json` records source dimensions,
hashes and role/sector assignments. `roster-preview.jpg` is a contact sheet.

Each board contains four poses for each of seven states, in order:
idle, movement, pain, firing, reload/charge, death, special. The packer segments
actual generated poses (including detached equipment), preserves source alpha,
uses a shared character scale, and grounds frames at the same baseline. It
never fabricates animation by translating a single render or chroma-keying
armor. Install `python3 -m pip install -r art/enemies-v4/requirements.txt`, then run
`python3 scripts/pack-enemies-v4.py` to reproduce runtime assets and catalog hashes.

Runtime files: `public/game/enemy_<slug>_<state>.png`, 256×256 pixels,
2×2 grid of 128×128 cells. Animation state and UV contracts remain unchanged.
New skin IDs 37–61 are sector-exclusive; each inherits a regular combat role,
projectile, hit material, armor, spatial SFX and voice profile. Boss IDs/rewards
are unchanged. Incoming projectile count remains 37; 25 redundant projectile
textures are deliberately avoided. All 779 atlas layers load before play.
