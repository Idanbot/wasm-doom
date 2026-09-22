import assert from "node:assert/strict";
import { chromium } from "playwright";

const browser = await chromium.launch({
  headless: true,
  args: ["--no-sandbox", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
});
try {
  const page = await browser.newPage();
  await page.addInitScript(() => Object.defineProperty(navigator, "gpu", { value: undefined }));
  await page.goto("http://127.0.0.1:8080/");
  const result = await page.evaluate(async () => {
    const { createBlitter } = await import("/src/game/blit.ts");
    const { TEX, TEX_N } = await import("/src/game/gpu-world.ts");
    const canvas = document.createElement("canvas");
    canvas.style.cssText = "width:64px;height:64px";
    document.body.append(canvas);
    const blit = await createBlitter(canvas);
    blit.setGfx({ crt: false, bloom: false, fog: false });
    const atlas = new Uint8Array(TEX_N * TEX * TEX * 4);
    for (let y = 0; y < TEX; y++)
      for (let x = 0; x < TEX; x++) {
        const i = (y * TEX + x) * 4;
        atlas[i + (y < TEX / 2 ? 0 : 2)] = 255;
        atlas[i + 3] = 255;
      }
    // A green world landmark makes horizontal camera motion observable in pixels.
    for (let i = TEX * TEX * 4; i < 2 * TEX * TEX * 4; i += 4) {
      atlas[i + 1] = 255;
      atlas[i + 3] = 255;
    }
    blit.uploadAtlas(atlas);
    const cols = new Float32Array(64 * 16);
    for (let x = 0; x < 64; x++)
      cols.set([1, 0, 0, 0, 0, 0, 63, 64, 0, 0, 0, 0, 0, 1, 0, 0], x * 16);
    const frame = {
      w: 64,
      h: 64,
      view: new Float32Array([4, 4, 1, 0, 0, 0.72, 32, 0, 0, 0, 64, 64, 0, 0, 0, 0]),
      cols,
      sprites: new Float32Array(8),
      spriteCount: 0,
      floor: new Uint8Array(48 * 32),
      light: new Float32Array(48 * 32 * 3),
    };
    const gl = canvas.getContext("webgl2");
    const sample = () => {
      const pixels = new Uint8Array(64 * 64 * 4);
      gl.readPixels(0, 0, 64, 64, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      return {
        top: [...pixels.slice((48 * 64 + 32) * 4, (48 * 64 + 32) * 4 + 3)],
        bottom: [...pixels.slice((16 * 64 + 32) * 4, (16 * 64 + 32) * 4 + 3)],
        error: gl.getError(),
      };
    };
    blit.drawWorld(frame);
    const world = sample();
    // Switch to the CPU upload path in the same context to catch leaked GPU state.
    const cpu = new Uint8Array(64 * 64 * 4);
    for (let y = 0; y < 64; y++)
      for (let x = 0; x < 64; x++) cpu[(y * 64 + x) * 4 + (y < 32 ? 0 : 2)] = 255;
    blit.draw(cpu, 64, 64);
    const upload = sample();
    const { instance } = await WebAssembly.instantiate(
      await (await fetch("/hellscan.wasm")).arrayBuffer(),
      {},
    );
    const e = instance.exports;
    e.hs_init(64, 64);
    e.hs_prepare_gpu();
    const initial = new Float32Array(e.memory.buffer, e.hs_gpu_view(), 16).slice();
    frame.sprites = new Float32Array([
      initial[0] + initial[2] * 3,
      initial[1] + initial[3] * 3,
      0,
      1,
      1,
      0,
      0,
      0,
    ]);
    frame.spriteCount = 1;
    for (let x = 0; x < 64; x++) frame.cols[x * 16 + 14] = 100;
    const landmark = (dx) => {
      e.hs_input(0, dx, 0);
      e.hs_tick(1 / 60);
      e.hs_prepare_gpu();
      frame.view = new Float32Array(e.memory.buffer, e.hs_gpu_view(), 16).slice();
      frame.view[10] = frame.view[11] = 64;
      frame.view[6] = 32;
      blit.drawWorld(frame);
      const pixels = new Uint8Array(64 * 64 * 4);
      gl.readPixels(0, 0, 64, 64, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      let sum = 0,
        count = 0;
      for (let y = 0; y < 64; y++)
        for (let x = 0; x < 64; x++) {
          const i = (y * 64 + x) * 4;
          if (pixels[i + 1] > pixels[i] + 40 && pixels[i + 1] > pixels[i + 2] + 40) {
            sum += x;
            count++;
          }
        }
      if (!count) throw Error("World landmark not visible");
      return sum / count;
    };
    const center = landmark(0),
      afterLeft = landmark(-20),
      afterRight = landmark(40);
    if (!(afterLeft > center && afterRight < center))
      throw Error(`Mouse scene motion inverted: ${center}, ${afterLeft}, ${afterRight}`);
    blit.dispose();
    return { world, upload };
  });
  for (const [path, sample] of Object.entries(result)) {
    assert.equal(sample.error, 0, `${path}: WebGL error`);
    assert.ok(sample.top[0] > sample.top[2], `${path}: top must be red: ${JSON.stringify(sample)}`);
    assert.ok(sample.bottom[2] > sample.bottom[0], `${path}: bottom must be blue`);
  }
  console.log("WebGL world and CPU upload orientation passed");
} finally {
  await browser.close();
}
