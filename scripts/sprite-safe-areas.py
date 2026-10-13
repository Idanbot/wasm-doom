#!/usr/bin/env python3
"""Publish/check reviewed frame bounds, transparent gutters and approved poses.

Art changes require an intentional --update after reviewing the new artwork.
This command does not crop, repack or modify any bitmap.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / 'art/sprite-safe-areas.json'
THRESHOLD = 10


def measure(image, grid, anchors=()):
    cols, rows = grid
    width, height = image.size
    assert width % cols == 0 and height % rows == 0, 'non-integral frame dimensions'
    cw, ch = width // cols, height // rows
    records = []
    for frame in range(cols * rows):
        x, y = frame % cols * cw, frame // cols * ch
        cell = image.crop((x, y, x + cw, y + ch)).convert('RGBA')
        bounds = cell.getchannel('A').point(lambda alpha: 255 if alpha > THRESHOLD else 0).getbbox()
        assert bounds, f'frame {frame}: blank silhouette'
        left, top, right, bottom = bounds
        gutters = dict(left=left, top=top, right=cw-right, bottom=ch-bottom)
        for side, gutter in gutters.items():
            assert side in anchors or gutter >= 2, f'frame {frame}: {side} gutter {gutter} < 2'
        records.append(dict(frame=frame, bounds=list(bounds), gutters=gutters,
                            rgba_sha256=hashlib.sha256(cell.tobytes()).hexdigest()))
    return [cw, ch], records


def specifications():
    specs = {}
    replacements = dict(re.findall(r'"([^"\n]+)": "([^"\n]+)"', (ROOT / "src/game/projectile-art.ts").read_text()))
    weapons = json.loads((ROOT / 'src/lib/draft-weapons-v2-data.json').read_text())
    for weapon in weapons:
        specs['public' + weapon['sheet_file']] = ([5, 5], ['right', 'bottom'], weapon)
    for path in sorted((ROOT / 'public/game').glob('enemy_*_*.png')):
        specs[str(path.relative_to(ROOT))] = ([2, 2], [], None)
    for path in sorted((ROOT / 'public/game/projectiles').rglob('*.png')):
        specs[str(path.relative_to(ROOT))] = ([1, 1], [], None)
    for name in ['casing', 'shell', 'fuel-debris', 'acid-debris', 'specimen-debris', 'pressure-debris']:
        specs[f'public/game/fx/{name}-hd.png'] = ([2, 2], [], None)
    for path in sorted((ROOT / 'public/game/fx/feeds').glob('*.png')):
        specs[str(path.relative_to(ROOT))] = ([2, 2], [], None)
    for name in ['rocket-forward', 'ion-bolt', 'cryo-shard', 'spore-cluster', 'phase-orb',
                 'ember-impact', 'pressure-impact', 'magnetic-impact', 'solar-impact']:
        grid = [2, 2] if name == 'rocket-forward' or name.endswith('-impact') else [1, 1]
        path = f'/game/fx25/{name}.png'
        specs['public' + replacements.get(path, path)] = (grid, [], None)
    for name, grid in [('spr_ball', 1), ('spr_ordnance', 2), ('spr_impact', 2), ('spr_muzzle', 2)]:
        specs[f'public/game/{name}.png'] = ([grid, grid], [], None)
    return specs


def snapshot():
    assets = []
    for path, (grid, anchors, weapon) in sorted(specifications().items()):
        raw = (ROOT / path).read_bytes()
        with Image.open(ROOT / path) as source:
            image = source.convert('RGBA')
        try:
            cell, frames = measure(image, grid, anchors)
            approved = None
            if weapon:
                assert image.size == (2560, 1920), 'weapon sheet must remain 5x5'
                with Image.open(ROOT / ('public' + weapon['aim_file'])) as reference:
                    approved = hashlib.sha256(reference.convert('RGBA').tobytes()).hexdigest()
                for index in [0, 9, 14, 19, 24]:
                    assert frames[index]['rgba_sha256'] == approved, f'approved aim changed in cell {index}'
                assert frames[5]['rgba_sha256'] == frames[3]['rgba_sha256'], 'dry fire must stay disabled'
        except AssertionError as error:
            raise AssertionError(f'{path}: {error}') from error
        entry = dict(file=path, sha256=hashlib.sha256(raw).hexdigest(), grid=grid,
                     cell=cell, anchored_edges=anchors, frames=frames)
        if approved:
            entry['approved_aim_rgba_sha256'] = approved
        assets.append(entry)
    return dict(version=1, alpha_threshold=THRESHOLD, minimum_gutter=2,
                bounds_format='left, top, right-exclusive, bottom-exclusive', assets=assets)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--update', action='store_true')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    current = snapshot()
    if args.update:
        CONTRACT.write_text(json.dumps(current, separators=(',', ':')) + '\n')
    else:
        approved = json.loads(CONTRACT.read_text())
        assert current == approved, 'sprite contracts changed; review art and explicitly republish with --update'
    print(f"[sprite-safe-areas] {len(current['assets'])} assets / "
          f"{sum(len(a['frames']) for a in current['assets'])} frames verified")


if __name__ == '__main__':
    main()
