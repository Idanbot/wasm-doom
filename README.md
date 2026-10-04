# BLACKSITE

A standalone tactical first-person raycaster set in Site Nadir-7. The Rust simulation runs as **WebAssembly**, with **WebGPU**, WebGL2 and Canvas2D rendering. The campaign has **25 distinct sectors, 25 unique bosses, 62 enemy/boss appearances and 33 weapons**, including one exclusive weapon reward per boss.

[Play BLACKSITE](https://idanbot.me/wasm-doom/)

## Play

| Input | Action |
| --- | --- |
| `W A S D` | Move; A strafes left and D strafes right |
| Arrow keys | Move forward/back and turn |
| Mouse with pointer lock | Look |
| Click / `Space` / `Ctrl` | Fire |
| Mouse wheel | Rotate the arsenal and equip an unlocked weapon |
| `1`–`8` | Select the eight standard weapons |
| `9`, `0`, `-`, `=`, `[`, `]`, `\`, `;`, `'`, backtick, `,` | Select boss rewards in weapon-slot order |
| `R` | Reload manually |
| `E` | Use nodes, terminals and override stations; open secret doors |
| `Shift` | Sprint |
| `P` / `Esc` | Pause |

Touch controls appear on coarse-pointer devices. Weapons stay locked until collected. The BR-12 loads one shell at a time; firing with a loaded shell interrupts its reload. Other guns use magazine or charge-cell reloads. Empty weapons wait for a manual reload and do not animate dry fire.

Each sector has its own layout, enemy cast and themed machinery. Complete its node objective, activate the override station, defeat the boss and collect the reward case to unlock the next sector. After sector 25, the campaign repeats its layouts in endless mode with gradually increasing enemy totals and health. Found weapons carry forward.

Each sector now includes an exclusive enemy silhouette designed for its machinery,
materials and colors. Twelve regular enemies and bosses in sectors 1–11 have
fresh 28-pose animation boards: four frames each for idle, movement, pain,
firing, reload/charge, death and special. One-shot states show every pose and
hold their last frame; movement and breathing loop. Native transparency avoids
chroma-key holes in armor. [Enemy art sources and packing](art/enemies-v4/README.md)
include prompts, hashes and a roster preview. Runtime sheets are 256×256,
with four 128×128 cells; all 804 world atlas layers preload before play.

## Sectors and boss rewards

| Sector | Boss | Reward |
| --- | --- | --- |
| Upper Works | Malik Veyran | VR-9 Override |
| Cryogenic Foundry | HECATE–9 | HC-9 Forge |
| Bioforge Depths | CHIMERA–9 | CM-9 Chimera |
| Data Spine | ORACLE–7 | OR-7 Predictor |
| Reactor Sink | GRAVEMIND–4 | GS-4 Sink |
| Null Archive | Null Archivist | AR-6 Archive |
| Cryo Reserve | HALCYON–3 | CR-3 Rime |
| Signal Crypt | RELAY–0 | SR-0 Relay |
| Siege Yard | TITAN–12 | TS-12 Titan |
| Command Bunker | Director Kest | KS-8 Kest |
| Obsidian Vault | MNEMOSYNE–6 | MN-6 Echo |
| Ashen Transit | VULCAN–2 | VX-2 CINDER |
| Tidal Pumpstation | LEVIATHAN–8 | LV-8 UNDERTOW |
| Optics Array | PRISM–5 | PR-5 SPECTRUM |
| Magnetic Forge | MAGNUS–4 | MG-4 POLARITY |
| Drone Hatchery | BROODMOTHER–3 | BM-3 SWARM |
| Fungal Research | MYCELIUM–9 | MY-9 SPORE |
| Orbital Uplink | ASTRA–7 | AS-7 STARFALL |
| Waste Reclaimer | SCRAPPER–6 | SC-6 SHREDDER |
| Shadow Lab | UMBRA–1 | UB-1 VEIL |
| Fusion Chamber | SOL–12 | SL-12 HELIOS |
| Flooded Silo | NAUTILUS–4 | NT-4 TORPEDO |
| Chrono Archive | CHRONOS–8 | CH-8 EPOCH |
| Black Ice Core | BOREAS–11 | BO-11 GLACIER |
| Apex Control | THE SOVEREIGN | SV-25 DOMINION |

Fourteen new boss sectors extend the campaign through Apex Control. Bosses use summons, magnetic pulls, relocation, shields, radial attacks and staggered salvos. See [the expansion guide](docs/CAMPAIGN_25.md) for the complete roster and assets.

The standard arsenal is MK23-S, BR-12 Breaker, KX-9 Vector, MR-4 Longbow, VLK-6 Warden, AX-12 Volt, M91 Cyclone and HX-8 Pyre.

## Art, audio and interface

- Weapon viewmodels use **5×5 sheets of 25 frames**, 2560×1920 overall, with 512×384 cells for ammunition states, pickup, reload, firing and spare special frames. Aim poses enter from the lower right toward the crosshair.
- **62 enemy/boss appearances** each have seven animation states with four frames per state: idle, movement, pain, fire, reload/charge, death and special. Runtime sheets are 256×256 with four 128×128 cells.
- Enemy ranged attacks use **37 isolated, type-specific projectile textures**, including distinct boss projectiles. This fixes the legacy atlas patch that mixed a quarter missile into an orange orb. Melee enemies still use melee attacks.
- Maps **10–25** use rebuilt room graphs with alternate routes, relocated sector nodes, terminals, supplies and boss arenas. The authoring source is `art/maps/layouts-10-25.json`; [layout overview](art/maps/layouts-10-25.png) shows every route.
- All **25 boss-reward weapons** have their own isolated projectile or hitscan travel artwork. **BM-3 SWARM** launches five visible mechanical microdrones with homing and rotor bob. The 37 enemy projectile images are a separate incoming set with head-on missile noses.
- Player missiles show their body and nose pointing upward/forward toward the target; enemy missiles show their nose head-on toward the player. Tactical, siege and naval designs have separate outgoing/incoming art, with smoke trails lasting less than a second. Nine campaign FX assets also provide ion bolts, cryo shards, spores, phase shots and animated impacts.
- Every sector has **five exclusive material textures and three exclusive destructible prop types**: 125 materials and 75 prop designs total. Runtime tiles are seamless 256×256 PNGs; 1024×1024 WebP copies are available for asset review. Breaking machinery produces sparks and supply drops without increasing enemy kills. See [sector art and projectile details](docs/SECTOR_DETAIL.md).
- All weapon handling sheets have cleaned alpha edges. Pickup and reload frames retain substantial detached hands/magazines while discarding neighboring-panel fragments. Approved aim poses and return-to-aim frames are preserved.
- The loading screen fetches and decodes weapon sheets, thumbnails and game assets before deployment. The wheel uses transparent gun thumbnails; its selected weapon is larger and fully opaque, while other weapons are dimmed and desaturated.
- **200 Cloudflare Aura-2 combat voice clips** cover 33 speaking voice profiles, reused by themed sector enemies; four profiles are nonverbal. All 25 bosses use fresh clean Aura-2 performances at natural pitch and pace, plus **50 death clips**, two per boss. Positional audio and enemy subtitles share the voice manifest. A shared voice gate prevents overlapping dialogue; interaction announcements and boss death events play once. Doors do not trigger dialogue.
- `public/game/music/menu.mp3` covers menus, pause, settings and other non-game screens. `bgm-remix.mp3` and `boss.mp3` provide gameplay music.
- The minimap sits below the handler panel; health and ammunition cards share a compact lower-left layout. Settings persist locally and include an enemy-subtitles toggle. The main menu has no resume-sector option.
- The asset catalog is available from the start menu only in local development. It shows the current weapon set and animation frames and is excluded from the production build.
- Boss reward cases have golden hard-shell exteriors and matching fitted guns; their arsenal wheel icons are strict side-profile cutouts. Obsidian Vault drops MN-6 ECHO, and both Esc and P pause/resume the operation.
- All twenty-five boss reward guns use regenerated pickup and mechanical reload poses in transparent 5×5 sheets (2560×1920, 512×384 cells), retaining their approved aim images. Source boards and packing details are in [the boss animation guide](docs/BOSS_WEAPON_ANIMATIONS.md).

| Folder | Contents |
| --- | --- |
| `public/game/draft/v2/` | Current 5×5 weapon sheets and aim images |
| `public/game/draft/v2/masters/` | Weapon source cutouts |
| `public/game/draft/v2/cases/` | Weapon case artwork |
| `public/game/ui/weapon-thumbs/` | Arsenal thumbnails |
| `public/game/` | Runtime enemy sheets, pickups, projectiles and effects |
| `public/game/theme/` | Wall and door texture variants |
| `public/game/voices/` | Runtime MP3 voices and subtitle manifest |
| `public/game/music/` | Runtime MP3 music |
| `public/game/sfx/v2/` | 103 CC0 recorded/mixed effects with MP3 fallbacks |
| `art/audio/` | Selected original recordings, licenses, creator credits, hashes and rebuild recipes |
| `public/game/fx25/` | Campaign projectile and four-frame impact atlases |
| `public/game/projectiles/` | Isolated enemy projectiles and outgoing/incoming missiles |
| `public/game/sectors/<sector>/` | Five exclusive materials and three destructible prop sprites per sector |
| `public/game/sectors/<sector>/hd/` | 1024×1024 WebP material review copies |
| `art/sector-detail/` | Native source boards, full prompts, hashes, sector themes and projectile mappings |
| `art/campaign25/` | Native source boards, full prompts, hashes and the level 12–25 roster |
| `art/bosses/` | Boss source poses |
| `art/source_hd/` | Source art and voice generation records |

The [25-sector expansion guide](docs/CAMPAIGN_25.md) lists new bosses, mechanics, asset paths and endless scaling. Combat and interface background are documented in [combat notes](docs/blacksite-ifrit/COMBAT.md) and [voice/interface notes](docs/blacksite-ifrit/VOICES_AND_UI.md). [Sound effect credits and integration](art/audio/README.md) document the CC0 replacement set for [issue #1](https://github.com/Idanbot/wasm-doom/issues/1): distinct firing sounds for all 33 guns, frame-synced magazine/cell reloads, incremental BR-12 shell inserts, spatial enemy/world effects, moving projectile loops, material impacts, distinct pickups and UI cues. All 103 effects decode during loading; MP3 fallbacks support compatibility. Recorded injury vocals share the dialogue gate, and mute/volume controls apply through the existing mixer.

## Architecture

```text
engine/src/                 Rust simulation → public/blacksite.wasm
  consts.rs                   map, atlas, entities, input and weapon constants
  map.rs                      sector layouts, spawns and objectives
  layouts.rs                  authored room graphs and sector contracts for maps 10–25
  field.rs                    nodes, interaction IDs and boss rewards
  combat.rs                   enemy combat profiles
  campaign.rs                 uncapped progression and expansion weapon stats
  enemies.rs                  enemy and pickup definitions
  hud.rs                      HUD wire format and compile-time assertions
  voices.rs                   enemy presentation snapshots
  sound.rs                    bounded positional world sound events
  lib.rs                      simulation, raycaster, lighting and FFI
src/game/
  runtime.ts                  WASM loader, fixed-60Hz loop, input and preload
  gpu-world.ts / blit.ts       world rendering and presentation fallbacks
  hud-abi.ts / save-abi.ts     shared Rust/TypeScript wire contracts
  weapon-assets.ts            viewmodel and thumbnail slot order
  audio.ts / enemy-audio.ts    music, effects and positional voices
  voice-gate.ts                serialized dialogue playback
src/components/game/        menus, HUD, arsenal, settings and touch controls
src/components/catalog/     local-development asset review
```

The HUD is a 228-byte Rust `#[repr(C)]` struct decoded with a DataView. Compile-time assertions, a boot-time `hs_hud_size` check and tests guard its layout. Saves cover 33 weapon slots in 300 bytes. Older supported checkpoints pad to the current slot count. The texture atlas has 804 layers.

## Develop

BLACKSITE has no builder SDK, injected branding, preview bridge, account broker,
connector gateway or runtime database. Its menus, manifest and social metadata
are owned by the game. Browser storage uses `blacksite-*` keys; existing player
settings, scores and checkpoints migrate automatically without overwriting new data.

Prerequisites: **Node 24** and **Rust stable** with the `wasm32-unknown-unknown` target.

```sh
npm ci
npm run build:wasm   # compile the engine and update its source hash
npm run dev         # development server on port 8080
```

`startup.sh` starts development through `npm run dev` and rebuilds WASM when needed. After changing engine sources, run `npm run build:wasm` so the committed binary and source hash stay synchronized. On the local development server, `?qa=1` starts a fresh run with all 33 weapons unlocked. Add `&lvl=x` to start at sector **1–25**, for example `http://localhost:8080/?qa=1&lvl=11` opens Obsidian Vault. Missing or invalid levels start at sector 1. Use `&lvl=25` to test the Sovereign fight. These shortcuts are ignored in production and on non-local hosts. Expanded rewards beyond the first eleven use the mouse wheel.

Install `python3 -m pip install -r art/enemies-v4/requirements.txt` for the enemy-art pipeline. Asset-generation scripts use Python/Pillow and ffmpeg where applicable. Cloudflare voice generation requires a server-side `CF_API_KEY` and `CF_ACCOUNT_ID`; credentials stay in the ignored `.env` or process environment and never ship in browser assets. Existing voice files are cached to avoid repeat generation.

## Verify

```sh
npm run typecheck
npm test
cargo test --manifest-path engine/Cargo.toml
npm run check:wasm
npm run check:assets
npm run check:environment
npm run check:projectiles
npm run check:voices
npm run check:motion
node --test scripts/sector-detail-assets.test.mjs scripts/weapon-frame-edges.test.mjs
npm run build
```

With the development server running:

```sh
npm run test:browser
node --test scripts/projectile-presentation.browser.mjs
node scripts/browser-smoke.mjs http://127.0.0.1:8080/ screenshots/qa-menu.png
node scripts/combat-smoke.mjs
node scripts/end-screen-smoke.mjs
```

The engine has **119 Rust tests**. Coverage includes sector reachability, all boss entries and phases, distinct reward drops, Echo firing and save roundtrips, manual reload behavior, projectiles and movement direction. Browser checks cover asset preload and failure handling, arsenal thumbnails, firing/reload frames, HUD containment, bosses, desktop/mobile rendering and end screens.

[CI](.github/workflows/ci.yml) runs typechecking, WASM synchronization, asset validation, production build, renderer performance checks, TypeScript/Rust tests and browser smoke checks on pushes and pull requests.

## Deploy

The game runs entirely in the browser and needs no runtime database or application server. Settings and the leaderboard use local storage.

```sh
npm run build:pages  # static output in dist-pages/, local base /blacksite/
PAGES_BASE=/custom-path/ npm run build:pages  # optional hosting path
```

The [Pages workflow](.github/workflows/pages.yml) builds and publishes on pushes to `main`, deriving its base path from `GITHUB_REPOSITORY`. The local default is `/blacksite/`; `PAGES_BASE` overrides either value. The existing user site owns `idanbot.me`; this repository serves [idanbot.me/wasm-doom/](https://idanbot.me/wasm-doom/) and should not have a separate custom domain.

### Boss arenas and balance

Every boss has a generated seamless arena material and a distinct floor hazard: induction spokes, acid rings, prediction grids, crushing lanes, pressure spirals and more. The spawn sequence stages sector-specific steam, sparks or impacts before the arena changes. Hazards warn for 1.6 seconds, strike for 1.2 seconds, then recover; the override console remains a refuge. Damage has a 0.6-second cooldown, and all hazard tiles restore when the boss dies or the sector changes.

CM-9 direct, splash and acid-pool damage now use 67% of the previous damage (integer hits round; pool damage accumulates fractional remainder). Shift sprint is 10% faster: 4.422 world units/second, also the maximum speed with Overdrive.

Arena masters and exact image-generation prompts: `art/boss-arenas-v1/`; runtime materials: `public/game/boss-arenas/`; 1024×1024 review copies: `public/game/boss-arenas/hd/`. Boss voice sources, prompts and estimated usage: `art/boss-voices-v3/`. Rebuild materials with `python3 scripts/pack-boss-arenas.py`; regenerate voices with `node scripts/generate-boss-voices-v3.mjs --account <account-id>` using ignored `CF_API_KEY`.

### Tactical combat, secrets and machinery

Weapon families now have complementary roles. Precision weapons bypass 40% of body armor absorption; frontal shields still stop them. Acid primes a target for three seconds: its next precision hit gains 25% damage and consumes the primer. Shock interrupts attacks and drains shields, with cooldowns that prevent boss stun locks. Rime slows movement; automatic suppression delays return fire; close-range shotgun hits stagger. Existing missile splash and Pyre pools supply blast and area denial. All 33 weapons retain their approved sprites and original firing patterns. Quick equip takes 0.28 seconds, while separate weapon cooldowns prevent switching from bypassing cadence.

Every sector has a concealed service cache or its existing secret rooms. Scratches, interrupted cable tracks and ventilation marks lead toward scuffed service panels; face one and press **E**. New caches are carved into unused wall space without replacing objectives or routes. Supplies and a recovered log are awarded once per cache, including paired door leaves.

Shoot machinery to use the room tactically. Aim at the sector-specific object to see its name and effect. Coolant slows nearby enemies for six seconds; power equipment disables local shield emitters; network equipment blocks queued arrivals and ambush reinforcements within its circuit. Pressure equipment staggers, biological containment primes targets, fuel explodes dangerously, and supply fixtures yield ammunition and armor. Cover blocks these effects; outages and status effects reset on sector changes.

Prop identities and effects: `src/game/tactical-machinery.json` and `engine/src/tactical_roles.rs`. Systems and tests: `engine/src/tactical.rs`, `scripts/tactical.browser.mjs`, and `scripts/tactical-data.test.mjs`. The HUD/save ABI and 804-layer atlas remain unchanged.
