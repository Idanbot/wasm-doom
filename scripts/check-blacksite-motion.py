#!/usr/bin/env python3
"""Reject enemy sheets whose runtime cells are effectively static duplicates."""
import json
from pathlib import Path
from PIL import Image, ImageChops, ImageStat

ROOT = Path(__file__).resolve().parents[1]
ANIMATIONS = ("idle", "move", "pain", "fire", "reload", "dead", "special")
SKINS = tuple(e['slug'] for e in json.loads((ROOT / 'art/enemies-v4/specs.json').read_text())['enemies'])

checked = 0
for skin in SKINS:
    for animation in ANIMATIONS:
        image = Image.open(ROOT / "public/game" / f"enemy_{skin}_{animation}.png").convert("RGBA")
        if image.size != (256, 256): raise SystemExit(f"{skin}/{animation}: wrong atlas dimensions")
        frames = [image.crop(((i & 1) * 128, (i >> 1) * 128, (i & 1) * 128 + 128, (i >> 1) * 128 + 128)) for i in range(4)]
        for index, frame in enumerate(frames):
            alpha = frame.getchannel('A')
            bounds = alpha.point(lambda value: 255 if value > 24 else 0).getbbox()
            if bounds is None or bounds[0] < 4 or bounds[1] < 4 or bounds[2] > 124 or bounds[3] > 124:
                raise SystemExit(f"{skin}/{animation}/{index}: blank or clipped silhouette {bounds}")
            if sum(alpha.histogram()[128:]) < 600:
                raise SystemExit(f"{skin}/{animation}/{index}: incomplete subject")
        required = 4
        for i in range(1, required):
            diff = ImageChops.difference(frames[i - 1], frames[i])
            mean = sum(ImageStat.Stat(diff).mean) / 4
            if mean < 0.45:
                raise SystemExit(f"{skin}/{animation}: frame {i} is effectively static ({mean:.3f})")
        checked += required
print(f"[check:motion] {checked} enemy animation frames vary and decode correctly.")
