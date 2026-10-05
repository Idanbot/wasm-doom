import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

for (const mode of ["webgl2", "canvas2d"])
  test(`paused resolution changes redraw ${mode} and restore graphics without restarting`, async () => {
    const browser = await chromium.launch({ headless: true });
    try {
      const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
      const errors = [];
      page.on("pageerror", (e) => errors.push(e.message));
      await page.addInitScript((mode) => {
        localStorage.setItem("blacksite-sensitivity", "1.4");
        Object.defineProperty(navigator, "gpu", { value: undefined });
        if (mode === "canvas2d") {
          const getContext = HTMLCanvasElement.prototype.getContext;
          HTMLCanvasElement.prototype.getContext = function (type, ...args) {
            return type === "webgl2" || type === "webgpu"
              ? null
              : getContext.call(this, type, ...args);
          };
        }
      }, mode);
      const url = new URL(process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/");
      url.searchParams.set("qa", "1");
      await page.goto(url.href);
      await page.waitForFunction(() => window.__controlsTest?.getReserve() === 72, null, {
        timeout: 120000,
      });
      await page.keyboard.press("p");
      await page.getByRole("button", { name: "Settings", exact: true }).click();
      await page.getByRole("button", { name: "Controls", exact: true }).click();
      // Legacy default migration is checked after renderer regression assertions.
      const sensitivity = await page.getByRole("slider", { name: "Look sensitivity" }).inputValue();
      await page.getByRole("button", { name: "Display", exact: true }).click();
      for (const res of ["320", "1440", "640", "320"])
        await page.getByRole("combobox", { name: "Render resolution" }).selectOption(res);
      await page.getByRole("button", { name: "Done", exact: true }).click();
      const state = () =>
        page.evaluate(() => ({
          x: window.__controlsTest.getX(),
          y: window.__controlsTest.getY(),
          ammo: window.__controlsTest.getAmmo(),
          reserve: window.__controlsTest.getReserve(),
        }));
      const frozen = await state();
      const checkWorld = async (name) => {
        await page.waitForTimeout(400);
        const png = await page
          .locator(".game-canvas")
          .screenshot({
            path: `screenshots/resolution-${mode}-${name}.png`,
            style: ".game-canvas ~ * { visibility: hidden !important; }",
          });
        const brightness = await page.evaluate(async (data) => {
          const img = new Image();
          img.src = `data:image/png;base64,${data}`;
          await img.decode();
          const c = document.createElement("canvas");
          c.width = img.width;
          c.height = img.height;
          const ctx = c.getContext("2d");
          ctx.drawImage(img, 0, 0);
          const p = ctx.getImageData(0, 0, c.width, c.height).data;
          let sum = 0;
          for (let i = 0; i < p.length; i += 4) sum += p[i] + p[i + 1] + p[i + 2];
          return sum / ((p.length / 4) * 3);
        }, png.toString("base64"));
        assert.ok(brightness > 20, `${name}: world canvas is blank (${brightness})`);
      };
      await page.setViewportSize({ width: 960, height: 600 });
      await checkWorld("paused-resize");
      assert.deepEqual(await state(), frozen);
      if (mode === "webgl2") {
        const supported = await page.evaluate(() => {
          const gl = document.querySelector(".game-canvas").getContext("webgl2");
          const ext = gl?.getExtension("WEBGL_lose_context");
          if (!ext) return false;
          ext.loseContext();
          setTimeout(() => ext.restoreContext(), 150);
          return true;
        });
        assert.ok(supported, "WebGL context loss extension is available");
        await page.waitForTimeout(1500);
        await checkWorld("restored");
        assert.deepEqual(await state(), frozen);
      }
      await page.keyboard.press("p");
      await checkWorld("resume");
      assert.equal(sensitivity, "0.5");
      assert.deepEqual(errors, []);
    } finally {
      await browser.close();
    }
  });
