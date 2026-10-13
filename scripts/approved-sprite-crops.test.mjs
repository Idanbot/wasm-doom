import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { chromium } from 'playwright';

const dataUrl = path => 'data:image/png;base64,' + readFileSync(path).toString('base64');

test('all weapons return to their approved aim after pickup, reload and firing', async () => {
  const weapons = JSON.parse(readFileSync('src/lib/draft-weapons-v2-data.json'));
  assert.equal(weapons.length, 33);
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    for (const weapon of weapons) {
      const differences = await page.evaluate(async ({sheet, aim}) => {
        const load = async src => { const im = new Image(); im.src = src; await im.decode(); return im; };
        const image = await load(sheet), reference = await load(aim);
        const canvas = document.createElement('canvas'); canvas.width = 512; canvas.height = 384;
        const ctx = canvas.getContext('2d', {willReadFrequently:true});
        ctx.drawImage(reference, 0, 0);
        const approved = ctx.getImageData(0, 0, 512, 384).data;
        return [0, 9, 14, 19, 24].map(frame => {
          ctx.clearRect(0, 0, 512, 384);
          ctx.drawImage(image, frame%5*512, Math.floor(frame/5)*384, 512, 384, 0, 0, 512, 384);
          const actual = ctx.getImageData(0, 0, 512, 384).data;
          let changed = 0;
          for (let i=0; i<actual.length; i++) if (actual[i] !== approved[i]) changed++;
          return changed;
        });
      }, {sheet:dataUrl('public'+weapon.sheet_file), aim:dataUrl('public'+weapon.aim_file)});
      assert.deepEqual(differences, [0,0,0,0,0], `${weapon.id}: approved pose altered or incorrectly cropped`);
    }
  } finally { await browser.close(); }
});

test('level 1 terminal retains its monitor housing, complete base and safe padding', async () => {
  const path = 'game/environment/props/datacenter_terminal_console.png';
  assert.deepEqual(readFileSync('public/'+path), readFileSync('art/source_hd/environment/props/datacenter/datacenter_terminal_console.png'));
  const browser = await chromium.launch({headless:true});
  try {
    const page = await browser.newPage();
    const bounds = await page.evaluate(async src => {
      const im = new Image(); im.src = src; await im.decode();
      const c = document.createElement('canvas'); c.width = c.height = 256;
      const ctx = c.getContext('2d'); ctx.drawImage(im,0,0);
      const d = ctx.getImageData(0,0,256,256).data;
      let top=256,bottom=-1,left=256,right=-1;
      for(let y=0;y<256;y++)for(let x=0;x<256;x++)if(d[(y*256+x)*4+3]>64) {
        top=Math.min(top,y);bottom=Math.max(bottom,y);left=Math.min(left,x);right=Math.max(right,x);
      }
      return {top,bottom,left,right};
    }, dataUrl('public/'+path));
    assert.ok(bounds.top >= 8 && bounds.top <= 20, JSON.stringify(bounds));
    assert.ok(bounds.bottom >= 225 && bounds.bottom < 232, JSON.stringify(bounds));
    assert.ok(bounds.left >= 8 && bounds.right < 248, JSON.stringify(bounds));
    assert.ok(bounds.bottom-bounds.top > 210, 'terminal silhouette lost its top or base');
  } finally { await browser.close(); }
});
