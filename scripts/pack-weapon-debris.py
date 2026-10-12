"""Pack native-alpha reload debris into padded, four-pose runtime sheets."""
from pathlib import Path
import hashlib
import json
from PIL import Image

root = Path(__file__).resolve().parent.parent
source = root / 'art/weapon-debris-v1/source.png'
image = Image.open(source).convert('RGBA')
assets = []
for row, name in enumerate(('fuel', 'acid', 'specimen', 'pressure')):
    sheet = Image.new('RGBA', (1024, 1024))
    for column in range(4):
        tile = image.crop((column * image.width // 4, row * image.height // 4,
                           (column + 1) * image.width // 4, (row + 1) * image.height // 4))
        box = tile.getchannel('A').getbbox()
        if not box:
            raise ValueError(f'Empty debris pose: {name}/{column}')
        tile = tile.crop(box)
        tile.thumbnail((400, 400), Image.Resampling.LANCZOS)
        sheet.paste(tile, ((column % 2) * 512 + (512 - tile.width) // 2,
                           (column // 2) * 512 + (512 - tile.height) // 2))
    target = root / f'public/game/fx/{name}-debris-hd.png'
    sheet.save(target)
    assets.append({'file': str(target.relative_to(root)), 'sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
                   'size': [1024, 1024], 'grid': [2, 2], 'cell': [512, 512]})
(root / 'art/weapon-debris-v1/manifest.json').write_text(json.dumps({'provider': 'OpenAI imagegen',
    'source': str(source.relative_to(root)), 'references': ['hx8', 'cm9', 'my9', 'lv8'], 'assets': assets}, indent=2) + '\n')
