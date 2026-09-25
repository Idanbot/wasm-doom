# BLACKSITE monster sprite design

Authoritative art specification for the current roster and every future hostile billboard.

Do not treat monsters as independent illustrations. Every sheet must look as if one artist produced it in one pass, for this renderer, on this pixel grid.

---

## 1. Runtime contract (do not violate)

Inspected from the BLACKSITE layers in `public/game/enemy_<skin>_<animation>.png`,
`src/game/runtime.ts` (`TEX_FILES`, `keySpriteAlpha`), and
`engine/src/lib.rs` (`EnemySkin`, animation state selection).

| Fact | Value |
|------|--------|
| Atlas | 256×256 PNG, RGBA per animation layer |
| Layout | 2×2 cells per layer; seven layers per enemy skin |
| Cell | 128×128 |
| Frame index | idle 2, move 4, pain 2, fire 2, reload/charge 2, dead 2, special 2 |
| Frame order | TL, TR, BL, BR; two-frame groups duplicate TR into the lower cells |
| Sampling | nearest, square billboard, `sprite_w = sprite_h` |
| World scale | Per-skin table in `engine/src/enemies.rs`; legacy archetype tuning remains shared |
| Vertical lift | Per-skin `zoff` (flying identities use a negative lift) |
| Transparency | deterministic edge flood plus enclosed neon-magenta key; final PNGs are RGBA |
| Upload | stretched to `TEX=256` with image smoothing **off** |

**Keep this format.** 128×128 cells are enough if the 1024px source is aligned
and downsampled deterministically. Do not raise atlas size unless a later
review proves 128px cells fail readability.

### Current-sheet audit (256×256 after Scale2x)

| | Unique opaque colors | Occupied cell (typ.) | Feet Y | Center X |
|--|---------------------|----------------------|--------|----------|
| Husk | ~1750 | 46×102 | **115** | ~63 |
| Brute | ~2985 | 66×102 | **115** | ~63 |
| Wraith | ~2756 | 60×92 | 107–113 (unstable) | 62–68 |

Magenta is unused. Transparency is black. Color counts are an order of magnitude too high (Scale2x + painterly source). That is the primary quality failure.

Strengths to keep: readable roles (thin walker / wide tank / floating skull-fire), Husk/Brute already share a foot line, front-facing billboard posing.

Failures to fix: noisy palettes, mushy mid-tones, no shared outline, Wraith vertical drift, Husk/Brute look like different resolutions, Brute surface noise will explode at 2.45× boss scale.

---

## 2. Shared art direction

Professional indie pixel art in the **Doom (1993) billboard language**, not a Doom asset clone.

- Front or 3/4-front. No 8-way camera set.
- Silhouette first. Gameplay role readable at ~32–48 on-screen pixels tall.
- Deliberate pixel clusters. No painterly strokes, no photo grain, no blur.
- One lighting model, one outline rule, one pixel size, one animation grammar.
- Materials read as clusters of 2–4 values (shadow / body / mid / rim), not gradients.

**Avoid:** digital painting, photoreal skin, soft AA rims, noisy pores, muddy browns, unique color per pixel, micro-detail that dies at gameplay scale, inconsistent cluster size between enemies.

---

## 3. Pixel density

Author as if the **virtual pixel is 2×2 display pixels inside a 128×128 cell** (effective 64×64 design grid, then nearest-neighbor 2×).

All three monsters share that cluster size. A Husk limb and a Brute plate edge must snap to the same grid.

Do not mix “hi-res pixel illustration” with “chunky 32px” in one roster.

Master art may be 512×512 per frame, but reduction is **box/median → nearest**, then cluster cleanup. Never bilinear onto the atlas.

---

## 4. Palette discipline

Target **12–18 opaque colors per monster**, plus 1 outline, plus transparency.

Shared neutrals (all sheets):

| Role | Hex (approx) | Notes |
|------|----------------|-------|
| Outline | `#3A2A24` | Must stay ≥ rgb 32 so `keySpriteAlpha` does not eat it |
| Deep shadow | `#4A3028` | Warm, not blue-black |
| Bone | `#C8B89A` | Skulls, teeth, Husk joints |

Enemy identities (stay distinct, same saturation ceiling):

- **Husk** — rust `#A85A3A`, ash `#6A5A52`, dried blood `#6E2C24`, bone. Cooler greys only in shadow, never as a second skin.
- **Brute** — crimson `#9A2420`, wine `#5A1418`, plate `#3E2A28`, dull brass `#8A6A3A` (sparse). No orange fire.
- **Wraith** — violet `#5A2A78`, gold fire `#E08A20` / `#F0C45A`, skull bone. Fire is the only high-chroma element in the roster.

Rules:

- Hue-shift shadows toward the identity hue (Husk rust-brown, Brute wine, Wraith purple), not toward grey-blue.
- Highlights: one step, small area, never white except a 1–2px fire core on Wraith.
- No color that appears on only one frame.

---

## 5. Lighting and shading

Single key: **upper-left, ~11 o’clock**, slightly above.

- Large shadow masses first (torso under, right-down side).
- One rim on the upper-left contour, 1 cluster thick.
- No ambient occlusion dirt, no baked noise.
- Value steps are discrete. If a ramp needs more than 4 values, the form is over-rendered.

Vault Master: the same Brute shading must survive a red multiply. Avoid green/teal fill lights and 1px checker dither that moirés at 2.45×.

---

## 6. Outline treatment

**Selective dark outline**, 1 authored pixel (2 display px at the 2× grid):

