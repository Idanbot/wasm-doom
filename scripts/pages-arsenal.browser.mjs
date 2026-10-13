import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

const baseUrl = process.env.BLACKSITE_PAGES_URL ?? "http://127.0.0.1:8082/blacksite/";

test("project-path deployment loads its arsenal thumbnails", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    const failed = [], requested = [];
    await page.addInitScript(()=>localStorage.setItem("blacksite-res","320"));
    page.on("request",request=>requested.push(request.url()));
    page.on("response", (response) => {
      if (response.status() >= 400) failed.push(response.url());
    });
    await page.goto(baseUrl);
    await page.waitForFunction(() => !document.querySelector(".deploy-button")?.hasAttribute("disabled"), null, { timeout: 90_000 });
    await page.locator(".deploy-button").click();
    await page.locator(".game-canvas").dispatchEvent("wheel", { deltaY: 100 });
    await page.locator(".weapon-spiral img").first().waitFor();
    const thumbnail = await page.locator(".weapon-spiral img").first().evaluate((img) => ({
      source: img.src,
      loaded: img.complete && img.naturalWidth > 0,
    }));
    assert.match(thumbnail.source,/^blob:/,"arsenal uses the validated decoded image URL");
    assert.ok(requested.some(url=>new URL(url).pathname.startsWith(new URL("game/ui/weapon-thumbs/",baseUrl).pathname) && url.includes("blacksite_version=")),"versioned thumbnails must load under the deployment base");
    assert.ok(thumbnail.loaded, "arsenal thumbnail was blank");
    assert.deepEqual(failed.filter((url) => url.includes("weapon-thumbs")), []);
  } finally {
    await browser.close();
  }
});
