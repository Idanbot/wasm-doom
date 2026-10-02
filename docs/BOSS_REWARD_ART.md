# Golden boss reward artwork

All 11 boss guns have newly generated golden hard cases and full side-profile arsenal thumbnails. The first eight standard gun assets are unchanged. Cases show each matching gun in fitted charcoal foam, inside an open gold case. Wheel thumbnails show the complete gun from the side, muzzle left, without hands or a case.

Weapon IDs: `vr9`, `hc9`, `cm9`, `ar6`, `or7`, `gs4`, `cr3`, `sr0`, `ts12`, `ks8`, `mn6`.

| Asset | Path | Published dimensions |
| --- | --- | --- |
| Original generated artwork | `art/boss-reward-art-v3/{id}-case.png`, `{id}-side.png` | Original generation resolution |
| Case catalog artwork | `public/game/draft/v2/cases/case_{id}.png` | 1024 × 768 |
| World case pickup | `public/game/spr_gun_{id}.png` | 256 × 256 |
| Arsenal side profile | `public/game/ui/weapon-thumbs/{id}.png` | 640 × 300 |

All published images are PNG with native transparency. The processor crops, scales and packs artwork; it does not remove magenta or recolor the gun. It clears almost invisible alpha values below 4, preserving opaque colored details.

Exact generation prompts, provider, source hashes and dimensions are recorded in `art/boss-reward-art-v3/generation.json`. To republish the approved artwork after another asset-generation pipeline runs:

```sh
python3 scripts/process-boss-reward-art.py
```

This also refreshes both asset catalog manifests. Run `node --test scripts/boss-reward-art.test.mjs scripts/arsenal-assets.test.mjs` to check dimensions, transparency, gold coloring and visible wheel thumbnails.

Obsidian Vault now labels and collects its actual MN-6 ECHO reward. Every boss case goes through the same pickup classifier, including the nineteenth weapon. The browser regression exercises the real reward spawn and overlap pickup, both pause keys, simulation freeze, and pointer-lock loss.
