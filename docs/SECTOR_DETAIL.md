# Sector materials, destructibles and projectile repairs

Every sector has five exclusive materials: structural wall, service wall, sliding door, floor and ceiling. Runtime PNG tiles are 256×256; 1024×1024 WebP material copies are available in each sector's `hd/` directory. Opposite runtime edges match exactly for seamless repetition. Sector selection also applies to the floor beneath the override station. Endless waves reuse their corresponding sector artwork.

Each sector adds three exclusive destructible machinery/item types, with two placements per type where open side spaces allow. Shots and explosions can destroy them; breaking scenery drops ammo, health or armor and does not increase enemy kills. Spark and impact effects accompany destruction. Placement avoids doors, player spawns, objectives and existing objects.

| Level | Sector folder | Three destructible types |
| --- | --- | --- |
| 1 | `public/game/sectors/upper-works/` | olive spare-ammunition locker; compressed-air cylinder trolley; worn electrical junction cabinet |
| 2 | `public/game/sectors/cryogenic-foundry/` | frosted coolant reservoir; cast-metal induction furnace capacitor; blue insulated pressure vessel |
| 3 | `public/game/sectors/bioforge-depths/` | green specimen containment tank; sealed biohazard waste cart; glass nutrient pump reservoir |
| 4 | `public/game/sectors/data-spine/` | cyan data-server tower; fiber-optic patch cabinet; rack-mounted backup battery unit |
| 5 | `public/game/sectors/reactor-sink/` | copper reactor coolant pump; yellow nuclear capacitor bank; armored turbine control cabinet |
| 6 | `public/game/sectors/null-archive/` | magnetic tape archive cabinet; amber memory cartridge trolley; bronze archive cooling unit |
| 7 | `public/game/sectors/cryo-reserve/` | liquid nitrogen dewar; frozen specimen freezer cabinet; cryo-compressor skid |
| 8 | `public/game/sectors/signal-crypt/` | red signal amplifier cabinet; satellite modem relay rack; sealed communications battery chest |
| 9 | `public/game/sectors/siege-yard/` | armored missile reload pallet; sand-colored hydraulic lift powerpack; heavy artillery battery cart |
| 10 | `public/game/sectors/command-bunker/` | tactical radio command cabinet; secure operations document case; olive command UPS cabinet |
| 11 | `public/game/sectors/obsidian-vault/` | violet memory-core reliquary; obsidian encryption server; sealed black memory cartridge case |
| 12 | `public/game/sectors/ashen-transit/` | orange railway power inverter; soot-covered pneumatic rail compressor; steel transit fuel canister |
| 13 | `public/game/sectors/tidal-pumpstation/` | blue pressure pump motor; brass water filtration canister; naval hydraulic accumulator |
| 14 | `public/game/sectors/optics-array/` | cyan laser calibration emitter; chrome lens storage cabinet; optical bench power supply |
| 15 | `public/game/sectors/magnetic-forge/` | red induction coil transformer; copper magnet cooling pump; laminated steel flux capacitor bank |
| 16 | `public/game/sectors/drone-hatchery/` | amber drone charging cradle; robotic assembly parts cassette; hexagonal drone battery tower |
| 17 | `public/game/sectors/fungal-research/` | sealed green fungal incubation tank; contaminated sample refrigeration cabinet; spore filter pressure unit |
| 18 | `public/game/sectors/orbital-uplink/` | violet satellite targeting power rack; orbital telemetry storage cabinet; compact antenna gimbal controller |
| 19 | `public/game/sectors/waste-reclaimer/` | rusted scrap sorting hopper; copper hydraulic fluid drum; industrial shredder motor cabinet |
| 20 | `public/game/sectors/shadow-lab/` | purple cloak-field generator; matte black stealth calibration rack; phase containment battery housing |
| 21 | `public/game/sectors/fusion-chamber/` | gold fusion containment capacitor; white ceramic coolant circulation pump; fusion fuel cartridge caddy |
| 22 | `public/game/sectors/flooded-silo/` | teal torpedo guidance locker; naval ballast pressure cylinder; brass silo hydraulic valve station |
| 23 | `public/game/sectors/chrono-archive/` | bronze gyroscope timing core; amber chronology tape cabinet; temporal calibration capacitor |
| 24 | `public/game/sectors/black-ice-core/` | ice-blue supercomputer coolant rack; black insulated refrigeration compressor; frosted emergency power cell cabinet |
| 25 | `public/game/sectors/apex-control/` | gold command encryption terminal rack; black apex security power cabinet; gold-plated tactical reserve cartridge chest |

