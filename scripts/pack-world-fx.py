#!/usr/bin/env python3
"""Pack the native-alpha casing master into four padded tumbling cells."""
from pathlib import Path
from PIL import Image
ROOT = Path(__file__).resolve().parents[1]
def pack(name):
    source = Image.open(ROOT / f'art/world-fx-v1/{name}-source.png').convert('RGBA')
    out = Image.new('RGBA', (1024, 1024))
    for i in range(4):
        x, y = i % 2, i // 2
        cell = source.crop((x * source.width // 2, y * source.height // 2,
                            (x + 1) * source.width // 2, (y + 1) * source.height // 2))
        box = cell.getchannel('A').getbbox()
        assert box, f'Empty casing frame {i}'
        cell = cell.crop(box)
        cell.thumbnail((400, 400), Image.Resampling.LANCZOS)
        out.alpha_composite(cell, (x * 512 + (512-cell.width)//2, y * 512 + (512-cell.height)//2))
    out.save(ROOT / f'public/game/fx/{name}-hd.png')

for name in ['casing', 'shell']:
    pack(name)
