#!/usr/bin/env python3
"""Crop native-alpha generated sprite boards into isolated world atlas layers."""
from pathlib import Path
import json,hashlib
from collections import deque
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
SRC=ROOT/'art/projectiles-v3'
OUT=ROOT/'public/game/projectiles'
def export_cell(path,index,cols,rows,out):
    im=Image.open(path).convert('RGBA')
    x=index%cols;y=index//cols
    cell=im.crop((round(x*im.width/cols),round(y*im.height/rows),round((x+1)*im.width/cols),round((y+1)*im.height/rows)))
    # Threshold only near-zero alpha dust, preserve translucent glow and opaque bodies.
    cell.putalpha(cell.getchannel('A').point(lambda a:0 if a<8 else a))
    alpha=bytearray(cell.getchannel('A').tobytes());w,h=cell.size;seen=bytearray(w*h);components=[]
    for start,value in enumerate(alpha):
        if value<8 or seen[start]:continue
        todo=[start];seen[start]=1;pixels=[]
        while todo:
            k=todo.pop();pixels.append(k);x=k%w;y=k//w
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,-1),(1,-1),(-1,1)]:
                nx=x+dx;ny=y+dy
                if 0<=nx<w and 0<=ny<h:
                    n=ny*w+nx
                    if alpha[n]>=8 and not seen[n]:seen[n]=1;todo.append(n)
        components.append(pixels)
    largest=max(map(len,components),default=0)
    # Discard only small disconnected islands: these can be fragments of a neighboring cell.
    for pixels in components:
        if len(pixels)<largest*0.04:
            for k in pixels:alpha[k]=0
    cell.putalpha(Image.frombytes('L',(w,h),bytes(alpha)))
    bounds=cell.getchannel('A').getbbox()
    if not bounds:raise ValueError(f'blank {out}')
    cell=cell.crop(bounds);cell.thumbnail((218,218),Image.Resampling.LANCZOS)
    canvas=Image.new('RGBA',(256,256));canvas.alpha_composite(cell,((256-cell.width)//2,(256-cell.height)//2))
    canvas.putalpha(canvas.getchannel('A').point(lambda a:0 if a<4 else a))
    canvas.save(out,optimize=True)
    return {'file':str(out.relative_to(ROOT)),'source':str(path.relative_to(ROOT)),'cell':index,'grid':[cols,rows],'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'native_size':list(im.size),'runtime_size':[256,256]}
def main():
    OUT.mkdir(parents=True,exist_ok=True);records=[]
    for i in range(25):records.append(export_cell(SRC/'boss-weapons.png',i,5,5,OUT/f'boss_weapon_{i+8}.png'))
    enemies=json.loads((ROOT/'art/sector-detail/projectiles.json').read_text())
    for en in enemies:
        skin=en['skin'];source='enemies-0-19.png' if skin<20 else 'enemies-20-36.png';cell=skin if skin<20 else skin-20
        records.append(export_cell(SRC/source,cell,5,4,OUT/f'enemy_{en["slug"]}.png'))
    # Public orientation aliases stay compatible with existing renderer tests/catalog.
    for name,skin,slot in [('tactical',29,25),('siege',20,16),('naval',33,29)]:
        slug=enemies[skin]['slug']
        for direction,source in [('incoming',OUT/f'enemy_{slug}.png'),('outgoing',OUT/f'boss_weapon_{slot}.png')]:
            (OUT/f'missile_{name}_{direction}.png').write_bytes(source.read_bytes())
    sources=[{'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(SRC.glob('*.png'))]
    (SRC/'manifest.json').write_text(json.dumps({'generator':'built-in image_gen','alpha':'native transparency; no chroma key','sources':sources,'assets':records},indent=2)+'\n')
if __name__=='__main__':main()
