import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { chromium } from 'playwright';

const root = new URL('../', import.meta.url);
const weapons = JSON.parse(readFileSync(new URL('src/lib/draft-weapons-v2-data.json', root)));
const bosses = weapons.filter((weapon) => weapon.is_boss);
const provenance = JSON.parse(readFileSync(new URL('art/weapon-animations-v3/generation.json', root)));

test('all boss sheets have distinct handling poses, clean alpha and exact approved aim returns', async () => {
  assert.equal(bosses.length, 11);
  assert.deepEqual(provenance.weapons.map((entry) => entry.id).sort(), bosses.map((entry) => entry.id).sort());
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    for (const weapon of bosses) {
      const source = provenance.weapons.find((entry) => entry.id === weapon.id);
      assert.equal(createHash('sha256').update(readFileSync(new URL(source.source, root))).digest('hex'), source.source_sha256, `${weapon.id}: source board changed`);
      assert.equal(source.reference_sha256, weapon.aim_hash, `${weapon.id}: source references a different approved aim`);
      const bytes = readFileSync(new URL(`public${weapon.sheet_file}`, root));
      assert.equal(createHash('sha256').update(bytes).digest('hex'), weapon.sheet_hash, `${weapon.id}: stale catalog hash`);
      assert.equal(weapon.animation_version, 3, `${weapon.id}: animation not reworked`);
      const result = await page.evaluate(async ({ sheet, aim }) => {
        const load = async (src) => { const image = new Image(); image.src = src; await image.decode(); return image; };
        const image = await load(sheet);
        const reference = await load(aim);
        const canvas = document.createElement('canvas');
        canvas.width = 512; canvas.height = 384;
        const ctx = canvas.getContext('2d', { willReadFrequently: true });
        const pixels = (index) => {
          ctx.clearRect(0, 0, 512, 384);
          ctx.drawImage(image, index % 5 * 512, Math.floor(index / 5) * 384, 512, 384, 0, 0, 512, 384);
          return ctx.getImageData(0, 0, 512, 384).data;
        };
        ctx.drawImage(reference, 0, 0);
        const original = ctx.getImageData(0, 0, 512, 384).data;
        const frames = Array.from({ length: 25 }, (_, index) => pixels(index));
        const difference = (a, b) => { let count = 0; for (let i = 0; i < a.length; i += 4) if (Math.abs(a[i] - b[i]) + Math.abs(a[i + 1] - b[i + 1]) + Math.abs(a[i + 2] - b[i + 2]) + Math.abs(a[i + 3] - b[i + 3]) > 20) count++; return count; };
        const coverage = frames.map((frame) => { let count = 0; for (let i = 3; i < frame.length; i += 4) if (frame[i] > 128) count++; return count / (512 * 384); });
        const clear = [3, 4, 6, 7, 8, 10, 11, 12, 13].every((index) => {
          for (let y = 184; y < 200; y++) for (let x = 248; x < 264; x++) if (frames[index][(y * 512 + x) * 4 + 3] >= 8) return false;
          return true;
        });
        return {
          size: [image.naturalWidth, image.naturalHeight], coverage, clear,
          returns: [0, 9, 14, 19, 24].map((index) => difference(frames[index], original)),
          dry: difference(frames[5], frames[3]),
          half: difference(frames[1], frames[0]),
          distinct: [[6, 7], [7, 8], [8, 9], [10, 11], [11, 12], [12, 13], [13, 14], [3, 4], [15, 19]].map(([a, b]) => difference(frames[a], frames[b])),
          corners: frames.map((frame) => frame[3]),
        };
      }, {
        sheet: `data:image/png;base64,${bytes.toString('base64')}`,
        aim: `data:image/png;base64,${readFileSync(new URL(`public${weapon.aim_file}`, root)).toString('base64')}`,
      });
      assert.deepEqual(result.size, [2560, 1920], weapon.id);
      assert.deepEqual(result.returns, [0, 0, 0, 0, 0], `${weapon.id}: approved aim changed`);
      assert.equal(result.dry, 0, `${weapon.id}: dry-fire animation must stay disabled`);
      if (['ks8', 'ts12', 'gs4'].includes(weapon.id)) assert.equal(result.half, 0, `${weapon.id}: opaque feed must not dim painted chassis as ammunition drops`);
      assert.ok(result.clear, `${weapon.id}: handling pose covers reticle`);
      assert.ok(result.coverage.every((coverage) => coverage > .025 && coverage < .65), `${weapon.id}: blank frame or opaque backdrop`);
      assert.ok(result.corners.every((alpha) => alpha === 0), `${weapon.id}: backdrop is not transparent`);
      assert.ok(result.distinct.every((count) => count > 1200), `${weapon.id}: repeated handling pose (${result.distinct})`);
    }
  } finally { await browser.close(); }
});
