"""Extract the approved expansion concept grids into gameplay-ready cutouts."""
from pathlib import Path
from PIL import Image, ImageEnhance, ImageFilter
import numpy as np
from scipy.ndimage import label

ROOT = Path(__file__).resolve().parents[1]
BOSSES = ('halcyon', 'relay', 'titan', 'kest')
GUNS = ('or7', 'gs4', 'cr3', 'sr0', 'ts12', 'ks8')

def subject(image: Image.Image) -> Image.Image:
    alpha = image.getchannel('A')
    bounds = alpha.point(lambda a: 255 if a > 15 else 0).getbbox()
    if bounds is None:
        raise ValueError('empty generated cell')
    return image.crop(bounds)

def main() -> None:
    boss_grid = Image.open(ROOT / 'art/expansion/bosses-source.png').convert('RGBA')
    gun_grid = Image.open(ROOT / 'art/expansion/weapons-source.png').convert('RGBA')
    for i, name in enumerate(BOSSES):
        x, y = i % 2, i // 2
        base = subject(boss_grid.crop((x * 627, y * 627, (x + 1) * 627, (y + 1) * 627)))
        out = ROOT / f'art/bosses/{name}-source.png'
        base.save(out)
        # Separate attack poses retain the same silhouette and equipment.
        for action, scale, shift, light in [('fire', 1.045, (-9, 0), 1.18), ('special', 1.06, (5, -9), 1.32)]:
            pose = ImageEnhance.Brightness(base).enhance(light)
            pose = pose.resize((round(pose.width * scale), round(pose.height * scale)), Image.Resampling.LANCZOS)
            canvas = Image.new('RGBA', (max(base.width, pose.width) + 30, max(base.height, pose.height) + 30))
            px = (canvas.width - pose.width) // 2 + shift[0]
            py = (canvas.height - pose.height) // 2 + shift[1]
            canvas.alpha_composite(pose, (px, py))
            canvas.save(ROOT / f'art/bosses/{name}-{action}-source.png')
    for i, name in enumerate(GUNS):
        x, y = i % 3, i // 3
        panel = gun_grid.crop((x * 483, y * 543, min((x + 1) * 483, gun_grid.width), min((y + 1) * 543, gun_grid.height)))
        rgba = np.array(panel)
        components, total = label(rgba[:, :, 3] > 35)
        if total:
            sizes = np.bincount(components.ravel())
            largest = sizes[1:].argmax() + 1
            rgba[:, :, 3][components != largest] = 0
        if name == 'ks8':
            # The neighboring siege cannon touches the left edge of this
            # concept cell below the rifle; remove that narrow grid bleed.
            rgba[300:, :18, 3] = 0
        gun = subject(Image.fromarray(rgba, 'RGBA'))
        gun.save(ROOT / f'public/game/draft/v2/masters/{name}.png')

if __name__ == '__main__':
    main()
