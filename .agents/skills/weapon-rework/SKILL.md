---
name: weapon-rework
description: >
  Standardized doctrine, frame specifications, and pipeline for FPS weapon viewmodels
  and 5x5 (25-frame) animation sprite sheets. Use whenever creating, reworking,
  or polishing weapon viewmodels, aim states, dry fire, gun case pickups,
  reloads, primary fire, or alt-fire sequences.
metadata:
  short-description: "25-frame 5x5 weapon viewmodel animation and rework standard"
user-invocable: true
---

# Weapon Rework & 5×5 Animation Sheet Specification

This skill defines the official architecture, state rules, frame mapping, and verification
doctrine for all first-person shooter weapon assets in this project.

---

## 1. Grid Architecture & Viewport Layout

Each weapon asset is built on a **5×5 uniform sprite sheet** (25 total frames):
- **Cell Dimensions**: `512 × 384` px per cell (4:3 aspect ratio).
- **Master Sheet Dimensions**: `2560 × 1920` px (5 columns × 5 rows).
- **Center Sightline Law**: The center of the screen (`50% X, 50% Y` / `256, 192` within the cell)
  is strictly reserved for the in-game crosshair (`<Crosshair flash={0} spread={0} danger={false} />`).
  The weapon viewmodel is anchored at the **bottom-right corner** of the frame, tilted and
  aimed diagonally upward toward the center crosshair.
  **The weapon must NEVER overlap or obscure the center crosshair.**

---

## 2. 25-Frame Layout (5×5 Grid)

```text
Row 0 (Aim States)  : [Cell 0: Full-Aim]  [Cell 1: Half-Aim]  [Cell 2: Low-Aim]  [Cell 3: Empty-Mag]  [Cell 4: No-Ammo]
Row 1 (Transitions) : [Cell 5: Dry-Fire]  [Cell 6: Pickup-1]  [Cell 7: Pickup-2] [Cell 8: Pickup-3]  [Cell 9: Pickup-4]
Row 2 (Reloading)   : [Cell 10: Reload-1] [Cell 11: Reload-2] [Cell 12: Reload-3][Cell 13: Reload-4] [Cell 14: Reload-5]
Row 3 (Primary Fire): [Cell 15: Fire-1]   [Cell 16: Fire-2]   [Cell 17: Fire-3]  [Cell 18: Fire-4]   [Cell 19: Fire-5]
Row 4 (Reserved FX): [Cell 20: Alt-1]    [Cell 21: Alt-2]    [Cell 22: Alt-3]   [Cell 23: Alt-4]    [Cell 24: Alt-5]
```

---

## 3. Frame-by-Frame Definition

### Row 0: Aim & Ammo States (Cells 0–4)

1. **Cell 0 — `full-ammo-aim` (1 frame)**:
   - **Definition**: Basic idle frame with magazine/cell inserted and locked. Static aim on center reticle with 100% full ammunition.
   - **Visuals**: Full magazine seated, energy bar at 100%, digital counter reading "100%" or "FULL", all fuel rods/conduits glowing bright.
2. **Cell 1 — `half-ammo-aim` (1 frame)**:
   - **Definition**: Static aim with half magazine capacity remaining (50%).
   - **Visuals**:
     - *Energy/Plasma/Tech*: Energy bar at 50%, digital counter at 50%, half the fuel rods/diodes unlit.
     - *Transparent magazines*: Half the bullet stack visible.
     - *Solid ballistic magazines*: Visually identical to `full-ammo-aim` (since internal bullets are hidden).
3. **Cell 2 — `low-ammo-aim` (1 frame)**:
   - **Definition**: Static aim with critical/low ammunition remaining (15–20%).
   - **Visuals**: Warning indicators active. Amber/red caution diode blinking on chassis, fuel conduit flickering faintly, digital readout reading "15% / LOW". For solid ballistic mags, caution diode illuminated.
4. **Cell 3 — `empty-mag-aim` (1 frame)**:
   - **Definition**: Active magazine is completely depleted, but player may have reserve ammo. Weapon requires reloading.
   - **Visuals**: Empty magazine well (or empty magazine casing), slide or bolt carrier locked back over an open ejection port, red warning diode, digital HUD displays "0% / EMPTY".
5. **Cell 4 — `no-ammo-aim` (1 frame)**:
   - **Definition**: Total ammunition exhaustion: magazine is empty AND reserve ammo is zero (cannot reload).
   - **Visuals**: Weapon is completely unpowered and dry. No magazine attached, digital display black/offline, all LEDs and energy conduits unpowered (dark metal).

---

### Row 1: Actions & Transitions (Cells 5–9)

6. **Cell 5 — `fire-empty` (1 frame)**:
   - **Definition**: Reserved cell only. Dry-fire animation is disabled; firing an empty weapon leaves its empty aim pose unchanged.
   - **Visuals**: Exact copy of cell 3. **Zero movement, muzzle flash, spark or projectile.**
7. **Cells 6–9 — `pick-up / equip` (4 frames)**:
   - **Definition**: Equipping/picking up the weapon from an open military Pelican case on the floor:
     - **Cell 6 (Pickup 1/4)**: First-person view looking down into open military foam case, tactical gloved hands reaching into the molded foam cutout to grip the weapon.
     - **Cell 7 (Pickup 2/4)**: Hands lifting the weapon upward out of the foam cutout, weapon angled up ~25° rising into view.
     - **Cell 8 (Pickup 3/4)**: Weapon brought up to chest height, rotating towards the player, quick tactical inspection check.
     - **Cell 9 (Pickup 4/4)**: Mounting weapon into shoulder firing position, hands locking onto foregrip, barrel swinging to level out onto center sightline.

