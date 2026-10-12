import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

const baseUrl = process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/";

test("world material shaders compile and create WebGPU pipelines", async t => {
  const browser = await chromium.launch({ args: ["--enable-unsafe-webgpu", "--use-angle=swiftshader"] });
  try {
    const page = await browser.newPage();
    await page.route("**/__materials_test.html", route => route.fulfill({ contentType: "text/html", body: "<!doctype html><body></body>" }));
    await page.goto(new URL("/__materials_test.html", baseUrl).href);
    const result = await page.evaluate(async () => {
      const adapter = await navigator.gpu?.requestAdapter();
      if (!adapter) return { unavailable: true };
      const device = await adapter.requestDevice();
      const { compileShader, WORLD_SHADER_SOURCES } = await import("/src/game/gpu-world.ts");
      device.pushErrorScope("validation");
      for (const [label, code] of Object.entries(WORLD_SHADER_SOURCES)) {
        const module = await compileShader(device, code, label);
        await device.createRenderPipelineAsync({ layout: "auto",
          vertex: { module, entryPoint: "vs" },
          fragment: { module, entryPoint: "fs", targets: [{ format: "rgba8unorm" }] },
          primitive: { topology: "triangle-list" } });
      }
      const error = await device.popErrorScope();
      device.destroy();
      return { error: error?.message ?? null };
    });
    if (result.unavailable) { t.skip("WebGPU adapter unavailable"); return; }
    assert.equal(result.error, null);
  } finally { await browser.close(); }
});

