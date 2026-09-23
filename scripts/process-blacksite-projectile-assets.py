#!/usr/bin/env python3
"""Build the runtime projectile/effect textures from the 1024px VFX master.

The generator produces one single-cell hostile plasma projectile and three
2x2 sheets consumed by the engine's existing sprite slots. Keeping the split
deterministic means the source master stays reviewable while the runtime
atlas remains a small, predictable contract.
"""

from __future__ import annotations

import json
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art" / "source_hd" / "projectiles"
MASTER = SOURCE / "projectile_effects_4x4.png"
CELLS = SOURCE / "cells"
RUNTIME = ROOT / "public" / "game"


# Source grid coordinates (column, row). Each runtime sheet uses four 256px
# cells laid out in the same 2x2 order the Rust renderer samples.
CELL_COORDS = {
    "plasma_bolt": (0, 0),
    "incendiary_projectile": (1, 0),
    "lance_beam": (2, 0),
    "acid_seeker": (3, 0),
    "pistol_muzzle": (0, 1),
    "shotgun_muzzle": (1, 1),
    "smg_muzzle": (2, 1),
    "rifle_muzzle": (3, 1),
    "metal_impact": (0, 2),
    "plasma_impact": (1, 2),
    "explosive_impact": (2, 2),
    "acid_impact": (3, 2),
    "fire_patch": (0, 3),
    "smoke_puff": (1, 3),
    "electric_sparks": (2, 3),
    "casing_burst": (3, 3),
}


SHEETS = {
    "spr_ordnance.png": ("plasma_bolt", "incendiary_projectile", "lance_beam", "acid_seeker"),
    "spr_muzzle.png": ("pistol_muzzle", "shotgun_muzzle", "smg_muzzle", "rifle_muzzle"),
    "spr_impact.png": ("metal_impact", "plasma_impact", "explosive_impact", "acid_impact"),
    "spr_flame.png": ("fire_patch", "smoke_puff", "electric_sparks", "casing_burst"),
}


def cell(master: Image.Image, name: str) -> Image.Image:
    column, row = CELL_COORDS[name]
    return master.crop((column * 256, row * 256, (column + 1) * 256, (row + 1) * 256)).copy()


def sheet(master: Image.Image, names: tuple[str, str, str, str]) -> Image.Image:
    out = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
    for index, name in enumerate(names):
        out.paste(cell(master, name), ((index & 1) * 256, (index >> 1) * 256))
    return out


def main() -> None:
    if not MASTER.exists():
        raise FileNotFoundError(f"missing generated master: {MASTER}")
    SOURCE.mkdir(parents=True, exist_ok=True)
    CELLS.mkdir(parents=True, exist_ok=True)
    master = Image.open(MASTER).convert("RGBA")
    if master.size != (1024, 1024):
        raise ValueError(f"projectile master must be 1024x1024, got {master.size}")

    cell_records: dict[str, dict[str, object]] = {}
    for name in CELL_COORDS:
        target = CELLS / f"{name}.png"
        cell(master, name).save(target, format="PNG", optimize=False)
        cell_records[name] = {"source": str(target.relative_to(ROOT)), "size": [256, 256]}

    runtime_records: dict[str, dict[str, object]] = {}
    plasma = cell(master, "plasma_bolt")
    plasma.save(RUNTIME / "spr_ball.png", format="PNG", optimize=False)
    runtime_records["spr_ball.png"] = {"cells": ["plasma_bolt"], "size": [256, 256], "sheet": "1x1"}
    for filename, names in SHEETS.items():
        target = RUNTIME / filename
        sheet(master, names).save(target, format="PNG", optimize=False)
        runtime_records[filename] = {"cells": list(names), "size": [512, 512], "sheet": "2x2"}

    report = {
        "source": str(MASTER.relative_to(ROOT)),
        "sourceSize": [1024, 1024],
        "cellSize": [256, 256],
        "cells": cell_records,
        "runtime": runtime_records,
        "alpha": "true RGBA transparency preserved from the generated master",
    }
    (SOURCE / "processing-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
