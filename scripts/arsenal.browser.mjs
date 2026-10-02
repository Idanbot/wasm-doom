import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

const baseUrl = process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080";
const slugs = ["mk23s", "br12", "kx9", "mr4", "vlk6", "ax12", "m91", "hx8", "vr9", "hc9", "cm9", "ar6", "or7", "gs4", "cr3", "sr0", "ts12", "ks8", "mn6", "vx2", "lv8", "pr5", "mg4", "bm3", "my9", "as7", "sc6", "ub1", "sl12", "nt4", "ch8", "bo11", "sv25"];
let browser;

before(async () => { browser = await chromium.launch({ headless: true }); });
after(async () => { await browser?.close(); });

async function checkpointPage() {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  await page.addInitScript(() => localStorage.setItem("hellscan-checkpoint", JSON.stringify({
    wave: 2, health: 100, armor: 0, weapon: 0, flags: 1,
    ammo: Array(19).fill(12), mag: Array(19).fill(12),
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

test("loading screen fetches and decodes every weapon sheet and thumbnail before deployment", async () => {
  const page = await browser.newPage();
  const requested = new Set();
  let releaseThumbnail;
  let thumbnailStarted;
  const heldThumbnail = new Promise((resolve) => { releaseThumbnail = resolve; });
  const thumbnailRequest = new Promise((resolve) => { thumbnailStarted = resolve; });
  await page.route("**/game/ui/weapon-thumbs/br12.png", async (route) => {
    thumbnailStarted();
    await heldThumbnail;
    await route.continue();
  });
  page.on("request", (request) => requested.add(new URL(request.url()).pathname));
  try {
    await page.goto(baseUrl, { waitUntil: "domcontentloaded" });
    await thumbnailRequest;
    assert.equal(await page.locator(".deploy-button").isDisabled(), true, "deployment became ready before a thumbnail loaded");
    releaseThumbnail();
    await page.waitForFunction(() => !document.querySelector(".deploy-button")?.hasAttribute("disabled"), null, { timeout: 90_000 });
    for (const slug of slugs) {
      assert.ok(requested.has(`/game/ui/weapon-thumbs/${slug}.png`), `${slug} thumbnail was not loaded before deployment`);
      assert.ok(requested.has(`/game/draft/v2/weap_${slug}_5x5.png`), `${slug} viewmodel was not loaded before deployment`);
    }
    for (const path of [
      "/game/ui/hud-panel.webp",
      "/game/ui/menu-reactor.webp",
      "/game/music/menu.mp3",
      "/game/music/bgm-remix.mp3",
      "/game/music/boss.mp3",
      "/game/sfx/fire0.ogg",
    ]) assert.ok(requested.has(path), `${path} was not loaded before deployment`);
  } finally { releaseThumbnail(); await page.close(); }
});

test("a missing thumbnail keeps deployment blocked with an asset error", async () => {
  const page = await browser.newPage();
  try {
    await page.route("**/game/ui/weapon-thumbs/br12.png", (route) => route.fulfill({ status: 404, body: "missing" }));
    await page.goto(baseUrl);
    await page.getByRole("alert").waitFor({ timeout: 90_000 });
    assert.equal(await page.locator(".deploy-button").isDisabled(), true);
    assert.match(await page.getByRole("alert").innerText(), /arsenal|asset|BR-12|br12/i);
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
      const hud = { weapon: 5, ...Object.fromEntries(Array.from({ length: 17 }, (_, i) => [`hasW${i + 2}`, true])) };
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
          transform: getComputedStyle(thumb).transform,
          opacity: Number(getComputedStyle(thumb.closest(".weapon-spiral-slot")).opacity),
          selected: thumb.closest(".weapon-spiral-slot").classList.contains("active"),
        })),
      };
    }, slugs);
    assert.equal(result.images.length, 33);
    for (const image of result.images) assert.ok(image.width > 0 && image.height > 0, `${image.slug} failed to decode`);
    assert.equal(result.slots.length, 5, "the compact wheel should show the selected gun and two neighbors on each side");
    for (const slot of result.slots) {
      assert.ok(slot.loaded, "a wheel thumbnail did not load");
      assert.ok(Number(slot.filter.match(/brightness\(([^)]+)\)/)?.[1]) >= 1.8, `wheel gun lacks contrast: ${slot.filter}`);
      assert.ok(Math.abs(slot.opacity - (slot.selected ? 1 : 0.5)) < 0.01, `incorrect wheel opacity: ${slot.opacity}`);
      if (slot.selected) assert.match(slot.transform, /matrix\(1\.2, 0, 0, 1\.2, 0, 0\)/);
      else assert.match(slot.filter, /saturate\(0\.5\)/);
    }
  } finally { await page.close(); }
});

test("dev catalog shows the current thirty-three weapon sheets and frame controls", async () => {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  try {
    await page.goto(`${baseUrl}/catalog`);
    await page.getByText("LATEST 5×5 WEAPON SET · 33 GUNS").waitFor();
    await page.getByRole("button", { name: /18\. KS-8 KEST/ }).click();
    await page.getByRole("button", { name: "Reload" }).click();
    await page.getByRole("button", { name: "4", exact: true }).click();
    await page.getByRole("img", { name: "KS-8 KEST Reload frame 4" }).waitFor();
    assert.equal(await page.locator(".weapon-spiral-slot").count(), 0);
  } finally { await page.close(); }
});