- Full outline where the sprite meets transparency (silhouette).
- Interior plate/limb splits use a **shadow cluster**, not a second outline color.
- Outline color is `#3A2A24` (or identity-dark), never `#000000` (runtime keys pure black).

Apply to Husk, Brute, and Wraith (Wraith outline follows skull + outer flame envelope).

---

## 7. Animation language (16 gameplay frames)

Alive, not a flipbook of new designs.

| Set | Frames | Direction |
|--|--:|--|
| Idle | 2 | restrained breathing and equipment sway |
| Move | 4 | readable weight shift or locomotion; flying units bob |
| Pain | 2 | recoil and red hit response |
| Fire | 2 | attack pose plus muzzle or energy cue |
| Reload/charge | 2 | weapon reset, charge-up or stance preparation |
| Dead | 2 | collapse and settled body |
| Special | 2 | identity-specific action (pounce, slam, detonation, optic lock, etc.) |

- Anatomy lock: head size, torso width, limb length constant across frames.
- No yaw/turn. Billboard is always “toward player.”
- Keep the same origin and lighting direction across every group.

---

## 8. Framing and alignment

Cell 128×128.

- Horizontal center of mass at **x = 64 ± 2**.
- **Husk + Brute feet baseline: y = 116** (2 px pad from cell bottom). Same line on all eight grounded frames.
- **Wraith:** bbox vertical center at **y = 64 ± 3**; lowest flame pixel ≥ y 110; never sit on the grounded baseline.
- Empty margin: ≥ 12 px left/right of silhouette; grounded heads ≥ 10 px from top.
- Occupancy (opaque bbox): Husk ~48×104, Brute ~72×104, Wraith ~64×96. Brute is wider, not taller, than Husk.

---

## 9. Technical export

- PNG RGBA, 256×256, 2×2, 128×128 cells. **Do not raise atlas size** unless 128px cells fail readability; `TEX=256` is shared with walls.
- Transparent background (`A=0`). Runtime still keys black/magenta as a safety net; never use `#000000` in the silhouette.
- Author on a **64×64 design grid**, nearest-neighbor 2× into each 128 cell (2×2 authored pixel).
- Masters: `art/monster-masters/<name>-f{0-3}.png` at 512×512 (nearest from the 64 grid).
- **Rework pass (this document):** originals in `public/game/spr_{husk,brute,wraith}.png` are **identity/gameplay references only**. New art ships as `public/game/spr_*_rework.png`. Do not quantize or Scale2x the old sheets.


---

## 10. Per-monster specs

### Husk

- Silhouette: inverted-triangle / hourglass, hunched, long thin legs, small skull.
- Pose: 3/4-front, left foot lead on even frames, right on odd.
- Decay: exposed ribs or torn abdomen as **large holes**, not speckles; rust skin, ash cloth scraps.
- Walk: shoulders counter-rotate slightly; head stays on one height.
- Must remain readable as “zombie human” at Husk world scale 0.95.

### Brute

- Silhouette: rectangle / inverted trapezoid, shoulders wider than cell half-width, short legs, no neck.
- Pose: square to camera, fists at sides or slightly forward.
- Armor: 4–6 plate planes, crimson, brass rivets as 2×2 clusters max.
- Walk: torso sways 1 px; legs almost planted.
- **Vault Master:** this sheet at 2.45× + red tint. No 1px filigree, no high-frequency spots, no thin antlers. Mass and horn/shoulder peaks must read as big shapes.

### Wraith

- Silhouette: skull in an oval/teardrop of fire, no body.
- Skull: bone, empty sockets, slight 3/4. Sits in the **upper two-thirds** of the flame.
- Flame: purple body, gold rim/core. Animate by deforming 4–8 outer clusters, not by redrawing the skull.
- Hover: skull may shift 1 px; flame breathes. Never a gait.
- At scale 0.7 the skull sockets and gold rim must still parse.

---

## 11. Vault Master reuse

Brute is the only boss art.

Pass if, when scaled 2.45× and mixed toward red:

- plates remain planes, not noise
- outline does not halo
- horns/shoulders still read
- no new colors appear from interpolation (nearest scale)

Fail if the large version looks like a blurry stamp or a carpet of sparkles.

---

## 12. Acceptance checklist

A sheet is accepted only if:

- [ ] Silhouette readable in 1 second as the correct role
- [ ] Shared pixel density with the other two
- [ ] Shared lighting (upper-left)
- [ ] ≤ 18 opaque colors (count unique RGB with A&gt;16)
- [ ] Selective dark outline, not `#000000`
- [ ] Detail density matches the roster
- [ ] Frames share anatomy; no design drift
- [ ] Reads at gameplay size (Husk ~1.0, Brute 1.25, Wraith 0.7, Brute-boss 2.45)
- [ ] Not painterly; no blur; no soft AA rims
- [ ] 256×256 / 2×2 / 128 cells
- [ ] Husk/Brute feet on y=116
- [ ] Wraith vertically centered, not grounded
- [ ] Transparency: empty cells are A=0; flood-key still safe
- [ ] Brute survives boss scale + red tint
- [ ] Looks like the same artist as the other two

---

## 13. Pipeline (from-scratch gothic pass)

1. Lock this document. Live sheets are identity/gameplay references only.
2. Generate **gothic demonic billboards** with the approved image-generation service: solid `#FF00FF` background, 2×2 grid, same scale per cell, front-facing retro-FPS creatures — not geometric primitives, not photo-quantize.
3. Chroma-key magenta, snap to the 64-grid / 128 cell, assemble `sheet4`.
4. Do **not** accept capsule/ellipse programmer art. If the image-generation service is unavailable, leave live sheets unchanged rather than shipping placeholders.
