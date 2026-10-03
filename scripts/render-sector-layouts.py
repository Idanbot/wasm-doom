#!/usr/bin/env python3
"""Render the authored room graph overview and derive arena entrance doors."""
import json
from pathlib import Path
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
path=ROOT/'art/maps/layouts-10-25.json';plans=json.loads(path.read_text())
canvas=Image.new('RGB',(1248,1032),(10,16,23));d=ImageDraw.Draw(canvas)
for i,s in enumerate(plans):
 grid=[[s['wall']]*48 for _ in range(32)]
 for x,y,w,h in s['rooms']:
  for yy in range(y,y+h):
   for xx in range(x,x+w):grid[yy][xx]=s['wall'] if xx in (x,x+w-1) or yy in (y,y+h-1) else 0
 for a,b,vert in s['edges']:
  ax,ay,aw,ah=s['rooms'][a];bx,by,bw,bh=s['rooms'][b];x0,y0,x1,y1=ax+aw//2,ay+ah//2,bx+bw//2,by+bh//2
  yh=y1 if vert else y0; xv=x0 if vert else x1
  for xx in range(min(x0,x1),max(x0,x1)+1):grid[yh][xx]=grid[yh+1][xx]=0
  for yy in range(min(y0,y1),max(y0,y1)+1):grid[yy][xv]=grid[yy][xv+1]=0
 for x,y,w,h in s['rooms'][1:]:
  if w>=9 and h>=9:grid[y+4][x+3]=s['wall']
 x,y,w,h=s['rooms'][-1]
 boundary=sorted(set([(xx,yy)for yy in (y,y+h-1)for xx in range(x,x+w)]+[(xx,yy)for xx in (x,x+w-1)for yy in range(y,y+h)]))
 s['doors']=[[xx,yy]for xx,yy in boundary if grid[yy][xx]==0]
 xoff=i%4*312+12;yoff=i//4*258+30
 d.text((xoff,yoff-22),f"{s['level']:02d}  {s['name']}",fill=(235,241,247))
 for yy in range(32):
  for xx in range(48):
   if grid[yy][xx]==0:d.rectangle((xoff+xx*6,yoff+yy*6,xoff+xx*6+5,yoff+yy*6+5),fill=(48,67,83))
 for xx,yy in s['doors']:d.rectangle((xoff+xx*6,yoff+yy*6,xoff+xx*6+5,yoff+yy*6+5),fill=(164,123,63))
 for label,p,col in [('S',s['start'],(89,220,163)),('N',s['node'],(78,193,250)),('B',s['bosses'][0],(255,107,107)),('O',s['override'],(255,184,65))]:d.text((xoff+int(p[0])*6,yoff+int(p[1])*6),label,fill=col)
 d.text((xoff,yoff+200),'S start / N node / O override / B boss',fill=(158,174,188))
canvas.save(ROOT/'art/maps/layouts-10-25.png')
path.write_text(json.dumps(plans,indent=2)+'\n')