test("world-space reflections, sector cosmetics, shafts and boss pulses preserve occlusion and weapon colours", async () => {
  const browser = await chromium.launch({ args: ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
  try {
    const page = await browser.newPage({ viewport: { width: 768, height: 576 } });
    const errors = []; page.on("pageerror", e => errors.push(e.message));
    await page.addInitScript(() => Object.defineProperty(navigator, "gpu", { value: undefined }));
    await page.route("**/__materials_test.html", route => route.fulfill({ contentType: "text/html", body: "<!doctype html><body></body>" }));
    await page.goto(new URL("/__materials_test.html", baseUrl).href);
    const result = await page.evaluate(async () => {
      const { createBlitter } = await import("/src/game/blit.ts");
      const { TEX, TEX_N, T_SECTOR_SURFACE } = await import("/src/game/gpu-world.ts");
      const canvas = document.createElement("canvas");
      canvas.style.cssText = "width:768px;height:576px"; document.body.replaceChildren(canvas);
      const blit = await createBlitter(canvas); blit.setGfx({ crt: false, fog: false, bloom: false });
      const w = 128, h = 96, atlas = new Uint8Array(TEX_N * TEX * TEX * 4);
      const texture = color => {
        const p = new Uint8Array(TEX * TEX * 4);
        for (let i = 0; i < p.length; i += 4) p.set([...color, 255], i);
        return p;
      };
      const floor = texture([60, 65, 70]), red = texture([220, 20, 20]), blue = texture([20, 20, 220]);
      for (let level = 0; level < 25; level++) {
        atlas.set(floor, (T_SECTOR_SURFACE + level * 5 + 3) * TEX * TEX * 4);
        atlas.set(red, (T_SECTOR_SURFACE + level * 5 + 4) * TEX * TEX * 4);
      }
      atlas.set(floor, 0); blit.uploadAtlas(atlas);
      const cols = new Float32Array(w * 16);
      for (let x = 0; x < w; x++) cols[x * 16 + 14] = 100;
      const frame = { w, h, view: new Float32Array([3, 2, 1, 0, 0, .72, 48, 2, 0, 0, w, h, .72, 0, 0, 0]),
        cols, sprites: new Float32Array(8), spriteCount: 0, floor: new Uint8Array(48 * 32),
        light: new Float32Array(48 * 32 * 3), smoke: new Float32Array(48 * 32) };
      const gl = canvas.getContext("webgl2");
      const read = fx => {
        if (!blit.drawWorld(frame, fx)) throw new Error("GPU world draw failed");
        const pixels = new Uint8Array(canvas.width * canvas.height * 4);
        gl.readPixels(0, 0, canvas.width, canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, pixels); return pixels;
      };
      // Bottom 30% only: no ceiling pixels or horizon post-filter taps.
      const floorDiff = (a, b) => a.slice(0, canvas.width * Math.floor(canvas.height * .30) * 4)
        .reduce((n, v, i) => n + Math.abs(v - b[i]), 0);
      const original = read();
      blit.uploadAtlasLayer(T_SECTOR_SURFACE + 4, blue);
      const reflected = read(), reflection = floorDiff(original, reflected);
      frame.floor.fill(3);
      const marked = read(); blit.uploadAtlasLayer(T_SECTOR_SURFACE + 4, red);
      const protectedMarkers = floorDiff(marked, read());
      frame.floor.fill(0); frame.view[14] = 1;
      const sectorVariation = floorDiff(original, read()); frame.view[14] = 0;
      blit.uploadAtlasLayer(T_SECTOR_SURFACE + 3, texture([0, 0, 0]));
      const dark = read(); frame.light.fill(.5);
      const scattering = floorDiff(dark, read());
      // A wall ends each view ray before the emitter region. Hidden emitters
      // cannot light the haze on our side of that wall.
      for (let x = 0; x < w; x++) cols.set([2, 0, 0, 0, 0, 0, h - 1, h, 0, 0, 0, 0, 0, 1, 2, 0], x * 16);
      frame.light.fill(0); const sealed = read();
      for (let y = 0; y < 32; y++) for (let x = 9; x < 48; x++) frame.light.set([1, .2, .1], (y * 48 + x) * 3);
      const hiddenLight = floorDiff(sealed, read());
      // Postprocessing phase changes must not tint zero-depth weapon pixels.
      const pixels = new Uint8Array(w * h * 4);
      for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) pixels.set([80, 90, 100, x < w / 2 ? 0 : 150], (y * w + x) * 4);
      const post = encounter => {
        blit.draw(pixels, w, h, { muzzle: 0, hurt: 0, time: 2, boss: -1, encounter });
        const out = new Uint8Array(canvas.width * canvas.height * 4);
        gl.readPixels(0, 0, canvas.width, canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, out); return out;
      };
      const neutral = post(), pulse = post({ phase: 2, pulse: .5, color: [.8, .2, 1] });
      let weaponDiff = 0, phaseDiff = 0;
      for (let y = 0; y < canvas.height; y++) for (let x = 0; x < canvas.width; x++) {
        if (x > canvas.width * .45 && x < canvas.width * .55) continue;
        const i = (y * canvas.width + x) * 4;
        const d = Math.abs(neutral[i] - pulse[i]) + Math.abs(neutral[i + 1] - pulse[i + 1]) + Math.abs(neutral[i + 2] - pulse[i + 2]);
        if (x < canvas.width / 2) weaponDiff += d; else phaseDiff += d;
      }
      // Leave the world fixture visible for screenshot inspection.
      for (let x = 0; x < w; x++) cols.set(new Float32Array(16), x * 16);
      for (let x = 0; x < w; x++) cols[x * 16 + 14] = 100;
      blit.uploadAtlasLayer(T_SECTOR_SURFACE + 3, floor); frame.light.fill(.35); read();
      return { reflection, protectedMarkers, sectorVariation, scattering, hiddenLight, weaponDiff, phaseDiff, kind: blit.kind, error: gl.getError() };
    });
    await page.screenshot({ path: "screenshots/world-materials.png" });
    assert.equal(result.kind, "webgl2"); assert.equal(result.error, 0);
    assert.ok(result.reflection > 1000, JSON.stringify(result));
    assert.equal(result.protectedMarkers, 0);
    assert.ok(result.sectorVariation > 1000, JSON.stringify(result));
    assert.ok(result.scattering > 1000, JSON.stringify(result));
    assert.equal(result.hiddenLight, 0);
    assert.equal(result.weaponDiff, 0); assert.ok(result.phaseDiff > 1000);
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
