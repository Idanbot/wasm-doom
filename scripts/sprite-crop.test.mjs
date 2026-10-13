import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { chromium } from 'playwright';
import { PROJECTILE_ART } from '../src/game/projectile-art.ts';

// Test decoded pixels rather than PNG headers: a valid sheet can still contain
// part of its neighbor, or have its own silhouette cut at a cell boundary.
test('projectiles, casings and reload ejections have transparent frame boundaries', async () => {
  const paths = readdirSync('public/game/projectiles', { recursive: true })
    .filter(p => p.endsWith('.png')).map(p => [`public/game/projectiles/${p}`, 1]);
  paths.push(['public/game/spr_ball.png', 1], ['public/game/spr_ordnance.png', 2]);
  for (const name of ['rocket-forward', 'ion-bolt', 'cryo-shard', 'spore-cluster', 'phase-orb'])
    paths.push([`public${PROJECTILE_ART[`/game/fx25/${name}.png`] ?? `/game/fx25/${name}.png`}`, name === 'rocket-forward' ? 2 : 1]);
  for (const name of ['casing', 'shell', 'fuel-debris', 'acid-debris', 'specimen-debris', 'pressure-debris'])
    paths.push([`public/game/fx/${name}-hd.png`, 2]);
  for (const p of readdirSync('public/game/fx/feeds'))
    if (p.endsWith('.png')) paths.push([`public/game/fx/feeds/${p}`, 2]);
  paths.push(['public/game/environment/props/datacenter_terminal_console.png', 1]);
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    for (const [path, grid] of paths) {
      const result = await page.evaluate(async ({ src, grid }) => {
        const image = new Image(); image.src = src; await image.decode();
        const c = document.createElement('canvas'); c.width = image.width; c.height = image.height;
        const ctx = c.getContext('2d'); ctx.drawImage(image, 0, 0);
        const w = c.width / grid, h = c.height / grid; const frames = [];
        for (let fy = 0; fy < grid; fy++) for (let fx = 0; fx < grid; fx++) {
          const d = ctx.getImageData(fx*w, fy*h, w, h).data;
          let border = 0, visible = 0;
          for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
            const alpha = d[(y*w+x)*4+3];
            if (alpha > 10) visible++;
            if (alpha > 10 && (x < 2 || y < 2 || x >= w-2 || y >= h-2)) border++;
          }
          frames.push({ frame: fy*grid+fx, border, visible });
        }
        return frames;
      }, { src: 'data:image/png;base64,' + readFileSync(path).toString('base64'), grid });
      assert.equal(result.length, grid*grid, `${path}: missing frame`);
      for (const frame of result) {
        assert.equal(frame.border, 0, `${path} frame ${frame.frame}: silhouette touches a frame boundary`);
        assert.ok(frame.visible > 100, `${path} frame ${frame.frame}: empty artwork`);
      }
    }
  } finally { await browser.close(); }
});
