#!/usr/bin/env python3
"""Deterministically turn 1024px enemy renders into seven 2x2 runtime sheets.

The image model supplies one clean, high-resolution design render per enemy.
This script owns the repeatable work the model should not do: magenta keying,
feet/origin alignment, animation poses, downsampling and sheet packing.
"""

from __future__ import annotations

import argparse
import json
from collections import deque
from pathlib import Path

from PIL import Image, ImageDraw


ANIMATIONS = ("idle", "move", "pain", "fire", "reload", "dead", "special")
FRAME_COUNTS = {"idle": 2, "move": 4, "pain": 2, "fire": 2, "reload": 2, "dead": 2, "special": 2}
SPECS = (
    "rifleman",
    "breacher",
    "subject",
    "hazmat",
    "gunner",
    "loader",
    "vatbrute",
    "marksman",
    "hornet",
    "hound",
    "spitter",
    "martyr",
    "veyran",
)


def is_magenta(r: int, g: int, b: int) -> bool:
    return r > 190 and b > 120 and g < 160 and b > g * 1.2 and r > b * 0.72


def is_studio_pink(r: int, g: int, b: int) -> bool:
    """Match the darker pink falloff used by generated floor shadows.

    A strict chroma key catches the flat background but can stop in the
    shadow's darker center. This broader hue test is used only while flood
    filling from the image edge, so magenta details enclosed by the subject
    remain intact.
    """
    return r > 135 and b > 62 and g < 112 and b > g * 1.15 and r > b * 0.72


def key_magenta(image: Image.Image) -> Image.Image:
    """Remove edge-connected studio background and enclosed magenta islands."""
    image = image.convert("RGBA")
    pixels = image.load()
    w, h = image.size
    corner = pixels[0, 0]
    br, bg, bb = corner[:3]

    def near_corner(r: int, g: int, b: int) -> bool:
        # The model sometimes renders a soft floor shadow or a pink-to-gray
        # gradient. Flood fill against the sampled corner removes that halo
        # while keeping the centered subject intact.
        d = (r - br) ** 2 + (g - bg) ** 2 + (b - bb) ** 2
        return d <= 92 * 92

    seen = bytearray(w * h)
    q = deque()
    for x in range(w):
        q.append((x, 0))
        q.append((x, h - 1))
    for y in range(h):
        q.append((0, y))
        q.append((w - 1, y))
    while q:
        x, y = q.popleft()
        i = y * w + x
        if seen[i]:
            continue
        seen[i] = 1
        r, g, b, _ = pixels[x, y]
        if not near_corner(r, g, b) and not is_magenta(r, g, b) and not is_studio_pink(r, g, b):
            continue
        pixels[x, y] = (r, g, b, 0)
        if x:
            q.append((x - 1, y))
        if x + 1 < w:
            q.append((x + 1, y))
        if y:
            q.append((x, y - 1))
        if y + 1 < h:
            q.append((x, y + 1))

    # Keep the enclosed-gap guarantee even where a limb closes a region.
    for y in range(image.height):
        for x in range(image.width):
            r, g, b, _ = pixels[x, y]
            if is_magenta(r, g, b):
                pixels[x, y] = (r, g, b, 0)
    return image


def translate(image: Image.Image, dx: int, dy: int) -> Image.Image:
    return image.transform(
        image.size,
        Image.Transform.AFFINE,
        (1, 0, -dx, 0, 1, -dy),
        resample=Image.Resampling.BICUBIC,
        fillcolor=(0, 0, 0, 0),
    )


