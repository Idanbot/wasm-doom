#!/usr/bin/env python3
"""Build BLACKSITE v2 review assets from individual weapon poses and masters."""
from __future__ import annotations
import hashlib, json, math, random
from functools import lru_cache
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw, ImageEnhance, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'public/game/draft/v2'
CELL = (512, 384)
SPECS = [
    ('mk23s','MK23-S',0,False,(244,181,84),(231,139)),
    ('br12','BR-12 BREAKER',1,False,(255,180,87),(238,166)),
    ('kx9','KX-9 VECTOR',2,False,(252,197,105),(217,157)),
    ('mr4','MR-4 LONGBOW',3,False,(249,208,132),(232,185)),
    ('vlk6','VLK-6 WARDEN',4,False,(255,174,89),(185,72)),
    ('ax12','AX-12 VOLT',5,False,(117,206,238),(192,92)),
    ('m91','M91 CYCLONE',6,False,(255,186,89),(169,119)),
    ('hx8','HX-8 PYRE',7,False,(255,125,68),(180,100)),
    ('vr9','VR-9 OVERRIDE',8,True,(255,181,61),(210,153)),
    ('hc9','HC-9 FORGE',9,True,(71,216,255),(250,189)),
    ('cm9','CM-9 CHIMERA',10,True,(143,245,106),(184,155)),
]
SAFE_FIT = {'mk23s':(1,10,0),'br12':(.90,5,0),'kx9':(.80,0,0),
            'mr4':(.90,0,5),'vlk6':(1,25,0),'ax12':(.90,20,0),
            'm91':(.85,0,35),'hx8':(1,25,0),'vr9':(.80,0,0),
            'hc9':(1.0,30,15),'cm9':(.90,0,20)}

def digest(path):
    h=hashlib.sha256(path.read_bytes()).hexdigest()
    return h,h[:8]

def save(im,path):
    path.parent.mkdir(parents=True,exist_ok=True)
    im.save(path,optimize=True)
    return digest(path)

def frame(base,dx=0,dy=0,angle=0,light=1):
    im=base
    if angle:
        im=im.rotate(angle,Image.Resampling.BICUBIC,center=(415,342),fillcolor=(0,0,0,0))
    if light!=1:
        rgb=ImageEnhance.Brightness(im.convert('RGB')).enhance(light)
        rgb.putalpha(im.getchannel('A'));im=rgb
    out=Image.new('RGBA',CELL)
    out.alpha_composite(im,(dx,dy))
    return out

