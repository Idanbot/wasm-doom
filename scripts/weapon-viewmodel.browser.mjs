import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

/**
 * Regression guard for the gun viewmodel disappearing after a resolution change
 * on the pause screen.
 *
 * The viewmodel canvas unmounts while the game is paused and remounts on
 * resume. The frame loop caches the canvas 2D context so it does not walk the
 * DOM 60 times a second; if that cache outlives the canvas it keeps drawing
 * into a detached element and the gun silently renders nothing. The canvas
 * exists and is the right size, so nothing else fails - only the pixels are
 * missing. This asserts on pixels, not on element presence.
 */
test("viewmodel keeps drawing across pause, resume and wave changes", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 960, height: 600 } });
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    const url = new URL(process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/");
    url.searchParams.set("qa", "1");
    await page.goto(url.href);
    await page.waitForFunction(() => window.__controlsTest?.isLive(), null, {
      timeout: 180000,
    });

    /** Opaque pixel count in the weapon viewmodel canvas. */
    const sample = () =>
      page.evaluate(async () => {
        // Let a few frames present before sampling.
        for (let i = 0; i < 8; i++) await new Promise((r) => requestAnimationFrame(r));
        const canvas = document.querySelector(".weapon-view canvas");
        // A mounted viewmodel with nothing drawn means the cached context is
        // bound to a detached canvas.
        if (!canvas) return { error: "not-mounted" };
        const data = canvas.getContext("2d").getImageData(0, 0, canvas.width, canvas.height)
          .data;
        let opaque = 0;
        for (let i = 3; i < data.length; i += 4) if (data[i] > 8) opaque++;
        return { opaque };
      });

    const first = await sample();
    assert.equal(first.error, undefined, `viewmodel should be mounted: ${first.error}`);
    assert.ok(
      first.opaque > 1000,
      `expected a drawn viewmodel, got ${first.opaque} opaque pixels`,
    );
    const baseline = first.opaque;

    // Pausing unmounts the viewmodel; resuming remounts it.
    await page.keyboard.press("Escape");
    await page.getByRole("heading", { name: "Operation paused" }).waitFor();
    await page.getByRole("button", { name: "Resume operation" }).click();
    await page.waitForFunction(() => !document.body.innerText.includes("Operation paused"));
    assert.equal(
      await sample().then((r) => r.opaque),
      baseline,
      "the viewmodel must still draw after a pause/resume cycle",
    );

    // The reported failure: change resolution on the pause screen, then resume.
    await page.keyboard.press("Escape");
    await page.getByRole("heading", { name: "Operation paused" }).waitFor();
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.getByRole("button", { name: "Display", exact: true }).click();
    await page.getByRole("combobox", { name: "Render resolution" }).selectOption("320");
    await page.getByRole("button", { name: "Done", exact: true }).click();
    await page.getByRole("button", { name: "Resume operation" }).click();
    await page.waitForFunction(() => !document.body.innerText.includes("Operation paused"));
    assert.equal(
      await sample().then((r) => r.opaque),
      baseline,
      "the viewmodel must still draw after a resolution change on the pause screen",
    );

    // And after advancing a sector, which swaps themes and remounts assets.
    await page.evaluate(() => window.__controlsTest.nextWave());
    await page.waitForFunction(() => window.__controlsTest.getReserve() > 0, null, {
      timeout: 60000,
    });
    assert.equal(
      await sample().then((r) => r.opaque),
      baseline,
      "the viewmodel must still draw after a sector change",
    );

    assert.deepEqual(errors, []);
  } finally {
    await browser.close();
  }
});
