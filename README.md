# BLACKSITE

A Doom-style first-person raycaster set in Site Nadir-7. The Rust simulation runs as **WebAssembly**, with **WebGPU**, WebGL2 and Canvas2D rendering. The campaign has **11 distinct sectors, 11 unique bosses and 19 weapons**, including one exclusive weapon reward per boss.

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

Each sector has its own layout, enemy cast and themed machinery. Complete its node objective, activate the override station, defeat the boss and collect the reward case to unlock the next sector. After sector 11, the campaign cycles with increasing difficulty. Found weapons carry forward.

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

The latest addition, **Obsidian Vault**, connects two memory galleries to a broad boss arena. MNEMOSYNE releases moving echo mines and radial pulse barrages at health thresholds. Its **MN-6 Echo** reward is a 12-charge lance that pierces up to three targets. See [the expansion notes](docs/OBSIDIAN_VAULT.md) for gameplay, assets and wire-format details.

The standard arsenal is MK23-S, BR-12 Breaker, KX-9 Vector, MR-4 Longbow, VLK-6 Warden, AX-12 Volt, M91 Cyclone and HX-8 Pyre.

## Art, audio and interface

- Weapon viewmodels use **5×5 sheets of 25 frames**, 2560×1920 overall, with 512×384 cells for ammunition states, pickup, reload, firing and spare special frames. Aim poses enter from the lower right toward the crosshair.
- **23 enemy skins** each have seven animation states with four frames per state: idle, movement, pain, fire, reload/charge, death and special. Runtime sheets are 256×256 with four 128×128 cells.
- The loading screen fetches and decodes weapon sheets, thumbnails and game assets before deployment. The wheel uses transparent gun thumbnails; its selected weapon is larger and fully opaque, while other weapons are dimmed and desaturated.
- **116 Cloudflare Aura-2 combat voice clips** cover 19 speaking profiles; four profiles are nonverbal. Bosses also have **22 death clips**, two per boss. Positional audio and enemy subtitles share the voice manifest. A shared voice gate prevents overlapping dialogue; interaction announcements and boss death events play once. Doors do not trigger dialogue.
- `public/game/music/menu.mp3` covers menus, pause, settings and other non-game screens. `bgm-remix.mp3` and `boss.mp3` provide gameplay music.
- The minimap sits below the handler panel; health and ammunition cards share a compact lower-left layout. Settings persist locally and include an enemy-subtitles toggle. The main menu has no resume-sector option.
- The asset catalog is available from the start menu only in local development. It shows the current weapon set and animation frames and is excluded from the production build.

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
| `art/bosses/` | Boss source poses |
| `art/source_hd/` | Source art and voice generation records |

Combat and interface background are documented in [combat notes](docs/blacksite-ifrit/COMBAT.md) and [voice/interface notes](docs/blacksite-ifrit/VOICES_AND_UI.md). Missing or replacement sound effects are tracked in [issue #1](https://github.com/Idanbot/wasm-doom/issues/1).

## Architecture

```text
engine/src/                 Rust simulation → public/hellscan.wasm
  consts.rs                   map, atlas, entities, input and weapon constants
  map.rs                      sector layouts, spawns and objectives
  field.rs                    nodes, interaction IDs and boss rewards
  combat.rs                   enemy combat profiles
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

The HUD is a 224-byte Rust `#[repr(C)]` struct decoded with a DataView. Compile-time assertions, a boot-time `hs_hud_size` check and tests guard its layout. Saves cover 19 weapon slots in 184 bytes. Older supported checkpoints pad to the current slot count. The texture atlas has 218 layers.

## Develop

Prerequisites: **Node 24** and **Rust stable** with the `wasm32-unknown-unknown` target.

```sh
npm ci
npm run build:wasm   # compile the engine and update its source hash
npm run dev         # development server on port 8080
```

`startup.sh` starts development through `npm run dev` and rebuilds WASM when needed. After changing engine sources, run `npm run build:wasm` so the committed binary and source hash stay synchronized. Add `?qa=1` for the browser QA controls.

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
node scripts/browser-smoke.mjs
node scripts/combat-smoke.mjs
node scripts/end-screen-smoke.mjs
```

The engine has **102 Rust tests**. Coverage includes sector reachability, all boss entries and phases, distinct reward drops, Echo firing and save roundtrips, manual reload behavior, projectiles and movement direction. Browser checks cover asset preload and failure handling, arsenal thumbnails, firing/reload frames, HUD containment, bosses, desktop/mobile rendering and end screens.

[CI](.github/workflows/ci.yml) runs typechecking, WASM synchronization, asset validation, production build, renderer performance checks, TypeScript/Rust tests and browser smoke checks on pushes and pull requests.

## Deploy

The game runs entirely in the browser and needs no runtime database or application server. Settings and the leaderboard use local storage.

```sh
npm run build:pages  # static output in dist-pages/, base /wasm-doom/
```

The [Pages workflow](.github/workflows/pages.yml) builds and publishes on pushes to `main`. The existing user site owns `idanbot.me`; this repository serves [idanbot.me/wasm-doom/](https://idanbot.me/wasm-doom/) and should not have a separate custom domain.
