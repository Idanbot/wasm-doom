#!/usr/bin/env python3
"""Build centered first-person weapon idle, fire and eight-frame reload sheets."""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageEnhance, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art/source_hd/weapons_v3"
COMBAT_V3 = ROOT / "art/source_hd/combat_v3"
OUT = ROOT / "public/game"
ATLAS = ROOT / "art/atlases"

WEAPONS = {
    "br12": {"name": "BR-12 BREAKER", "height": 600, "flash_y": 38},
    "kx9": {"name": "KX-9 VECTOR", "height": 600, "flash_y": 38},
    "mr4": {"name": "MR-4 LONGBOW", "height": 620, "flash_y": 34},
    "vlk6": {"name": "VLK-6 WARDEN", "height": 590, "flash_y": 150},
    "ax12": {"name": "AX-12 VOLT", "height": 610, "flash_y": 42},
    "m91": {"name": "M91 CYCLONE", "height": 610, "flash_y": 58},
    "hx8": {"name": "HX-8 PYRE", "height": 600, "flash_y": 120},
    "vr9": {"name": "VR-9 OVERRIDE", "height": 620, "flash_y": 30},
    "hc9": {"name": "HC-9 FORGE", "height": 600, "flash_y": 150},
    "cm9": {"name": "CM-9 CHIMERA", "height": 600, "flash_y": 90},
}
FRAME_W = 800
FRAME_H = 480


def clean_native_alpha(image: Image.Image) -> Image.Image:
    """Remove the faint studio wash left in otherwise transparent renders."""
    image = image.convert("RGBA")
    pixels = image.load()
    for y in range(image.height):
        for x in range(image.width):
            r, g, b, a = pixels[x, y]
            # Cloudflare concept renders use the same neon-magenta key as the
            # enemy pipeline. Remove it before bounding-box fitting so boss
            # weapons do not carry a pink rectangle into the view model.
            if r > 120 and b > 90 and g < 170 and r > g * 1.18 and b > g * 1.08:
                pixels[x, y] = (0, 0, 0, 0)
                continue
            if a < 72:
                pixels[x, y] = (0, 0, 0, 0)
            else:
                alpha = min(255, round((a - 56) * 255 / 199))
                pixels[x, y] = (r, g, b, alpha)
    # Remove the compressed magenta fringe that remains around hard edges.
    for y in range(image.height):
        for x in range(image.width):
            r, g, b, a = pixels[x, y]
            if a > 0 and r > 45 and b > 45 and g < min(r, b) * 0.86:
                pixels[x, y] = (0, 0, 0, 0)
    return image


