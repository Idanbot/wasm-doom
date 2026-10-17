# Controls and effects

Settings → Controls supports up to four keyboard/mouse bindings per action,
including all 33 direct weapon selections. Select a binding to replace it, use
**+ key** to add an alternate, or **×** to clear it. Conflicts identify the
existing action. Escape cancels capture and always pauses/resumes gameplay;
browser/OS keys are reserved. The mouse wheel continues to cycle weapons.
Prompts, reload reminders and pressure hints use the current binding.

Settings → Accessibility provides independent 0–100% controls:

- **Flash intensity:** muzzle lighting, viewmodel brightness, damage tint and
  decorative boss pulses. At zero, the existing ammo aim pose replaces firing
  cells with baked-in flashes; reload and pickup sequences retain their frames.
- **Shake intensity:** procedural weapon bob/kick, muzzle heat shimmer and
  explosion distortion. Mouse aim and simulation coordinates stay independent.
- **Debris amount:** a stable subset of cosmetic gibs, casings and removed feeds,
  including their contact shadows, plus ambient motes. Projectiles, impact cues,
  hazards and attack markers remain visible.

Bindings are versioned in browser-local `blacksite-controls`; effects extend
`blacksite-gfx`. Missing effect fields default to 100% for existing saves.
**Restore defaults** restores bindings and the other settings together.

The new WASM export `hs_effects(flash, shake, debris)` clamps each float to 0–1,
using 1 for non-finite values. Preferences survive restart, sector changes,
checkpoint restoration and renderer recovery. They are not part of the run-save
or HUD layouts. Filtering affects CPU/GPU presentation; the simulation still
generates identical effects, random values, sound events and combat outcomes.

Rust accessibility tests cover validation, immediate feedback changes,
deterministic combat, essential-sprite visibility, stable debris selection,
CPU drawing and transition persistence. `scripts/accessibility.browser.mjs`
covers input translation, conflict/cancel behavior, local persistence, restored
defaults and real movement/fire/reload/pause/weapon controls.
