#!/usr/bin/env python3
"""Publish v2 draft surfaces as 256px runtime textures.

Global replacements overwrite their live slots (same names, same dims).
Per-theme walls/doors go to public/game/theme/ and are swapped into the
T_TECH/T_DOOR atlas slots at runtime by wave (see hs_apply_theme).
Square center-crops keep the seamless tiling intact.
"""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageStat


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "public" / "game" / "draft" / "v2" / "textures"
OUT = ROOT / "public" / "game"
THEME_OUT = OUT / "theme"

# draft file -> live runtime file (same 256px contract as before).
GLOBAL = {
    "wall_old_brick_1080p.png": "wall_brick.png",
    "wall_dark_alloy_1080p.png": "wall_metal.png",
    "floor_military_grate_1080p.png": "floor_grate.png",
    "floor_service_concrete_1080p.png": "floor_concrete.png",
    "floor_hazard_steel_1080p.png": "wall_hazard_tile2x2.png",
    "ceil_dark_panels_1080p.png": "ceil_pipes.png",
}

THEMES = ("hangar", "plaza", "security", "datacenter", "foundry", "biotech", "nuclear", "vault")


def to_tile(source: Path) -> Image.Image:
    image = Image.open(source).convert("RGB")
    side = min(image.width, image.height)
    left = (image.width - side) // 2
    top = (image.height - side) // 2
    tile = image.crop((left, top, left + side, top + side)).resize(
        (256, 256), Image.Resampling.LANCZOS
    )
    # Opaque alpha channel: the legacy environment gate requires RGBA
    # runtime files, and the engine ignores alpha on world surfaces.
    out = Image.new("RGBA", (256, 256), (0, 0, 0, 255))
    out.paste(tile, (0, 0))
    return out


def tiling_seam(image: Image.Image) -> float:
    """Mean edge discontinuity (0 = perfectly seamless). Informational."""
    px = image.load()
    w, h = image.size
    diff = 0
    for y in range(h):
        a, b = px[0, y], px[w - 1, y]
        diff += abs(a[0] - b[0]) + abs(a[1] - b[1]) + abs(a[2] - b[2])
    for x in range(w):
        a, b = px[x, 0], px[x, h - 1]
        diff += abs(a[0] - b[0]) + abs(a[1] - b[1]) + abs(a[2] - b[2])
    return diff / (2 * (w + h) * 3)


def main() -> None:
    THEME_OUT.mkdir(parents=True, exist_ok=True)
    report: dict[str, object] = {"global": {}, "theme": {}}
    for draft, live in GLOBAL.items():
        tile = to_tile(SOURCE / draft)
        tile.save(OUT / live, optimize=True)
        seam = tiling_seam(tile)
        report["global"][live] = {"source": draft, "seam": round(seam, 2)}
        print(f"{live}: seam {seam:.1f}")
    for theme in THEMES:
        for kind in ("wall", "door"):
            name = f"{kind}_{theme}_sector_1080p.png" if kind == "wall" else f"{kind}_{theme}_1080p.png"
            tile = to_tile(SOURCE / name)
            target = THEME_OUT / f"{kind}_{theme}.png"
            tile.save(target, optimize=True)
            seam = tiling_seam(tile) if kind == "wall" else 0.0
            report["theme"][target.name] = {"source": name, "seam": round(seam, 2)}
            print(f"{target.name}: seam {seam:.1f}")
    (SOURCE / "processing-report.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
