#!/usr/bin/env python3
"""Key weapon masters and build stable idle/fire/reload viewmodel sheets."""
from __future__ import annotations

import json
from collections import deque
from pathlib import Path
from PIL import Image, ImageDraw, ImageEnhance

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "art/source_hd/weapons_v2"
OUT = ROOT / "public/game"
ATLAS = ROOT / "art/atlases"
WEAPONS = ("mk23s", "m870k", "vx9", "shrike", "raven", "arc12", "m56")

def key(image: Image.Image) -> Image.Image:
    image = image.convert("RGBA")
    px = image.load(); w, h = image.size
    seen = bytearray(w*h); q = deque()
    for x in range(w): q.extend(((x,0),(x,h-1)))
    for y in range(h): q.extend(((0,y),(w-1,y)))
    while q:
        x,y=q.popleft(); i=y*w+x
        if seen[i]: continue
        seen[i]=1; r,g,b,_=px[x,y]
        if not (r>145 and b>80 and g<150 and r>b*.68 and b>g*1.08): continue
        px[x,y]=(r,g,b,0)
        if x:q.append((x-1,y))
        if x+1<w:q.append((x+1,y))
        if y:q.append((x,y-1))
        if y+1<h:q.append((x,y+1))
    for y in range(h):
        for x in range(w):
            r,g,b,a=px[x,y]
            if r>115 and b>65 and r-g>38 and b-g>24 and r>b*.62: px[x,y]=(r,g,b,0)
    return image

def fitted(image: Image.Image) -> Image.Image:
    bbox=image.getchannel("A").getbbox()
    if not bbox: raise ValueError("weapon vanished during chroma key")
    image=image.crop(bbox)
    scale=min(900/image.width, 930/image.height)
    image=image.resize((round(image.width*scale),round(image.height*scale)),Image.Resampling.LANCZOS)
    canvas=Image.new("RGBA",(1024,1024),(0,0,0,0))
    canvas.alpha_composite(image,((1024-image.width)//2,1020-image.height))
    return canvas

def decontaminate(image: Image.Image) -> Image.Image:
    """Neutralize pink studio spill retained on otherwise opaque antialiased edges."""
    image=image.copy(); px=image.load()
    for y in range(image.height):
        for x in range(image.width):
            r,g,b,a=px[x,y]
            magenta = r>80 and b>40 and r-g>25 and b-g>15 and abs(r-b)<125
            if a < 64 or (magenta and a < 220):
                px[x,y]=(0,0,0,0)
            elif magenta:
                neutral=max(4,min(255,round(g*.78)))
                px[x,y]=(neutral,neutral,neutral,a)
    return image

def shifted(base: Image.Image, dx=0, dy=0, angle=0, bright=1.0) -> Image.Image:
    frame=base.rotate(angle,Image.Resampling.BICUBIC,center=(512,940),fillcolor=(0,0,0,0))
    if bright != 1: frame=ImageEnhance.Brightness(frame).enhance(bright)
    out=Image.new("RGBA",base.size,(0,0,0,0)); out.alpha_composite(frame,(dx,dy)); return out

def flash(frame: Image.Image, slug: str, power: float) -> Image.Image:
    out=frame.copy(); glow=Image.new("RGBA",out.size,(0,0,0,0)); d=ImageDraw.Draw(glow)
    cx,cy=(512,120 if slug in {"raven","shrike"} else 170)
    color=(110,225,255) if slug in {"arc12","shrike"} else (255,175,66)
    radius=round(50+power*45)
    d.ellipse((cx-radius,cy-radius,cx+radius,cy+radius),fill=(*color,round(45+power*55)))
    d.polygon(((cx,cy-radius-45),(cx-22,cy+22),(cx+22,cy+22)),fill=(*color,220))
    return Image.alpha_composite(out,glow)

def sheet(frames: list[Image.Image]) -> Image.Image:
    out=Image.new("RGBA",(1024,1024),(0,0,0,0))
    for i,frame in enumerate(frames): out.alpha_composite(frame.resize((512,512),Image.Resampling.LANCZOS),((i&1)*512,(i>>1)*512))
    return out

def main():
    ATLAS.mkdir(parents=True, exist_ok=True)
    report={}
    for slug in WEAPONS:
        path=SOURCE/f"{slug}.jpg"
        if not path.exists(): raise FileNotFoundError(path)
        base=decontaminate(key(fitted(key(Image.open(path)))))
        base.resize((640,640),Image.Resampling.LANCZOS).save(OUT/f"weap_{slug}.png",optimize=True)
        recoil=[shifted(base,0,0),flash(shifted(base,-4,32,-1.2,1.08),slug,.55),flash(shifted(base,4,58,1.6,1.14),slug,1),shifted(base,0,20,0,.96)]
        reload=[shifted(base,0,25,0),shifted(base,-55,85,-7),shifted(base,48,95,6),shifted(base,0,35,0)]
        sheet(recoil).save(OUT/f"weap_{slug}_fire.png",optimize=True)
        sheet(reload).save(OUT/f"weap_{slug}_reload.png",optimize=True)
        report[slug]={"source":str(path.relative_to(ROOT)),"idle":f"public/game/weap_{slug}.png","fireFrames":4,"reloadFrames":4}
        metadata={"id":f"weapon_{slug}","frameWidth":512,"frameHeight":512,"origin":{"x":256,"y":480},"animations":{"idle":{"start":0,"frames":1,"fps":1},"fire":{"start":0,"frames":4,"fps":10},"reload":{"start":0,"frames":4,"fps":7}}}
        (ATLAS/f"weapon_{slug}.json").write_text(json.dumps(metadata,indent=2)+"\n")
    (SOURCE/"processing-report.json").write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps(report,indent=2))

if __name__ == "__main__": main()
