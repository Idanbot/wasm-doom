#!/usr/bin/env python3
"""Remove disconnected panel remnants from mechanical handling poses.

This only repairs alpha masks; it does not redraw guns, hands, ammo or effects.
Approved aim poses, silhouette positions and timing stay unchanged.
"""
from pathlib import Path
import json, hashlib
import numpy as np
from PIL import Image
from scipy.ndimage import label, find_objects, binary_dilation
ROOT=Path(__file__).resolve().parents[1]
MANIFEST=ROOT/'src/lib/draft-weapons-v2-data.json'
HANDLING=(3,4,5,6,7,8,10,11,12,13)
FRAMES=tuple(i for i in range(25) if i not in (0,9,14,19,24))
def clean(frame, handling=True):
 a=np.asarray(frame).copy(); labels,_=label(a[:,:,3]>10);sizes=np.bincount(labels.ravel());sizes[0]=0
 keep=sizes>=(max(128,sizes.max()*.01) if handling else 8)
 for index,box in enumerate(find_objects(labels),1):
  if box and min(b.stop-b.start for b in box)<=3:keep[index]=False
 mask=binary_dilation(keep[labels],iterations=2)
 a[:,:,3][~mask]=0;a[:,:,3][(labels>0)&~keep[labels]]=0;a[:,:,3][a[:,:,3]<4]=0
 return Image.fromarray(a)
def main():
 rows=json.loads(MANIFEST.read_text());report_path=ROOT/'art/sector-detail/weapon-edge-cleanup.json';report=json.loads(report_path.read_text()) if report_path.exists() else []
 for row in rows:
  path=ROOT/'public'/row['sheet_file'].lstrip('/');sheet=Image.open(path).convert('RGBA');removed=0
  for i in FRAMES:
   box=(i%5*512,i//5*384,(i%5+1)*512,(i//5+1)*384);original=sheet.crop(box);frame=clean(original,i in HANDLING)
   before=np.asarray(original)[:,:,3];after=np.asarray(frame)[:,:,3];removed+=int(np.count_nonzero((before>10)&(after<=10)));sheet.paste(frame,box[:2])
  # The reserved dry-fire frame remains pixel-identical to empty-mag aim.
  empty=sheet.crop((1536,0,2048,384));sheet.paste(empty,(0,384))
  if row["id"] in ("ks8", "ts12", "gs4"):
   sheet.paste(sheet.crop((0,0,512,384)),(512,0))
  if removed or row["id"] in ("ks8", "ts12", "gs4"):
   sheet.save(path,optimize=True)
   existing=next((r for r in report if r['weapon']==row['id']),None)
   if existing:existing['removed_pixels']+=removed
   else:report.append({'weapon':row['id'],'removed_pixels':removed})
  h=hashlib.sha256(path.read_bytes()).hexdigest();row['sheet_hash']=h;row['sheet_short_hash']=h[:8]
 MANIFEST.write_text(json.dumps(rows,indent=2)+'\n')
 (ROOT/'art/sector-detail/weapon-edge-cleanup.json').write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps(report))
if __name__=='__main__':main()
