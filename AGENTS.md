# BLACKSITE

BLACKSITE is a standalone browser FPS. The Rust engine compiles to WebAssembly;
React supplies the HUD, menus and local development asset catalog. No builder
platform, account service, connector gateway or runtime database is required.

## Project structure

- `engine/src/`: simulation, combat, sector layouts, rendering and WASM ABI.
- `src/game/`: runtime, graphics, input, audio and asset loading.
- `src/components/game/`: menus, HUD, settings and touch controls.
- `public/game/`: published art, music, sound effects and voices.
- `art/`: authored source assets, prompts, generation records and licenses.
- `.agents/skills/weapon-rework/SKILL.md`: weapon animation pipeline.

## Development

Use Node 24 and Rust stable. Read `package.json` before changing scripts.
Run `npm run dev` on `0.0.0.0:8080`; `startup.sh` starts it idempotently.
Keep secrets in ignored `.env` files or process variables. Never expose
Cloudflare credentials in browser assets or logs.

Build engine changes with `npm run build:wasm`; commit `public/blacksite.wasm`
and its synchronized `public/blacksite.sha256`. Keep Rust/TypeScript ABI and
texture indices synchronized. Do not change approved weapon aim poses while
editing their other animation frames. Keep catalog and local QA shortcuts
restricted to the local development server.

## Verification

Run appropriate tests, `npm run typecheck`, `npm run check:wasm`, `npm run build`
and `npm run build:pages`. Verify desktop and mobile rendering with
`scripts/browser-smoke.mjs`; inspect both screenshots and fix browser errors.
Use the existing Playwright checks for input, combat, preloading and rewards.
Test the built Pages output too. Store local QA output in ignored `screenshots/`
and temporary server state in ignored `.blacksite/`. Preserve art/audio licenses
and credits. Update README and asset documentation when contracts change.

## Deployment

GitHub Pages publishes static output from `dist-pages/`; its base path is
configurable using `PAGES_BASE`. The TanStack/Vercel build is also supported.
Settings, scores and checkpoints are browser-local. Do not add authentication,
a database or external services unless the user explicitly requests them.
