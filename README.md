# BLACKSITE

A Doom-style raycaster that runs entirely in the browser. The simulation is written in **Rust**, compiled to **WebAssembly**, and presented through **WebGPU** (with WebGL2 / Canvas2D fallbacks). Sweep three distinct sectors of Site Nadir-7 and defeat Malik Veyran, HECATE–9, and CHIMERA–9.

## Play

| Input                    | Action                                                   |
| ------------------------ | -------------------------------------------------------- |
| `W A S D` / arrows       | Move + turn                                              |
| Mouse (pointer lock)     | Look                                                     |
| Click / `Space` / `Ctrl` | Fire                                                     |
| `1`–`8`                  | Weapons: MK23-S, BR-12, KX-9, MR-4, VLK-6, AX-12, M91, HX-8 |
| `R`                      | Reload                                                   |
| `E`                      | Use doors / pickups                                      |
| `Shift`                  | Sprint                                                   |
| `P` / `Esc`              | Pause                                                    |

Touch controls appear automatically on coarse-pointer devices. Add `?qa=1` to the URL for autoplay QA mode.

## Deploy

The game is entirely in the browser: simulation, weapons, voices, music, settings, and the local leaderboard. A static host can serve that. It does not need the TanStack/Nitro server.

It is published at [idanbot.me/wasm-doom](https://idanbot.me/wasm-doom/). `npm run build:pages` writes `dist-pages/` with base `/wasm-doom/`. The user site already owns `idanbot.me`, so this project is served on that path. Do not set a separate custom domain on this repo.


## Architecture

```
engine/src/          Rust simulation → hellscan.wasm (136KB)
  consts.rs            map / texture / entity / input / weapon constants
  events.rs            audio event bit flags
  hud.rs               HUD wire format (+ compile-time layout assert)
  types.rs             entity / effect types
  lib.rs               sim, raycaster, lighting, FFI exports
src/game/
  runtime.ts           WASM loader, fixed-60Hz loop, HUD decode, input
  hud-abi.ts           TS copy of the HUD layout (asserted at boot)
  blit.ts              WebGPU → WebGL2 → Canvas2D presenter
  audio.ts             file samples + synth fallback, pooled voices
src/components/game/ React shell: Menu, Pause, EndCard, HUD, touch, settings
```

The HUD struct is `#[repr(C)]` and read from WASM memory with a DataView. The layout is guarded three ways: a Rust compile-time assert, a boot-time `hs_hud_size` check, and unit tests on both sides.

## Develop

Prerequisites: Node 24, Rust stable with the `wasm32-unknown-unknown` target.

```sh
npm ci
npm run build:wasm   # compile engine → public/hellscan.wasm (+ source hash)
npm run dev          # serve on http://127.0.0.1:8080
```

`startup.sh` rebuilds the WASM automatically when `engine/src` is newer than the committed binary.

## Verify

```sh
npm run typecheck    # tsc --noEmit
npm run lint         # eslint
npm test             # script + TS unit tests
npm run check:wasm   # engine sources match the committed WASM
npm run build        # production build
cargo test --manifest-path engine/Cargo.toml
```

Push and pull request runs [`.github/workflows/ci.yml`](.github/workflows/ci.yml): typecheck, wasm sync, asset checks, the production build, the game unit tests, and `cargo test`. It uses Node 24.

## Deploy

The game is entirely in the browser: simulation, weapons, voices, music, settings, and the local leaderboard. A static host can serve that. It does not need the TanStack/Nitro server.

It is published at [idanbot.me/wasm-doom](https://idanbot.me/wasm-doom/). `npm run build:pages` writes `dist-pages/` with base `/wasm-doom/`. The user site already owns `idanbot.me`, so this project is served on that path. Do not set a separate custom domain on this repo.

[`.github/workflows/pages.yml`](.github/workflows/pages.yml) publishes that folder on every push to `main`. Action pins, verified 2026-09-25:

| Step | Pin |
| ---- | --- |
| `actions/checkout` | `v7.0.1` |
| `actions/setup-node` | `v7.0.0` (Node 24) |
| `actions/upload-pages-artifact` | `v5.0.0` (`include-hidden-files` so `.nojekyll` ships) |
| `actions/deploy-pages` | `v5.0.1` |
| `dtolnay/rust-toolchain` | `master` + `toolchain: stable` (only release tag is `v1`, 2022) |

## Project status

- Default render size is 1080p, with 1440p and 2160p available. 640×400 still holds 60 on the CPU raycaster; higher sizes rely on WebGPU or WebGL2
- WebGPU with automatic WebGL2/Canvas2D fallback; renderer switchable without restarting the sim
- 11 weapons, 13 enemy skins with seven animation states each, projectile/effect atlas, three boss sigils, wave system, local leaderboard
- 83 Rust tests plus TS and script tests. CI runs typecheck, wasm-sync, asset validation, the production build, and both test suites

Combat roles, attack windups, guided missiles and effect rendering are
documented in [the BLACKSITE combat notes](docs/blacksite-ifrit/COMBAT.md).
With the game running, `node scripts/combat-smoke.mjs` checks all seven guns,
reloads, low-ammo feedback, the boss HUD and strafe direction in a real browser.

The [voice and interface notes](docs/blacksite-ifrit/VOICES_AND_UI.md) cover
54 Cloudflare Aura-2 lines, positional enemy audio, subtitles, the generated
HUD artwork, and saved settings. Run `npm run check:voices` and
`npm run test:voices`; `node scripts/voices-smoke.mjs` verifies them in-game.
