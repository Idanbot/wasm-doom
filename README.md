# BLACKSITE

A standalone browser FPS set in Site Nadir-7. Disable each sector's defenses,
defeat its boss, and turn the recovered weapon against the next one.

**[Play BLACKSITE](https://idanbot.me/wasm-doom/)**

- **25 sectors and bosses**, each with a distinct layout, enemy cast, machinery,
  attack patterns and reward. Beyond sector 25, the campaign repeats with rising
  enemy counts and health.
- **33 weapons** with magazine, shell and charge reloads. Boss rewards add marks,
  chain arcs, frost shatters, timed bursts and pressure banking.
- **Four difficulties:** Recruit, Operative, Veteran and Blacksite. Each has a
  separate local leaderboard with expandable combat totals, sector splits and routes.
- Spatial voices, enemy subtitles, material-based sound effects, sector lighting
  and atmospheric effects. Keyboard/mouse and touch controls are supported.

The Rust/WASM simulation runs entirely in the browser, with React 19 and
TypeScript for the interface. Rendering uses WebGPU, WebGL2 or Canvas2D fallback.
Settings, records and checkpoints stay on the device; no accounts or backend.

## Play

| Action | Control |
| --- | --- |
| Move / sprint | `WASD` / `Shift` |
| Aim / fire | Mouse / left click |
| Equip weapon | Mouse wheel or number keys |
| Reload / interact | `R` / `E` |
| Pause / resume | `Esc` or `P` |

Choose a difficulty after loading completes; it stays fixed for the run.
Mouse look responds every display frame, with matching horizontal/vertical
sensitivity, wider vertical look and interpolated camera movement.
Sensitivity remains adjustable in Settings.
Settings → Controls supports alternate keyboard/mouse bindings for every action
and weapon. Accessibility has separate flash, shake and cosmetic debris sliders.
For GS-4 SINK, LV-8 UNDERTOW and SL-12 HELIOS, **R** vents stored pressure;
firing during the vent spends a round on a heavy, inaccurate discharge.

## Develop

Requires **Node 24**, **Rust stable**, and the `wasm32-unknown-unknown` target
(installed by the WASM build script).

```sh
npm install
npm run build:wasm
npm run dev                 # http://localhost:8080, bound to 0.0.0.0
```

`startup.sh` starts the local server idempotently. After engine edits, rebuild
and commit `public/blacksite.wasm` with its synchronized `public/blacksite.sha256`.
Keep credentials in ignored `.env` files or process variables.

| Path | Contents |
| --- | --- |
| `engine/src/` | Deterministic simulation, combat, maps, rendering and WASM ABI |
| `src/game/` | Runtime, graphics, input, audio and asset loading |
| `src/components/game/` | Menus, HUD, settings and touch controls |
| `public/game/` | Published textures, sprites, music, effects and voices |
| `art/` | Source assets, prompts, generation records and licenses |
| `scripts/` | Asset checks, benchmarks and browser tests |

### Local tools

Use `?qa=1` for the full arsenal and QA helpers, or `?qa=1&lvl=N` to start in
sector N. Open the asset catalog from the main menu or `/catalog`. These tools
are restricted to the local development server and excluded from production.

**Settings → Display → Reload all assets** repairs missing or damaged visual
assets. Loading validates content hashes, caches valid files and retries failures
up to twice before offering **Retry loading**.

## Verify

```sh
npm run typecheck
npm run lint
npm test
cargo test --release --manifest-path engine/Cargo.toml -- --test-threads=2
npm run check:wasm
npm run check:assets
npm run check:sprites
npm run build
npm run build:pages
```

With the dev server running:

```sh
node scripts/browser-smoke.mjs http://127.0.0.1:8080/ screenshots/menu.png
node scripts/combat-smoke.mjs http://127.0.0.1:8080/
npm run test:browser
```

CI runs source, asset and engine checks in parallel, then tests production builds
and browser behavior. Only a successful `main` run deploys the tested Pages build.
See [CI and deployment](docs/CI.md) for jobs, artifacts and diagnostics.

## Deploy and reference

GitHub Pages serves `dist-pages/`; set `PAGES_BASE` to the deployment path
(`/wasm-doom/` for this repository). The standard `npm run build` also produces
the TanStack/Vercel output.

- Gameplay: [boss counterplay](docs/BOSS_COUNTERPLAY.md),
  [movement](docs/BOSS_MOVEMENT.md), [weapon mechanics](docs/BOSS_WEAPON_MECHANICS.md).
- Art: [sprite safe areas](docs/SPRITE_SAFE_AREAS.md),
  [weapon animations](docs/BOSS_WEAPON_ANIMATIONS.md),
  [materials](art/world-materials-v1/README.md), [audio credits](art/audio/README.md).
- [Improvement suggestions and implementation status](docs/GAME_IMPROVEMENT_SUGGESTIONS.md).
