# BLACKSITE projectile rework

Generated with the built-in image generation tool using native transparent backgrounds. `boss-weapons.png` is a 5×5 board: 25 outgoing views, in weapon slot order 8–32. `enemies-0-19.png` and `enemies-20-36.png` are 5×4 boards: 37 incoming views in enemy skin order. Remaining three cells on the second enemy board are spare particle concepts.

The prompt specification was: realistic industrial FPS cutouts, centered isolated sprites in uniform cells with large padding; no dividers, lettering, ground shadows or backdrops. Player projectiles show their body pointing up/forward away from the viewer; enemy projectiles show their incoming nose/core. The player board follows tungsten, electricity, acid, archive, prediction, gravity, coolant, signal, siege, tactical, memory, ember, harpoon, optics, magnetic, microdrone, spores, smart missile, flechettes, phase, solar, torpedo, temporal, cryogenic and command designs. The swarm uses visible mechanical rotors/chassis, not an orb.

`python3 scripts/pack-projectiles-v3.py` exports 62 isolated 256×256 layers, retains alpha, removes tiny disconnected neighboring-cell fragments, and packs with at least 19 pixels of border space. It also updates the six directional missile compatibility aliases. No magenta keying is used. `manifest.json` records source dimensions, cell mapping and source/export hashes. The developer catalog includes the new layers.

Player visual IDs are independent of element/ownership/collision state. Hitscan guns retain instant damage and show brief unique trail samples. The BM-3 SWARM fires five allied homing drones with a wide launch formation and subtle rotor bob. Enemy missiles always use their separately authored incoming texture.

Tests validate sources, uniqueness across player/enemy sets, transparent borders, actual renderer visibility, unique weapon selection, homing and allied collision behavior.