def fit_source(source: Image.Image, spec: str) -> Image.Image:
    source = key_magenta(source)
    alpha = source.getchannel("A")
    bbox = alpha.getbbox()
    if bbox is None:
        raise ValueError(f"{spec}: source has no visible subject after magenta keying")
    source = source.crop(bbox)
    target_height = 216 if spec not in {"hornet", "hound", "martyr"} else 188
    scale = target_height / source.height
    width = max(1, round(source.width * scale))
    source = source.resize((width, target_height), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    x = (256 - source.width) // 2
    if spec in {"hornet", "marksman", "spitter"}:
        y = (256 - source.height) // 2 + 8
    else:
        y = 236 - source.height
    canvas.alpha_composite(source, (x, y))
    return canvas


def add_fire_flash(image: Image.Image, side: int) -> Image.Image:
    out = image.copy()
    glow = Image.new("RGBA", out.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(glow)
    cx = 198 if side > 0 else 58
    cy = 142
    draw.ellipse((cx - 20, cy - 20, cx + 20, cy + 20), fill=(255, 205, 90, 96))
    draw.ellipse((cx - 9, cy - 9, cx + 9, cy + 9), fill=(255, 245, 190, 220))
    return Image.alpha_composite(out, glow)


def add_special_glow(image: Image.Image, spec: str) -> Image.Image:
    out = image.copy()
    glow = Image.new("RGBA", out.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(glow)
    color = (224, 154, 54, 66) if spec in {"rifleman", "breacher", "hazmat", "gunner", "loader"} else (69, 186, 177, 62)
    draw.ellipse((40, 26, 216, 244), outline=(*color[:3], 44), width=5)
    return Image.alpha_composite(out, glow)


def frames_for(base: Image.Image, spec: str, animation: str) -> list[Image.Image]:
    if animation == "idle":
        return [translate(base, -1, 0), translate(base, 1, 1)]
    if animation == "move":
        return [translate(base, -3, 1), translate(base, -1, -1), translate(base, 1, 1), translate(base, 3, -1)]
    if animation == "pain":
        red = Image.new("RGBA", base.size, (224, 38, 34, 0))
        red.putalpha(base.getchannel("A").point(lambda p: int(p * 0.42)))
        return [Image.alpha_composite(translate(base, -2, 0), red), Image.alpha_composite(translate(base, 2, 1), red)]
    if animation == "fire":
        return [add_fire_flash(translate(base, -2, 0), 1), add_fire_flash(translate(base, 1, 1), -1)]
    if animation == "reload":
        return [translate(base, 0, 1), translate(base, 0, -1)]
    if animation == "dead":
        return [base.rotate(7, resample=Image.Resampling.BICUBIC, center=(128, 236), fillcolor=(0, 0, 0, 0)),
                base.rotate(16, resample=Image.Resampling.BICUBIC, center=(128, 236), fillcolor=(0, 0, 0, 0)).transform(base.size, Image.Transform.AFFINE, (1, 0, 5, 0, 1, -4), fillcolor=(0, 0, 0, 0))]
    if animation == "special":
        return [add_special_glow(translate(base, -1, 0), spec), add_special_glow(translate(base, 1, -1), spec)]
    raise ValueError(animation)


def pack_sheet(frames: list[Image.Image]) -> Image.Image:
    sheet = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    cells = [*frames]
    while len(cells) < 4:
        cells.append(cells[-1])
    for i, frame in enumerate(cells[:4]):
        frame = frame.resize((128, 128), Image.Resampling.LANCZOS)
        sheet.alpha_composite(frame, ((i & 1) * 128, (i >> 1) * 128))
    return sheet


def build(spec: str, source_dir: Path, output_dir: Path, atlas_dir: Path) -> None:
    source_path = source_dir / f"enemy_{spec}.png"
    if not source_path.exists():
        # Cloudflare returns JPEG bytes; the batch generator keeps those as
        # `.jpg` until an artist or a later conversion pass needs PNG.
        source_path = source_dir / f"enemy_{spec}.jpg"
    if not source_path.exists():
        raise FileNotFoundError(source_dir / f"enemy_{spec}.png")
    base = fit_source(Image.open(source_path), spec)
    for animation in ANIMATIONS:
        sheet = pack_sheet(frames_for(base, spec, animation))
        sheet.save(output_dir / f"enemy_{spec}_{animation}.png", "PNG", optimize=True)
    atlas_dir.mkdir(parents=True, exist_ok=True)
    metadata = {
        "id": f"enemy_{spec}",
        "frameWidth": 128,
        "frameHeight": 128,
        "origin": {"x": 64, "y": 118},
        "animations": {
            name: {"start": sum(FRAME_COUNTS[a] for a in ANIMATIONS[:i]), "frames": FRAME_COUNTS[name], "fps": 8 if name == "move" else 4}
            for i, name in enumerate(ANIMATIONS)
        },
        "layers": [f"enemy_{spec}_{name}.png" for name in ANIMATIONS],
        "background": "transparent after deterministic #ff00ff key",
        "source": str(source_path),
    }
    (atlas_dir / f"enemy_{spec}.json").write_text(json.dumps(metadata, indent=2) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-dir", type=Path, default=Path("art/source_hd/enemies"))
    parser.add_argument("--output-dir", type=Path, default=Path("public/game"))
    parser.add_argument("--atlas-dir", type=Path, default=Path("art/atlases"))
    parser.add_argument("--spec", choices=SPECS, action="append")
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    selected = args.spec or list(SPECS)
    for spec in selected:
        build(spec, args.source_dir, args.output_dir, args.atlas_dir)
        print(f"built enemy_{spec}: {len(ANIMATIONS)} layers")


if __name__ == "__main__":
    main()
