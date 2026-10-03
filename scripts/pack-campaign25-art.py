#!/usr/bin/env python3
"""Pack native-alpha campaign artwork into the live sprite contracts."""
from pathlib import Path
import json,hashlib
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'art/campaign25'
SPECS=json.loads((SOURCE/'specs.json').read_text())
OUT=ROOT/'public/game'
ACTIONS=('idle','move','pain','fire','reload','dead','special')
def save(image,path):
 path.parent.mkdir(parents=True,exist_ok=True)
 image.putalpha(image.getchannel('A').point(lambda a: 0 if a<4 else a))
 image.save(path,optimize=True)
def panels(board,cols,rows):
 return [board.crop((round(c*board.width/cols),round(r*board.height/rows),round((c+1)*board.width/cols),round((r+1)*board.height/rows))) for r in range(rows) for c in range(cols)]
def main():
 for s in SPECS:
  w=s['weapon']; board=Image.open(SOURCE/f'{w}-sheet.png').convert('RGBA')
  cells=[]
  for pose in panels(board,5,5):
   pose=pose.resize((320,240),Image.Resampling.LANCZOS)
   cell=Image.new('RGBA',(512,384));cell.alpha_composite(pose,(192,144));cells.append(cell)
  # One translation for the entire sequence keeps handling clear of the reticle.
  shift=0
  while shift < 96 and any(cell.crop((248,184-shift,264,200-shift)).getchannel('A').getextrema()[1]>=8 for cell in [cells[i] for i in (0,1,2,3,4,6,7,8,10,11,12,13)]):shift+=4
  if shift:
   translated=[]
   for cell in cells:
    canvas=Image.new('RGBA',(512,384));canvas.alpha_composite(cell,(0,shift));translated.append(canvas)
   cells=translated
  aim=cells[0]
  # Fixed-size packing preserves continuity and exact return-to-aim pixels.
  for index in (9,14,19,24):cells[index]=aim.copy()
  cells[5]=cells[3].copy()
  sheet=Image.new('RGBA',(2560,1920))
  for i,cell in enumerate(cells):sheet.alpha_composite(cell,((i%5)*512,(i//5)*384))
  save(sheet,OUT/f'draft/v2/weap_{w}_5x5.png');save(aim,OUT/f'draft/v2/weap_{w}_aim.png')
  enemy=Image.open(SOURCE/f'{s["slug"]}-boss.png').convert('RGBA')
  poses=panels(enemy,4,7)
  # One common bounding scale, never chroma-key native alpha or shift frames independently.
  for row,action in enumerate(ACTIONS):
   atlas=Image.new('RGBA',(256,256))
   for frame in range(4):
    pose=poses[row*4+frame].resize((120,120),Image.Resampling.LANCZOS)
    atlas.alpha_composite(pose,((frame%2)*128+4,(frame//2)*128+4))
   save(atlas,OUT/f'enemy_{s["slug"]}_{action}.png')
  attachment=SOURCE/f'{w}-reward.png'
  if attachment.exists():
   case,side=panels(Image.open(attachment).convert('RGBA'),2,1)
   save(case.resize((1024,768),Image.Resampling.LANCZOS),OUT/f'draft/v2/cases/case_{w}.png')
   bounds=case.getchannel('A').point(lambda a:255 if a>20 else 0).getbbox()
   case=case.crop(bounds);case.thumbnail((230,218),Image.Resampling.LANCZOS)
   icon=Image.new('RGBA',(256,256));icon.alpha_composite(case,((256-case.width)//2,238-case.height));save(icon,OUT/f'spr_gun_{w}.png')
   bounds=side.getchannel('A').point(lambda a:255 if a>20 else 0).getbbox()
   side=side.crop(bounds);side.thumbnail((570,246),Image.Resampling.LANCZOS)
   icon=Image.new('RGBA',(640,300));icon.alpha_composite(side,((640-side.width)//2,(300-side.height)//2));save(icon,OUT/f'ui/weapon-thumbs/{w}.png')
 for name in ('rocket-forward','ion-bolt','cryo-shard','spore-cluster','phase-orb','ember-impact','pressure-impact','magnetic-impact','solar-impact'):
  source=Image.open(SOURCE/f'{name}.png').convert('RGBA')
  if name=='rocket-forward':
   frames=panels(source,3,1);frames.append(frames[1].copy())
  elif 'impact' in name:frames=panels(source,4,1)
  else:
   save(source.resize((256,256),Image.Resampling.LANCZOS),OUT/f'fx25/{name}.png');continue
  atlas=Image.new('RGBA',(256,256))
  for i,frame in enumerate(frames):
   frame=frame.resize((120,120),Image.Resampling.LANCZOS)
   atlas.alpha_composite(frame,((i%2)*128+4,(i//2)*128+4))
  save(atlas,OUT/f'fx25/{name}.png')
 # Legacy rocket catalog entries and ordnance atlas share the corrected outgoing art.
 rocket=Image.open(OUT/'fx25/rocket-forward.png').convert('RGBA').crop((0,0,128,128))
 save(rocket.resize((512,512),Image.Resampling.LANCZOS),OUT/'draft/v2/fx/projectile_rocket.png')
 ordnance=Image.open(OUT/'spr_ordnance.png').convert('RGBA')
 cell=ordnance.width//2
 ordnance.paste(rocket.resize((cell,cell),Image.Resampling.LANCZOS),(cell,0));save(ordnance,OUT/'spr_ordnance.png')
 print('Packed 14 boss animation sets, 14 weapon sheets and 9 effect assets')
if __name__=='__main__':main()
