# Boss voice recommendation (4 October 2026)

The current generator already uses Cloudflare `@cf/deepgram/aura-2-en`.
Its `style` text is metadata: the API receives only dialogue and speaker, not
those acting directions. Several post-processing chains change sample rate,
add ring modulation or echoes, and narrow the frequency range. This is a
plausible contributor to the robotic result; no listening comparison was run.

For an inexpensive Cloudflare-only pass, retain Aura-2, audition several of its
40 English speakers on the same short lines, and compare clean recordings
before adding subtle radio EQ. Use one stable speaker per character, natural
punctuation and shorter lines. Reserve modulation for explicitly robotic
characters. Cloudflare's documented parameters offer no emotion/style prompt.
MeloTTS is another option but is not my recommended upgrade for dramatic acting.

For a quality upgrade, audition ElevenLabs `eleven_v4` using Text to Dialogue.
Official documentation recommends it for game character performances and
emotional dialogue; `eleven_v3` is another expressive option with audio tags.
Keep a stable voice profile, generate several takes and select by listening.
Paid plans include a commercial license for non-beta services; free generations
lack commercial rights and have publication attribution requirements. Generate
final release recordings under the appropriate plan. A human voice actor is
still the strongest option for a few important bosses.

An offline alternative is the open-source Chatterbox family: Turbo supports
English speech with performance tags; Nano runs on CPU. Both need local compute
and a short reference voice recording that you have permission to use. Audition
them before choosing; no listening comparison has been performed here.

Sources:
- https://developers.cloudflare.com/workers-ai/models/aura-2-en/
- https://developers.cloudflare.com/workers-ai/models/?tasks=Text-to-Speech
- https://github.com/resemble-ai/chatterbox
- https://elevenlabs.io/docs/overview/models
- https://elevenlabs.io/docs/overview/capabilities/text-to-speech/best-practices
- https://help.elevenlabs.io/hc/en-us/articles/13313564601361-Can-I-publish-the-content-I-generate-on-the-platform

Suggested next art/gameplay work: directional enemy views, eight-frame locomotion,
boss armor damage states, one unique mechanic per sector enemy, and attack
telegraphs aligned to animation contact frames.
