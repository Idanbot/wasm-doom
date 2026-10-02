import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { chromium } from 'playwright';

const bosses = ['vr9', 'hc9', 'cm9', 'ar6', 'or7', 'gs4', 'cr3', 'sr0', 'ts12', 'ks8', 'mn6', ...JSON.parse(readFileSync(new URL('../art/campaign25/specs.json', import.meta.url))).map(b => b.weapon)];

test('all boss cases are golden native-alpha pickups and wheel icons are wide side profiles', async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    for (const id of bosses) {
      for (const kind of ['case', 'pickup', 'side']) {
        const path = kind === 'case' ? `game/draft/v2/cases/case_${id}.png` : kind === 'pickup' ? `game/spr_gun_${id}.png` : `game/ui/weapon-thumbs/${id}.png`;
        const bytes = readFileSync(new URL(`../public/${path}`, import.meta.url));
        const image = await page.evaluate(async (src) => {
          const image = new Image(); image.src = src; await image.decode();
          const canvas = document.createElement('canvas'); canvas.width = image.width; canvas.height = image.height;
          const ctx = canvas.getContext('2d', { willReadFrequently: true }); ctx.drawImage(image, 0, 0);
          const pixels = ctx.getImageData(0, 0, image.width, image.height).data;
          let opaque = 0, gold = 0, minX = image.width, maxX = 0, minY = image.height, maxY = 0;
          for (let y = 0; y < image.height; y++) for (let x = 0; x < image.width; x++) {
            const i = (y * image.width + x) * 4;
            if (pixels[i + 3] < 128) continue;
            opaque++;
            if (pixels[i] > 110 && pixels[i + 1] > 65 && pixels[i + 1] > pixels[i + 2] * 1.3 && pixels[i] > pixels[i + 1] * 1.05) gold++;
            minX = Math.min(minX, x); maxX = Math.max(maxX, x); minY = Math.min(minY, y); maxY = Math.max(maxY, y);
          }
          return { width: image.width, height: image.height, corner: pixels[3], coverage: opaque / (image.width * image.height), gold: gold / opaque, aspect: (maxX - minX + 1) / (maxY - minY + 1), minX, maxX };
        }, `data:image/png;base64,${bytes.toString('base64')}`);
        assert.equal(image.corner, 0, `${id} ${kind}: opaque background`);
        assert.ok(image.coverage > .08 && image.coverage < .8, `${id} ${kind}: blank or opaque rectangle`);
        if (kind === 'side') {
          assert.deepEqual([image.width, image.height], [640, 300]);
          assert.ok(image.aspect > 1.8, `${id}: not a readable horizontal gun profile`);
          assert.ok(image.minX >= 25 && image.maxX < 615, `${id}: gun touches icon edge`);
        } else {
          assert.deepEqual([image.width, image.height], kind === 'case' ? [1024, 768] : [256, 256]);
          assert.ok(image.gold > .06, `${id}: case isn't golden (${image.gold})`);
        }
      }
    }
    const manifest = JSON.parse(readFileSync(new URL('../art/boss-reward-art-v3/generation.json', import.meta.url)));
    assert.equal(manifest.assets.length, 22);
    for (const source of manifest.assets) assert.equal(createHash('sha256').update(readFileSync(new URL(`../${source.file}`, import.meta.url))).digest('hex'), source.sha256);
  } finally { await browser.close(); }
});
