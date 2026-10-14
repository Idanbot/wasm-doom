# Boss reward mechanics

Ten rewards were selected for overlapping instant-hit bursts, simple single
projectiles or low utility beside stronger boss drops. Their approved 5×5
viewmodels, pickup cases and ownership slots stay intact.

| Weapon (engine slot) | New identity | Practical use |
| --- | --- | --- |
| AR-6 ARCHIVE (11) | First hit opens a 2.5-second dossier; the next hit adds 54 damage and ruptures nearby enemies. | Reward deliberate follow-up on an armored priority target. |
| OR-7 PREDICTOR (12) | A 52-damage precision strike schedules a 46-damage pulse 0.18 seconds ahead, using observed target velocity. | Lead moving enemies; the second strike stays at its committed location. |
| GS-4 SINK (13) | Nine gravity pellets pull visible enemies into the firing lane; banks pressure. | Group a crowd, then release a powerful, inaccurate piercing scatter. |
| CR-3 RIME (14) | Three slowing coolant slugs prime frost. A different volley within three seconds shatters it for bonus damage and nearby slowing damage. | Alternate crowd control and shatter volleys; slugs from one volley cannot proc each other. |
| SR-0 RELAY (15) | A 34-damage arc jumps to three distinct nearby enemies for 26/20/16 damage, draining shields and interrupting. | Clear clustered guards; walls block every link. |
| KS-8 KEST (17) | Three 24-damage rounds fire at 0/0.085/0.17 seconds, following current aim. Each bullet spends a round. | Sweep a burst across targets; switching, reloading, death or an empty magazine cancels unfired rounds. |
| LV-8 UNDERTOW (20) | A 94-damage harpoon reels the victim toward you and splashes nearby enemies; banks pressure. | Pull an exposed threat into range, or spend pressure on a three-harpoon fan. |
| MG-4 POLARITY (22) | A magnetic beam pierces three targets with damage falloff, strips extra armor and pulls victims toward its firing line. | Align crowds for a follow-up lane shot. |
| SL-12 HELIOS (28) | Solar projectiles burst into an enemy-only area blast followed by three short pulses; banks pressure. | Deny a small area, or discharge a larger, slower and less accurate solar core. Solar damage uses the burn role. |
| CH-8 EPOCH (30) | A precision hit leaves a world-space mark with two area echoes at 0.35/0.7 seconds. | Predict where enemies will stay; echoes keep their control role after switching guns. |

## Pressure controls

GS-4 adds 22%, LV-8 26% and SL-12 20% pressure per successful primary shot.
Pressure belongs to each gun and caps at 100%. At the cap, primary fire waits
for a manual vent. A gauge above the ammunition card shows the bank and the
available action on desktop and touch layouts.

- Press **R** to vent over 0.9 seconds. Pressure falls continuously. Venting has
  a valve sound, a lowered aim pose, and no muzzle flash, smoke, feed ejection
  or ammunition transfer.
- Wait for cooling to finish and the ordinary magazine reload begins, provided
  ammunition is needed and reserve rounds remain. A full or exhausted weapon
  can still vent safely.
- A **fresh fire press** during venting releases the remaining bank if at least
  20% pressure and one magazine round remain. Holding fire while pressing R
  does not discharge accidentally.
- A discharge spends one round, empties the pressure bank, cancels the reload
  and incurs 0.95 seconds of recovery. More bank increases payload damage;
  wider spread makes discharge less reliable at distance.
- Switching cancels handling without moving pressure into another gun.
  Sector changes and checkpoint restoration reset transient pressure and
  pending attacks. Checkpoints preserve saved ammunition for every gun; only
  entering the next sector refills owned weapons. The checkpoint format is unchanged.

## Simulation and presentation contracts

`engine/src/boss_weapons.rs` owns pressure, contacts, delayed damage and bounded
attack scheduling. Delayed effects retain their originating shot and weapon
role for accuracy, kills and elemental combinations. Secondary area damage
does not recursively trigger weapon procs. Pull effects have 78% boss
resistance and remain subject to normal collision resolution.

Pending attacks cap at 32. Scatter tracers cap at three dots per lane so a
discharge leaves entity headroom for active enemies and projectiles. Area
effects and arc links check line of sight. Player projectiles start at the
player position and travel through collision checks rather than appearing on
the far side of nearby walls.

The HUD ABI is now **256 bytes**, appending floats `pressure` at 244, `venting`
at 248 and `pressure_release` at 252. Rust, TypeScript, the WASM runner and
ABI tests share the updated contract.

The existing 25 weapon-specific projectile textures supply the correct color
and silhouette for every new attack. Charged projectiles scale with their
bank; rings and arcs arrange those textures in world space. No replacement
raster assets or changes to approved viewmodel pixels are required. Texture
indices, transparent safe areas, source credits and the 5×5 animation contract
are unchanged. Magazine reloads continue to use cells 10–14; pressure venting
uses the gun's current ammunition aim pose. Cells 20–24 remain reserved.

## Verification

`npm run test:weapons` runs the focused Rust and HUD/audio contract tests.
The full Rust suite covers fixed-seed replay, pressure accounting, fresh-trigger
release, empty ammunition, cooldowns, damage timing, cover, pulls, chain arcs,
frost volleys, armor, cancellation and checkpoint/sector transitions.
`scripts/boss-weapons.browser.mjs` exercises the compiled WASM controls;
`scripts/hud-insets.browser.mjs` checks gauge alignment and text containment at
four desktop/mobile sizes. The combat CI group includes the new WASM test.
