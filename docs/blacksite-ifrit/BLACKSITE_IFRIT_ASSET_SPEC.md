# BLACKSITE IFRIT — HD ASSET GENERATION SPECIFICATION

Version: 1.0

Purpose:
Define exact visual, technical, dimensional, and post-processing requirements for all AI-generated game assets.

This document should be supplied to any image-generation model or coding agent used to create assets for BLACKSITE IFRIT.

---

# 1. Master Pipeline

Never generate directly at final game resolution when a higher-resolution source can be used.

Preferred workflow:

1. generate high-resolution source
2. approve visual design
3. lock reference image
4. generate variations / animation frames from reference
5. remove background
6. crop
7. normalize scale
8. align origin
9. resize
10. sharpen if required
11. validate alpha
12. pack atlas / sprite sheet
13. export metadata
14. archive original HD source

Source art must always be preserved.

Directory structure:

```text
art/
├── references/
├── source_hd/
│   ├── enemies/
│   ├── weapons/
│   ├── props/
│   ├── projectiles/
│   ├── textures/
│   ├── decals/
│   └── ui/
├── processed/
└── atlases/
```

---

# 2. Global Art Requirements

All assets must match the BLACKSITE IFRIT game bible.

Visual style:

- dark military sci-fi
- underground special-weapons facility
- industrial realism
- retro FPS readability
- pre-rendered sprite aesthetic
- high-resolution source art
- strong silhouettes
- controlled grime
- practical engineering
- limited glossy surfaces

Avoid:

- generic futuristic concept art
- random neon cyberpunk
- anime styling
- excessive tiny detail
- overly clean surfaces
- fantasy magic unless explicitly biotech/AI related
- obvious Doom enemy copying

---

# 3. High Resolution Source Standards

## Character / Enemy Source

Preferred generation resolution:

**2048 x 2048 px minimum**

Ideal:

**3072 x 3072 px**

Acceptable maximum when available:

**4096 x 4096 px**

Never rely on the model to create the final sprite at 256 px.

---

## Weapon Source

Preferred:

**3072 x 3072 px**

Minimum:

**2048 x 2048 px**

For wide muzzle-flash compositions:

**3072 x 2048 px** or greater.

---

## Prop Source

Small props:

**2048 x 2048 px**

Large machinery:

**3072 x 3072 px**

Very large industrial props:

**4096 x 4096 px** where available.

---

## Texture Source

Preferred:

**2048 x 2048 px**

Minimum:

**1024 x 1024 px**

Hero / major wall textures:

**4096 x 4096 px** if the generator supports stable output.

Textures must always be generated square unless the asset specifically requires another ratio.

---

# 4. Enemy Sprite Specification

## Source Composition

Enemy must be:

- full body
- centered
- fully visible
- no cropped limbs
- neutral background or transparent background
- no environment
- no floor geometry
- no cast shadow extending far from feet
- no text
- no border

Camera:

- straight-on or predefined rotation angle
- approximately orthographic
- minimal lens distortion
- camera positioned around chest height
- no dramatic perspective

Lighting:

- neutral studio-style key light
- subtle rim light
- same direction across all frames
- no environment-colored lighting
- no dramatic spotlight variation

---

# 5. Final Enemy Sprite Dimensions

Default final gameplay sprite:

**256 x 256 px**

Large enemies:

**384 x 384 px**

Bosses:

**512 x 512 px** minimum

Hero final boss:

**768 x 768 px** or **1024 x 1024 px** if engine budget allows.

All source assets should remain significantly higher resolution.

---

# 6. Enemy Canvas Occupancy

Default humanoid enemy:

- height: 78–84% of canvas
- width: variable
- feet baseline: 91–94% of canvas height
- horizontal center: 50%

Example for 256x256:

- target body height: ~205 px
- ground baseline: y = 236 px
- center x = 128 px

Do not normalize by bounding-box center alone.

Always align by gameplay origin / feet.

---

# 7. Enemy Origin

Default enemy origin:

```text
origin_x = center
origin_y = feet
```

For 256 x 256:

```text
origin_x = 128
origin_y = 236
```

Flying enemies:

```text
origin_x = center
origin_y = visual center
```

Flying enemy metadata must include visual hover offset.

---

# 8. Enemy Animation Sets

Default humanoid:

```text
idle      2 frames
walk      8 frames
attack    5 frames
pain      2 frames
death     7 frames
```

Optional:

```text
alert     2-4
reload    4-8
special   variable
```

Robots may use fewer frames when mechanical motion supports it.

Bosses may use significantly more.

---

# 9. Walk Cycle Standard

8-frame walk:

```text
01 contact left
02 down left
03 passing left
04 up left
05 contact right
06 down right
07 passing right
08 up right
```

Maintain identical:

