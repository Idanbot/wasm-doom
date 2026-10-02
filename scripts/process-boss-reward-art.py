#!/usr/bin/env python3
"""Publish golden boss cases and strict side-profile arsenal thumbnails."""
from pathlib import Path
import hashlib
import json
import importlib.util
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'art/boss-reward-art-v3'
BOSSES = ('vr9', 'hc9', 'cm9', 'ar6', 'or7', 'gs4', 'cr3', 'sr0', 'ts12', 'ks8', 'mn6')
spec = importlib.util.spec_from_file_location('weapon_cases', ROOT / 'scripts/process-blacksite-weapon-cases.py')
cases = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cases)


def save(image, path):
    path.parent.mkdir(parents=True, exist_ok=True)
    image.putalpha(image.getchannel('A').point(lambda value: 0 if value < 4 else value))
    image.save(path, optimize=True)


def main():
    changed = set()
    for slug in BOSSES:
        case = Image.open(SOURCE / f'{slug}-case.png').convert('RGBA')
        case = case.resize((1024, 768), Image.Resampling.LANCZOS)
        draft = ROOT / f'public/game/draft/v2/cases/case_{slug}.png'
        runtime = ROOT / f'public/game/spr_gun_{slug}.png'
        save(case, draft)
        save(cases.fit(case), runtime)
        side = Image.open(SOURCE / f'{slug}-side.png').convert('RGBA')
        bounds = side.getchannel('A').point(lambda value: 255 if value > 20 else 0).getbbox()
        if not bounds:
            raise ValueError(f'{slug}: empty side view')
        side = side.crop(bounds)
        side.thumbnail((570, 246), Image.Resampling.LANCZOS)
        thumb = Image.new('RGBA', (640, 300))
        thumb.alpha_composite(side, ((640 - side.width) // 2, (300 - side.height) // 2))
        target = ROOT / f'public/game/ui/weapon-thumbs/{slug}.png'
        save(thumb, target)
        changed.update('/' + str(path.relative_to(ROOT / 'public')) for path in (draft, runtime, target))
    for name in ('asset-catalog-data.json', 'draft-assets-v2-data.json'):
        path = ROOT / 'src/lib' / name
        data = json.loads(path.read_text())
        for row in data:
            if row['file'] not in changed:
                continue
            file = ROOT / 'public' / row['file'].lstrip('/')
            digest = hashlib.sha256(file.read_bytes()).hexdigest()
            row.update(hash=digest, shortHash=digest[:8], size=file.stat().st_size)
            row['width'], row['height'] = Image.open(file).size
        path.write_text(json.dumps(data, indent=2) + '\n')
    print(f'Published {len(BOSSES)} golden cases, world pickups and side-profile thumbnails.')


if __name__ == '__main__':
    main()