def glow(im,at,color,radius=7):
    layer=Image.new('RGBA',CELL)
    d=ImageDraw.Draw(layer)
    x,y=at
    d.ellipse((x-radius*3,y-radius*3,x+radius*3,y+radius*3),fill=(*color,45))
    layer=layer.filter(ImageFilter.GaussianBlur(radius))
    im.alpha_composite(layer)
    d=ImageDraw.Draw(im)
    d.ellipse((x-radius//2,y-radius//2,x+radius//2,y+radius//2),fill=(*color,230))
    return im

def flare(im,at,color,scale=1):
    x,y=at
    source=muzzle_art()
    w,h=round(112*scale),round(78*scale)
    art=source.resize((w,h),Image.Resampling.LANCZOS)
    if color[2]>color[0]*1.1 or color[1]>color[0]*1.2:
        a=np.array(art)
        intensity=a[:,:,:3].max(axis=2).astype(np.float32)/255
        for c in range(3):a[:,:,c]=np.clip(color[c]*intensity+35,0,255)
        art=Image.fromarray(a,'RGBA')
    im.alpha_composite(art,(round(x-w*.24),round(y-h*.5)))
    return im

@lru_cache(maxsize=1)
def muzzle_art():
    source=Image.open(OUT/'masters/fx_muzzle.png').convert('RGBA')
    source=source.crop(source.getchannel('A').getbbox())
    return source.rotate(180,expand=True)

def mag(im,step,color):
    # Detached metal magazine changes position and rotation across reload cells.
    x=266+step*11; y=327-(max(0,step-1)*25)
    layer=Image.new('RGBA',CELL);d=ImageDraw.Draw(layer)
    poly=[(x,y),(x+25,y-6),(x+18,y+49),(x+2,y+54)]
    d.polygon(poly,fill=(24,29,33,245),outline=(116,121,119,255),width=2)
    for i in range(4):
        d.line((x+5,y+10+i*9,x+18,y+7+i*9),fill=(75,82,83,180),width=2)
    if step>1: glow(layer,(x+13,y+4),color,3)
    im.alpha_composite(layer)
    return im

AMMO_PARTS = {
    'kx9': [(317,265),(348,267),(351,384),(320,384)],
    'ax12': [(314,239),(343,241),(348,350),(315,350)],
    'm91': [(262,278),(319,279),(321,381),(255,381)],
    'mr4': [(379,286),(411,283),(412,384),(380,384)],
    'vr9': [(376,286),(414,282),(421,384),(381,384)],
    'hx8': [(434,214),(512,211),(512,316),(447,300)],
    'hc9': [(342,260),(421,250),(435,365),(354,378)],
    'cm9': [(362,203),(449,198),(457,288),(371,303)],
}
AMMO_GAUGES = {
    'mk23s':(382,253),'br12':(394,256),'kx9':(391,270),'mr4':(404,280),
    'vlk6':(424,271),'ax12':(413,255),'m91':(401,259),'hx8':(421,272),
    'vr9':(414,273),'hc9':(450,283),
}

def ammo_gauge(im,slug,fraction,color):
    if slug not in AMMO_GAUGES:return im
    x,y=AMMO_GAUGES[slug]
    # Four recessed receiver lamps make the ammunition state legible at game scale.
    overlay=Image.new('RGBA',CELL)
    d=ImageDraw.Draw(overlay)
    d.rounded_rectangle((x-3,y-3,x+36,y+11),radius=3,fill=(7,12,14,245),outline=(103,115,113,230),width=1)
    lit=2 if fraction>=.5 else 1 if fraction>0 else 0
    for index in range(4):
        tint=(*color,240) if index<lit else (29,36,37,245)
        d.rounded_rectangle((x+index*9,y,x+index*9+6,y+7),radius=1,fill=tint)
    # Blend only over the existing weapon, never into transparent scenery.
    mask=im.getchannel('A').point(lambda a:255 if a>150 else 0)
    alpha=overlay.getchannel('A');alpha=Image.fromarray(np.minimum(np.asarray(alpha),np.asarray(mask)).astype(np.uint8),'L')
    overlay.putalpha(alpha)
    im.alpha_composite(overlay)
    return im

def ammo_state(base,slug,fraction,color,remove=False):
    """Preserve approved aim silhouette while changing visible ammo hardware."""
    out=base.copy()
    if slug=='cm9':
        a=np.array(out)
        yy,xx=np.mgrid[:384,:512]
        in_tubes=((xx>364)&(xx<440)&(yy>238)&(yy<274))
        green=(a[:,:,1]>a[:,:,0]*1.18)&(a[:,:,1]>a[:,:,2]*1.12)
        above=yy<270-round(26*fraction)
        mask=in_tubes&green&above
        a[:,:,:3][mask]=(a[:,:,:3][mask]*.2+np.array([37,43,40])*.8).astype(np.uint8)
        out=Image.fromarray(a,'RGBA')
    elif slug in ('hc9','vr9','ax12'):
        a=np.array(out)
        yy,xx=np.mgrid[:384,:512]
        bright=(a[:,:,2]>a[:,:,0]*1.18) if slug in ('hc9','ax12') else (a[:,:,0]>a[:,:,2]*1.35)
        # Discharge segments progressively, leaving body metal unchanged.
        cutoff={1:0,.5:2,.15:3,0:4}.get(fraction,4)
        bands=((xx//17+yy//19)%4)<cutoff
        mask=bright&bands&(a[:,:,3]>40)
        a[:,:,:3][mask]=(a[:,:,:3][mask]*.43).astype(np.uint8)
        out=Image.fromarray(a,'RGBA')
    elif slug=='m91':
        a=np.array(out)
        yy,xx=np.mgrid[:384,:512]
        belt=(xx>260)&(xx<305)&(yy>240)&(yy<309)
        brass=(a[:,:,0]>a[:,:,2]*1.15)
        above=yy<309-(69*fraction)
        mask=belt&brass&above
        a[:,:,:3][mask]=(a[:,:,:3][mask]*.4).astype(np.uint8)
        out=Image.fromarray(a,'RGBA')
    if fraction<=.15:
        # A small chassis warning is visible on low or empty, without obscuring the art.
        x,y=(394,247) if slug!='cm9' else (345,258)
        glow(out,(x,y),(245,142,53) if fraction else (228,64,61),3)
    if fraction<1 and not remove:
        out=ammo_gauge(out,slug,fraction,color)
    if remove:
        if slug in AMMO_PARTS:
            mask=Image.new('L',CELL);ImageDraw.Draw(mask).polygon(AMMO_PARTS[slug],fill=255)
            # Remove detachable hardware. The visible dark edge reads as an open socket.
            alpha=out.getchannel('A');alpha.paste(0,mask=mask);out.putalpha(alpha)
        elif slug=='mk23s':
            d=ImageDraw.Draw(out,'RGBA')
            d.polygon([(346,352),(389,353),(385,370),(348,369)],fill=(8,12,15,230))
            d.line((348,352,387,353),fill=(110,114,108,230),width=2)
        elif slug in ('br12','vlk6'):
            # Tube-fed weapons have no detachable box magazine; their feed mouths are empty.
            d=ImageDraw.Draw(out,'RGBA')
            d.ellipse((269,180,292,198),fill=(8,11,12,235),outline=(102,108,108,180),width=2)
    return out

def pose_frames(slug,kind,count):
    path=OUT/'masters'/f'anim_{kind}_{slug}.png'
    if not path.exists():return None
    strip=Image.open(path).convert('RGBA')
    frames=[]
    for i in range(count):
        panel=strip.crop((round(i*strip.width/count),0,round((i+1)*strip.width/count),strip.height))
        target_w=246 if kind=='reload' else 270
        target_h=round(panel.height*target_w/panel.width)
        panel=panel.resize((target_w,target_h),Image.Resampling.LANCZOS)
        y=35 if kind=='reload' else 4
        f=Image.new('RGBA',CELL)
        f.alpha_composite(panel,(512-target_w,y))
        # Slightly reduce only the rare panel that blocks the crosshair.
        for _ in range(6):
            if np.asarray(f.getchannel('A'))[180:204,244:268].mean()<4:break
            small=f.resize((round(f.width*.94),round(f.height*.94)),Image.Resampling.LANCZOS)
            f=Image.new('RGBA',CELL)
            f.alpha_composite(small,(512-small.width,384-small.height))
        frames.append(f)
    return frames

def variant_frame(slug,kind):
    path=OUT/'masters'/f'anim_{kind}_{slug}.png'
    if not path.exists():return None
    variant=Image.open(path).convert('RGBA').resize(CELL,Image.Resampling.LANCZOS)
    if slug=='hc9':
        moved=Image.new('RGBA',CELL)
        moved.alpha_composite(variant,(30,15))
        return moved
    return variant

def make_sheet(slug,boss,color,muzzle):
    source=OUT/'masters'/f'{slug}.png'
    if not source.exists(): raise FileNotFoundError(source)
    raw=Image.open(source).convert('RGBA')
    # Full generated canvas is the composition reference; retain transparent safety area.
    fitted=raw.copy()
    fitted.thumbnail(CELL,Image.Resampling.LANCZOS)
    base=Image.new('RGBA',CELL)
    base.alpha_composite(fitted,(CELL[0]-fitted.width,CELL[1]-fitted.height))
    safe_scale,safe_dx,safe_dy=SAFE_FIT[slug]
    if safe_scale != 1:
        scaled=base.resize((round(512*safe_scale),round(384*safe_scale)),Image.Resampling.LANCZOS)
        base=Image.new('RGBA',CELL)
        base.alpha_composite(scaled,(512-scaled.width+safe_dx,384-scaled.height+safe_dy))
    elif safe_dx or safe_dy:
        moved=Image.new('RGBA',CELL)
        moved.alpha_composite(base,(safe_dx,safe_dy))
        base=moved
    muzzle=(round(512-(512-muzzle[0])*safe_scale+safe_dx),
            round(384-(384-muzzle[1])*safe_scale+safe_dy))
    a=np.asarray(base.getchannel('A'))
    # Viewmodel must enter from bottom/right and leave an unobstructed reticle.
    if np.count_nonzero(a[185:200,249:264]>40)>30:
        print(f'NOTICE {slug}: reticle clearance should be checked by eye')
    frames=[]
    zero=variant_frame(slug,'zero') if slug=='cm9' else None
    empty=variant_frame(slug,'empty')
    frames.extend([base.copy(),ammo_state(base,slug,.5,color),ammo_state(base,slug,.15,color),zero or ammo_state(base,slug,0,color),empty or ammo_state(base,slug,0,color,remove=True)])
    frames.append(frame(frames[3],dx=1,dy=-2,light=.92))
    pickup=pose_frames(slug,'pickup',4)
    if pickup:
        frames.extend([*pickup[:3],base.copy()])
    else:
        frames.extend([frame(base,dx=62,dy=125,angle=-9,light=.65),frame(base,dx=35,dy=78,angle=-5,light=.78),frame(base,dx=12,dy=32,angle=-2,light=.9),base.copy()])
    reload=pose_frames(slug,'reload',5)
    if reload:
        frames.extend([*reload[:4],base.copy()])
    else:
        for i,(dx,dy,angle) in enumerate([(13,37,-3),(24,65,-5),(17,49,-4),(7,24,-1),(0,0,0)]):
            f=frame(base,dx,dy,angle,light=.91 if i<3 else 1)
            if i in (0,1,2,3): mag(f,i,color)
            frames.append(f)
    for i,(dx,dy,angle) in enumerate([(2,-6,1),(8,-17,5),(7,-13,4),(3,-5,2),(0,0,0)]):
        f=frame(base,dx,dy,angle,light=1.05 if i<3 else 1)
        if i in (0,1): flare(f,(muzzle[0]+dx,muzzle[1]+dy),color,1.1 if i==1 else .8)
        if i==2 and not boss:
            ImageDraw.Draw(f).ellipse((340,149,348,156),fill=(216,169,76,220))
        frames.append(f)
    if boss:
        for i in range(5):
            f=frame(base,dx=[0,1,7,4,0][i],dy=[0,-3,-19,-8,0][i],angle=[0,1,7,3,0][i],light=[1,1.06,1.16,1.03,1][i])
            if i in (0,1,2,3): glow(f,(muzzle[0],muzzle[1]-i*2),color,5+i*3)
            if i==2: flare(f,(muzzle[0]+7,muzzle[1]-19),color,1.5)
            frames.append(f)
    else:
        frames.extend([frames[i].copy() for i in range(15,20)])
    sheet=Image.new('RGBA',(2560,1920))
    for i,f in enumerate(frames): sheet.alpha_composite(f,((i%5)*512,(i//5)*384))
    sh,ss=save(sheet,OUT/f'weap_{slug}_5x5.png')
    ah,ash=save(base,OUT/f'weap_{slug}_aim.png')
    mh,msh=digest(source)
    return dict(id=slug,sheet_file=f'/game/draft/v2/weap_{slug}_5x5.png',sheet_hash=sh,sheet_short_hash=ss,aim_file=f'/game/draft/v2/weap_{slug}_aim.png',aim_hash=ah,aim_short_hash=ash,gen_master_file=f'/game/draft/v2/masters/{slug}.png',gen_master_hash=mh,gen_master_short_hash=msh,cell_w=512,cell_h=384,cols=5,rows=5,frames_total=25)

def tile_noise(w,h,seed):
    rng=np.random.default_rng(seed)
    out=np.zeros((h,w),dtype=np.float32)
    for n,weight in [(8,7),(24,5),(80,4),(240,3),(960,2)]:
        small=rng.normal(0,1,(max(2,int(h*n/w)),n)).astype(np.float32)
        sy,sx=small.shape
        # wrap by copying first row/column; final full-resolution border is equal.
        small[-1,:]=small[0,:];small[:,-1]=small[:,0]
        layer=np.asarray(Image.fromarray(small,mode='F').resize((w,h),Image.Resampling.BICUBIC))
        out+=layer*weight
    return out

def texture(name,base,seed,kind):
    w,h=1920,1080
    noise=tile_noise(w,h,seed)
    yy,xx=np.mgrid[:h,:w]
    bands=3*np.sin(2*np.pi*xx/w*8)+2*np.sin(2*np.pi*yy/h*5)
    grey=noise+bands
    arr=np.zeros((h,w,3),dtype=np.uint8)
    for c in range(3):arr[:,:,c]=np.clip(base[c]+grey,0,255)
    im=Image.fromarray(arr,'RGB');d=ImageDraw.Draw(im,'RGBA')
    if kind=='panel':
        for x in range(0,w,480):
            d.line((x,0,x,h),fill=(5,8,12,150),width=10)
            d.line((x+12,0,x+12,h),fill=(190,191,179,45),width=2)
        for y in range(0,h,270):
            d.line((0,y,w,y),fill=(5,8,12,150),width=9)
            d.line((0,y+11,w,y+11),fill=(170,177,177,35),width=2)
        for x in range(50,w,480):
            for y in range(48,h,270):
                for px in (x,x+380):
                    d.ellipse((px-5,y-5,px+5,y+5),fill=(22,23,22,240),outline=(150,157,151,120),width=2)
    elif kind=='concrete':
        for x in range(0,w,480):d.line((x,0,x,h),fill=(18,22,24,100),width=6)
        for y in range(0,h,360):d.line((0,y,w,y),fill=(18,22,24,100),width=6)
        rng=random.Random(seed)
        for _ in range(160):
            x=rng.randrange(w);y=rng.randrange(h);r=rng.randrange(1,5)
            d.ellipse((x-r,y-r,x+r,y+r),fill=(15,19,19,rng.randrange(30,100)))
    elif kind=='grate':
        for x in range(0,w,120):
            for y in range(0,h,90):
                d.rounded_rectangle((x+10,y+8,x+106,y+74),radius=12,fill=(12,16,18,160),outline=(166,172,167,90),width=4)
                d.line((x+16,y+18,x+100,y+66),fill=(151,156,151,35),width=2)
    elif kind=='ceramic':
        for x in range(0,w,240):d.line((x,0,x,h),fill=(20,26,30,120),width=8)
        for y in range(0,h,180):d.line((0,y,w,y),fill=(20,26,30,120),width=8)
        for x in range(60,w,240):
            for y in range(45,h,180): d.line((x,y,x+75,y),fill=(244,247,233,30),width=3)
    elif kind=='hazard':
        for x in range(-300,w+300,320):
            d.polygon([(x,0),(x+118,0),(x+400,h),(x+282,h)],fill=(179,127,40,75))
        for y in range(0,h,270):d.line((0,y,w,y),fill=(8,10,10,130),width=9)
    elif kind=='brick':
        for y in range(0,h,180):
            d.line((0,y,w,y),fill=(10,13,15,160),width=9)
            offset=0 if (y//180)%2==0 else 240
            for x in range(-offset,w,480):d.line((x,y,x,y+180),fill=(10,13,15,160),width=8)
    # Exact edge match guarantees seamless repeat in both axes.
    ar=np.asarray(im).copy();ar[:,-1]=ar[:,0];ar[-1,:]=ar[0,:]
    return Image.fromarray(ar,'RGB')

def particle(name,color,kind):
    source_map={
        'muzzle_ballistic':('muzzle',1),'muzzle_suppressed':('muzzle',0),'muzzle_rocket':('muzzle',3),
        'impact_sparks':('impact',0),'impact_electric':('impact',1),'impact_acid':('impact',3),
        'projectile_rocket':('ordnance',1),'projectile_rail':('ordnance',0),
        'projectile_arc':('ordnance',2),'projectile_acid':('ordnance',3),
        'smoke_muzzle':('ambient',1),'smoke_explosion':('ambient',1),
        'explosion_core':('impact',2),
    }
    if name in source_map:
        atlas_name,index=source_map[name]
        atlas=ROOT/'art/source_hd/projectiles'/f'{atlas_name}_v3_2x2.png'
        if atlas.exists():
            sheet=Image.open(atlas).convert('RGBA')
            x,y=(index&1)*512,(index>>1)*512
            image=sheet.crop((x,y,x+512,y+512))
            if name=='smoke_muzzle':
                image.putalpha(ImageEnhance.Brightness(image.getchannel('A')).enhance(.55))
            return image
    if name.startswith('muzzle_'):
        source=muzzle_art().copy()
        source.thumbnail((480,480),Image.Resampling.LANCZOS)
        im=Image.new('RGBA',(512,512))
        im.alpha_composite(source,((512-source.width)//2,(512-source.height)//2))
        return im
    w=h=512; yy,xx=np.mgrid[:h,:w];cx=cy=255
    dx=xx-cx;dy=yy-cy;r=np.sqrt(dx*dx+dy*dy)
    if kind=='smoke':
        n=tile_noise(w,h,int(hashlib.sha256(name.encode()).hexdigest()[:4],16))
        alpha=np.clip((135-r)*1.5+n*3,0,130)
    elif kind=='ring':
        alpha=np.clip(140-np.abs(r-130)*8,0,210)
    elif kind=='trail':
        alpha=np.clip((255-xx)*1.4,0,190)*np.clip((45-np.abs(dy))*0.06,0,1)
    else:
        alpha=np.clip(255-r*2.4,0,240)
    rgba=np.empty((h,w,4),dtype=np.uint8)
    for c in range(3): rgba[:,:,c]=np.clip(color[c]+(255-color[c])*np.clip(1-r/180,0,1),0,255)
    rgba[:,:,3]=alpha.astype(np.uint8)
    im=Image.fromarray(rgba,'RGBA')
    if kind=='spark':
        d=ImageDraw.Draw(im,'RGBA');rng=random.Random(len(name))
        for _ in range(22):
            a=rng.random()*math.tau;length=rng.randrange(75,230)
            d.line((cx,cy,cx+math.cos(a)*length,cy+math.sin(a)*length),fill=(*color,210),width=rng.randrange(1,4))
    if kind=='flash':
        d=ImageDraw.Draw(im,'RGBA')
        for a in range(10):
            ang=a*math.tau/10;ln=195 if a%2 else 115
            d.polygon([(cx,cy),(cx+math.cos(ang-.1)*25,cy+math.sin(ang-.1)*25),(cx+math.cos(ang)*ln,cy+math.sin(ang)*ln),(cx+math.cos(ang+.1)*25,cy+math.sin(ang+.1)*25)],fill=(*color,190))
        d.ellipse((235,235,277,277),fill=(255,250,217,255))
    return im

def case(slug,boss,color):
    source='case_pistol.png' if slug=='mk23s' else 'case_long.png'
    art=Image.open(OUT/'masters'/source).convert('RGBA')
    art=art.crop(art.getchannel('A').getbbox())
    width={'mk23s':690,'kx9':840,'br12':930,'m91':990,'vlk6':1000,
           'hx8':980,'hc9':990,'cm9':960}.get(slug,950)
    height=round(art.height*width/art.width)
    if height>720:
        width=round(width*720/height);height=720
    art=art.resize((width,height),Image.Resampling.LANCZOS)
    canvas=Image.new('RGBA',(1024,768))
    x=(1024-width)//2;y=(768-height)//2
    canvas.alpha_composite(art,(x,y))
    return canvas

def object_sprite(kind,color):
    source=OUT/'masters'/f'{kind}.png'
    if source.exists():
        art=Image.open(source).convert('RGBA')
        if color in ((234,81,67),(138,225,242)):
            a=np.array(art,dtype=np.float32)
            yy,xx=np.mgrid[:art.height,:art.width]
            bright=np.min(a[:,:,:3],axis=2)
            area=((yy>art.height*.30)&(yy<art.height*.78)) if kind=='lantern' else (yy<art.height*.29)
            mask=(bright>105)&area&(a[:,:,3]>100)
            target=np.array(color,dtype=np.float32)
            energy=np.clip(bright[mask]/230,0.35,1.0)[:,None]
            a[:,:,:3][mask]=np.clip(target[None,:]*energy+35,0,255)
            art=Image.fromarray(a.astype(np.uint8),'RGBA')
        art.thumbnail((728,984),Image.Resampling.LANCZOS)
        canvas=Image.new('RGBA',(768,1024))
        canvas.alpha_composite(art,((768-art.width)//2,(1024-art.height)//2))
        return canvas
    im=Image.new('RGBA',(768,1024));d=ImageDraw.Draw(im,'RGBA')
    if kind=='lantern':
        d.rounded_rectangle((266,220,502,760),radius=26,fill=(35,41,43,255),outline=(170,177,166,255),width=11)
        d.rounded_rectangle((296,298,472,663),radius=13,fill=(*color,130),outline=(213,217,202,255),width=8)
        d.ellipse((310,393,458,543),fill=(*color,185))
        d.rounded_rectangle((315,175,453,250),radius=15,fill=(36,43,45,255),outline=(151,160,156,255),width=9)
        d.arc((311,105,456,260),180,360,fill=(185,191,185,255),width=16)
        for x in (309,459):d.line((x,280,x,680),fill=(26,31,32,255),width=17)
    elif kind=='worklight':
        d.line((380,450,260,901),fill=(87,93,91,255),width=22)
        d.line((380,450,510,901),fill=(87,93,91,255),width=22)
        d.rounded_rectangle((220,202,548,500),radius=29,fill=(32,37,39,255),outline=(167,172,160,255),width=12)
        d.rounded_rectangle((261,249,507,448),radius=18,fill=(*color,155),outline=(234,237,219,255),width=9)
        for x in range(290,510,35):d.line((x,260,x,430),fill=(255,248,212,75),width=8)
    elif kind=='beacon':
        d.rounded_rectangle((222,800,546,890),radius=14,fill=(41,47,48,255),outline=(153,158,149,255),width=10)
        d.rounded_rectangle((270,280,499,800),radius=27,fill=(30,36,37,255),outline=(163,168,160,255),width=10)
        d.rounded_rectangle((303,311,466,555),radius=44,fill=(*color,175),outline=(238,235,219,255),width=8)
        for y in range(575,750,35):d.line((286,y,482,y),fill=(116,122,116,255),width=14)
    elif kind=='medkit':
        d.rounded_rectangle((140,290,630,745),radius=38,fill=(74,83,80,255),outline=(185,191,179,255),width=14)
        d.rounded_rectangle((260,235,508,310),radius=24,fill=(51,59,58,255),outline=(165,171,162,255),width=12)
        d.rectangle((338,395,430,640),fill=(209,217,201,255))
        d.rectangle((255,470,513,565),fill=(209,217,201,255))
        for x in (180,550):d.rectangle((x,680,x+40,735),fill=(*color,210))
    elif kind=='ammo':
        d.rounded_rectangle((165,315,603,748),radius=25,fill=(69,76,70,255),outline=(151,157,144,255),width=13)
        for x in (190,540):d.rounded_rectangle((x,340,x+32,720),radius=6,fill=(34,40,39,255))
        d.rectangle((270,460,500,555),fill=(*color,205))
        d.rounded_rectangle((250,275,520,328),radius=18,fill=(50,55,50,255),outline=(146,154,141,255),width=8)
    return im.filter(ImageFilter.GaussianBlur(.45))

SECTORS = [
    ('hangar',(62,76,79)),('plaza',(83,76,65)),('security',(55,68,79)),
    ('datacenter',(43,69,86)),('foundry',(91,58,43)),('biotech',(57,79,69)),
    ('nuclear',(70,77,52)),('vault',(82,68,73)),
]

def sector_panel(name,color,index):
    """A repeat-safe 1080p wall treatment with a distinct sector palette."""
    im=texture(name,color,401+index*37,'panel').convert('RGBA')
    d=ImageDraw.Draw(im,'RGBA')
    accent=tuple(min(255,round(c*1.9)) for c in color)
    # Work inside each repeat cell so the outer tile edges remain seamless.
    for x in range(0,1920,480):
        for y in range(0,1080,270):
            d.rounded_rectangle((x+41,y+36,x+439,y+234),radius=9,
                fill=(12,17,20,75),outline=(*accent,105),width=5)
            d.line((x+58,y+75,x+168,y+75),fill=(*accent,165),width=8)
            d.line((x+58,y+94,x+267,y+94),fill=(*accent,80),width=3)
            d.rectangle((x+382,y+53,x+405,y+85),fill=(*accent,125))
    ar=np.asarray(im.convert('RGB')).copy()
    ar[:,-1]=ar[:,0];ar[-1,:]=ar[0,:]
    return Image.fromarray(ar,'RGB')

def door_texture(sector):
    source=Image.open(OUT/'masters'/f'door_{sector}.png').convert('RGB')
    width,height=source.size
    desired=16/9
    if width/height>desired:
        crop=round(height*desired);left=(width-crop)//2
        source=source.crop((left,0,left+crop,height))
    else:
        crop=round(width/desired);top=(height-crop)//2
        source=source.crop((0,top,width,top+crop))
    return source.resize((1920,1080),Image.Resampling.LANCZOS)

def machinery_sprite(name):
    source=Image.open(OUT/'masters'/f'{name}.png').convert('RGBA')
    bbox=source.getchannel('A').getbbox()
    if bbox: source=source.crop(bbox)
    source.thumbnail((752,1008),Image.Resampling.LANCZOS)
    canvas=Image.new('RGBA',(768,1024))
    canvas.alpha_composite(source,((768-source.width)//2,1024-source.height))
    return canvas

def main():
    records=[];extras=[]
    for slug,name,slot,boss,color,muzzle in SPECS:
        meta=make_sheet(slug,boss,color,muzzle)
        meta.update(name=name,slot=slot,is_boss=boss,alt_fire_name='Boss special' if boss else 'Primary fire variant',alt_fire_desc='Visual draft for review.',alt_available=boss,ammo_visual='Visual draft: mechanical magazine/charge state progression.')
        records.append(meta)
        path=OUT/'cases'/f'case_{slug}.png';h,s=save(case(slug,boss,color),path)
        extras.append(dict(file=f'/game/draft/v2/cases/case_{slug}.png',name=f'case_{slug}_v2.png',size=path.stat().st_size,hash=h,shortHash=s,width=1024,height=768,group='items'))
    for name,base,kind in [('wall_dark_alloy',(46,52,55),'panel'),('wall_service_concrete',(92,95,91),'concrete'),('floor_military_grate',(57,64,65),'grate'),('floor_hazard_steel',(48,53,52),'hazard'),('wall_medical_ceramic',(153,163,158),'ceramic'),('wall_old_brick',(70,67,65),'brick'),('ceil_dark_panels',(37,43,46),'panel'),('floor_service_concrete',(72,76,73),'concrete')]:
        path=OUT/'textures'/f'{name}_1080p.png';h,s=save(texture(name,base,len(name)*17,kind),path)
        extras.append(dict(file=f'/game/draft/v2/textures/{name}_1080p.png',name=f'{name}_v2_1080p.png',size=path.stat().st_size,hash=h,shortHash=s,width=1920,height=1080,group='textures'))
    for index,(sector,color) in enumerate(SECTORS):
        name=f'wall_{sector}_sector'
        path=OUT/'textures'/f'{name}_1080p.png';h,s=save(sector_panel(name,color,index),path)
        extras.append(dict(file=f'/game/draft/v2/textures/{name}_1080p.png',name=f'{name}_v2_1080p.png',size=path.stat().st_size,hash=h,shortHash=s,width=1920,height=1080,group='textures'))
        name=f'door_{sector}'
        path=OUT/'textures'/f'{name}_1080p.png';h,s=save(door_texture(sector),path)
        extras.append(dict(file=f'/game/draft/v2/textures/{name}_1080p.png',name=f'{name}_v2_1080p.png',size=path.stat().st_size,hash=h,shortHash=s,width=1920,height=1080,group='textures',seamless=False))
    for name,color,kind in [('muzzle_ballistic',(255,187,71),'flash'),('muzzle_suppressed',(184,200,179),'flash'),('muzzle_rocket',(255,107,42),'flash'),('impact_sparks',(255,201,107),'spark'),('impact_electric',(70,208,255),'spark'),('impact_acid',(149,234,87),'spark'),('projectile_rocket',(255,135,53),'trail'),('projectile_rail',(253,174,64),'trail'),('projectile_arc',(103,224,255),'trail'),('projectile_acid',(144,230,80),'orb'),('smoke_muzzle',(146,151,149),'smoke'),('smoke_explosion',(77,83,83),'smoke'),('shockwave',(150,214,232),'ring'),('explosion_core',(255,134,47),'orb')]:
        path=OUT/'fx'/f'{name}.png';h,s=save(particle(name,color,kind),path)
        extras.append(dict(file=f'/game/draft/v2/fx/{name}.png',name=f'{name}_v2.png',size=path.stat().st_size,hash=h,shortHash=s,width=512,height=512,group='particles'))
    for name,color in [('lantern_amber',(255,188,100)),('lantern_red',(234,81,67)),('worklight_white',(249,245,204)),('worklight_cyan',(138,225,242)),('beacon_warning',(255,112,57)),('medkit',(211,78,72)),('ammo_cache',(209,169,73))]:
        kind=name.split('_')[0]
        path=OUT/'items'/f'{name}.png';h,s=save(object_sprite(kind,color),path)
        extras.append(dict(file=f'/game/draft/v2/items/{name}.png',name=f'{name}_v2.png',size=path.stat().st_size,hash=h,shortHash=s,width=768,height=1024,group='items'))
    for name in ('reactor_unit','server_rack','ac_power_unit','ventilation_array'):
        path=OUT/'items'/f'{name}.png';h,s=save(machinery_sprite(name),path)
        extras.append(dict(file=f'/game/draft/v2/items/{name}.png',name=f'{name}_v2.png',size=path.stat().st_size,hash=h,shortHash=s,width=768,height=1024,group='items'))
    (ROOT/'src/lib/draft-weapons-v2-data.json').write_text(json.dumps(records,indent=2)+'\n')
    (ROOT/'src/lib/draft-assets-v2-data.json').write_text(json.dumps(extras,indent=2)+'\n')
    print(f'generated {len(records)} 5x5 sheets and {len(extras)} supporting assets')

if __name__=='__main__':main()