- body proportions
- equipment
- armor
- colors
- lighting
- camera
- scale

Only pose changes.

---

# 10. Rotational Enemy Sprites

If classic Doom-style directional sprites are used:

Preferred directions:

```text
front
front-right
right
back-right
back
back-left
left
front-left
```

8 directions total.

Minimum acceptable:

5 unique directions with mirrored equivalents.

Never mirror asymmetrical characters unless design permits it.

Examples of non-mirrorable features:

- one-sided cybernetic arm
- shoulder weapon
- asymmetric armor
- equipment pouch
- damaged limb

---

# 11. Transparency

Preferred format:

**PNG RGBA**

Background:

**fully transparent**

Alpha requirements:

- clean silhouette
- no white fringe
- no matte halo
- no semi-transparent background haze
- smoke / energy may use partial alpha

Alpha threshold cleanup should preserve:

- hair
- cables
- antennae
- small weapon silhouettes

---

# 12. Weapon Sprite Specification

Weapons are first-person screen sprites.

Preferred source size:

**3072 x 3072 px**

Final gameplay output:

**1024 x 1024 px** recommended.

Lower-memory variant:

**512 x 512 px**

Weapon composition:

- centered
- first-person perspective
- held from bottom of frame
- hands included where applicable
- camera consistent
- no environment
- transparent background
- no HUD
- no crosshair

---

# 13. Weapon Origin

Default:

```text
origin_x = 50%
origin_y = bottom center
```

Weapons must not visibly jump between frames unless intentional recoil is being animated.

---

# 14. Weapon Animation Sets

Typical:

```text
idle        1-4
fire        3-6
reload      6-16
equip       4-8
holster     4-8
```

Optional:

```text
inspect
charge
overheat
alt_fire
```

Fire animation must preserve weapon geometry.

Do not regenerate the weapon from scratch per frame.

Use reference-image editing whenever possible.

---

# 15. Muzzle Flash

Muzzle flash should be generated separately where possible.

Benefits:

- reusable
- compositable
- easier frame control
- easier lighting effects

Recommended muzzle-flash source:

**1024 x 1024**

Final:

**256–512 px**

Transparent PNG.

---

# 16. Prop Specification

Props should use:

- orthographic-like view
- transparent background
- consistent lighting
- clear silhouette
- no environmental floor
- no perspective exaggeration

Examples:

- ammo crates
- medkits
- barrels
- terminals
- laboratory containers
- reactor equipment
- cables
- server racks
- drone racks
- warning signs
- machinery

---

# 17. Prop Final Sizes

Default:

Small:

```text
128 x 128
```

Medium:

```text
256 x 256
```

Large:

```text
512 x 512
```

Very large machinery:

```text
1024 x 1024
```

Choose canvas based on world-space importance.

---

# 18. Texture Specification

All tiling textures must be:

- seamless horizontally
- seamless vertically
- orthographic
- perpendicular camera
- flat / neutral lighting
- no perspective
- no cast shadows
- no obvious focal object
- no readable text unless intentionally designed
- no border

---

# 19. Texture Categories

Required families:

## Concrete

- clean bunker concrete
- dirty concrete
- cracked concrete
- reinforced concrete
- painted concrete
- blast-wall concrete

## Metal

- gunmetal panel
- painted steel
- galvanized steel
- brushed stainless steel
- rusted industrial metal
- armored plating

## Floors

- diamond plate
- industrial grate
- server-room raised floor
- laboratory tile
- bunker concrete
- hazard-striped metal

## Technical

- server panels
- cable trays
- vent panels
- cooling panels
- electrical cabinets
- robotic assembly surfaces

## Biotech

- containment floor
- sterile wall
- biological contamination
- synthetic tissue
- medical paneling

## Nuclear

- hazard panel
- reactor metal
- containment wall
- heat-stressed metal
- radiation-sector flooring

---

# 20. Texture Source and Final Resolution

Source:

```text
2048 x 2048
```

Preferred hero source:

```text
4096 x 4096
```

Final:

```text
512 x 512
```

Low-resolution retro variant:

```text
256 x 256
```

Keep original high-resolution source.

---

# 21. Texture Tiling Validation

Every seamless texture must be tested as:

```text
3 x 3 tiled preview
```

Check:

- horizontal seams
- vertical seams
- repeating focal patterns
- brightness mismatch
- edge discoloration
- geometry discontinuities

Reject texture if seam is noticeable at gameplay scale.

---

# 22. Texture Detail Frequency

Avoid excessive high-frequency AI noise.

Texture must remain readable after downscaling.

Prefer:

- broad panel forms
- large scratches
- visible bolts
- controlled grime
- medium-scale damage

Avoid:

- random tiny scratches everywhere
- microtext
- fake unreadable labels
- excessive pseudo-detail

