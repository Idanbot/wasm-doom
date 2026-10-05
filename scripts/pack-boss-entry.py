#!/usr/bin/env python3
"""Extract native-alpha boss entrance motifs with safe transparent margins."""
from pathlib import Path
from PIL import Image
root=Path(__file__).resolve().parents[1]
im=Image.open(root/'art/boss-entry-v1/source.png').convert('RGBA')
for i in range(8):
    x,y=i%4,i//4
    cell=im.crop((x*im.width//4,y*im.height//2,(x+1)*im.width//4,(y+1)*im.height//2))
    cell.thumbnail((800,800),Image.Resampling.LANCZOS)
    out=Image.new('RGBA',(1024,1024));out.alpha_composite(cell,((1024-cell.width)//2,(1024-cell.height)//2))
    out.save(root/f'public/game/fx/boss-entry-{i}.png')