---

### Row 2: Reload Sequence (Cells 10–14, 5 frames)

8. **Cell 10 (Reload 1/5)**:
   - Weapon lowered slightly (`rot=-4°, dy=14`), off-hand drops/ejects empty magazine or depleted power cell downward out of frame. Empty magazine well exposed.
9. **Cell 11 (Reload 2/5)**:
   - Tactical gloved off-hand reaches down and brings a fresh, glowing, full magazine/cell up into the lower-left viewport.
10. **Cell 12 (Reload 3/5)**:
    - Guiding and sliding the fresh magazine/cell into the receiver well. Alignment glow/tactile positioning.
11. **Cell 13 (Reload 4/5)**:
    - Off-hand operates the charging handle, bolt catch or feed latch after the fresh feed is seated.
12. **Cell 14 (Reload 5/5)**:
    - Hand returns to the forward grip and weapon recovers to the exact approved aim pose; charge indicators are full again.

---

### Row 3: Primary Fire Sequence (Cells 15–19, 5 frames)

13. **Cell 15 (Fire 1/5)**:
    - Trigger break! High-energy muzzle flash / ignition bloom at barrel tip, initial barrel jump (`dy=-6, rot=2°`).
14. **Cell 16 (Fire 2/5)**:
    - Peak recoil buck! Weapon kicks up and back (`dx=6..8, dy=-14..-16, rot=5.5..6.5°`), barrel heat vents glowing.
15. **Cell 17 (Fire 3/5)**:
    - Action cycling at maximum rearward travel. Ejection of spent casing / heat slug / plasma venting.
16. **Cell 18 (Fire 4/5)**:
    - Return stroke forward, chambering next round/cell (`dx=2, dy=-4, rot=1.5°`).
17. **Cell 19 (Fire 5/5)**:
    - Settle frame: weapon damping back onto exact center sightline (`dx=0, dy=0, rot=0°`).

---

### Row 4: Alt-Fire / Special (Cells 20–24, 5 frames)

18. **Cells 20–24 — `alt-fire` (5 frames)**:
    - **Heavy / Boss / Energy Weapons**:
      - Reserved special artwork. The game does not currently expose an alt-fire input; do not imply an implemented Right Mouse Button ability.
      - **Cell 20**: Power spool-up / vortex suction lines / containment louvers open.
      - **Cell 21**: Secondary projectile condensation (massive glowing orb / arc buildup / payload arming).
      - **Cell 22**: Massive blast release! Extreme recoil shockwave (`rot=8..10°, dy=-20..-24`), lens flare bloom.
      - **Cell 23**: Gravity wake / thermal exhaust dissipation / shockwave rings.
      - **Cell 24**: Clamps lock, cooling cycle complete, weapon returns to ready aim.
    - **Standard Ballistic Weapons**:
      - If the weapon does not possess an exotic alt-fire mechanic, Row 4 is **identical to primary fire** (or a tactical melee bash / burst mode).

---

## 4. Visual Ammo Cues Matrix

| Weapon Type | Full Aim (Cell 0) | Half Aim (Cell 1) | Low Aim (Cell 2) | Empty Mag (Cell 3) | No Ammo (Cell 4) |
|:---|:---|:---|:---|:---|:---|
| **Energy / Boss (VR-9, HC-9)** | 4 rods glowing, 100% HUD | 2 rods glowing, 50% HUD | 1 rod glowing, 15% HUD + amber diode | Mag well empty, slide locked, 0% HUD | Unpowered chassis, dark HUD, cold |
| **Chemical / Fluid (CM-9)** | Twin vials 100% full green | Vials 50% level | Vials 15% level + amber warn | Vials empty glass, red alert | Drain valve unpowered, dark |
| **Transparent Mag (KX-9)** | 36 rounds visible in stack | 18 rounds visible in stack | 5 rounds visible + caution diode | Mag empty / follower visible | No mag in well, unpowered |
| **Solid Ballistic (MK23-S, BR-12)** | Solid mag inserted | Identical to Cell 0 | Amber caution diode on receiver | Slide locked back, chamber open | Zero reserve, slide locked, cold |

---

## 5. Verification & Asset Hash Workflow

Every live weapon is recorded in `src/lib/draft-weapons-v2-data.json`:
1. `sheet_file`: `/game/draft/v2/weap_{id}_5x5.png`, with full SHA-256 and short hash.
2. `aim_file`: `/game/draft/v2/weap_{id}_aim.png`, the approved aim reference.
3. Generated boss pose sources and prompts: `art/weapon-animations-v3/` (outside public deployment assets).

Boss rework pipeline: `python3 scripts/rework-boss-animations.py [weapon ids]`.
The source board has nine poses: three pickup, four reload, empty feed, feed removed.
Packing preserves approved aim exactly in cells 0, 9, 14, 19, 24; cell 5 equals cell 3.
Use generated detailed hand/mechanical poses, rather than drawing a fake magazine over a rotated idle image.

Verification:
- `node --test scripts/boss-animation-assets.test.mjs`: geometry, hashes, alpha, distinct poses, reticle clearance and exact return-to-aim pixels.
- `npm run typecheck`, `npm test`, `npm run check:environment`, `npm run build`.
- Dev-only Asset Catalog is accessed from the main menu; inspect Pickup, Reload, Fire, Empty and No ammo.
- `node scripts/combat-smoke.mjs`: runtime fire/reload transitions for all 19 weapons.
