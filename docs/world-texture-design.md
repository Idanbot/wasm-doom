# Hellscan world texture design

Source of truth for every wall, floor, ceiling, door, and hazard tile. Same universe as `docs/monster-sprite-design.md` (rust, wine, bone, steel, upper-left light, no painterly smear).

The map is **one 48×32 layout**, replayed every wave. Art must fight repetition with **families, trims, and hash variants** — not a new maze.

---

## 1. Runtime contract

| Fact | Value |
|------|--------|
| Atlas size | `TEX = 256` (upload nearest; sources should be 256×256) |
| Wrap | `tex_x & 255` — **must tile** |
| Cell → wall | 1 metal, 2 brick, 3 flesh, 4 pipes, 5 bone/skull, 6 tech, 7 hazard, 8 door, 9 secret, 10 vault floor (walkable) |
| Floor styles | 0 grate, 1 concrete (extendable) |
| Hash variants | metal→hazard (`hash%7==0`), brick→skull (`hash%5==0`) |
| Hell | metal/brick/tech/pipes/door → flesh; grate/conc/hazard/ceil/secret → skull |
| Ceiling | always `T_CEIL` (pipes) except hell skull |
| Current PNG size | **128×128 photos**, bilinear-stretched to 256 — blur + seam fatigue |

Rooms (unchanged topology):

| Zone | Walls | Floor today | Role |
|------|-------|-------------|------|
| Hangar 1,1 15×12 | metal | grate | start, steel |
| Lab 18,2 13×11 | tech | grate | CRT / computers |
| Chapel 33,1 14×14 | brick | concrete | masonry / occult |
| Barracks 1,15 16×13 | hazard | concrete | caution |
| Flesh pit 20,16 16×13 | flesh | grate | organic |
| Vault 37,18 10×11 | bone | concrete | end landmark |
| Secret 40,1 7×5 | brick | concrete | alcove |

`mix_edge` already sprinkles sibling kinds on room borders. Keep that; enrich palettes so mixes read as **trims**, not random wallpaper.

---

## 2. Shared environment style

**Nadir-7 industrial occult.** A UAC-ish facility rotting into a chapel and a meat pit.

- Pixel-authored tiles, 256×256, **seamless**.
- Readability in motion: large 32–64 px modules (panels, bricks, pipe runs). No 1px noise carpets.
- Materials: steel, rust, mortar, flesh, bone, caution paint.
- Light: same **upper-left** as monsters. Panel faces slightly brighter top-left; mortar/gaps darker.
- Palette ceiling ~12–20 colors per tile. Hue-shift shadows toward rust/wine, not cool grey.
- Not photoreal, not painterly, not blurry, not high-frequency rust photos.

A tile that looks good in isolation but **shows a seam or a 64px stamp** when repeated is failed.

---

## 3. Texture families

### A. Industrial (hangar, lab, barracks, vault steel)

| Slot | File | Module | Notes |
|------|------|--------|-------|
| Metal | `wall_metal.png` | 64 px plates, 4 px trim, sparse rivets | Primary hangar |
| Pipes | `wall_pipes.png` | 32–64 px wrapped pipe runs | Accent / metal variant |
| Hazard | `wall_hazard.png` | 32 px diagonal caution | Barracks + metal hash |
| Tech | `wall_tech.png` / `wall_tech_tile2x2.png` | 64 px CRT/panel | Lab identity |
| Grate | `floor_grate.png` | 16 px bars + 64 px frame | Hangar / pit floor |
| Ceil | `ceil_pipes.png` | 64 px duct grid | All ceilings |

### B. Masonry (chapel, secret)

| Slot | File | Module |
|------|------|--------|
| Brick | `wall_brick.png` | 32×16 bricks, 2 px mortar, row offset 16 |
| Concrete | `floor_concrete.png` | 128×128 slabs, 4 px joints, one wrapped crack |

### C. Occult (pit, vault, hell swap)

| Slot | File | Module |
|------|------|--------|
| Flesh | `wall_flesh.png` | large 48–80 px lobes, dark creases that wrap |
| Bone | `wall_bone.png` | dark brick + bone glyphs on 64 grid |

### D. Interactive

| Slot | File | Module |
|------|------|--------|
| Door | `wall_door.png` | vertical stiles, mid caution bar, rivets | Must stay readable when `open` shifts U |

Secret uses door language, darker, no caution (engine `T_SECRET`).

---

## 4. Tiling rules

- Every motif period **divides 256** (8, 16, 32, 64, 128).
- No unique “hero” stain in the tile center.
- Test: 2×2 and 3×3 of the tile; no cross, no plus, no obvious square.
- Diagonal hazard: `(x+y) % period` so it wraps.
- Pipes: run off all four edges and re-enter.

---

## 5. Endless-run presentation (same map)

Do **not** rebuild rooms. Reduce fatigue with:

1. **Better base tiles** (this pass).
2. **Hash variants inside a family** (metal↔pipes↔hazard, brick↔bone).
3. **Floor zoning:** hangar grate, lab metal plate, chapel/secret concrete, barracks concrete, pit grate, vault bone.
4. **mix_edge** as trim: hangar metal+hazard+pipes; chapel brick+bone; pit flesh+brick.
5. Landmarks already in map (hazard bulkheads, bone niches, vault 10 cells) — keep; make those textures louder.

---

## 6. Acceptance

- [ ] 256×256 PNG, opaque
- [ ] Seamless on 2×2 repeat
- [ ] Module size ≥ 16 px; no photo grain
- [ ] ≤ ~24 unique colors
- [ ] Matches monster rust/wine/bone/steel
- [ ] Door still reads as a door when U-shifted
- [ ] Hazard reads at a glance (yellow-black)
- [ ] Flesh/bone distinct from industrial
- [ ] No runtime atlas-size change unless documented

---

## 7. This pass

Replace live world PNGs in `public/game/` with authored 256 tiles. Optional masters in `art/world-texture-masters/`. Small engine tweaks: extra hash variants + floor style 2/3. No new map topology.
