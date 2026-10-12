"""Pack native-alpha, weapon-specific feed sheets and axial projectile sprites."""
from pathlib import Path
import hashlib
import json
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def fit(source, size, content):
    bounds = source.getchannel('A').getbbox()
    if not bounds:
        raise ValueError('Generated artwork has no visible pixels')
    image = source.crop(bounds)
    image.thumbnail((content, content), Image.Resampling.LANCZOS)
    output = Image.new('RGBA', (size, size))
    output.paste(image, ((size - image.width) // 2, (size - image.height) // 2))
    return output

for folder, category in [('weapon-feeds-v2', 'feed'), ('projectiles-v4', 'projectile')]:
    records = []
    for job in json.loads((ROOT / 'art' / folder / 'prompts.json').read_text()):
        source = ROOT / job['source']
        if not source.exists():
            raise FileNotFoundError(source)
        image = Image.open(source).convert('RGBA')
        if image.getchannel('A').getextrema()[0] != 0:
            raise ValueError(f'Artwork lacks native transparency: {source}')
        if category == 'feed':
            output = Image.new('RGBA', (1024, 1024))
            for frame in range(4):
                x = frame % 2
                y = frame // 2
                tile = image.crop((x * image.width // 2, y * image.height // 2,
                                   (x + 1) * image.width // 2, (y + 1) * image.height // 2))
                output.paste(fit(tile, 512, 400), (x * 512, y * 512))
        else:
            output = fit(image, 1024, 800)
        target = ROOT / job['file']
        target.parent.mkdir(parents=True, exist_ok=True)
        output.save(target)
        records.append({**job, 'sha256': digest(target), 'source_sha256': digest(source),
                        'size': [1024, 1024], 'grid': [2, 2] if category == 'feed' else [1, 1],
                        'padding': 56 if category == 'feed' else 112})
    (ROOT / 'art' / folder / 'manifest.json').write_text(json.dumps({
        'provider': 'OpenAI imagegen (built-in)', 'assets': records}, indent=2) + '\n')
    print(f'{folder}: packed {len(records)} assets')
