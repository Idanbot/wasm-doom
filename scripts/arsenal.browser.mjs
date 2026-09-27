import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

const baseUrl = process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080";
const slugs = ["mk23s", "br12", "kx9", "mr4", "vlk6", "ax12", "m91", "hx8", "vr9", "hc9", "cm9"];
let browser;

before(async () => { browser = await chromium.launch({ headless: true }); });
after(async () => { await browser?.close(); });

async function checkpointPage() {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  await page.addInitScript(() => localStorage.setItem("hellscan-checkpoint", JSON.stringify({
    wave: 2, health: 100, armor: 0, weapon: 0, flags: 1,
    ammo: Array(11).fill(12), mag: Array(11).fill(12),
  })));
  await page.goto(baseUrl);
  await page.locator(".deploy-button").waitFor();
  return page;
}

test("saved checkpoint never adds Resume last sector to the main menu", async () => {
  const page = await checkpointPage();
  try {
    await page.waitForTimeout(500);
    assert.equal(await page.getByText("Resume last sector").count(), 0);
  } finally { await page.close(); }
});

test("mouse-wheel arsenal renders readable transparent gun thumbnails", async () => {
  const page = await checkpointPage();
  try {
    await page.waitForFunction(() => !document.querySelector(".deploy-button")?.hasAttribute("disabled"), null, { timeout: 60_000 });
    await page.locator(".deploy-button").click();
    await page.locator(".game-canvas").dispatchEvent("wheel", { deltaY: 100 });
    await page.locator(".weapon-spiral.visible").waitFor();
    // Render the real wheel component with the full inventory so every visible
    // slot is checked even when a new run begins with only the pistol.
    await page.evaluate(async () => {
      const { WeaponSpiral } = await import("/src/components/game/WeaponSpiral.tsx");
      const React = (await import("/@id/react")).default;
      const { createRoot } = (await import("/@id/react-dom/client")).default;
      const host = document.createElement("div");
      host.id = "arsenal-test-host";
      host.style.cssText = "position:absolute;inset:0;pointer-events:none";
      document.querySelector("main")?.appendChild(host);
      const hud = { weapon: 5, ...Object.fromEntries(Array.from({ length: 10 }, (_, i) => [`hasW${i + 2}`, true])) };
      createRoot(host).render(React.createElement(WeaponSpiral, { hud, visible: true }));
    });
    await page.locator("#arsenal-test-host .weapon-spiral-slot img").first().waitFor();
    const result = await page.evaluate(async (slugs) => {
      const images = await Promise.all(slugs.map((slug) => new Promise((resolve) => {
        const img = new Image();
        img.onload = () => resolve({ slug, width: img.naturalWidth, height: img.naturalHeight });
        img.onerror = () => resolve({ slug, width: 0, height: 0 });
        img.src = `/game/ui/weapon-thumbs/${slug}.png`;
      })));
      const thumbs = [...document.querySelectorAll("#arsenal-test-host .weapon-spiral-slot img")];
      return {
        images,
        slots: thumbs.map((thumb) => ({
          loaded: thumb.complete && thumb.naturalWidth > 0,
          filter: getComputedStyle(thumb).filter,
          opacity: Number(getComputedStyle(thumb.closest(".weapon-spiral-slot")).opacity),
        })),
      };
    }, slugs);
    assert.equal(result.images.length, 11);
    for (const image of result.images) assert.ok(image.width > 0 && image.height > 0, `${image.slug} failed to decode`);
    assert.equal(result.slots.length, 5, "the compact wheel should show the selected gun and two neighbors on each side");
    for (const slot of result.slots) {
      assert.ok(slot.loaded, "a wheel thumbnail did not load");
      assert.ok(Number(slot.filter.match(/brightness\(([^)]+)\)/)?.[1]) >= 1.8, `wheel gun lacks contrast: ${slot.filter}`);
      assert.ok(slot.opacity >= 0.95, `wheel gun is faded: ${slot.opacity}`);
    }
  } finally { await page.close(); }
});
