#!/usr/bin/env python3
"""Deterministically turn 1024px enemy renders into seven 4-frame runtime sheets.

The image model supplies one clean, high-resolution design render per enemy.
This script owns the repeatable work the model should not do: magenta keying,
feet/origin alignment, animation poses, downsampling and sheet packing.
"""

from __future__ import annotations

import argparse
import json
from collections import deque
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter


ANIMATIONS = ("idle", "move", "pain", "fire", "reload", "dead", "special")
FRAME_COUNTS = {name: 4 for name in ANIMATIONS}
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
    "hecate",
    "chimera",
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
    # Current master art is generated with real transparency. Chroma-keying a
    # transparent image by its hidden RGB values can eat dark armor connected
    # to the canvas edge, so transparent sources only need alpha cleanup.
    if image.getchannel("A").getextrema()[0] < 250:
        return clean_alpha_noise(image)
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


def clean_alpha_noise(image: Image.Image) -> Image.Image:
    """Drop isolated generation specks without damaging the main silhouette."""
    image = image.convert("RGBA")
    alpha = image.getchannel("A")
    w, h = image.size
    mask = alpha.point(lambda value: 255 if value > 8 else 0)
    pixels = mask.load()
    seen = bytearray(w * h)
    components: list[list[tuple[int, int]]] = []

    for sy in range(h):
        for sx in range(w):
            index = sy * w + sx
            if seen[index] or not pixels[sx, sy]:
                continue
            seen[index] = 1
            queue = deque([(sx, sy)])
            component: list[tuple[int, int]] = []
            while queue:
                x, y = queue.popleft()
                component.append((x, y))
                for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    if nx < 0 or nx >= w or ny < 0 or ny >= h:
                        continue
                    ni = ny * w + nx
                    if seen[ni] or not pixels[nx, ny]:
                        continue
                    seen[ni] = 1
                    queue.append((nx, ny))
            components.append(component)

    if not components:
        return image
    largest = max(len(component) for component in components)
    minimum = max(48, round(largest * 0.0005))
    cleaned = alpha.copy()
    cleaned_pixels = cleaned.load()
    for component in components:
        if len(component) >= minimum:
            continue
        for x, y in component:
            cleaned_pixels[x, y] = 0
    image.putalpha(cleaned)
    return image


def translate(image: Image.Image, dx: int, dy: int) -> Image.Image:
    return image.transform(
        image.size,
        Image.Transform.AFFINE,
        (1, 0, -dx, 0, 1, -dy),
        resample=Image.Resampling.BICUBIC,
        fillcolor=(0, 0, 0, 0),
    )


