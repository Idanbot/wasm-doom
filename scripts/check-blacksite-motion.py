#!/usr/bin/env python3
"""Reject enemy sheets whose runtime cells are effectively static duplicates."""
from pathlib import Path
from PIL import Image, ImageChops, ImageStat

ROOT = Path(__file__).resolve().parents[1]
ANIMATIONS = ("idle", "move", "pain", "fire", "reload", "dead", "special")
SKINS = ("rifleman", "breacher", "subject", "hazmat", "gunner", "loader", "vatbrute", "marksman", "hornet", "hound", "spitter", "martyr", "veyran")

checked = 0
for skin in SKINS:
    for animation in ANIMATIONS:
        image = Image.open(ROOT / "public/game" / f"enemy_{skin}_{animation}.png").convert("RGBA")
        frames = [image.crop(((i & 1) * 128, (i >> 1) * 128, (i & 1) * 128 + 128, (i >> 1) * 128 + 128)) for i in range(4)]
        required = 4 if animation == "move" else 2
        for i in range(1, required):
            diff = ImageChops.difference(frames[0], frames[i])
            mean = sum(ImageStat.Stat(diff).mean) / 4
            if mean < 0.45:
                raise SystemExit(f"{skin}/{animation}: frame {i} is effectively static ({mean:.3f})")
        checked += required
print(f"[check:motion] {checked} enemy animation frames vary and decode correctly.")
