#!/usr/bin/env python3
"""Publish the existing weapon renders under the BLACKSITE weapon IDs.

The first-pass weapon renders already match the game's dark military-sci-fi
style. This pass gives them stable spec names, keeps an HD source archive, and
normalizes every delivered file to RGBA without changing the DOM sheet layout.
"""

from __future__ import annotations

import json
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art" / "source_hd" / "weapons"
RUNTIME = ROOT / "public" / "game"

WEAPONS = {
    "weapon_mk23s": "pistol",
    "weapon_m870k": "shotgun",
    "weapon_vx9": "ripper",
    "weapon_shrike": "lance",
    "weapon_raven": "pyre",
}


def main() -> None:
    SOURCE.mkdir(parents=True, exist_ok=True)
    records = {}
    for spec_id, old_id in WEAPONS.items():
        short = spec_id.removeprefix("weapon_")
        entries = {}
        for animation, suffix in (("idle", ""), ("fire", "_fire"), ("reload", "_reload")):
            source_path = RUNTIME / f"weap_{old_id}{suffix}.png"
            if not source_path.exists():
                raise FileNotFoundError(f"missing existing weapon render: {source_path}")
            image = Image.open(source_path).convert("RGBA")
            # Preserve the current runtime resolution for the DOM compositor;
            # the source archive is a 1024px review master for every animation.
            hd = image.resize((1024, 1024), Image.Resampling.LANCZOS)
            hd_path = SOURCE / f"{short}_{animation}.png"
            hd.save(hd_path, format="PNG", optimize=False)
            runtime_path = RUNTIME / f"weap_{short}{suffix}.png"
            image.save(runtime_path, format="PNG", optimize=False)
            entries[animation] = {
                "source": str(hd_path.relative_to(ROOT)),
                "runtime": str(runtime_path.relative_to(ROOT)),
                "runtimeSize": list(image.size),
            }
        records[spec_id] = entries
    (SOURCE / "processing-report.json").write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(records, indent=2))


if __name__ == "__main__":
    main()
