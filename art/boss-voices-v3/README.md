# Clean boss voice rework

Cloudflare Workers AI `@cf/deepgram/aura-2-en` generated 202 fresh clips: 152 combat clips across all 25 bosses and 50 death performances. Each boss retains one speaker across combat and death. Natural performance relies on speaker selection and concise wording; Aura-2 has no emotion/style prompt field.

`plan.json` contains all exact text and speaker requests. `raw/` preserves lossless 24 kHz WAV source audio plus resumable request signature sidecars. Published files are mono MP3 at 192 kbps, normalized to -16 LUFS with a 65 Hz high-pass, mild compression and a 12 ms fade-in. No pitch shift, ring modulation or echo. Boss playback preserves the source pace; spatial occlusion is gentler for intelligibility. The shared voice gate and once-per-death rules remain intact.

`usage.json` records 202 completed clips and 6,375 text characters. The estimated list-price cost was $0.19125, not an account billing measurement. No credentials are included. Regenerate with `node scripts/generate-boss-voices-v3.mjs --account <account-id>` and an ignored `.env` containing `CF_API_KEY`. Cached requests are reused unless their model, speaker or text changes.

Subtitles: `src/game/boss-voice-lines.json`; combat manifest: `public/game/voices/manifest.json`; death plan: `art/boss-voices.json`.
