"""Pack generated boss cutouts into the engine's seven 2x2 animation atlases."""

from pathlib import Path
from PIL import Image, ImageEnhance

ROOT = Path(__file__).resolve().parents[1]
SOURCES = {
    "oracle": ROOT / "art/bosses/oracle-source.png",
    "gravemind": ROOT / "art/bosses/gravemind-source.png",
    "archivist": ROOT / "art/bosses/archivist-source.png",
}
ACTIONS = ("idle", "move", "pain", "fire", "reload", "dead", "special")


def cutout(source: Path) -> Image.Image:
    original = Image.open(source).convert("RGBA")
    alpha = original.getchannel("A")
    bounds = alpha.point(lambda value: 255 if value > 12 else 0).getbbox()
    if bounds is None:
        raise ValueError(f"{source} has no visible subject")
    subject = original.crop(bounds)
    subject.thumbnail((109, 112), Image.Resampling.LANCZOS)
    return subject


def pack(name: str, source: Path) -> None:
    subject = cutout(source)
    firing = cutout(source.with_name(f"{name}-fire-source.png"))
    charging = cutout(source.with_name(f"{name}-special-source.png"))
    for action in ACTIONS:
        sheet = Image.new("RGBA", (256, 256))
        for frame in range(4):
            source_pose = firing if action == "fire" and frame in (1, 2) else charging if action == "special" and frame in (1, 2) else subject
            pose = source_pose.copy()
            if action == "pain":
                rgb = ImageEnhance.Color(pose.convert("RGB")).enhance(0.7)
                pose = Image.merge("RGBA", (*rgb.split(), pose.getchannel("A")))
            elif action == "dead":
                pose = pose.rotate(66 + frame * 4, resample=Image.Resampling.BICUBIC, expand=True)
                pose.thumbnail((116, 84), Image.Resampling.LANCZOS)
            elif action == "special":
                rgb = ImageEnhance.Brightness(pose.convert("RGB")).enhance(1.1 + frame * 0.1)
                pose = Image.merge("RGBA", (*rgb.split(), pose.getchannel("A")))
            dx = (0, -2, 1, 2)[frame] if action == "move" else (0, 1, 0, -1)[frame]
            dy = (1, 0, -1, 0)[frame] if action in ("idle", "move") else 0
            if action == "fire":
                dx -= frame * 2
            if action == "reload":
                dx += (0, 3, -3, 0)[frame]
            x = (frame % 2) * 128 + (128 - pose.width) // 2 + dx
            y = (frame // 2) * 128 + 121 - pose.height + dy
            sheet.alpha_composite(pose, (x, y))
        sheet.save(ROOT / f"public/game/enemy_{name}_{action}.png", optimize=True)


for boss_name, boss_source in SOURCES.items():
    pack(boss_name, boss_source)