---

# 23. Decals

Decals should be separate transparent PNGs.

Examples:

- blood
- oil
- warning labels
- scorch marks
- bullet impacts
- leaks
- hazard symbols
- graffiti
- cracked glass
- biotech residue

Preferred source:

```text
1024 x 1024
```

Final:

```text
256–512 px
```

---

# 24. Signage

Game signage should use actual authored text after generation.

Do not rely on image generators for readable text.

Workflow:

1. generate sign/background plate
2. remove AI text
3. add real typography in post-processing
4. export final texture

Fonts:

- military stencil
- condensed sans
- monospaced technical fonts

Avoid copyrighted logo imitation.

---

# 25. Palette Discipline

Do not force a strict indexed palette during generation.

Instead:

1. generate HD full-color source
2. normalize contrast
3. apply game color treatment
4. optionally reduce palette at final stage

Final sprites should favor:

- readable midtones
- clear shadows
- controlled highlights
- limited saturated accents

---

# 26. Lighting Consistency

Character asset master light:

```text
key light:
front-left, slightly above

fill:
weak front-right

rim:
subtle rear edge

background:
transparent
```

Do not change this between frames.

---

# 27. Shadow Policy

Enemy sprites:

- no large ground shadow baked into image

Optional:

- tiny contact shadow may be retained
- preferably generate engine-side shadow separately

Props:

same policy unless asset naturally requires baked shadow.

---

# 28. Material Readability

AI-generated assets must clearly communicate materials.

Examples:

Metal:
- controlled specular highlights
- edge wear
- solid surface

Rubber:
- low reflectance
- subtle roughness

Glass:
- controlled transparency
- strong edge reflection

Biological tissue:
- moist but not excessively glossy
- clear structural form

Concrete:
- matte
- porous
- broad surface variation

---

# 29. Robot Design Rules

Robots should look:

- engineered
- manufactured
- maintainable
- modular
- military / industrial

Use:

- panels
- bolts
- actuators
- exposed joints
- sensor clusters
- replaceable modules

Avoid overly magical floating parts.

---

# 30. Biotech Design Rules

Biotech enemies should feel like failed military science.

Combine:

- human anatomy
- surgical implants
- synthetic tissue
- cybernetic support
- reinforced skeletal structures
- medical hardware

Avoid generic fantasy demon anatomy.

---

# 31. Final Boss Source Requirements

VEYRAN // MALIK should receive the highest-quality source generation.

Preferred master reference:

```text
4096 x 4096 minimum
```

Create:

- front neutral reference
- 3/4 reference
- side reference
- material closeups
- phase 1 reference
- phase 2 reference
- phase 3 reference

Lock design before generating animations.

Final sprite:

```text
1024 x 1024
```

Minimum:

```text
768 x 768
```

---

# 32. File Naming

Use lowercase snake_case.

Examples:

```text
enemy_rifleman_idle_01.png
enemy_rifleman_walk_01.png

enemy_hound_attack_03.png

weapon_shotgun_fire_02.png

prop_server_rack_a.png

tex_concrete_bunker_01.png

decal_scorch_large_02.png
```

---

# 33. Asset IDs

Recommended logical IDs:

```text
enemy_rifleman
enemy_breacher
enemy_hornet
enemy_hound
enemy_janissary
enemy_brute

weapon_mk23s
weapon_kx9
weapon_br12
weapon_mr4
weapon_vlk6
weapon_rotary
weapon_arc
weapon_biodisruptor
weapon_sunhammer

projectile_plasma_bolt
projectile_incendiary
projectile_lance_beam
projectile_acid_seeker
vfx_muzzle_flash
vfx_impact
vfx_fire_patch
```

---

# 34. Sprite Sheet Packing

Do not ask generative AI to create production sprite sheets.

Generate individual frames.

Pack deterministically.

Default sheet:

```text
frame_size: 256x256
columns: 8
padding: 0-2 px
format: PNG RGBA
```

Boss sheet may use:

```text
512x512
768x768
1024x1024
```

depending on final implementation.

---

# 35. Metadata

Each sprite sheet should have JSON metadata.

Example:

```json
{
  "id": "enemy_rifleman",
  "frameWidth": 256,
  "frameHeight": 256,
  "origin": {
    "x": 128,
    "y": 236
  },
  "animations": {
    "idle": {
      "start": 0,
      "frames": 2,
      "fps": 3
    },
    "walk": {
      "start": 2,
      "frames": 8,
      "fps": 8
    },
    "attack": {
      "start": 10,
      "frames": 5,
      "fps": 10
    }
  }
}
```

---

# 36. Processing Filter

When reducing HD renders to game size:

Recommended first pass:

- high-quality Lanczos or bicubic downscale

If stronger retro look desired:

