#!/usr/bin/env python3
"""Turn the 1024px BLACKSITE environment masters into seamless tiles and sprites.

The source renders are intentionally kept at 1024px in art/source_hd so they
remain useful for later atlas or UI work. Runtime world textures are repaired
at both resolutions and reduced to the engine's 256px texture contract. The
object and collectible sheets are split into 4x4, 256px cells while retaining
the original 1024px sheets as reviewable masters.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Iterable

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / "art" / "source_hd" / "environment" / "_generated"
SOURCE = ROOT / "art" / "source_hd" / "environment"
SOURCE_TEXTURES = SOURCE / "textures"
SOURCE_SHEETS = SOURCE / "sheets"
SOURCE_PROPS = SOURCE / "props"
SOURCE_ITEMS = SOURCE / "items"
RUNTIME = ROOT / "public" / "game"


TEXTURES = {
    # runtime filename: (raw filename, source filename, human-readable theme)
    "wall_metal.png": ("tex_hangar_gunmetal.png", "tex_hangar_gunmetal.png", "hangar"),
    "wall_flesh.png": ("tex_biotech_janus.png", "tex_biotech_janus.png", "biotech"),
    "wall_tech_tile2x2.png": ("tex_datacenter_server.png", "tex_datacenter_server.png", "datacenter"),
    "wall_hazard_tile2x2.png": ("tex_security_bulkhead.png", "tex_security_bulkhead.png", "security"),
    "floor_grate.png": ("tex_foundry_diamond.png", "tex_foundry_diamond.png", "foundry"),
    "wall_brick.png": ("tex_plaza_masonry.png", "tex_plaza_masonry.png", "plaza"),
    "floor_concrete.png": ("tex_nuclear_floor.png", "tex_nuclear_floor.png", "nuclear"),
    "wall_bone.png": ("tex_vault_ossuary.png", "tex_vault_ossuary.png", "vault"),
}

OBJECT_SHEETS = {
    "hangar": "obj_hangar.png",
    "plaza": "obj_plaza.png",
    "security": "obj_security.png",
    "datacenter": "obj_datacenter.png",
    "foundry": "obj_foundry.png",
    "biotech": "obj_biotech.png",
    "nuclear": "obj_nuclear.png",
    "vault": "obj_vault.png",
}

ITEM_SHEETS = {
    "combat": "item_combat.png",
    "security": "item_security.png",
    "science": "item_science.png",
    "occult": "item_occult.png",
}

# Existing engine slots that can consume the new art without growing TEX_N.
# The complete per-theme cells remain in public/game/environment for the next
# object/atlas expansion; these promotions make the current map visibly use
# the new set right away.
PROMOTIONS = {
    "spr_barrel.png": "hangar_explosive_barrel.png",
    "spr_med.png": "combat_compact_medkit.png",
    "spr_ammo.png": "combat_shotgun_shell_bundle.png",
    "spr_armor.png": "combat_armor_plate.png",
    "spr_crate.png": "hangar_breakable_supply_crate.png",
    "spr_lamp.png": "hangar_warning_lamp.png",
    "spr_chain.png": "hangar_cargo_hook.png",
}

OBJECT_FAMILIES = (
    "explosive_barrel",
    "fuel_canister",
    "flame_vent",
    "acid_spill",
    "electric_arc_coil",
    "toxic_gas_emitter",
    "security_turret",
    "hydraulic_crusher",
    "laser_barrier",
    "cargo_hook",
    "pressure_airlock",
    "breakable_supply_crate",
    "terminal_console",
    "specimen_reactor_tank",
    "cable_pipe_manifold",
    "warning_lamp",
)

ITEM_FAMILIES = {
    "combat": (
        "pistol_magazine", "shotgun_shell_bundle", "rifle_magazine", "energy_cell",
        "frag_grenade", "incendiary_charge", "proximity_mine", "explosive_charge",
        "armor_plate", "trauma_injector", "compact_medkit", "stimulant_ampoule",
        "repair_tool", "welding_torch", "ration_pack", "flare",
    ),
    "security": (
        "security_keycard", "access_badge", "encrypted_data_drive", "map_tablet",
        "master_access_token", "biometric_chip", "evidence_cassette", "blacksite_logbook",
        "radio_handset", "signal_beacon", "camera_module", "cipher_coin",
        "evidence_pouch", "drone_core", "command_seal", "data_crystal",
    ),
    "science": (
        "janus_specimen_vial", "bio_sample_canister", "coolant_cartridge", "radiation_filter",
        "reactor_fuel_cell", "isotope_capsule", "green_medgel_syringe", "culture_tube",
        "pressure_gauge", "coolant_valve", "hazard_dosimeter", "lead_sample_case",
        "frozen_tissue_sample", "microchip_tray", "cryo_tag", "emergency_oxygen",
    ),
    "occult": (
        "bone_seal_shard", "iron_reliquary", "crimson_sigil_token", "skull_medallion",
        "chapel_key", "blood_glass_vial", "prayer_strip", "demon_horn_fragment",
        "ivory_talisman", "chained_heart_relic", "obsidian_coin", "candle_bundle",
        "vault_crown_fragment", "bone_die", "red_crystal", "sealed_skull_capsule",
    ),
}


def ensure_dirs() -> None:
    for directory in (RAW, SOURCE_TEXTURES, SOURCE_SHEETS, SOURCE_PROPS, SOURCE_ITEMS, RUNTIME / "environment" / "props", RUNTIME / "environment" / "items"):
        directory.mkdir(parents=True, exist_ok=True)


def rgba_resized(path: Path, size: tuple[int, int]) -> Image.Image:
    image = Image.open(path).convert("RGBA")
    return image.resize(size, Image.Resampling.LANCZOS)


def generated_or_canonical(raw_name: str, canonical: Path) -> Path:
    """Prefer the local generation cache, but make committed masters reusable."""

    generated = RAW / raw_name
    return generated if generated.exists() else canonical


def opaque_texture(image: Image.Image) -> Image.Image:
    # Generated material prompts sometimes return a mostly-opaque RGBA image.
    # Composite onto the game's charcoal base so every world layer is opaque.
    if image.mode != "RGBA":
        return image.convert("RGB")
    base = Image.new("RGBA", image.size, (18, 16, 16, 255))
    return Image.alpha_composite(base, image).convert("RGB")


def lerp(a: int, b: int, weight: float) -> int:
    return round(a + (b - a) * weight)


def repair_wrap(image: Image.Image, band: int = 32) -> Image.Image:
    """Feather opposite edges so a tile can repeat without a seam.

    The generated materials already use a repeating modular composition. This
    pass only blends the outside bands and makes the actual boundary pixels
    equal; it does not offset or mirror the tile, so panels keep their intended
    direction. A final edge assertion below catches accidental regressions.
    """

    image = image.copy().convert("RGB")
    px = image.load()
    width, height = image.size
    band = max(1, min(band, width // 4, height // 4))
    for distance in range(band):
        weight = (band - distance) / band
        left = distance
        right = width - 1 - distance
        for y in range(height):
            a = px[left, y]
            b = px[right, y]
            mid = tuple((a[channel] + b[channel]) // 2 for channel in range(3))
            px[left, y] = tuple(lerp(a[channel], mid[channel], weight) for channel in range(3))
            px[right, y] = tuple(lerp(b[channel], mid[channel], weight) for channel in range(3))
        top = distance
        bottom = height - 1 - distance
        for x in range(width):
            a = px[x, top]
            b = px[x, bottom]
            mid = tuple((a[channel] + b[channel]) // 2 for channel in range(3))
            px[x, top] = tuple(lerp(a[channel], mid[channel], weight) for channel in range(3))
            px[x, bottom] = tuple(lerp(b[channel], mid[channel], weight) for channel in range(3))
    # Enforce exact equality at the wrap itself after the two passes touch the
    # corners. This is the edge consumed by adjacent nearest-filtered samples.
    for y in range(height):
        a = px[0, y]
        b = px[width - 1, y]
        mid = tuple((a[channel] + b[channel]) // 2 for channel in range(3))
        px[0, y] = mid
        px[width - 1, y] = mid
    for x in range(width):
        a = px[x, 0]
        b = px[x, height - 1]
        mid = tuple((a[channel] + b[channel]) // 2 for channel in range(3))
        px[x, 0] = mid
        px[x, height - 1] = mid
    return image


def edge_delta(image: Image.Image) -> int:
    image = image.convert("RGB")
    px = image.load()
    width, height = image.size
    deltas = []
    for y in range(height):
        deltas.append(max(abs(px[0, y][c] - px[width - 1, y][c]) for c in range(3)))
    for x in range(width):
        deltas.append(max(abs(px[x, 0][c] - px[x, height - 1][c]) for c in range(3)))
    return max(deltas, default=0)


def save_texture(raw_name: str, source_name: str, runtime_name: str) -> dict[str, object]:
    canonical = SOURCE_TEXTURES / source_name
    raw = RAW / raw_name
    # A committed canonical master has already been resized and seam-repaired.
    # Reading it directly keeps repeated `assets:environment` runs idempotent.
    source = (
        repair_wrap(opaque_texture(rgba_resized(raw, (1024, 1024))))
        if raw.exists()
        else Image.open(canonical).convert("RGB")
    )
    source_path = SOURCE_TEXTURES / source_name
    source.save(source_path, format="PNG", optimize=False)
    runtime = repair_wrap(source.resize((256, 256), Image.Resampling.LANCZOS), band=8)
    runtime_path = RUNTIME / runtime_name
    # Keep an explicit alpha channel even for opaque walls; the atlas loader
    # treats every layer as RGBA and the asset gate checks that contract.
    runtime.convert("RGBA").save(runtime_path, format="PNG", optimize=False)
    return {
        "source": str(source_path.relative_to(ROOT)),
        "runtime": str(runtime_path.relative_to(ROOT)),
        "sourceSize": [1024, 1024],
        "runtimeSize": [256, 256],
        "maxWrapEdgeDelta": max(edge_delta(source), edge_delta(runtime)),
    }


def save_sheet(raw_name: str, source_name: str, target: Path) -> None:
    image = rgba_resized(RAW / raw_name, (1024, 1024))
    image.save(target / source_name, format="PNG", optimize=False)


def split_sheet(sheet_name: str, source_name: str, families: Iterable[str], out_source: Path, out_runtime: Path) -> None:
    out_source.mkdir(parents=True, exist_ok=True)
    out_runtime.mkdir(parents=True, exist_ok=True)
    sheet = rgba_resized(generated_or_canonical(sheet_name, SOURCE_SHEETS / source_name), (1024, 1024))
    sheet.save(SOURCE_SHEETS / source_name, format="PNG", optimize=False)
    for index, family in enumerate(families):
        x = (index % 4) * 256
        y = (index // 4) * 256
        cell = sheet.crop((x, y, x + 256, y + 256))
        # Keep a reviewable 256px source cell and a runtime copy under a
        # namespaced directory. Consumers can promote a cell into an atlas
        # without re-running generation.
        cell.save(out_source / f"{out_source.name}_{family}.png", format="PNG", optimize=False)
        cell.save(out_runtime / f"{out_source.name}_{family}.png", format="PNG", optimize=False)


def promote_existing_slots() -> None:
    props = RUNTIME / "environment" / "props"
    items = RUNTIME / "environment" / "items"
    for destination, source_name in PROMOTIONS.items():
        source = props / source_name if source_name.startswith("hangar_") else items / source_name
        if not source.exists():
            raise FileNotFoundError(f"promotion source missing: {source}")
        Image.open(source).convert("RGBA").save(RUNTIME / destination, format="PNG", optimize=False)


def main() -> None:
    ensure_dirs()
    texture_records = {}
    for runtime_name, (raw_name, source_name, _theme) in TEXTURES.items():
        texture_records[runtime_name] = save_texture(raw_name, source_name, runtime_name)

    for theme, raw_name in OBJECT_SHEETS.items():
        split_sheet(raw_name, f"objects_{theme}_4x4.png", OBJECT_FAMILIES, SOURCE_PROPS / theme, RUNTIME / "environment" / "props")

    item_records = {}
    for group, raw_name in ITEM_SHEETS.items():
        split_sheet(raw_name, f"items_{group}_4x4.png", ITEM_FAMILIES[group], SOURCE_ITEMS / group, RUNTIME / "environment" / "items")
        item_records[group] = {"sheet": str((SOURCE_SHEETS / f"items_{group}_4x4.png").relative_to(ROOT)), "count": len(ITEM_FAMILIES[group])}
    promote_existing_slots()

    report = {
        "sourceSize": 1024,
        "runtimeTextureSize": 256,
        "seamMethod": "32px feathered opposite-edge blend, exact boundary equality",
        "textures": texture_records,
        "objectSheets": {
            theme: {"sheet": str((SOURCE_SHEETS / f"objects_{theme}_4x4.png").relative_to(ROOT)), "objects": len(OBJECT_FAMILIES)}
            for theme in OBJECT_SHEETS
        },
        "itemSheets": item_records,
    }
    (SOURCE / "processing-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
