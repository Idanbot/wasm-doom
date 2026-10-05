# Blacksite sound effects

130 short, mono recordings and authored mixes replace the old shared/generated effects. All nine approved source packs use **CC0 1.0 Universal**. Original creator credits and source-page license records live in `licenses/<pack>/`; selected unmodified recordings live in `sources/`. `manifest.json` records each original filename/SHA-256 and every trim, pitch, filter, layer, delay, normalization target and export hash.

This includes 25 distinct sector ambience accents and two casing landing mixes. Map accents are finite quiet machinery, coolant, phase, biological and industrial textures, played only during active simulation every 6–11 seconds. Casings play a spatial landing cue on actual floor bounces. Both use the existing mixer, voice ducking and loading gate. The [world effects guide](../world-fx-v1/README.md) maps every sector to its visual and sound signature. To rebuild selected clips without touching other exports, pass IDs to `python3 scripts/process-blacksite-sfx.py sector-1 casing-brass`.

Rebuild with `npm run assets:sfx` (Python 3 and FFmpeg with libvorbis/libmp3lame). The script trims leading silence, retains attack transients, applies short boundary fades and normalizes each mix to −6 dBFS or lower. It exports 44.1 kHz mono Vorbis and MP3. `true-peaks.json` records SHA-bound FFmpeg true-peak measurements of every Vorbis export; the current maximum is −5 dBFS. Runtime clips are in `public/game/sfx/v2/`; its manifest intentionally omits source recordings and recipes from the loading path. Only selected originals are retained, rather than the complete downloaded packs.

## Credits

- Ben Jaszczak, Brian Nelson, Kevin Heras and Matthew Nanney — [Free Firearm Sound Library](https://opengameart.org/content/the-free-firearm-sound-library).
- SpringySpringo — [Gun reload sounds](https://opengameart.org/content/gun-reload-sounds).
- Kenney — [Interface Sounds](https://kenney.nl/assets/interface-sounds), [Impact Sounds](https://kenney.nl/assets/impact-sounds), [RPG Audio](https://kenney.nl/assets/rpg-audio), [Sci-fi Sounds](https://kenney.nl/assets/sci-fi-sounds).
- qubodup — [15 vocal male strain/hurt/pain/jump sounds](https://opengameart.org/content/15-vocal-male-strainhurtpainjump-sounds). The author changed these recordings to CC0 on 2024-08-30.
- rubberduck — [80 CC0 creature SFX](https://opengameart.org/content/80-cc0-creature-sfx), [50 CC0 Sci-Fi SFX](https://opengameart.org/content/50-cc0-sci-fi-sfx).

[CC0 1.0 legal text](https://creativecommons.org/publicdomain/zero/1.0/legalcode.en) permits redistribution and modification, including commercial use. Credits are voluntary. Existing music and generated dialogue retain their existing provenance and are outside this CC0 effects set.

## Runtime behavior

`SfxDirector` maps all 33 weapons and 37 enemy skins. Reload cues follow fixed-step animation frames; BR-12 inserts are driven by actual shell increments. No future reload stages are scheduled with timers. Firing or switching weapons stops active reload sounds. Material impacts, scenery destruction, distinct pickups and doors originate in the Rust world event queue, preserving positions across multiple simulation steps per rendered frame. Projectile and fire-patch loops follow live entity positions and stop when the entity disappears, on pause, on sector transitions, on mute and on disposal.

`SfxPlayer` preloads/decodes with six workers during loading and falls back to MP3 if Vorbis fetch/decode fails. Missing both formats blocks readiness with a named asset error, rather than silently substituting a beep. Sources use the existing SFX/master gain buses, a master compressor, spatial distance rolloff and occlusion filtering where enemy visibility is available. Playback is bounded to 24 sources, eight enemy effects, four hazard bursts and six travel loops. Recorded nonverbal human/creature vocals use the same `VoiceGate` as dialogue, preventing overlapping voices. Mechanical effects are ducked during voice playback.

The local development asset catalog contains playable previews of all 130 effects. Production keeps local asset URLs and the deployment base path. No remote audio services are used during play.

## Verification

`npm run check:sfx` verifies source/export hashes, CC0 records, event coverage, unique weapon mixes and the primary-download budget. `src/game/sfx-director.test.ts` covers state transitions, reload interruption, shell insertion, entity identity and boss phases. Rust tests cover the bounded positional ABI, material impacts, door repetition, pickups and scenery destruction. `scripts/sfx.browser.mjs` decodes both formats, measures actual Web Audio output on desktop/mobile, checks gain/mute, shared vocal exclusivity, loop limits, missing-file fallback/failure and real game firing/reload/pause events.

Waveform checks verify format, signal energy, peaks and loading behavior; they do not substitute for a human aesthetic listening review. These are selected CC0 recordings and detailed authored mixes, ready for further listening adjustments in the catalog.
