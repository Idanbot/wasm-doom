# Blue/cyan projectile rework

18 individually generated OpenAI imagegen assets: 16 upgraded incoming/outgoing
projectile sprites plus two beam cross-sections. Exact prompts, reference paths
and replacement contracts are in `prompts.json`; source/output hashes are in
`manifest.json`. Sources use native transparency; packing preserves interior
alpha and places artwork within an 800px area of each 1024×1024 runtime image.
Repack using `python3 scripts/pack-weapon-feeds.py`.

Runtime files: `public/game/projectiles/v4/`. `src/game/projectile-art.ts` maps
existing live blue/cyan texture paths to new files, preserving all combat values
and existing projectile identities. Incoming missiles face the camera;
outgoing missiles expose their exhaust, never a side profile or leading nose.
Frost, optical prisms and ion/arc projectiles retain distinct designs.

Both GPU renderers and the Rust CPU renderer now sample projectile artwork for
beams. Previously a procedural blue circle replaced every ray's texture,
including boss-specific sprites. Untagged rays use the new arc/precision core;
tagged boss rays keep their level-specific art. Beam texture slots follow weapon
feed slots in the shared Rust/TypeScript atlas layout.

Original images remain as recorded references for earlier art manifests.
