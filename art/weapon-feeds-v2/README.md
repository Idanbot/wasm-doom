# Weapon-specific reload feeds

28 individually generated native-alpha OpenAI imagegen sheets complement the
four fuel/chemical/pressure containers in `art/weapon-debris-v1`. Every one of
BLACKSITE's 32 removable feeds has its own artwork; BR-12 has a fixed tube and
continues loading individual shells without ejecting a magazine.

Each weapon's approved aim image is a material/design reference. The prompt
requests four tumbling orientations of a detached, depleted magazine, power
cell, ammunition box or cassette: realistic metal, mechanical catches, no
hands, weapons, loose ammunition or full-charge glow. Exact per-gun prompts and
references are in `prompts.json`; source/output hashes are in `manifest.json`.
Approved weapon viewmodels remain unchanged.

Runtime files: `public/game/fx/feeds/<weapon>.png`, 1024×1024, 2×2 poses, 512×512
cells with at least 56px transparent padding. Repack using
`python3 scripts/pack-weapon-feeds.py`.

A valid reload ejects one feed during initial removal. BR-12 refill ticks eject
none; rejected or repeated reload inputs eject none. The captured material
survives weapon switching. Feed bodies share casing gravity, floor bounces,
wall collision, impact audio, a 24-object budget and six-second cleanup. They
remain cosmetic, never damage objects or block movement, and cannot be picked up.

New feed toss variation derives from simulation time and position, without
advancing the combat RNG. Sector completion and exact replay remain unchanged.
