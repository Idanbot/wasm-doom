"""Refresh catalog metadata for the ten-boss expansion's delivered files."""
import hashlib
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
path = ROOT / 'src/lib/asset-catalog-data.json'
rows = json.loads(path.read_text())
rows = [row for row in rows if row['file'] not in {
    '/game/music/bgm-menu.ogg', '/game/music/bgm-remix.ogg', '/game/music/boss.ogg',
}]
index = {row['file']: row for row in rows}
files = []
for slug in ('or7', 'gs4', 'cr3', 'sr0', 'ts12', 'ks8', 'mn6'):
    files += [
        f'game/draft/v2/weap_{slug}_5x5.png',
        f'game/draft/v2/weap_{slug}_aim.png',
        f'game/draft/v2/masters/{slug}.png',
        f'game/draft/v2/cases/case_{slug}.png',
        f'game/spr_gun_{slug}.png',
        f'game/ui/weapon-thumbs/{slug}.png',
    ]
for boss in ('halcyon', 'relay', 'titan', 'kest', 'mnemosyne'):
    files += [f'game/enemy_{boss}_{state}.png' for state in
              ('idle', 'move', 'pain', 'fire', 'reload', 'dead', 'special')]
    files += [f'game/voices/{boss}-{n}.mp3' for n in range(1, 7)]
for boss in ('veyran', 'hecate', 'chimera', 'oracle', 'gravemind', 'archivist',
             'halcyon', 'relay', 'titan', 'kest', 'mnemosyne'):
    files.append(f'game/voices/boss-{boss}-v2.mp3')
for boss in ('oracle', 'gravemind', 'halcyon', 'relay', 'titan', 'kest', 'mnemosyne'):
    files.append(f'game/voices/boss-{boss}.mp3')
files += ['game/music/menu.mp3', 'game/music/bgm-remix.mp3', 'game/music/boss.mp3']

for name in files:
    source = ROOT / 'public' / name
    if not source.exists():
        raise FileNotFoundError(source)
    data = source.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    url = '/' + name
    row = index.setdefault(url, {'file': url, 'name': source.name})
    row.update(size=len(data), hash=digest, shortHash=digest[:8])
    if source.suffix == '.png':
        with Image.open(source) as im:
            row.update(width=im.width, height=im.height)
    if row not in rows:
        rows.append(row)

path.write_text(json.dumps(rows, indent=2) + '\n')
print(f'catalog refreshed: {len(files)} expansion files')
