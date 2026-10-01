# Obsidian Vault expansion

Sector 11, NADIR–7K, adds a distinct twin-gallery map that reconnects at a wide memory vault. Restore the memory bus, activate the override station, defeat MNEMOSYNE–6, then collect the MN-6 Echo case to clear the sector. The campaign now cycles through 11 bosses and 19 weapons, including 11 unique boss rewards.

MNEMOSYNE–6 is an obsidian-armored memory android. Its ranged attack has a long 1.05-second windup, six projectiles and a 1.7-second cooldown. At health thresholds it releases eight Martyr echo mines plus a 20-projectile ring. Mines retain the existing visible detonation windup and can be destroyed. The late-sector boss health cap remains 6000.

MN-6 Echo uses a 12-charge cell, 72 reserve cap, 1.9-second manual reload and a 0.52-second fire interval. Its pulse pierces up to three targets, with a brief tracer afterimage. It is initially locked and only unlocked by its boss reward. Ownership, ammunition, wheel selection, preload, dev catalog and save/load all include slot 19.

## Art delivery

- Boss source poses: `art/bosses/mnemosyne-{source,fire-source,special-source}.png`.
- Runtime boss states: `public/game/enemy_mnemosyne_{idle,move,pain,fire,reload,dead,special}.png`, each 256×256, four 128×128 cells in a 2×2 grid.
- Gun master: `public/game/draft/v2/masters/mn6.png`.
- Gun animation: `public/game/draft/v2/weap_mn6_5x5.png`, 2560×1920, 25 cells of 512×384; follows the weapon-rework frame contract.
- Aim: `public/game/draft/v2/weap_mn6_aim.png`, 512×384.
- Case: `public/game/draft/v2/cases/case_mn6.png`, 1024×768.
- Runtime pickup: `public/game/spr_gun_mn6.png`, 256×192.
- Wheel thumbnail: `public/game/ui/weapon-thumbs/mn6.png`, 640×300.
- Voices: `public/game/voices/mnemosyne-{1..6}.mp3` and `boss-mnemosyne{,-v2}.mp3`; Cloudflare Aura-2, Luna voice, synthetic processing. Subtitles use the same text as the voice manifest.

Art generated with the built-in imagegen tool, then packed with the existing Pillow animation pipeline. Final prompts: full-body obsidian ceramic security android with violet optic/core and a compact graviton rifle, detailed gritty tactical FPS cutout; and matching realistic first-person obsidian/violet rifle with tactical hands, muzzle at 50%/42%, held bottom-right toward the center, isolated transparent background and a clear crosshair. Firing and charging source poses are derived from the boss master; runtime cells add recoil, movement, damage, reload and collapse transforms. Regenerate the gun sheet with `python3 scripts/generate-expansion-weapons.py mn6`.

## Wire contracts

Weapon slots: 19. Save layout: 184 bytes, ammo offset 32, magazine offset 108. HUD: 224 bytes, appended `hasW19` at offset 220. Atlas: 218 layers, 23 enemy skins. Previous HUD offsets and ownership flags are preserved. Older 8/11/12/18-slot checkpoints pad to the current slot count.

Regression coverage verifies every boss entry and phase transition, map/objective reachability, unique reward cases, Echo firing and save roundtrip, visible weapon thumbnails, voice files and varied enemy frames. Browser combat coverage includes Echo selection/fire/reload and the Obsidian Vault boss HUD.
