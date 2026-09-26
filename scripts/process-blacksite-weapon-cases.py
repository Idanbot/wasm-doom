#!/usr/bin/env python3
"""Key generated weapon cases and publish one 256px world sprite per gun."""
from __future__ import annotations

import json
from collections import deque
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art" / "source_hd" / "weapon_cases"
# v2 draft cases (1024x768 native-alpha renders) supersede the v1 masters
# and are processed second so they win the runtime sprite slots.
V2_SOURCE = ROOT / "public" / "game" / "draft" / "v2" / "cases"
OUT = ROOT / "public" / "game"
WEAPONS = ("mk23s", "br12", "kx9", "mr4", "vlk6", "ax12", "m91", "hx8", "vr9", "hc9", "cm9")


def key_magenta(image: Image.Image) -> Image.Image:
    image = image.convert("RGBA")
    pixels = image.load()
    width, height = image.size
    seen = bytearray(width * height)
    queue = deque()
    for x in range(width):
        queue.extend(((x, 0), (x, height - 1)))
    for y in range(height):
        queue.extend(((0, y), (width - 1, y)))
    while queue:
        x, y = queue.popleft()
        index = y * width + x
        if seen[index]:
            continue
        seen[index] = 1
        red, green, blue, _ = pixels[x, y]
        is_pink = red > 120 and blue > 55 and red - green > 35 and blue - green > 14
        if not is_pink:
            continue
        pixels[x, y] = (0, 0, 0, 0)
        if x:
            queue.append((x - 1, y))
        if x + 1 < width:
            queue.append((x + 1, y))
        if y:
            queue.append((x, y - 1))
        if y + 1 < height:
            queue.append((x, y + 1))
    for y in range(height):
        for x in range(width):
            red, green, blue, alpha = pixels[x, y]
            if alpha < 32 or (red > 135 and blue > 65 and red - green > 42 and blue - green > 18):
                pixels[x, y] = (0, 0, 0, 0)
    return image


def fit(image: Image.Image) -> Image.Image:
    bbox = image.getchannel("A").getbbox()
    if not bbox:
        raise ValueError("weapon case vanished during chroma key")
    image = image.crop(bbox)
    scale = min(230 / image.width, 218 / image.height)
    image = image.resize(
        (max(1, round(image.width * scale)), max(1, round(image.height * scale))),
        Image.Resampling.LANCZOS,
    )
    canvas = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    canvas.alpha_composite(image, ((256 - image.width) // 2, 238 - image.height))
    return canvas


def despill(image: Image.Image) -> Image.Image:
    """Clear enclosed pink fringe the edge flood fill cannot reach.
    Guarded to spare amber lamps and true reds."""
    pixels = image.load()
    for y in range(image.height):
        for x in range(image.width):
            red, green, blue, alpha = pixels[x, y]
            if alpha >= 32 and red > 150 and green < 80 and blue > 90 and (red - blue) > 30:
                pixels[x, y] = (0, 0, 0, 0)
    return image


def main() -> None:
    report = {}
    for slug in WEAPONS:
        v2 = V2_SOURCE / f"case_{slug}.png"
        if v2.exists():
            source = v2
        else:
            source = SOURCE / (f"{slug}.png" if (SOURCE / f"{slug}.png").exists() else f"{slug}.jpg")
        sprite = fit(despill(key_magenta(Image.open(source))))
        target = OUT / f"spr_gun_{slug}.png"
        sprite.save(target, "PNG", optimize=True)
        report[slug] = {
            "source": str(source.relative_to(ROOT)),
            "runtime": str(target.relative_to(ROOT)),
            "size": [256, 256],
        }
        if slug == "br12":
            sprite.save(OUT / "spr_gun.png", "PNG", optimize=True)
    (SOURCE / "processing-report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