def deform(image: Image.Image, sx: float = 1.0, sy: float = 1.0, shear: float = 0.0, dx: int = 0, dy: int = 0) -> Image.Image:
    """Pose the isolated body around its bottom-center gameplay anchor."""
    scaled = image.resize((max(1, round(256 * sx)), max(1, round(256 * sy))), Image.Resampling.BICUBIC)
    canvas = Image.new("RGBA", (320, 320), (0, 0, 0, 0))
    canvas.alpha_composite(scaled, ((320 - scaled.width) // 2 + dx, 300 - scaled.height + dy))
    warped = canvas.transform(
        canvas.size,
        Image.Transform.AFFINE,
        (1, shear, -shear * 210, 0, 1, 0),
        resample=Image.Resampling.BICUBIC,
        fillcolor=(0, 0, 0, 0),
    )
    return warped.crop((32, 44, 288, 300))


def recoil_pose(base: Image.Image, amount: int) -> Image.Image:
    """Move the torso independently from planted legs for a readable recoil."""
    out = base.copy()
    upper = base.crop((0, 0, 256, 184))
    out.paste((0, 0, 0, 0), (0, 0, 256, 176))
    out.alpha_composite(upper.rotate(-amount * 0.45, Image.Resampling.BICUBIC, center=(128, 178), fillcolor=(0, 0, 0, 0)), (-amount, amount // 2))
    return out


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


def add_special_glow(image: Image.Image, spec: str, strength: float = 1.0) -> Image.Image:
    out = image.copy()
    glow = Image.new("RGBA", out.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(glow)
    color = (224, 154, 54, 66) if spec in {"rifleman", "breacher", "hazmat", "gunner", "loader"} else (69, 186, 177, 62)
    draw.ellipse((40, 26, 216, 244), outline=(*color[:3], round(32 + 44 * strength)), width=max(3, round(3 + 4 * strength)))
    return Image.alpha_composite(out, glow)


def pain_tint(image: Image.Image, strength: float) -> Image.Image:
    red = Image.new("RGBA", image.size, (224, 38, 34, 0))
    red.putalpha(image.getchannel("A").point(lambda p: int(p * strength)))
    return Image.alpha_composite(image, red)


def frames_for(base: Image.Image, spec: str, animation: str) -> list[Image.Image]:
    if animation == "idle":
        return [deform(base, 0.992, 1.008, -0.018, -2, -1), deform(base, 1.008, 0.992, -0.006, -1, 2),
                deform(base, 1.016, 0.985, 0.018, 2, 4), deform(base, 1.004, 0.998, 0.006, 1, 1)]
    if animation == "move":
        return [deform(base, 0.94, 1.02, -0.055, -5, -1), deform(base, 1.05, 0.96, 0.02, -2, 5),
                deform(base, 0.94, 1.02, 0.055, 5, -1), deform(base, 1.05, 0.96, -0.02, 2, 5)]
    if animation == "pain":
        return [pain_tint(deform(base, 0.92, 1.0, -0.12, -13, 1), 0.52),
                pain_tint(deform(base, 0.84, 0.96, -0.07, -8, 8), 0.62),
                pain_tint(deform(base, 1.04, 0.93, 0.08, 8, 11), 0.46),
                pain_tint(deform(base, 1.01, 0.98, 0.025, 3, 4), 0.24)]
    if animation == "fire":
        return [recoil_pose(base, 2), add_fire_flash(recoil_pose(base, 8), 1),
                add_fire_flash(recoil_pose(base, 14), 1), recoil_pose(deform(base, 1.01, 0.98, 0.015, 2, 4), 5)]
    if animation == "reload":
        return [recoil_pose(deform(base, 0.98, 0.99, -0.075, -8, 3), -5),
                recoil_pose(deform(base, 0.94, 0.96, -0.035, -4, 10), -2),
                recoil_pose(deform(base, 1.04, 0.94, 0.065, 8, 12), 7),
                recoil_pose(deform(base, 1.01, 0.98, 0.025, 3, 5), 3)]
    if animation == "dead":
        fall = [
            deform(base, 0.98, 1.0, -0.04, -4, 1),
            deform(base, 0.94, 0.96, -0.10, -9, 8),
            deform(base, 0.88, 0.88, -0.16, -13, 18),
            deform(base, 0.82, 0.72, -0.22, -16, 42),
        ]
        return [frame.rotate(angle, resample=Image.Resampling.BICUBIC, center=(128, 224), fillcolor=(0, 0, 0, 0))
                for frame, angle in zip(fall, (5, 18, 38, 62))]
    if animation == "special":
        return [add_special_glow(deform(base, 0.91, 0.97, -0.055, -5, 7), spec, 0.35),
                add_special_glow(deform(base, 1.02, 1.01, -0.015, -1, -1), spec, 0.7),
                add_special_glow(deform(base, 1.10, 1.05, 0.045, 5, -10), spec, 1.0),
                add_special_glow(deform(base, 1.01, 0.99, 0.015, 2, 2), spec, 0.55)]
    raise ValueError(animation)


def pack_sheet(frames: list[Image.Image]) -> Image.Image:
    sheet = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    cells = [*frames]
    while len(cells) < 4:
        cells.append(cells[-1])
    for i, frame in enumerate(cells[:4]):
        frame = frame.resize((128, 128), Image.Resampling.LANCZOS)
        # Restore edge definition lost in the 1024px -> 128px reduction. The
        # restrained radius sharpens plates, seams and optics without adding
        # a halo around the transparent silhouette.
        frame = frame.filter(ImageFilter.UnsharpMask(radius=0.7, percent=115, threshold=3))
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
            name: {"start": sum(FRAME_COUNTS[a] for a in ANIMATIONS[:i]), "frames": FRAME_COUNTS[name], "fps": 8 if name == "move" else 6}
            for i, name in enumerate(ANIMATIONS)
        },
        "layers": [f"enemy_{spec}_{name}.png" for name in ANIMATIONS],
        "background": "native transparency with deterministic alpha cleanup; legacy #ff00ff key supported",
        "source": str(source_path),
        "motion": "four-frame anchored deformation with independent torso recoil",
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
