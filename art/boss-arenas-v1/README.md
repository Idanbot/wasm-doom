# Boss arena materials

25 individually generated materials, one per campaign boss. Generated with the built-in image-generation tool (generate mode, opaque background). `prompts.json` preserves every exact prompt; `generation.json` records original dimensions, hashes and runtime paths. Native 1254×1254 masters are retained. No Cloudflare image quota was used for this set.

`specs.json` defines the boss identity, material, hazard pattern, color and intro timing. Runtime mirrors live in `src/game/boss-arena-data.json` and `engine/src/boss_arena.rs`; tests verify 25 distinct patterns and warning/impact/death cleanup across every map.

`python3 scripts/pack-boss-arenas.py` creates opaque 256×256 PNG atlas layers and lossless 1024×1024 WebP review textures. Opposite pixel edges are blended and made identical for seamless placement. `preview.jpg` shows all 25 textures.

The renderer tints hazard tile borders during warnings, then fills them during impact. The authored shape remains fixed through both phases. Cover blocks hazard placement, the override console stays safe, and bounded steam/electrical/debris effects avoid floating flames.
