# HELLSCAN

A Doom-style raycaster that runs entirely in the browser. The simulation is written in **Rust**, compiled to **WebAssembly**, and presented through **WebGPU** (with WebGL2 / Canvas2D fallbacks). Sweep the corridors of Site Nadir-7, fight through waves, and face the Vault Master.

## Play

| Input | Action |
|---|---|
| `W A S D` / arrows | Move + turn |
| Mouse (pointer lock) | Look |
| Click / `Space` / `Ctrl` | Fire |
| `1`–`5` | Weapons: MK23-S, M870K, VX-9, SHRIKE, RAVEN |
| `R` | Reload |
| `E` | Use doors / pickups |
| `Shift` | Sprint |
| `P` / `Esc` | Pause |

Touch controls appear automatically on coarse-pointer devices. Add `?qa=1` to the URL for autoplay QA mode.

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

Prerequisites: Node 22, Rust with the `wasm32-unknown-unknown` target.

```sh
npm install
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

## Project status

- 60fps at 640×400 on the CPU raycaster; 1280 holds 60 when WebGPU or WebGL2 fills the frame
- WebGPU with automatic WebGL2/Canvas2D fallback; renderer switchable without restarting the sim
- 5 delivered BLACKSITE weapon sets, 13 enemy skins with seven animation states each, projectile/effect atlas, wave system, local leaderboard
- 49 Rust tests plus TS and script integration tests; CI runs typecheck, wasm-sync, asset validation, and build

Combat roles, attack windups, incendiary grenades and effect rendering are
documented in [the BLACKSITE combat notes](docs/blacksite-ifrit/COMBAT.md).
With the game running, `node scripts/combat-smoke.mjs` checks all five guns,
reloads and strafe direction in a real browser.
