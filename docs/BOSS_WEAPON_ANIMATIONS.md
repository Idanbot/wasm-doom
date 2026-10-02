# Boss weapon animation revision 3

All eleven boss rewards use regenerated mechanical handling poses while retaining
their approved full-aim image: VR-9, HC-9, CM-9, AR-6, OR-7, GS-4, CR-3, SR-0,
TS-12, KS-8 and MN-6. The eight standard weapons are unchanged.

Runtime assets: `public/game/draft/v2/weap_{id}_5x5.png`.
Each transparent sheet is **2560×1920**, five columns by five rows, with
**512×384** cells. Existing runtime frame indices remain unchanged.

| Cells | Animation |
| --- | --- |
| 0–4 | Approved full aim, half charge, low charge, depleted feed, feed removed |
| 5 | Reserved empty pose; firing empty does not animate |
| 6–9 | Reach/lower, lift, shoulder/inspect, approved aim |
| 10–14 | Spent feed out, fresh feed presented, feed inserted, handle/latch operated, approved aim |
| 15–19 | Muzzle ignition, recoil, recovery, return, approved aim |
| 20–24 | Reserved FX artwork, currently mirrors primary fire |

Opaque ballistic magazines do not reveal their internal ammunition. Energy
accents dim in contiguous regions; chemical bottles lower their visible fluid
level. Empty and feed-removed poses are separately generated. Alternate-fire
art does not imply a gameplay ability.

Generated nine-pose source boards and exact generation prompts are retained in
`art/weapon-animations-v3/`. They are outside `public/`, so the browser loads
only the packed runtime sheets. Alpha cleanup discards small disconnected
panel fragments while preserving substantial hands and removable feed parts.
Packing uses consistent scale within each sequence and keeps handling poses
clear of the reticle. Approved aim PNGs are never rewritten.

Reproduce packing with `python3 scripts/rework-boss-animations.py` (Pillow,
NumPy and SciPy); pass weapon IDs to rebuild a subset. SHA-256 catalog metadata
is refreshed automatically. Sources and provenance are recorded in
`generation.json` beside the boards.

Verification: `node --test scripts/boss-animation-assets.test.mjs` decodes every
sheet, checks alpha and frame occupancy, confirms distinct handling poses and
reticle clearance, compares catalog hashes and verifies exact pixel returns
to approved aim. The combat browser smoke test covers fire/reload transitions
for all nineteen weapons. CI runs both checks.

Repository cleanup removed obsolete untracked generators, duplicate weapon
specs and temporary screenshots. Ten unique unselected music tracks remain
locally in ignored `art/music-candidates/`; they are not deployed. Existing
tracked screenshot evidence and selected music were retained.
