# Mouse look and camera

Mouse displacement applies once per display frame through `hs_look(mx, my)`.
It updates both rendered aim and combat yaw immediately, without acceleration or
filtering. Sensitivity still scales the raw deltas. The legacy `hs_input` mouse
arguments remain supported for headless simulation clients.

Physics stays deterministic at 60 Hz. Before rendering, `hs_camera(alpha)` returns
a pointer to two `f32` values (world x, y), interpolated between the previous and
current physics positions using the remaining frame fraction. This adds at most
one physics step of positional delay; mouse look has no interpolation delay.
World rendering, enemy overlays and projected effects share this position.
Teleports, sector changes, checkpoint restores and paused redraws snap to the
current position.

Vertical look is a raycaster horizon shift, limited symmetrically to ±0.85
(previously ±0.42); these values are not angles in radians. Both renderers use
`height * (0.5 + pitch * 0.9)` for the horizon. Weapon bob remains independent.

`scripts/aim.browser.mjs` exercises the real WASM/runtime bridge at 30, 60, 120,
144 and 240 Hz, including sub-tick mouse input, walking interpolation, pause and
firing without losing mouse capture. Rust camera tests cover range, invalid
inputs, simulation isolation, overlays and transition resets.