1. downscale smoothly
2. sharpen slightly
3. optionally reduce palette
4. final resize using nearest-neighbor only if deliberately pixelating

Do not simply nearest-neighbor scale directly from 4K source.

---

# 37. Edge Cleanup

Before export:

Check:

- white halo
- black halo
- transparency fringe
- accidental background pixels
- detached fragments
- cropped weapon
- clipped antennae
- missing feet

Automated alpha bounding-box validation is encouraged.

---

# 38. Duplicate Frame Detection

Processing tool should compute image similarity.

Warn if:

- animation frames are near-duplicates
- generator returned identical poses
- mirrored frame accidentally replaced unique pose

Do not automatically delete without review.

---

# 39. Character Consistency Validation

Compare every animation frame to reference.

Reject frame if any major feature changes:

- armor layout
- weapon model
- facial structure
- limb count
- helmet shape
- colors
- insignia
- cybernetic components

Pose variation is allowed.

Design variation is not.

---

# 40. Generation Prompt Template — Enemy

Use:

```text
Create a high-resolution full-body game sprite source render.

GAME:
BLACKSITE IFRIT

STYLE:
dark military sci-fi
industrial underground special-weapons facility
1990s pre-rendered FPS sprite readability
modern high-resolution source artwork
gritty but controlled
strong silhouette
realistic materials

CHARACTER:
[character description]

POSE:
[pose]

CAMERA:
straight-on
orthographic-like
camera at chest height
minimal perspective distortion

LIGHTING:
neutral studio lighting
front-left key light
subtle rim light
consistent game-asset lighting

COMPOSITION:
entire body visible
centered
feet aligned
no cropped limbs

BACKGROUND:
transparent
no environment
no scenery
no text
no border

IMPORTANT:
Preserve character proportions and equipment exactly.
This image will be downsampled into a retro FPS sprite.
Large readable shapes are more important than tiny detail.
```

---

# 41. Generation Prompt Template — Weapon

```text
Create a high-resolution first-person FPS weapon sprite source.

GAME:
BLACKSITE IFRIT

WEAPON:
[weapon description]

STYLE:
dark military sci-fi
industrial engineering
heavy mechanical design
retro FPS presentation
modern high-resolution source

VIEW:
first-person
centered
weapon held from bottom center
consistent camera
hands visible if appropriate

BACKGROUND:
transparent

LIGHTING:
neutral studio lighting
subtle dramatic rim light

NO:
environment
HUD
crosshair
text
border

IMPORTANT:
The weapon geometry must remain identical across animation frames.
```

---

# 42. Generation Prompt Template — Texture

```text
Create a seamless tileable game texture.

GAME:
BLACKSITE IFRIT

MATERIAL:
[material]

STYLE:
dark military-industrial special-weapons bunker
used but maintained
realistic material definition
retro FPS readability

CAMERA:
perfectly perpendicular
orthographic
no perspective

LIGHTING:
flat neutral material lighting
no dramatic shadows

REQUIREMENTS:
seamless X
seamless Y
no focal object
no text
no symbols unless requested
no border

SOURCE:
high-resolution square texture
designed to be downsampled to 512x512
```

---

# 43. Generation Prompt Template — Prop

```text
Create a high-resolution isolated game prop.

GAME:
BLACKSITE IFRIT

PROP:
[prop description]

STYLE:
military-industrial sci-fi
functional engineering
used but maintained
strong readable silhouette
retro FPS asset design

CAMERA:
orthographic-like
slightly elevated only if necessary
minimal perspective

BACKGROUND:
transparent

LIGHTING:
neutral studio asset lighting

NO:
environment
floor
text
border

The entire object must be visible.
```

---

# 44. Asset Review Checklist

Before approving any asset:

## Style

- [ ] matches BLACKSITE IFRIT
- [ ] does not look like generic AI sci-fi
- [ ] silhouette is readable
- [ ] materials are clear

## Technical

- [ ] high-resolution source saved
- [ ] transparent background where required
- [ ] no clipping
- [ ] correct camera
- [ ] correct lighting
- [ ] correct scale
- [ ] correct origin
- [ ] correct filename

## Consistency

- [ ] matches reference
- [ ] same equipment
- [ ] same color scheme
- [ ] same anatomy
- [ ] same weapon

## Gameplay

- [ ] recognizable at distance
- [ ] role visually obvious
- [ ] not overloaded with detail

---

# 45. Final Quality Principle

The AI generator creates the artwork.

The asset pipeline creates the game asset.

Never accept dimensional accuracy, frame alignment, sprite-sheet packing, transparency cleanup, or metadata from the image model without deterministic validation.

BLACKSITE IFRIT assets should begin as polished HD illustrations and end as highly controlled, readable retro-FPS production assets.
