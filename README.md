# BLACKSITE

A Doom-style first-person raycaster set in Site Nadir-7. The Rust simulation runs as **WebAssembly**, with **WebGPU**, WebGL2 and Canvas2D rendering. The campaign has **25 distinct sectors, 25 unique bosses and 33 weapons**, including one exclusive weapon reward per boss.

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
- **37 enemy skins** each have seven animation states with four frames per state: idle, movement, pain, fire, reload/charge, death and special. Runtime sheets are 256×256 with four 128×128 cells.
- Nine new effect assets provide forward-facing missiles, ion bolts, cryo shards, spores, phase shots, and animated ember, pressure, magnetic and solar impacts. Weapon and enemy attack types select their matching effects. Outgoing and incoming rockets use different views; smoke trails expire within one second.
- The loading screen fetches and decodes weapon sheets, thumbnails and game assets before deployment. The wheel uses transparent gun thumbnails; its selected weapon is larger and fully opaque, while other weapons are dimmed and desaturated.
- **116 Cloudflare Aura-2 combat voice clips** cover 19 speaking profiles; four profiles are nonverbal. Bosses also have **50 death clips**, two per boss. Positional audio and enemy subtitles share the voice manifest. A shared voice gate prevents overlapping dialogue; interaction announcements and boss death events play once. Doors do not trigger dialogue.
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
| `public/game/fx25/` | Campaign projectile and four-frame impact atlases |
| `art/campaign25/` | Native source boards, full prompts, hashes and the level 12–25 roster |
| `art/bosses/` | Boss source poses |
| `art/source_hd/` | Source art and voice generation records |

The [25-sector expansion guide](docs/CAMPAIGN_25.md) lists new bosses, mechanics, asset paths and endless scaling. Combat and interface background are documented in [combat notes](docs/blacksite-ifrit/COMBAT.md) and [voice/interface notes](docs/blacksite-ifrit/VOICES_AND_UI.md). Missing or replacement sound effects are tracked in [issue #1](https://github.com/Idanbot/wasm-doom/issues/1).

## Architecture

```text
engine/src/                 Rust simulation → public/hellscan.wasm
  consts.rs                   map, atlas, entities, input and weapon constants
  map.rs                      sector layouts, spawns and objectives
  field.rs                    nodes, interaction IDs and boss rewards
  combat.rs                   enemy combat profiles
  campaign.rs                 uncapped progression and expansion weapon stats
  enemies.rs                  enemy and pickup definitions
  hud.rs                      HUD wire format and compile-time assertions
  voices.rs                   enemy presentation snapshots
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

The HUD is a 228-byte Rust `#[repr(C)]` struct decoded with a DataView. Compile-time assertions, a boot-time `hs_hud_size` check and tests guard its layout. Saves cover 33 weapon slots in 300 bytes. Older supported checkpoints pad to the current slot count. The texture atlas has 339 layers.

## Develop

Prerequisites: **Node 24** and **Rust stable** with the `wasm32-unknown-unknown` target.

```sh
npm ci
npm run build:wasm   # compile the engine and update its source hash
npm run dev         # development server on port 8080
```

`startup.sh` starts development through `npm run dev` and rebuilds WASM when needed. After changing engine sources, run `npm run build:wasm` so the committed binary and source hash stay synchronized. On the local development server, `?qa=1` starts a fresh run with all 33 weapons unlocked. Add `&lvl=x` to start at sector **1–25**, for example `http://localhost:8080/?qa=1&lvl=11` opens Obsidian Vault. Missing or invalid levels start at sector 1. Use `&lvl=25` to test the Sovereign fight. These shortcuts are ignored in production and on non-local hosts. Expanded rewards beyond the first eleven use the mouse wheel.

Asset-generation scripts use Python/Pillow and ffmpeg where applicable. Cloudflare voice generation requires a server-side `CF_API_KEY` and `CF_ACCOUNT_ID`; credentials stay in the ignored `.env` or process environment and never ship in browser assets. Existing voice files are cached to avoid repeat generation.

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
npm run build
```

With the development server running:

```sh
npm run test:browser
node scripts/browser-smoke.mjs http://127.0.0.1:8080/ screenshots/qa-menu.png
node scripts/combat-smoke.mjs
node scripts/end-screen-smoke.mjs
```

The engine has **108 Rust tests**. Coverage includes sector reachability, all boss entries and phases, distinct reward drops, Echo firing and save roundtrips, manual reload behavior, projectiles and movement direction. Browser checks cover asset preload and failure handling, arsenal thumbnails, firing/reload frames, HUD containment, bosses, desktop/mobile rendering and end screens.

[CI](.github/workflows/ci.yml) runs typechecking, WASM synchronization, asset validation, production build, renderer performance checks, TypeScript/Rust tests and browser smoke checks on pushes and pull requests.

## Deploy

The game runs entirely in the browser and needs no runtime database or application server. Settings and the leaderboard use local storage.

```sh
npm run build:pages  # static output in dist-pages/, base /wasm-doom/
```

The [Pages workflow](.github/workflows/pages.yml) builds and publishes on pushes to `main`. The existing user site owns `idanbot.me`; this repository serves [idanbot.me/wasm-doom/](https://idanbot.me/wasm-doom/) and should not have a separate custom domain.
