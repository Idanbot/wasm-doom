import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import { chromium } from 'playwright';

const base = process.argv[2] || 'http://127.0.0.1:8080';
await mkdir('screenshots', { recursive: true });
const browser = await chromium.launch({ headless: true, args: [
  '--no-sandbox', '--enable-unsafe-webgpu', '--enable-features=Vulkan',
  '--use-angle=vulkan', '--use-vulkan=swiftshader', '--use-webgpu-adapter=swiftshader',
  '--disable-vulkan-surface',
] });
try {
  for (const [mobile, backend] of [[false, 'webgpu'], [true, 'webgl2'], [false, 'canvas2d']]) {
    const context = await browser.newContext({
      viewport: mobile ? { width: 390, height: 844 } : { width: 1280, height: 800 },
      isMobile: mobile, hasTouch: mobile,
    });
    if (backend !== 'webgpu') await context.addInitScript(backend => {
      Object.defineProperty(navigator, 'gpu', { value: undefined });
      if (backend === 'canvas2d') {
        const getContext = HTMLCanvasElement.prototype.getContext;
        HTMLCanvasElement.prototype.getContext = function (kind, ...args) {
          return kind === 'webgl2' ? null : getContext.call(this, kind, ...args);
        };
      }
    }, backend);
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
    await page.goto(`${base}/?qa=1`);
    await page.waitForFunction(() => window.__controlsTest && document.body.innerText.includes('HEALTH'));
    assert.ok((await page.locator('body').innerText()).includes(backend === 'canvas2d' ? '2D' : backend.toUpperCase()));
    await page.evaluate(async () => {
      const t = window.__controlsTest;
      const hold = async keys => {
        t.setKeys(keys);
        // Advance real frames: software GPU startup can exceed a wall-clock key hold.
        for (let i = 0; i < 6; i++) await new Promise(requestAnimationFrame);
        t.setKeys([]);
      };
      const yaw = t.getYaw();
      let x = t.getX(), y = t.getY();
      await hold(['KeyW', 'KeyA']);
      let right = -(t.getX() - x) * Math.sin(yaw) + (t.getY() - y) * Math.cos(yaw);
      if (right >= -0.05) throw Error('A must strafe left while moving forward');
      x = t.getX(); y = t.getY();
      await hold(['KeyW', 'KeyD']);
      right = -(t.getX() - x) * Math.sin(yaw) + (t.getY() - y) * Math.cos(yaw);
      if (right <= 0.05) throw Error('D must strafe right while moving forward');
      const ammo = t.getAmmo();
      await hold(['Space']);
      if (t.getAmmo() >= ammo) throw Error('Firing must consume ammunition');
      await hold(['KeyR']);
      const deadline = performance.now() + 10000;
      while (t.getAmmo() !== 12 && performance.now() < deadline) {
        await new Promise(r => setTimeout(r, 50));
      }
      if (t.getAmmo() !== 12) throw Error('Reload must refill magazine');
    });
    const screenshot = await page.screenshot({ path: `screenshots/game-${mobile ? 'mobile' : backend}.png` });
    const colors = await page.evaluate(async encoded => {
      const image = new Image();
      image.src = `data:image/png;base64,${encoded}`;
      await image.decode();
      const canvas = document.createElement('canvas');
      canvas.width = 200; canvas.height = 100;
      const ctx = canvas.getContext('2d');
      ctx.drawImage(image, image.width * 0.25, image.height * 0.2,
        image.width * 0.5, image.height * 0.2, 0, 0, 200, 100);
      const pixels = new Uint32Array(ctx.getImageData(0, 0, 200, 100).data.buffer);
      return new Set(pixels).size;
    }, screenshot.toString('base64'));
    assert.ok(colors > 32, `${backend}: world is blank (${colors} colors)`);
    await page.keyboard.press('p');
    await page.getByRole('button', { name: 'Resume', exact: true }).waitFor();
    const position = await page.evaluate(() => [window.__controlsTest.getX(), window.__controlsTest.getY()]);
    await page.waitForTimeout(200);
    assert.deepEqual(await page.evaluate(() => [window.__controlsTest.getX(), window.__controlsTest.getY()]), position);
    await page.getByRole('button', { name: 'Restart', exact: true }).click();
    await page.waitForTimeout(200);
    assert.equal(await page.evaluate(() => window.__controlsTest.getAmmo()), 12);
    if (mobile) {
      const cdp = await context.newCDPSession(page);
      const send = (type, touchPoints) => cdp.send('Input.dispatchTouchEvent', { type, touchPoints });
      await send('touchStart', [{ x: 80, y: 550, id: 1 }]);
      await send('touchMove', [{ x: 80, y: 510, id: 1 }]);
      await send('touchStart', [{ x: 80, y: 510, id: 1 }, { x: 270, y: 450, id: 2 }]);
      await send('touchMove', [{ x: 80, y: 510, id: 1 }, { x: 290, y: 450, id: 2 }]);
      // A stationary move finger must keep its original origin while the look finger moves.
      await send('touchMove', [{ x: 81, y: 510, id: 1 }, { x: 290, y: 450, id: 2 }]);
      await page.waitForTimeout(150);
      assert.ok(await page.evaluate(() => window.__controlsTest.getX() > 4.55));
      await send('touchEnd', []);
      await page.waitForTimeout(100);
      assert.equal(await page.evaluate(() => window.__controlsTest.getSpeed()), 0);
      assert.ok(await page.evaluate(() => {
        const canvas = document.querySelector('canvas');
        const [w, h] = canvas.dataset.resolution.split(' × ').map(Number);
        return Math.abs(w / h - innerWidth / innerHeight) < 0.01;
      }));
    }
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
    assert.deepEqual(errors, []);
    console.log(`${mobile ? 'mobile' : 'desktop'} ${backend}: movement, firing, reload, pause, restart, console${mobile ? ', multitouch and aspect ratio' : ''} passed`);
    await context.close();
  }
} finally { await browser.close(); }
