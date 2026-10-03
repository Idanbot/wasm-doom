# Sound effect replacement shortlist — issue #1

Research date: 2026-10-03. Approved by the user and implemented: 103 CC0 effects, covering all 33 weapons and all 37 enemy skins. Exact source records, credits, recipes and export hashes are in [art/audio/README.md](../art/audio/README.md) and `art/audio/manifest.json`. This document preserves the approved design mapping; acceptance evidence is recorded below.

[Issue #1](https://github.com/Idanbot/wasm-doom/issues/1) explicitly says: “Review and choose the candidates before downloading or wiring them.” Keep the issue open until implementation and acceptance checks pass. The issue mentions 18 weapons; current coverage must include all 33 weapons and all 37 enemy skins.

## Audit

`src/game/audio.ts` loads five gunshot samples for 33 weapons. Several standard guns and all later boss guns use generic synthesized fallbacks. Reloading uses a shared one-shot rather than animation steps. Enemy death uses one generic sample pair; footsteps and several interactions are synthesized. UI clicks lack a dedicated event mapping. Existing combat dialogue, boss death voices and MP3 music stay; speech continues through the shared `VoiceGate`.

## Approved source packs

All recommended packs are listed as **CC0** on their linked source pages. [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) permits copying, modification and distribution, including commercial use, without mandatory attribution. Preserve downloaded license files and source URLs, and voluntarily credit all creators. Recheck the actual archive license before selecting clips. qubodup’s human pack specifically states that it became CC0 on 2024-08-30.

| ID | Pack / source | Creator | Proposed use |
| --- | --- | --- | --- |
| `firearms` | [Free Firearm Sound Library](https://opengameart.org/content/the-free-firearm-sound-library) | Ben Jaszczak, Brian Nelson, Kevin Heras, Matthew Nanney | Real recorded pistol, shotgun, carbine, rifle and automatic fire; distinct ballistic foundations. |
| `handling` | [Gun reload sounds](https://opengameart.org/content/gun-reload-sounds) | SpringySpringo | Airsoft pistol/rifle mechanisms and shotgun cocking; edit into magazine out/in, bolt and pump steps. |
| `interface` | [Interface Sounds](https://kenney.nl/assets/interface-sounds) | Kenney | Buttons, confirmation, rejection, menu transitions and pickup confirmation. |
| `impacts` | [Impact Sounds](https://kenney.nl/assets/impact-sounds) | Kenney | Metal, body/armor contact, debris, destructible machinery and death thuds. |
| `footsteps` | [RPG Audio](https://kenney.nl/assets/rpg-audio) | Kenney | Footstep and equipment foley; select grounded boot impacts. |
| `mechanical-fx` | [Sci-fi Sounds](https://kenney.nl/assets/sci-fi-sounds) | Kenney | Launchers, energy weapons, explosions, motor/servo layers and boss attacks. |
| `human` | [15 vocal male strain/hurt/pain/jump sounds](https://opengameart.org/content/15-vocal-male-strainhurtpainjump-sounds) | qubodup | Short nonverbal human injury and strain; no dialogue samples. |
| `creature` | [80 CC0 creature SFX](https://opengameart.org/content/80-cc0-creature-sfx) | rubberduck | Mutant pain/death, alien breaths, roars, spit and wet biological layers. |
| `terminals` | [50 CC0 Sci-Fi SFX](https://opengameart.org/content/50-cc0-sci-fi-sfx) | rubberduck | Terminal cues, rocket and teleport layers; avoid retro/gamey presets. |

Sonniss GDC bundles were investigated but are not in the recommended set: the current [bundle license](https://sonniss.com/gdc-bundle-license/) restricts supplying the recordings as sound-effect files. Prefer CC0 sources for checked-in audio in this public repository. Michel Baradari’s monster pack is CC BY 3.0, not CC0, so it is also outside this proposed CC0-only set.

## Implemented gunshot design

Each weapon gets its own curated sample or authored multi-source mix, rather than assigning the same sound to every boss gun. Exact source filenames, edit points and layer recipes are recorded in `art/audio/manifest.json`.

| Slot | Weapon | Intended signature / source family |
| --- | --- | --- |
| 1 | MK23-S | short suppressed pistol crack; firearms + handling |
| 2 | BR-12 BREAKER | heavy shotgun report; firearms + pump mechanism |
| 3 | KX-9 VECTOR | tight PDW transient; firearms |
| 4 | MR-4 LONGBOW | precise heavy rifle crack with short magnetic tail; firearms + mechanical-fx |
| 5 | VLK-6 WARDEN | mechanical launch thump and short rocket ignition; mechanical-fx + impacts |
| 6 | AX-12 VOLT | sharp electrical discharge; mechanical-fx + terminals |
| 7 | M91 CYCLONE | heavy automatic cannon pulse; firearms + impacts |
| 8 | HX-8 PYRE | short pressurized ignition, restrained flame tail; mechanical-fx |
| 9 | VR-9 OVERRIDE | rail transient with metallic resonance; mechanical-fx + firearms |
| 10 | HC-9 FORGE | industrial cutter pulse and servo; mechanical-fx + impacts |
| 11 | CM-9 CHIMERA | wet acid discharge; creature + mechanical-fx |
| 12 | AR-6 ARCHIVE | dry amber rail pulse; mechanical-fx + impacts |
| 13 | OR-7 PREDICTOR | two precise predictive pulses; mechanical-fx + terminals |
| 14 | GS-4 SINK | dense reactor pressure blast; mechanical-fx + impacts |
| 15 | CR-3 RIME | cold slug launch with glassy detail; mechanical-fx + impacts |
| 16 | SR-0 RELAY | rapid signal-carbine crack; firearms + terminals |
| 17 | TS-12 TITAN | heavy siege launch thump; mechanical-fx + impacts |
| 18 | KS-8 KEST | controlled tactical rifle burst; firearms + handling |
| 19 | MN-6 ECHO | memory lance with short echo; mechanical-fx + terminals |
| 20 | VX-2 CINDER | furnace kick with ember tail; mechanical-fx + impacts |
| 21 | LV-8 UNDERTOW | compressed hydraulic harpoon launch; mechanical-fx + handling |
| 22 | PR-5 SPECTRUM | bright refracted pulse; mechanical-fx + terminals |
| 23 | MG-4 POLARITY | deep magnetic discharge; mechanical-fx + impacts |
| 24 | BM-3 SWARM | insect drone-launch chirr; creature + mechanical-fx |
| 25 | MY-9 SPORE | wet spore burst; creature |
| 26 | AS-7 STARFALL | missile ignition and restrained launch report; mechanical-fx |
| 27 | SC-6 SHREDDER | sharp flechette burst; firearms + impacts |
| 28 | UB-1 VEIL | soft phase snap and relocation wash; terminals + mechanical-fx |
| 29 | SL-12 HELIOS | weighty solar pulse; mechanical-fx + impacts |
| 30 | NT-4 TORPEDO | naval pressure launch and ignition; mechanical-fx |
| 31 | CH-8 EPOCH | staggered echo pulse; terminals + mechanical-fx |
| 32 | BO-11 GLACIER | cryo launch with brittle shard detail; impacts + mechanical-fx |
| 33 | SV-25 DOMINION | layered command cannon salvo; firearms + mechanical-fx + impacts |

## Event mapping and integration

| Event | Sources / behavior |
| --- | --- |
| Magazine reload | Handling: mag out, mag in, charging handle/chamber; sync to weapon frame transitions and cancel pending stages on interruption. |
| BR-12 reload | Handling: one shell insert per actual ammo increment, ending with the appropriate pump/chamber cue. |
| Energy/boss reload | Per-weapon cell removal/insertion and chamber/charge accents from handling, impacts and mechanical-fx. |
| Empty trigger / weapon pickup | Dry mechanical click / case latch and handling; no dry-fire visual animation. |
| Enemy pain / death | Humans: short nonverbal human takes; mutants: creature; robots: metal/servo shutdown. Detect entity state changes with enemy identity and position, not the aggregate kill bit alone. |
| Player injury / death | Restrained nonverbal human takes plus body/armor foley; short tails. |
| Footsteps | Boot foley varied by sector surface; randomized take selection, small gain variation. |
| Doors | Servo/motor and metal latch; no spoken lines. |
| Console / panel / terminal | Distinct short confirmation/rejection cues from interface and terminals; preserve once-per-interaction speech. |
| UI buttons / menus | Consistent click, confirm/back and transition set; respects master mute and appropriate volume routing. |
| Health/ammo/armor and reward pickups | Separate concise pickup cues; reward-case latch and a short confirmation accent. |
| Impacts / destructibles | Material-appropriate metal, flesh, debris and mechanical breakup; do not inflate enemy kill sounds for scenery. |
| Explosions / projectiles | Separate launch, travel and impact/detonation; spatial world effects with distance rolloff and limited concurrent tails. |
| Boss attacks / hazards | Bind distinct electrical, magnetic, pressure, corrosive, cryo, ember and phase cues to actual attack/state transitions; stop loops on pause, death, sector changes and disposal. |

Preload and decode selected audio during loading. Keep browser assets local. Archive original licenses, a creator/source manifest, exact source-file hashes and edit recipes. Retain only selected sources/runtime clips, not entire unused packs. Trim leading silence, use short fades, preserve attack transients, measure loudness and true peak, leave mix headroom, and audition every mechanical effect for unintended speech. Exports use mono 44.1 kHz Vorbis with MP3 fallback.

Route SFX through the existing SFX/master buses. Keep spoken lines on the shared voice gate; nonverbal vocalizations must respect active dialogue. Limit repetitions and simultaneous world effects; use per-entity state transitions and bounded per-event cooldowns. Preserve existing music and the enemy-subtitle settings.

## Acceptance checks before issue closure

- Exact source/license/credit/hash records in the repository for every selected clip and derived mix.
- All 33 weapons have intentional firing profiles; reload stages follow actual frames/ammo changes and cancel on interruption.
- Every UI, interaction, pickup, impact, enemy pain/death, boss/hazard event has a verified trigger.
- Real browser playback/decoded waveforms for desktop and mobile; master mute and SFX volume controls; first-gesture audio unlock.
- Missing-asset behavior is reported and safely handled without silent failures or repeated fetches.
- No overlapping spoken lines; vocal injury/death clips do not mask dialogue; doors contain no speech.
- Build/typecheck, targeted audio/state tests, browser smoke and production-path checks pass before closing issue #1 with implementation evidence.

## Implementation and validation

Delivered 103 effects in 206 exports (Vorbis plus MP3), with 33 unique firing mixes and three injury/death takes for each human, creature and robot family. There are eight boss/hazard signatures and four moving projectile/fire-patch loops. All selected recordings come from the nine approved CC0 packs; 123 original files, license evidence, credits and exact processing recipes are retained in `art/audio/`. The 26 superseded runtime effects were removed and the local catalog now indexes playable replacements.

Magazine and cell reloads follow frame transitions; BR-12 shell inserts follow ammo increments. Door opening, lockdown closure, impacts, scenery breakup and pickups use the bounded positional Rust event queue. Mechanical empty-trigger feedback preserves the existing empty weapon pose. Speech and nonverbal vocal recordings share one voice gate, while mechanical effects duck beneath active voices. Pause, mute, sector changes and disposal stop world loops and reload sounds.

Verified: 113 Rust tests; 227 script tests and 96 TypeScript tests; all four audio browser tests; production Pages preload and MP3 fallback checks; typecheck and both builds. Desktop/mobile checks decode all 206 exports and measure audible output, volume scaling, silence at zero, bounded loops and vocal exclusivity. Missing Vorbis falls back once to MP3; missing both produces a named loading error. Every Vorbis export was measured with FFmpeg `ebur128=peak=true`, with a maximum true peak of −5 dBFS. Desktop/mobile menu screenshots were inspected with clean browser consoles. CI includes the source/hash, state, real-playback and production-path checks.

Waveform and browser checks establish playback and signal quality; human listening preferences can be refined using the development catalog without changing the licensing or event wiring.
