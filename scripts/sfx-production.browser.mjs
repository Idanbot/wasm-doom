import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
const base = process.env.BLACKSITE_PRODUCTION_URL ?? "http://127.0.0.1:8081/";
test("production SFX preload and MP3 exports respect the deployment base path", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage(),
      requested = new Set(),
      errors = [];
    page.on("request", (r) => requested.add(new URL(r.url()).pathname));
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(base, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(
      () => {
        const b = document.querySelector(".deploy-button");
        return b && !b.hasAttribute("disabled");
      },
      null,
      { timeout: 90000 },
    );
    const exports = await page.evaluate(async (base) => {
      const manifest = await (await fetch(new URL("game/sfx/v2/manifest.json", base))).json();
      const ctx = new AudioContext();
      let decoded = 0;
      // Ready already guarantees Vorbis decode; explicitly verify each production MP3 fallback.
      for (const c of manifest.clips) {
        const url = new URL(c.fallback.replace(/^\//, ""), base);
        const r = await fetch(url);
        if (!r.ok) throw Error("Missing " + url);
        const b = await ctx.decodeAudioData(await r.arrayBuffer());
        if (b.numberOfChannels !== 1 || b.duration <= 0) throw Error("Bad " + url);
        decoded++;
      }
      await ctx.close();
      return { decoded, clips: manifest.clips };
    }, base);
    assert.equal(exports.decoded, 103);
    for (const c of exports.clips)
      assert.ok(
        requested.has(new URL(c.url.replace(/^\//, ""), base).pathname),
        c.id + " not preloaded",
      );
    assert.deepEqual(errors, []);
    assert.equal(await page.evaluate(() => typeof window.__controlsTest), "undefined");
  } finally {
    await browser.close();
  }
});
