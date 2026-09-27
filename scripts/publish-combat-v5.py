#!/usr/bin/env python3
"""Publish forward-facing ordnance and the repaired crate from reviewed masters."""
from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art/source_hd/combat_v5"
DRAFT = ROOT / "public/game/draft/v2/fx"
RUNTIME = ROOT / "public/game"


def record(path: Path) -> dict:
    with Image.open(path) as image:
        width, height = image.size
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    return {"size": path.stat().st_size, "hash": digest, "shortHash": digest[:8], "width": width, "height": height}


def update_catalog(path: Path, assets: list[Path], *, group: str | None = None) -> None:
    rows = json.loads(path.read_text())
    for asset in assets:
        file = "/" + str(asset.relative_to(ROOT / "public"))
        found = next((row for row in rows if row["file"] == file), None)
        if found is None:
            found = {"file": file, "name": asset.stem + "_v2.png"}
            if group:
                found["group"] = group
            rows.append(found)
        found.update(record(asset))
    path.write_text(json.dumps(rows, indent=2) + "\n")


def main() -> None:
    fx = []
    for name in ("projectile_ball", "projectile_rocket", "projectile_rail", "projectile_acid"):
        target = DRAFT / f"{name}.png"
        Image.open(SOURCE / f"{name}.png").convert("RGBA").resize((512, 512), Image.Resampling.LANCZOS).save(target, optimize=True)
        fx.append(target)
    subprocess.run(["python3", "scripts/process-blacksite-projectile-assets.py"], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)

    crate = Image.open(SOURCE / "crate.png").convert("RGBA")
    crate.thumbnail((236, 236), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (256, 256))
    canvas.alpha_composite(crate, ((256 - crate.width) // 2, (256 - crate.height) // 2))
    if canvas.getpixel((128, 128))[3] < 240:
        raise ValueError("crate center is unexpectedly transparent")
    canvas.save(RUNTIME / "spr_crate.png", optimize=True)

    update_catalog(ROOT / "src/lib/draft-assets-v2-data.json", fx, group="particles")
    update_catalog(ROOT / "src/lib/asset-catalog-data.json", [RUNTIME / name for name in ("spr_ball.png", "spr_ordnance.png", "spr_crate.png")])
    print("published forward-facing ordnance, plasma bolt, rail bolt and opaque crate")


if __name__ == "__main__":
    main()
