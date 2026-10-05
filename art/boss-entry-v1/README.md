# Boss entrance effects

Generated native-alpha source atlas: `source.png`, 4×2 motifs. The eight runtime
PNGs are 1024×1024, padded and preloaded in world atlas slots 806–813. The renderer
uses 256×256 layers. Repack with `python3 scripts/pack-boss-entry.py`.

`manifest.json` records source/output hashes, dimensions, provider and prompt.
No chroma key processing is used. Existing art/audio licenses remain unchanged.

Sector choreography is authored in `Engine::boss_intro_effect`: five stages,
sector-dependent radial beam count, phase rotation, contraction radius and motif.
`src/game/sector-effects.ts` handles pause-safe projection; `src/game/blit.ts`
implements depth-aware lightning/laser bloom in both GPU backends. Dynamic world
lighting is checked against line-of-sight in `Engine::add_light`.
