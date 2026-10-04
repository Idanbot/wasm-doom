#!/usr/bin/env python3
"""Publish boss materials with periodic edges, keeping native sources unchanged."""
from pathlib import Path
from PIL import Image,ImageDraw
import json,hashlib,numpy as np
ROOT=Path(__file__).resolve().parent.parent
source=ROOT/'art/boss-arenas-v1';out=ROOT/'public/game/boss-arenas';out.mkdir(parents=True,exist_ok=True)
plan=json.loads((source/'specs.json').read_text());catalog_path=ROOT/'src/lib/asset-catalog-data.json';catalog=json.loads(catalog_path.read_text())
def tile(image,size):
 a=np.asarray(image.resize((size,size),Image.Resampling.LANCZOS).convert('RGBA')).astype(float);a[:,:,3]=255
 for axis in (0,1):
  width=max(6,size//32)
  for i in range(width):
   lo=[slice(None)]*3;hi=[slice(None)]*3;lo[axis]=i;hi[axis]=-1-i
   left=a[tuple(lo)].copy();right=a[tuple(hi)].copy();mid=(left+right)/2;weight=1-i/width
   a[tuple(lo)]=left*(1-weight)+mid*weight;a[tuple(hi)]=right*(1-weight)+mid*weight
 return Image.fromarray(a.clip(0,255).astype('uint8'))
report=[];board=Image.new('RGB',(5*258,5*290),'#0a1014');draw=ImageDraw.Draw(board)
for i,b in enumerate(plan['bosses']):
 master=source/(b['slug']+'-source.png');im=Image.open(master)
 runtime=out/(b['slug']+'.png');tile(im,256).save(runtime,optimize=True)
 hd=out/'hd'/(b['slug']+'.webp');hd.parent.mkdir(exist_ok=True);tile(im,1024).save(hd,lossless=True,method=4)
 for path in (runtime,hd):
  file='/'+str(path.relative_to(ROOT/'public'));blob=path.read_bytes();h=hashlib.sha256(blob).hexdigest();row=next((x for x in catalog if x['file']==file),None)
  if row is None:row={'file':file};catalog.append(row)
  size=1024 if path.suffix=='.webp' else 256
  row.update(name=path.name,size=len(blob),hash=h,shortHash=h[:8],width=size,height=size,group='textures',seamless=True,level=b['level'],boss=b['boss'])
 x=i%5*258;y=i//5*290;board.paste(tile(im,256).convert('RGB'),(x,y));draw.text((x+4,y+260),f"{b['level']:02} {b['boss']}",fill='#edf2ee')
 report.append({'level':b['level'],'slug':b['slug'],'source':str(master.relative_to(ROOT)),'sourceDimensions':list(im.size),'sourceHash':hashlib.sha256(master.read_bytes()).hexdigest(),'runtime':str(runtime.relative_to(ROOT)),'hd':str(hd.relative_to(ROOT))})
(source/'generation.json').write_text(json.dumps(report,indent=2)+'\n');board.save(source/'preview.jpg',quality=94)
catalog_path.write_text(json.dumps(catalog,indent=2)+'\n');print(f'Packed {len(report)} distinct seamless boss materials.')
