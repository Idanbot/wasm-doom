#!/usr/bin/env python3
"""Pack reviewed nine-pose sources into the existing 25-cell runtime contract.

Approved aim PNGs are never rewritten. Source pose boards and the generation
manifest live outside public/ so unused high-resolution artwork isn't deployed.
"""
from pathlib import Path
import hashlib
from io import BytesIO
import importlib.util
import json
import sys

import numpy as np
from PIL import Image
from scipy.ndimage import label

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ROOT / 'art/weapon-animations-v3'
OUT = ROOT / 'public/game/draft/v2'
CELL = (512, 384)
spec = importlib.util.spec_from_file_location('weapon_art', ROOT / 'scripts/generate-blacksite-draft-v2.py')
art = importlib.util.module_from_spec(spec)
spec.loader.exec_module(art)

# Barrel tips measured in the approved aim renders, before recoil transforms.
MUZZLES = {'vr9': (270, 214), 'hc9': (275, 185), 'cm9': (219, 200),
           'ar6': (296, 189), 'or7': (214, 100), 'gs4': (216, 102),
           'cr3': (221, 115), 'sr0': (239, 112), 'ts12': (246, 104),
           'ks8': (214, 112), 'mn6': (260, 180)}


def poses(slug):
    board = Image.open(SOURCES / f'{slug}-poses.png').convert('RGBA')
    panels = []
    for index in range(9):
        x, y = index % 3, index // 3
        box = (round(x * board.width / 3), round(y * board.height / 3),
               round((x + 1) * board.width / 3), round((y + 1) * board.height / 3))
        panel = board.crop(box)
        # Discard only panel boundary pixels, which can contain divider remnants.
        alpha = np.asarray(panel.getchannel('A')).copy()
        alpha[:2] = alpha[-2:] = 0
        alpha[:, :2] = alpha[:, -2:] = 0
        components, _ = label(alpha > 10)
        sizes = np.bincount(components.ravel())
        sizes[0] = 0
        # A neighboring panel can leave a thin disconnected sliver at an edge.
        # Keep substantial detached hands/cells, discard only small fragments.
        keep = sizes >= max(1200, sizes.max() * .035)
        alpha[~keep[components]] = 0
        panel.putalpha(Image.fromarray(alpha))
        panels.append(panel)
    # One scale and placement for the entire sequence prevents per-cell jitter.
    scale = min(318 / panels[0].width, 260 / panels[0].height)
    frames = []
    for index, panel in enumerate(panels):
        panel = panel.resize((round(panel.width * scale), round(panel.height * scale)), Image.Resampling.LANCZOS)
        frame = Image.new('RGBA', CELL)
        drop = (70, 30, 0)[index] if index < 3 else 0
        frame.alpha_composite(panel, (512 - panel.width, 384 - panel.height + drop))
        for _ in range(12):
            if np.asarray(frame.getchannel('A'))[184:200, 248:264].max() < 8:
                break
            frame = art.frame(frame, 4, 6)
        frames.append(frame)
    return frames


def charge(base, fraction, color, slug):
    """Dim contiguous sections of visible emissive hardware, not the chassis."""
    if slug in ('ks8', 'ts12', 'gs4'):
        return base.copy()  # Opaque ballistic/pressure feeds hide internal rounds.
    a = np.asarray(base).copy()
    rgb = a[:, :, :3].astype(float)
    intensity = rgb.max(axis=2)
    saturation = intensity - rgb.min(axis=2)
    target = np.array(color, dtype=float)
    target /= np.linalg.norm(target)
    direction = rgb / np.maximum(np.linalg.norm(rgb, axis=2, keepdims=True), 1)
    colored = ((direction @ target) > .98) & (saturation > 55) & (intensity > 110) & (a[:, :, 3] > 150)
    yy, xx = np.mgrid[:384, :512]
    if slug == 'cm9':
        colored &= (xx > 355) & (yy > 200) & (yy < 310)
        off = colored & (yy < 290 - 70 * fraction)
    else:
        positions = xx[colored]
        cutoff = np.quantile(positions, 1 - fraction) if positions.size else 0
        off = colored & (xx <= cutoff)
    a[:, :, :3][off] = (rgb[off] * .24).astype(np.uint8)
    return Image.fromarray(a)


def build(slug, color):
    base = Image.open(OUT / f'weap_{slug}_aim.png').convert('RGBA')
    p = poses(slug)
    frames = [base.copy(), charge(base, .5, color, slug), charge(base, .15, color, slug), p[7], p[8]]
    # Dry fire is disabled: the reserved cell exactly matches empty-mag aim.
    frames += [p[7].copy(), *p[:3], base.copy()]
    frames += [*p[3:7], base.copy()]
    for dx, dy, angle, flash in [(0, -3, .8, .75), (4, -9, 2.5, .45), (3, -6, 1.8, 0), (1, -2, .6, 0), (0, 0, 0, 0)]:
        f = art.frame(base, dx, dy, angle)
        tip = art.rotated_point(MUZZLES[slug], angle)
        if flash:
            art.flare(f, (tip[0] + dx, tip[1] + dy), color, flash,
                      kind='signal' if slug == 'sr0' else None)
        frames.append(f)
    # Row four is reserved artwork, not a claim of an implemented alt-fire.
    frames += [f.copy() for f in frames[15:20]]
    sheet = Image.new('RGBA', (2560, 1920))
    for index, f in enumerate(frames):
        sheet.alpha_composite(f, ((index % 5) * 512, (index // 5) * 384))
    path = OUT / f'weap_{slug}_5x5.png'
    encoded = BytesIO()
    sheet.save(encoded, format='PNG', optimize=True)
    contents = encoded.getvalue()
    # Avoid needless asset reloads when a regeneration produced identical art.
    if not path.exists() or path.read_bytes() != contents:
        path.write_bytes(contents)
    return hashlib.sha256(contents).hexdigest()


def main():
    data_path = ROOT / 'src/lib/draft-weapons-v2-data.json'
    data = json.loads(data_path.read_text())
    chosen = set(sys.argv[1:]) or set(MUZZLES)
    for slug, _, _, boss, color, _ in art.SPECS:
        if not boss or slug not in chosen:
            continue
        digest = build(slug, color)
        row = next(row for row in data if row['id'] == slug)
        row.update(sheet_hash=digest, sheet_short_hash=digest[:8], animation_version=3,
                   ammo_visual='Approved full aim; depleted charge states; generated empty feed and feed removed poses.')
        print(slug, digest[:8])
    data_path.write_text(json.dumps(data, indent=2) + '\n')
    catalog_path = ROOT / 'src/lib/asset-catalog-data.json'
    catalog = json.loads(catalog_path.read_text())
    for row in catalog:
        if any(row['file'] == f'/game/draft/v2/weap_{slug}_5x5.png' for slug in chosen):
            path = ROOT / 'public' / row['file'].lstrip('/')
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            row.update(hash=digest, shortHash=digest[:8], size=path.stat().st_size)
    catalog_path.write_text(json.dumps(catalog, indent=2) + '\n')


if __name__ == '__main__':
    main()