def fit_view(image: Image.Image, target_height: int) -> Image.Image:
    image = clean_native_alpha(image)
    bbox = image.getchannel("A").getbbox()
    if not bbox:
        raise ValueError("weapon vanished during alpha cleanup")
    image = image.crop(bbox)
    scale = min(780 / image.width, target_height / image.height)
    image = image.resize(
        (max(1, round(image.width * scale)), max(1, round(image.height * scale))),
        Image.Resampling.LANCZOS,
    )
    image = image.filter(ImageFilter.UnsharpMask(radius=0.65, percent=110, threshold=3))
    canvas = Image.new("RGBA", (FRAME_W, FRAME_H), (0, 0, 0, 0))
    canvas.alpha_composite(image, ((FRAME_W - image.width) // 2, 8))
    return canvas


def shifted(base: Image.Image, dx: int = 0, dy: int = 0, angle: float = 0, bright: float = 1.0) -> Image.Image:
    frame = base.rotate(
        angle,
        Image.Resampling.BICUBIC,
        center=(FRAME_W // 2, FRAME_H - 35),
        fillcolor=(0, 0, 0, 0),
    )
    if bright != 1.0:
        frame = ImageEnhance.Brightness(frame).enhance(bright)
    out = Image.new("RGBA", base.size, (0, 0, 0, 0))
    out.alpha_composite(frame, (dx, dy))
    return out


def effect_cutout(path: Path) -> Image.Image:
    image = Image.open(path).convert("RGBA")
    pixels = image.load()
    for y in range(image.height):
        for x in range(image.width):
            r, g, b, a = pixels[x, y]
            if r > 115 and b > 65 and r - g > 38 and b - g > 24 and r > b * 0.62:
                pixels[x, y] = (0, 0, 0, 0)
    bbox = image.getchannel("A").getbbox()
    if not bbox:
        raise ValueError(f"effect vanished during key: {path}")
    return image.crop(bbox)


def add_flash(frame: Image.Image, slug: str, strength: float, muzzle: Image.Image) -> Image.Image:
    out = frame.copy()
    size = 92 if slug in {"kx9", "ax12"} else 118
    if slug == "vlk6":
        size = 132
    aspect = muzzle.height / max(1, muzzle.width)
    burst = muzzle.resize((size, max(1, round(size * aspect))), Image.Resampling.LANCZOS)
    if slug in {"mr4", "ax12"}:
        tint = Image.new("RGBA", burst.size, (105, 225, 255, 0))
        tint.putalpha(burst.getchannel("A"))
        burst = Image.blend(burst, tint, 0.5)
    burst = ImageEnhance.Brightness(burst).enhance(0.88 + strength * 0.22)
    cx, cy = FRAME_W // 2, WEAPONS[slug]["flash_y"]
    out.alpha_composite(burst, (cx - burst.width // 2, cy - burst.height // 2))
    return out


def pack(frames: list[Image.Image], columns: int, rows: int) -> Image.Image:
    if len(frames) != columns * rows:
        raise ValueError(f"expected {columns * rows} frames, got {len(frames)}")
    sheet = Image.new("RGBA", (FRAME_W * columns, FRAME_H * rows), (0, 0, 0, 0))
    for index, frame in enumerate(frames):
        sheet.alpha_composite(frame, ((index % columns) * FRAME_W, (index // columns) * FRAME_H))
    return sheet


def reload_frames(path: Path, target_height: int) -> list[Image.Image]:
    raw = Image.open(path).convert("RGBA")
    cell_w = raw.width // 4
    cell_h = raw.height // 2
    return [
        fit_view(
            raw.crop(
                (
                    (index % 4) * cell_w,
                    (index // 4) * cell_h,
                    (index % 4 + 1) * cell_w,
                    (index // 4 + 1) * cell_h,
                )
            ),
            target_height,
        )
        for index in range(8)
    ]


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    ATLAS.mkdir(parents=True, exist_ok=True)
    muzzle = effect_cutout(COMBAT_V3 / "muzzle_flash_front.jpg")
    report = {
        "mk23s": {
            "name": "MK23-S",
            "source": "art/source_hd/combat_v3/mk23s_pov.png",
            "idle": "public/game/weap_mk23s.png",
            "fireFrames": 4,
            "reloadFrames": 4,
        }
    }

    # Handling cycle when no multi-pose reload master exists: dip the
    # muzzle down, cant side to side, rise back to ready. Same fake as the
    # fire sheets, and consistent by construction.
    HANDLING = [(0, 0, 0), (0, 14, -4), (0, 30, -9), (-4, 44, -14),
                (4, 44, 14), (0, 30, 9), (0, 14, 4), (0, 0, 0)]

    for slug, spec in WEAPONS.items():
        source_path = SOURCE / f"{slug}.png"
        reload_path = SOURCE / f"{slug}_reload_raw.png"
        if not source_path.exists():
            raise FileNotFoundError(source_path)
        base = fit_view(Image.open(source_path), spec["height"])
        base.save(OUT / f"weap_{slug}.png", optimize=True)
        recoil = [
            shifted(base),
            add_flash(shifted(base, -3, 18, -0.65, 1.05), slug, 0.55, muzzle),
            add_flash(shifted(base, 3, 34, 0.85, 1.10), slug, 1.0, muzzle),
            shifted(base, 0, 12, 0, 0.98),
        ]
        pack(recoil, 2, 2).save(OUT / f"weap_{slug}_fire.png", optimize=True)
        if reload_path.exists():
            reload = reload_frames(reload_path, spec["height"])
            reload_source = str(reload_path.relative_to(ROOT))
        else:
            reload = [shifted(base, dx, dy, ang) for (dx, dy, ang) in HANDLING]
            reload_source = f"{source_path.relative_to(ROOT)} (synthetic handling cycle)"
        pack(reload, 4, 2).save(OUT / f"weap_{slug}_reload.png", optimize=True)
        report[slug] = {
            "name": spec["name"],
            "source": str(source_path.relative_to(ROOT)),
            "reloadSource": reload_source,
            "idle": f"public/game/weap_{slug}.png",
            "fireFrames": 4,
            "reloadFrames": 8,
        }
        metadata = {
            "id": f"weapon_{slug}",
            "frameWidth": FRAME_W,
            "frameHeight": FRAME_H,
            "origin": {"x": FRAME_W // 2, "y": FRAME_H},
            "animations": {
                "idle": {"start": 0, "frames": 1, "fps": 1},
                "fire": {"start": 0, "frames": 4, "fps": 12},
                "reload": {"start": 0, "frames": 8, "fps": 12},
            },
            "view": "centered first-person sightline",
        }
        (ATLAS / f"weapon_{slug}.json").write_text(json.dumps(metadata, indent=2) + "\n")
    (SOURCE / "processing-report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
