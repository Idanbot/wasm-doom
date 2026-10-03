# Campaign expansion: 25 sectors

Levels 12–25 add fourteen bosses and fourteen exclusive rewards. The full campaign has 25 bosses, 33 guns and 37 enemy skins. Claim a boss's golden case to finish its sector. Weapon ownership and ammunition survive checkpoint saves; all unlocked weapons carry into endless mode.

| Level | Sector | Boss | Reward | Attack identity |
| --- | --- | --- | --- | --- |
| 12 | ASHEN TRANSIT | VULCAN–2 | VX-2 CINDER | telegraphed ember lanes |
| 13 | TIDAL PUMPSTATION | LEVIATHAN–8 | LV-8 UNDERTOW | pressure harpoon fan |
| 14 | OPTICS ARRAY | PRISM–5 | PR-5 SPECTRUM | refracted crossfire |
| 15 | MAGNETIC FORGE | MAGNUS–4 | MG-4 POLARITY | charged magnetic sweep |
| 16 | DRONE HATCHERY | BROODMOTHER–3 | BM-3 SWARM | drone reinforcement swarm |
| 17 | FUNGAL RESEARCH | MYCELIUM–9 | MY-9 SPORE | corrosive spore clusters |
| 18 | ORBITAL UPLINK | ASTRA–7 | AS-7 STARFALL | delayed missile barrage |
| 19 | WASTE RECLAIMER | SCRAPPER–6 | SC-6 SHREDDER | flechette wall sweep |
| 20 | SHADOW LAB | UMBRA–1 | UB-1 VEIL | phase relocation and ambush |
| 21 | FUSION CHAMBER | SOL–12 | SL-12 HELIOS | solar pulse rings |
| 22 | FLOODED SILO | NAUTILUS–4 | NT-4 TORPEDO | pressure torpedo volleys |
| 23 | CHRONO ARCHIVE | CHRONOS–8 | CH-8 EPOCH | staggered echo shots |
| 24 | BLACK ICE CORE | BOREAS–11 | BO-11 GLACIER | cryo shard fan and shielding |
| 25 | APEX CONTROL | THE SOVEREIGN | SV-25 DOMINION | command escorts and converging salvos |

## Artwork and wiring

The exact native image-generation prompts, source paths and SHA-256 hashes are in [`art/campaign25/generation.json`](../art/campaign25/generation.json). [`art/campaign25/specs.json`](../art/campaign25/specs.json) records weapon stats, colors, designs and the level roster. The playable roster is mirrored in `src/game/campaign25.ts`, with attack parameters in `engine/src/campaign.rs`.

Each weapon `{weapon}` has:

- `public/game/draft/v2/weap_{weapon}_5x5.png`: 2560×1920, 5×5, 512×384 cells.
- `public/game/draft/v2/weap_{weapon}_aim.png`: 512×384 reference aim.
- `public/game/draft/v2/cases/case_{weapon}.png`: 1024×768 golden fitted case.
- `public/game/spr_gun_{weapon}.png`: 256×256 world pickup.
- `public/game/ui/weapon-thumbs/{weapon}.png`: 640×300 horizontal side view.

The sheet includes full/half/low/empty/removed-feed aim states, three pickup handling poses and four mechanical reload poses: remove feed, bring fresh feed, insert, operate charging mechanism, return to aim. Firing and reserved special poses occupy the final two rows. Cells 9, 14, 19 and 24 exactly match aim; dry-fire cell 5 matches the empty pose. The entire sequence shares one scale and translation so frames remain continuous and handling clears the reticle.

Each boss `{slug}` has seven `public/game/enemy_{slug}_{state}.png` atlases: `idle`, `move`, `pain`, `fire`, `reload`, `dead`, `special`. Each is 256×256 with four 128×128 frames. Source boards are 4×7, 28 distinct native poses.

`public/game/fx25/` contains `rocket-forward.png` (outgoing/incoming/heavy views in a 2×2 atlas), `ion-bolt.png`, `cryo-shard.png`, `spore-cluster.png`, `phase-orb.png`, and four 2×2 animation atlases: `ember-impact.png`, `pressure-impact.png`, `magnetic-impact.png`, `solar-impact.png`. Ordinary and boss rocket launchers use the same corrected missile family. Projectile velocity relative to camera direction selects outgoing or incoming art, with a visible metal nose, fins and rear exhaust. Trails live 0.75–0.9 seconds.

Pack source boards with `python3 scripts/pack-campaign25-art.py` (Pillow), then refresh catalogs with `python3 scripts/refresh-campaign25-catalog.py`. Packing preserves native transparency; it never keys out magenta. Gold cases and side views use matching gun references. Two Cloudflare Aura-2 death clips per new boss live in `public/game/voices/`; `art/boss-voices.json` stores the exact text and voice shaping. The shared voice gate and existing one-shot boss event handling apply to all 25 bosses.

## Combat and endless progression

Broodmother summons explosive drones; Umbra relocates between firing lanes; Magnus pulls the player toward its magnetic field; Boreas rebuilds a frontal shield after its first phase; Sol emits radial shots; Chronos staggers projectile arrival speeds; Sovereign calls shield guards. Other bosses vary projectile families, aimed spreads, burst counts, ranges and anticipation times. The Swarm reward gently steers its drones toward enemies ahead. Precision rewards use hitscan beams, while missiles and other bolts use swept projectile collision.

At level 26, the layout returns to Upper Works without resetting difficulty. Enemy health rises by about 2.5% of the level-25 baseline per additional level; boss health uses the same multiplier. Enemy totals grow by about 1.8% of the four-copy roster per level, with at least 16 base spawns so revisiting smaller legacy casts never resets encounter pressure. Small fractional-power terms gradually increase long-run pressure. There is no gameplay cap on either health or total enemies; integer storage still has its ordinary machine limits.

Up to 72 hostiles are active simultaneously. Remaining enemies stay in a queue and arrive every 1.25 seconds when a slot becomes available, at least seven world units away, with a 0.9-second anticipation pause. The sector cannot be overridden while reinforcements remain. This separates an uncapped encounter total from a bounded renderer/entity budget and leaves space for effects, props, rewards and bosses.

## Verification

Rust checks all 25 distinct layouts and reachable objectives, all 25 reward pickups, expansion firing/reloads/save ownership, endless growth, reinforcement arrival safety and rocket view selection. Asset checks cover all 33 visible thumbnails, all 25 golden cases, native-alpha handling poses and exact aim returns. Browser checks exercise the expanded wheel, loading failures, every gun's firing/reload, and the Sovereign reward-to-level-26 transition. Production ignores local `qa`/`lvl` shortcuts.

Sector-specific surfaces, destructible props, isolated enemy projectiles and corrected missile directions are documented in [sector detail](SECTOR_DETAIL.md). The current atlas has 579 layers. Run its packer and weapon edge cleanup after this original campaign packer so repaired art and catalog hashes stay current.
