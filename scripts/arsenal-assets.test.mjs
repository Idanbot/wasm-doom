import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { chromium } from "playwright";

const slugs = ["mk23s", "br12", "kx9", "mr4", "vlk6", "ax12", "m91", "hx8", "vr9", "hc9", "cm9", "ar6", "or7", "gs4", "cr3", "sr0", "ts12", "ks8", "mn6"];

test("every arsenal thumbnail decodes as a visible transparent gun cutout", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const files = slugs.map((slug) => ({
      slug,
      src: `data:image/png;base64,${readFileSync(new URL(`../public/game/ui/weapon-thumbs/${slug}.png`, import.meta.url)).toString("base64")}`,
    }));
    const results = await page.evaluate(async (files) => Promise.all(files.map(async ({ slug, src }) => {
      const image = new Image();
      image.src = src;
      await image.decode();
      const canvas = document.createElement("canvas");
      canvas.width = image.naturalWidth;
      canvas.height = image.naturalHeight;
      const context = canvas.getContext("2d", { willReadFrequently: true });
      context.drawImage(image, 0, 0);
      const rgba = context.getImageData(0, 0, canvas.width, canvas.height).data;
      let opaque = 0;
      let luminance = 0;
      for (let i = 0; i < rgba.length; i += 4) {
        if (rgba[i + 3] < 128) continue;
        opaque++;
        luminance += rgba[i] * 0.2126 + rgba[i + 1] * 0.7152 + rgba[i + 2] * 0.0722;
      }
      return {
        slug,
        width: canvas.width,
        height: canvas.height,
        coverage: opaque / (canvas.width * canvas.height),
        luminance: opaque ? luminance / opaque : 0,
        cornerAlpha: rgba[3],
      };
    })), files);
    assert.equal(results.length, 19);
    for (const image of results) {
      assert.equal(image.width, 640, `${image.slug} width`);
      assert.equal(image.height, 300, `${image.slug} height`);
      assert.ok(image.coverage > 0.08, `${image.slug} is blank`);
      assert.ok(image.coverage < 0.7 && image.cornerAlpha === 0, `${image.slug} has a solid square background`);
      assert.ok(image.luminance > 45, `${image.slug} has invisible gun detail`);
    }
  } finally {
    await browser.close();
  }
});
