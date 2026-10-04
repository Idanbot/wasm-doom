import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

const baseUrl = process.env.BLACKSITE_PAGES_URL ?? "http://127.0.0.1:8082/blacksite/";

test("project-path deployment loads its arsenal thumbnails", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    const failed = [];
    page.on("response", (response) => {
      if (response.status() >= 400) failed.push(response.url());
    });
    await page.goto(baseUrl);
    await page.waitForFunction(() => !document.querySelector(".deploy-button")?.hasAttribute("disabled"), null, { timeout: 90_000 });
    await page.locator(".deploy-button").click();
    await page.locator(".game-canvas").dispatchEvent("wheel", { deltaY: 100 });
    await page.locator(".weapon-spiral img").first().waitFor();
    const thumbnail = await page.locator(".weapon-spiral img").first().evaluate((img) => ({
      path: new URL(img.src).pathname,
      loaded: img.complete && img.naturalWidth > 0,
    }));
    assert.ok(thumbnail.path.startsWith(new URL("game/ui/weapon-thumbs/", baseUrl).pathname), thumbnail.path);
    assert.ok(thumbnail.loaded, "arsenal thumbnail was blank");
    assert.deepEqual(failed.filter((url) => url.includes("weapon-thumbs")), []);
  } finally {
    await browser.close();
  }
});