## Runtime assets

- `public/game/sectors/<sector>/wall.png`, `service.png`, `door.png`, `floor.png`, `ceiling.png`: 125 exclusive 256×256 seamless materials.
- `public/game/sectors/<sector>/hd/<material>.webp`: 125 high-resolution 1024×1024 material copies, kept outside gameplay preload.
- `public/game/sectors/<sector>/prop_1.png`, `prop_2.png`, `prop_3.png`: 75 transparent 256×256 exclusive destructible prop sprites.
- `public/game/projectiles/enemy_<skin>.png`: 37 isolated 256×256 type-specific projectile textures. Ranged attacks select their own texture; melee enemies remain melee. This includes a distinct image for every boss; Broodmother's special attack remains a drone summon.
- `public/game/projectiles/missile_<tactical|siege|naval>_<outgoing|incoming>.png`: six 256×256 native-alpha missile cutouts. Player shots use the upward/forward three-quarter body view with nose visible; incoming enemy missiles use a head-on nose view. Camera rotation never swaps an outgoing shot to an incoming model. Smoke expires within one second.
- `public/game/spr_ordnance.png`: repaired legacy 512×512 atlas, four complete 256×256 cells. Live enemy attacks use isolated textures rather than this mixed legacy sheet.

The atlas has 579 layers. Rust constants and WebGPU/WebGL texture-array sizes agree. Gameplay preloads all runtime materials, props, enemy projectiles and weapon sheets; HD review copies are not downloaded by the loading screen.

## What caused the glitches

The previous ordnance patch placed a 128-pixel missile into a 512-pixel atlas at coordinates intended for a 256-pixel atlas. It therefore overwrote a quarter of the first orange orb. The packing code now derives cell dimensions from the actual atlas, and live attacks use separate complete images.

Weapon animation cells contained small disconnected strips from neighboring source panels. Alpha-mask cleanup removes these fragments from pickup, reload, ammo-state and firing cells without redrawing weapons or changing frame placement. Substantial detached hands and magazines stay. Approved aim and exact return-to-aim cells stay unchanged; dry fire remains disabled. The canvas isolates a one-pixel cell border, and both GPU sprite shaders clamp animated UVs to the chosen cell's texel centers.

## Source and regeneration

`art/sector-detail/specs.json` defines themes and prop types. `projectiles.json` maps all 37 projectile textures to native source cells. `generation.json` contains the complete built-in imagegen prompt set, portable source paths and SHA-256 hashes. Transparent prop replacement boards take precedence over mixed material/prop boards when necessary.

```sh
python3 scripts/pack-sector-detail.py
python3 scripts/clean-weapon-frame-edges.py
python3 scripts/refresh-campaign25-catalog.py
npm run build:wasm
node --test scripts/sector-detail-assets.test.mjs scripts/weapon-frame-edges.test.mjs
cargo test --manifest-path engine/Cargo.toml
```

The packer only crops, scales and packs generated artwork, cleans disconnected alpha fragments and joins texture edges. It does not synthesize new raster art. `--props-only` refreshes prop replacements without rebuilding material exports. After regenerating older campaign art, run the sector packer and weapon edge cleanup again before refreshing catalog hashes.
