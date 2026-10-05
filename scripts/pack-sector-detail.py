#!/usr/bin/env python3
"""Pack native-alpha sector boards and isolated projectiles; make material edges periodic."""
from pathlib import Path
import json
import sys
import numpy as np
from PIL import Image
from scipy.ndimage import label, binary_dilation
ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'art/sector-detail'
OUT=ROOT/'public/game'
SURFACES=('wall','service','door','floor','ceiling')
def panel(board,c,r,cols,rows):
 return board.crop((round(c*board.width/cols),round(r*board.height/rows),round((c+1)*board.width/cols),round((r+1)*board.height/rows)))
def save(im,path):
 path.parent.mkdir(parents=True,exist_ok=True);im.save(path,optimize=True)
def clean_alpha(im):
 a=np.asarray(im).copy(); alpha=a[:,:,3]; labels,_=label(alpha>10);sizes=np.bincount(labels.ravel());sizes[0]=0
 keep=sizes>=max(100,float(sizes.max())*.004)
 mask=binary_dilation(keep[labels],iterations=2)
 a[:,:,3][~mask]=0;a[:,:,3][a[:,:,3]<4]=0
 return Image.fromarray(a)
def isolated(im,size=256):
 im=clean_alpha(im);bounds=im.getchannel('A').getbbox()
 if not bounds:raise ValueError('Empty sprite')
 im=im.crop(bounds);im.thumbnail((round(size*.88),round(size*.88)),Image.Resampling.LANCZOS)
 result=Image.new('RGBA',(size,size));result.alpha_composite(im,((size-im.width)//2,(size-im.height)//2));return result

def periodic(im, size=256):
 a=np.asarray(im.resize((size,size),Image.Resampling.LANCZOS)).astype(float);a[:,:,3]=255
 # Join opposite edge texels without changing the central material or inventing art.
 for axis in [0,1]:
  for i in range(max(4,size//64)):
   lo=[slice(None)]*3;hi=[slice(None)]*3;lo[axis]=i;hi[axis]=-1-i
   left=a[tuple(lo)].copy();right=a[tuple(hi)].copy();mid=(left+right)*.5;weight=1-i/max(4,size//64)
   a[tuple(lo)]=left*(1-weight)+mid*weight;a[tuple(hi)]=right*(1-weight)+mid*weight
 return Image.fromarray(np.clip(a,0,255).astype('uint8'))

def main():
 for s in json.loads((SOURCE/'specs.json').read_text()):
  path=SOURCE/(s['slug']+'.png')
  if not path.exists():continue
  board=Image.open(path).convert('RGBA');folder=OUT/'sectors'/s['slug']
  for i,name in enumerate(SURFACES if "--props-only" not in sys.argv else []):
   source=panel(board,i%4,i//4,4,2)
   tile=periodic(source);save(tile,folder/(name+'.png'))
   hd=folder/'hd'/(name+'.webp');hd.parent.mkdir(parents=True,exist_ok=True);periodic(source,1024).save(hd,quality=94,method=4)
  override=SOURCE/(s['slug']+'-props.png')
  prop_board=Image.open(override).convert('RGBA') if override.exists() else None
  for j in range(3):
   i=j+5;pose=panel(prop_board,j,0,3,1) if prop_board is not None else panel(board,i%4,i//4,4,2)
   save(isolated(pose,256),folder/f'prop_{j+1}.png')
 for s in json.loads((SOURCE/'projectiles.json').read_text()):
  path=SOURCE/(s['board']+'.png')
  if not path.exists():continue
  board=Image.open(path).convert('RGBA');cell=panel(board,s['cell']%s['cols'],s['cell']//s['cols'],s['cols'],s['rows'])
  save(isolated(cell),OUT/'projectiles'/('enemy_'+s['slug']+'.png'))
 board=Image.open(SOURCE/'missiles.png').convert('RGBA')
 for i,name in enumerate(['tactical','siege','naval']):
  for r,direction in enumerate(['outgoing','incoming']):
   rear=SOURCE/f'missile-{name}-rear.png'
   image=Image.open(rear).convert('RGBA') if direction=='outgoing' and rear.exists() else panel(board,i,r,3,2)
   save(isolated(image),OUT/'projectiles'/f'missile_{name}_{direction}.png')
 # The three missile bosses use their own incoming model, with a centered nose.
 for name,model in [('titan','siege'),('astra','tactical'),('nautilus','naval')]:
  save(Image.open(OUT/'projectiles'/f'missile_{model}_incoming.png').convert('RGBA'),OUT/'projectiles'/f'enemy_{name}.png')
 # Repair the legacy 2x2 atlas with full cells at the original 512px dimensions.
 paths=['enemy_rifleman.png','missile_tactical_outgoing.png','enemy_hornet.png','enemy_spitter.png']
 if all((OUT/'projectiles'/p).exists() for p in paths):
  atlas=Image.new('RGBA',(512,512))
  for i,p in enumerate(paths):atlas.alpha_composite(Image.open(OUT/'projectiles'/p).convert('RGBA'),((i%2)*256,(i//2)*256))
  save(atlas,OUT/'spr_ordnance.png')
 rocket=Image.open(OUT/'projectiles/missile_tactical_outgoing.png').convert('RGBA')
 rear=SOURCE/'missile-tactical-rear.png'
 save(isolated(Image.open(rear).convert('RGBA'),512) if rear.exists() else rocket.resize((512,512),Image.Resampling.LANCZOS),OUT/'draft/v2/fx/projectile_rocket.png')
 print('Packed sector-specific materials, props and isolated directional projectiles')
if __name__=='__main__':main()
