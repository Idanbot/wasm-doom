#!/usr/bin/env python3
"""Publish reviewed 1024+ source art to the v2 catalog and runtime inputs."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art/source_hd/combat_v4"
DRAFT = ROOT / "public/game/draft/v2"
CATALOG = ROOT / "src/lib/draft-assets-v2-data.json"


def publish(image: Image.Image, path: Path, size: int = 512) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    image = image.convert("RGBA").resize((size, size), Image.Resampling.LANCZOS)
    image.save(path, optimize=True)
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    records = json.loads(CATALOG.read_text())
    replacements = {
        "security_sensor_pylon": "items",
        "projectile_rocket": "fx",
        "projectile_acid": "fx",
        "muzzle_ballistic": "fx",
        "muzzle_suppressed": "fx",
        "muzzle_rocket": "fx",
    }
    atlas_cells = {
        "impact_atlas": ["impact_sparks", "impact_electric", "explosion_core", "impact_acid"],
        "ambient_atlas": ["smoke_muzzle", "smoke_explosion", "shockwave", "impact_shrapnel"],
    }
    output = {}
    for name, kind in replacements.items():
        output[name] = (kind, publish(Image.open(SOURCE / f"{name}.png"), DRAFT / kind / f"{name}.png", 1024 if kind == "items" else 512))
    for atlas_name, names in atlas_cells.items():
        atlas = Image.open(SOURCE / f"{atlas_name}.png").convert("RGBA")
        half_x, half_y = atlas.width // 2, atlas.height // 2
        for index, name in enumerate(names):
            x, y = (index % 2) * half_x, (index // 2) * half_y
            cell = atlas.crop((x, y, x + half_x, y + half_y))
            output[name] = ("fx", publish(cell, DRAFT / "fx" / f"{name}.png"))
    for record in records:
        if record.get("name") == "beacon_warning_v2.png":
            record["name"] = "security_sensor_pylon_v2.png"
            record["file"] = "/game/draft/v2/items/security_sensor_pylon.png"
        stem = Path(record["file"]).stem
        if stem in output:
            digest = output[stem][1]
            record["hash"] = digest
            record["shortHash"] = digest[:8]
            record["size"] = (DRAFT / output[stem][0] / f"{stem}.png").stat().st_size
            record["width"] = record["height"] = 1024 if output[stem][0] == "items" else 512
    if not any(record["name"] == "impact_shrapnel_v2.png" for record in records):
        records.append({
            "file": "/game/draft/v2/fx/impact_shrapnel.png",
            "name": "impact_shrapnel_v2.png",
            "hash": output["impact_shrapnel"][1],
            "shortHash": output["impact_shrapnel"][1][:8],
            "size": (DRAFT / "fx/impact_shrapnel.png").stat().st_size,
            "width": 512,
            "height": 512,
            "group": "particles",
        })
    CATALOG.write_text(json.dumps(records, indent=2) + "\n")
    (DRAFT / "items/beacon_warning.png").unlink(missing_ok=True)
    print(f"published {len(output)} reviewed assets")


if __name__ == "__main__":
    main()
