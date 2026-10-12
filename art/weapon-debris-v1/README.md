# Reload debris

Native-alpha OpenAI imagegen artwork, referenced against approved HX-8, CM-9,
MY-9 and LV-8 aim poses. Four tumbling views per container, packed as 1024×1024
2×2 sheets with 512×512 cells and at least 56 pixels of padding. Sources and
SHA-256 hashes are recorded in manifest.json. Repack with
`python3 scripts/pack-weapon-debris.py`.

Prompt: realistic isolated empty olive fuel tanks, surgical-steel capped acid
vials, green-tinted specimen cartridges and blue/brass pressure cylinders;
four orientations each, worn tactical materials, native transparency, no
weapons, hands, labels or external shadows. Glass keeps residue after use.

Reload emission: slots 7/10/24/20 respectively, once on feed removal. Debris
uses existing gravity, floor bounces, wall collision, impact audio and six-second
cleanup; all cases and containers share the 24-object budget. No gameplay hazard
or collectible is created. Approved weapon aim artwork remains unchanged.
