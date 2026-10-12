# BLACKSITE world materials and atmosphere

These effects are authored in code; they reuse the licensed sector textures and
boss palettes already shipped in `public/game`. There are no new image downloads
or texture slots, and the WASM/TypeScript ABI remains unchanged.

- `src/game/world-materials.ts`: deterministic world-space wet patches, grime,
  hazard paint, frost veins, bio residue and circuit etching. All 25 sectors use
  their own seed and palette, with eight surface families matching their themes.
- Wet floors sample the current sector's ceiling and nearby light grid, with a
  Fresnel reflection capped at 18%. Small world-space ripples distort the probe.
  Wall panels receive narrow highlights from local light. These are approximate
  material reflections; they do not reflect moving enemies or trace geometry.
- Eight bounded samples along the camera-to-surface ray scatter local light in
  dust and ceiling-slat beams. Rays end at visible geometry and reach at most
  24 cells. Scattering is capped at 0.14 per color channel.
- `engine/src/lighting.rs`: reactors, servers and power units cast sector-colored
  light, with two indirect passes that never duplicate direct illumination or
  propagate across sealed walls. Active boss phases pulse the authored palette.
- `WorldEffects` tracks boss phase transitions on fixed simulation steps. Each
  transition triggers one 1.4-second ring/palette pulse. Rendering and pause do
  not advance its clock; death and sector changes clear the effect.

WebGPU and WebGL2 run the same material and scattering formulas. Canvas2D retains
shared Rust lighting and reduced particle/phase overlays, without GPU surface
reflections or light shafts. Cosmetic paint and reflections are excluded from
objective and arena hazard tiles. Phase postprocessing excludes zero-depth
weapon pixels; React HUD elements are composited separately.

Regression coverage: `scripts/world-materials.browser.mjs` checks real shader
compilation, localized ceiling reflections, protected hazard marks, sector
variation, light scattering, a sealed wall and weapon-color preservation.
`world-effects.test.mjs` tests phase timing/reset behavior; Rust tests cover light
energy, wall occlusion, machinery illumination and render determinism. The new
browser checks run in the combat group of CI.
