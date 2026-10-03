#!/usr/bin/env python3
"""Index the native campaign expansion and refresh changed asset hashes."""
import hashlib,json
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
roster=json.loads((ROOT/'art/campaign25/specs.json').read_text())
def info(file):
 p=ROOT/'public'/file.lstrip('/'); data=p.read_bytes(); h=hashlib.sha256(data).hexdigest()
 with Image.open(p) as im:w,height=im.size
 return dict(file=file,name=p.name,size=len(data),hash=h,shortHash=h[:8],width=w,height=height)
for name in ['asset-catalog-data','draft-assets-v2-data']:
 p=ROOT/f'src/lib/{name}.json';rows=json.loads(p.read_text()); indexed={r['file']:r for r in rows}
 files=[]
 for b in roster:
  w=b['weapon']; files += [f'/game/draft/v2/cases/case_{w}.png',f'/game/spr_gun_{w}.png',f'/game/ui/weapon-thumbs/{w}.png']
  files += [f"/game/enemy_{b['slug']}_{state}.png" for state in ['idle','move','pain','fire','reload','dead','special']]
 files += ['/game/'+str(p.relative_to(ROOT/'public/game')) for p in (ROOT/'public/game/fx25').glob('*.png')]
 files += ['/game/'+str(p.relative_to(ROOT/'public/game')) for folder in ['sectors','projectiles'] for p in (ROOT/'public/game'/folder).rglob('*') if p.suffix in ['.png','.webp']]
 for file in files:
  if name=='draft-assets-v2-data' and '/cases/' not in file and '/fx25/' not in file and '/sectors/' not in file and '/projectiles/' not in file:continue
  indexed.setdefault(file,{}).update(info(file))
  if name=='draft-assets-v2-data':indexed[file]['group']='items' if '/cases/' in file or '/prop_' in file else 'textures' if '/sectors/' in file else 'fx'
 for row in indexed.values():
  if row['file'].endswith('.png') and (ROOT/'public'/row['file'].lstrip('/')).exists():row.update(info(row['file']))
 p.write_text(json.dumps(list(indexed.values()),indent=2)+'\n')
p=ROOT/'src/lib/draft-weapons-v2-data.json';rows=json.loads(p.read_text());indexed={r['id']:r for r in rows}
for i,b in enumerate(roster):
 w=b['weapon'];sheet=info(f'/game/draft/v2/weap_{w}_5x5.png');aim=info(f'/game/draft/v2/weap_{w}_aim.png')
 row=dict(id=w,name=b['gun'],slot=19+i,is_boss=True,cell_w=512,cell_h=384,cols=5,rows=5,frames_total=25,alt_fire_name=b['ability'],alt_fire_desc='Primary attack: '+b['ability'],alt_available=False,ammo_visual='Full, half, low, empty feed and removed feed; mechanical reload and pickup sequence.')
 for label,a in [('sheet',sheet),('aim',aim),('gen_master',aim)]:row.update({label+'_file':a['file'],label+'_hash':a['hash'],label+'_short_hash':a['shortHash']})
 indexed[w]=row
p.write_text(json.dumps(list(indexed.values()),indent=2)+'\n')
print('Catalog indexes 33 weapons and all expansion assets')
