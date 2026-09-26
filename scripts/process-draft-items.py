#!/usr/bin/env python3
"""Publish v2 draft items/machinery as 256px runtime sprites.

Live pickups (medkit, ammo cache, amber lantern) overwrite their runtime
slots. Machinery and lights become new static prop sprites wired to engine
kinds (see EnemyDef entries); the engine patch lands alongside.
"""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "public" / "game" / "draft" / "v2" / "items"
OUT = ROOT / "public" / "game"

# draft file -> runtime sprite. None means keep the draft basename.
ITEMS = {
    "medkit.png": "spr_med.png",
    "ammo_cache.png": "spr_ammo.png",
    "lantern_amber.png": "spr_lamp.png",
    "reactor_unit.png": "spr_prop_reactor.png",
    "server_rack.png": "spr_prop_server.png",
    "ac_power_unit.png": "spr_prop_ac.png",
    "ventilation_array.png": "spr_prop_vent.png",
    "worklight_cyan.png": "spr_prop_worklight_cyan.png",
    "worklight_white.png": "spr_prop_worklight_white.png",
    "beacon_warning.png": "spr_prop_beacon.png",
    "lantern_red.png": None,
}


def fit(image: Image.Image) -> Image.Image:
    image = image.convert("RGBA")
    bbox = image.getchannel("A").getbbox()
    if not bbox:
        raise ValueError("item vanished during alpha cleanup")
    image = image.crop(bbox)
    scale = min(230 / image.width, 230 / image.height)
    image = image.resize(
        (max(1, round(image.width * scale)), max(1, round(image.height * scale))),
        Image.Resampling.LANCZOS,
    )
    canvas = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    canvas.alpha_composite(image, ((256 - image.width) // 2, (256 - image.height) // 2))
    return canvas


def main() -> None:
    report = {}
    for draft, runtime in ITEMS.items():
        source = SOURCE / draft
        if not source.exists():
            raise FileNotFoundError(source)
        sprite = fit(Image.open(source))
        if runtime is None:
            print(f"staged {draft} (no runtime slot)")
            report[draft] = {"staged": True}
            continue
        target = OUT / runtime
        sprite.save(target, "PNG", optimize=True)
        report[draft] = {"runtime": str(target.relative_to(ROOT)), "size": [256, 256]}
        print(f"{draft} -> {runtime}")
    (SOURCE / "processing-report.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
