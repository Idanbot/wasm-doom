# Sprite safe-area contracts

`art/sprite-safe-areas.json` records 598 published assets and 2,818 frames:
33 weapon sheets, 434 enemy animation sheets, projectiles, impacts, casings and
reload feeds. Each entry records its file hash, grid, cell size, per-frame RGBA
hash, silhouette bounds and transparent gutters. Bounds exclude right/bottom;
alpha greater than 10 defines the visible silhouette.

Nonanchored edges require two transparent pixels in the source cell. Weapon
sheets retain their intentional right/bottom anchoring; top/left stay clear.
The 5×5 weapon contract uses 512×384 cells in a 2560×1920 sheet. Cells
0, 9, 14, 19 and 24 must exactly match the approved aim pixels; cell 5 must
match the empty-magazine cell 3. Existing connected-fragment and reticle tests
remain part of the publication checks.

Before publishing:

1. Review changed animation poses and gutters in the local asset catalog.
2. Preserve approved aim files; a changed handling pose cannot change an aim return.
3. Run `python3 scripts/sprite-safe-areas.py --update` to record reviewed artwork.
4. Run `npm run check:sprites` and `npm test`; both run in CI.

The checker never edits images. A missing/new asset, changed frame, modified
approved pose, blank cell, nonintegral grid or lost gutter fails validation.

Reload removal happens at 10% of the simulation reload duration, halfway through
cell 10. Both the engine and sound director use this milestone, with crossings
instead of wall-clock timers. BR-12 keeps its fixed tube and ejects no magazine.
Weapon switches before removal cancel the feed; emitted feeds retain their
original weapon material. Frame-rate and combat-RNG checks cover all 33 slots.
