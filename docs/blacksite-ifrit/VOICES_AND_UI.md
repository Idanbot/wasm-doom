# BLACKSITE voices and field interface

Nine speaking enemy profiles have six unique, prerecorded lines each (54
clips). Cloudflare Workers AI runs `@cf/deepgram/aura-2-en`; no inference,
credentials or API calls are needed during gameplay. The script and exact
text live in `art/blacksite-voices.json`. Source WAVs, request provenance and
the estimate are in `art/source_hd/voices/`; processed mono MP3s and timing
metadata ship in `public/game/voices/` (under 1 MB combined).

| Enemy | Voice | Delivery / treatment |
| --- | --- | --- |
| Rifleman | Orion | American tactical radio |
| Breacher | Hyperion | Australian enforcer / radio |
| Hazmat | Pandora | British containment officer / respirator |
| Heavy Gunner | Zeus | Deep American voice / lower register |
| Loader | Neptune | Industrial low robotic modulation |
| Marksman | Draco | Measured British baritone / comms |
| Hornet | Electra | Bright synthetic targeting announcements |
| Martyr Drone | Luna | Calm, unsettling synthetic warnings |
| Veyran | Saturn | Lowered commander voice with short echo |

Subjects, vat brutes, hounds and spitters never speak. Their local sounds
use biological or mechanical synthesis. Speech cues include sighting,
attack, pain and taunt. One spoken line plays at a time; per-enemy cooldowns
and line rotation prevent a crowd from talking continuously. A dead or
removed speaker is cut off. Pause, restart and wave changes clear audio.

`engine/src/voices.rs` exports a read-only 40-byte-per-enemy snapshot with
identity, animation, health, world position, projected head anchor, distance
and line of sight. Captions use the same camera projection as sprites and
only appear for living speakers in view with unblocked sight. The audio
listener follows the camera; HRTF panners follow each enemy, attenuate with
distance, and muffle sources behind walls. The spatial toggle centers audio
while retaining distance attenuation. Voice volume is separate from effects.

Settings include Audio, Display, Controls and Accessibility. Enemy subtitles
are on by default; their size, visibility, voice volume and spatial preference
persist locally. Voice-check buttons preview three voice styles. Look
sensitivity now persists. Performance diagnostics are optional. Generated
menu art and HUD frames are layered beneath real DOM text and controls;
provenance and exact image prompts are in `art/source_hd/ui/`. Oxanium and
Rajdhani are self-hosted with their OFL licenses in `public/fonts/`.

## Reproduction and validation

Generate once using `npm run assets:voices -- --account ACCOUNT_ID` with
`CF_API_KEY` in the ignored `.env`. `--dry-run` estimates the batch; `--limit`
bounds new requests. Matching WAV/request signatures are reused on reruns.
Changing processing parameters invalidates the signature. The successful
batch used 1,232 input characters: estimated $0.03696, or 3,361 neurons.
This is a pricing estimate, not an account-usage measurement.

Run `npm run check:voices`, `npm run test:voices`, Rust tests, typecheck and
build. `node scripts/voices-smoke.mjs` exercises actual decoded speech,
captions, listener rotation, pause cleanup, volume and saved settings, plus
mobile menus/HUD. It accepts a built-preview URL as its first argument.

Sources: [Cloudflare Aura-2](https://developers.cloudflare.com/workers-ai/models/aura-2-en/),
[Cloudflare pricing](https://developers.cloudflare.com/workers-ai/platform/pricing/),
[Deepgram voice catalog](https://developers.deepgram.com/docs/tts-models).
