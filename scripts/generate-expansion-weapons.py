"""Build only the six expansion sheets and matching pickups from concept cutouts."""
import importlib.util
import json
from pathlib import Path
from PIL import Image, ImageEnhance

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('draft', ROOT / 'scripts/generate-blacksite-draft-v2.py')
draft = importlib.util.module_from_spec(spec)
spec.loader.exec_module(draft)
weapons_path = ROOT / 'src/lib/draft-weapons-v2-data.json'
assets_path = ROOT / 'src/lib/draft-assets-v2-data.json'
weapons = json.loads(weapons_path.read_text())
assets = json.loads(assets_path.read_text())

for slug, name, slot, boss, color, muzzle in draft.SPECS[12:]:
    meta = draft.make_sheet(slug, boss, color, muzzle)
    meta.update(name=name, slot=slot, is_boss=True, alt_fire_name='Boss special',
                alt_fire_desc='Visual draft for review.', alt_available=True,
                ammo_visual='Charge progression')
    weapons = [row for row in weapons if row['id'] != slug] + [meta]
    case = draft.case(slug, boss, color)
    aim = Image.open(draft.OUT / f'weap_{slug}_aim.png').convert('RGBA')
    bbox = aim.getchannel('A').getbbox()
    if bbox:
        aim = aim.crop(bbox)
        aim.thumbnail((480, 260), Image.Resampling.LANCZOS)
        case.alpha_composite(aim, ((case.width - aim.width) // 2,
                                   (case.height - aim.height) // 2 + 50))
    path = draft.OUT / 'cases' / f'case_{slug}.png'
    digest, short = draft.save(case, path)
    assets = [row for row in assets if row['file'] != f'/game/draft/v2/cases/case_{slug}.png']
    assets.append(dict(file=f'/game/draft/v2/cases/case_{slug}.png', name=f'case_{slug}_v2.png',
                       size=path.stat().st_size, hash=digest, shortHash=short,
                       width=1024, height=768, group='items'))
    pickup = case.copy()
    pickup.thumbnail((256, 256), Image.Resampling.LANCZOS)
    pickup.save(ROOT / f'public/game/spr_gun_{slug}.png')
    thumb = Image.new('RGBA', (640, 300))
    gun = Image.open(draft.OUT / f'weap_{slug}_aim.png').convert('RGBA')
    bbox = gun.getchannel('A').getbbox()
    gun = gun.crop(bbox)
    gun.thumbnail((580, 285), Image.Resampling.LANCZOS)
    if slug == 'sr0':
        gun = ImageEnhance.Brightness(gun).enhance(1.55)
    thumb.alpha_composite(gun, ((640 - gun.width) // 2, (300 - gun.height) // 2))
    thumb.save(ROOT / f'public/game/ui/weapon-thumbs/{slug}.png')
    print(f'{slug}: {meta["sheet_short_hash"]}')

weapons_path.write_text(json.dumps(weapons, indent=2) + '\n')
assets_path.write_text(json.dumps(assets, indent=2) + '\n')
