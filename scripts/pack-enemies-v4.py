#!/usr/bin/env python3
"""Pack real generated poses; never fabricate motion from a single cutout."""
import hashlib,json
from pathlib import Path
import numpy as np
from scipy.ndimage import label, find_objects
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
DIR=ROOT/'art/enemies-v4'
plan=json.loads((DIR/'specs.json').read_text())
catalog_path=ROOT/'src/lib/asset-catalog-data.json'
catalog=json.loads(catalog_path.read_text());catalog_by_file={e['file']:e for e in catalog}

def extract(path, row_seams=()):
 image=Image.open(path).convert('RGBA'); a=np.asarray(image.getchannel('A')).copy()
 # A glow can bridge adjacent columns. Treat the authored column boundary as
 # a packing seam so two genuine poses cannot become one connected component.
 for column in range(1,4): a[:,round(image.width*column/4)] = 0
 for y in row_seams: a[y,:] = 0
 labels,n=label(a>24); sizes=np.bincount(labels.ravel()); sizes[0]=0
 objects=find_objects(labels)
 # Each generated subject is connected; tiny detached equipment/effects are
 # reassigned to the nearest figure rather than discarded or keyed by color.
 major=sorted(range(1,n+1),key=lambda i:sizes[i],reverse=True)[:28]
 if len(major)!=28 or min(sizes[i] for i in major)<400:raise ValueError(f'{path}: expected 28 substantial figures')
 boxes=[]
 for i in major:
  ys,xs=objects[i-1];boxes.append([xs.start,ys.start,xs.stop,ys.stop,i])
 boxes.sort(key=lambda b:(b[1]+b[3])/2)
 boxes=[b for j in range(0,28,4) for b in sorted(boxes[j:j+4],key=lambda b:(b[0]+b[2])/2)]
 owner=np.zeros(n+1,dtype=np.int32)
 for k,b in enumerate(boxes):owner[b[4]]=k+1
 for i in range(1,n+1):
  if owner[i] or sizes[i]<max(60, max(sizes[j] for j in major)*.004):continue
  ys,xs=objects[i-1];x,y=(xs.start+xs.stop)/2,(ys.start+ys.stop)/2
  column=min(3,int(x/image.width*4))
  distances=[(max(b[0]-x,0,x-b[2])**2+max(b[1]-y,0,y-b[3])**2)
             if k%4==column else float('inf') for k,b in enumerate(boxes)]
  k=int(np.argmin(distances))
  if distances[k]<min(image.size)**2*.014:owner[i]=k+1
 poses=[]
 for k,b in enumerate(boxes):
  mask=owner[labels]==k+1
  pixels=np.asarray(image).copy();pixels[:,:,3]=np.where(mask,a,0)
  pose=Image.fromarray(pixels);bounds=pose.getchannel('A').getbbox();poses.append(pose.crop(bounds))
 return poses

report=[]; preview=Image.new('RGB',(8*144,6*160),(23,28,32));draw=ImageDraw.Draw(preview)
for e in plan['enemies']:
 path=DIR/f"{e['slug']}-source.png"
 if not path.exists():continue
 poses=extract(path,e.get("source_row_seams",()))
 scale=min(112/max(p.height for p in poses),116/max(p.width for p in poses))
 for row,action in enumerate(plan['animations']):
  sheet=Image.new('RGBA',(256,256))
  for f in range(4):
   p=poses[row*4+f];p=p.resize((max(1,round(p.width*scale)),max(1,round(p.height*scale))),Image.Resampling.LANCZOS)
   sheet.alpha_composite(p,((f%2)*128+(128-p.width)//2,(f//2)*128+121-p.height))
  runtime=ROOT/f"public/game/enemy_{e['slug']}_{action}.png"
  sheet.save(runtime,optimize=True)
  file='/'+str(runtime.relative_to(ROOT/'public'))
  digest=hashlib.sha256(runtime.read_bytes()).hexdigest()
  if file not in catalog_by_file:
   catalog_by_file[file]={'file':file,'name':runtime.name};catalog.append(catalog_by_file[file])
  catalog_by_file[file].update(size=runtime.stat().st_size,hash=digest,shortHash=digest[:8],width=256,height=256)
 i=len(report);p=poses[0].copy();p.thumbnail((130,130));preview.paste(p,((i%8)*144+7,(i//8)*160),p);draw.text(((i%8)*144+3,(i//8)*160+136),e['name'][:22],fill='white')
 report.append(dict(slug=e['slug'],id=e['id'],level=e.get('level'),base=e['base'],source=str(path.relative_to(ROOT)),source_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),source_size=Image.open(path).size,frames=28))
catalog_path.write_text(json.dumps(catalog,indent=2)+'\n')
(DIR/'generation.json').write_text(json.dumps(dict(generator='built-in image_gen',enemies=report),indent=2)+'\n');preview.save(DIR/'roster-preview.jpg')
print('Packed',len(report),'characters /',len(report)*28,'real poses')
