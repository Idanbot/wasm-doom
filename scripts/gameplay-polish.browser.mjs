import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

test("Escape transitions debounce; held empty triggers repeat clicks; first-play reload hint expires", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    const url = new URL(process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/");
    url.searchParams.set("qa", "1");
    await page.addInitScript(() => localStorage.setItem("blacksite-res", "640"));
    await page.goto(url.href);
    await page.waitForFunction(() => window.__controlsTest?.getReserve() === 72, null, {
      timeout: 120000,
    });
    await page.evaluate(
      () => (window.__queuedEscape = new KeyboardEvent("keydown", { code: "Escape" })),
    );
    await page.keyboard.press("Escape");
    await page.getByRole("heading", { name: "Operation paused" }).waitFor();
    await page.evaluate(() => window.dispatchEvent(window.__queuedEscape));
    assert.equal(await page.getByRole("heading", { name: "Operation paused" }).count(), 1);
    await page.waitForTimeout(350);
    await page.evaluate(
      () => (window.__queuedEscape = new KeyboardEvent("keydown", { code: "Escape" })),
    );
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => !document.body.innerText.includes("Operation paused"));
    await page.evaluate(() => window.dispatchEvent(window.__queuedEscape));
    await page.waitForTimeout(100);
    assert.equal(await page.getByRole("heading", { name: "Operation paused" }).count(), 0);
    await page.locator(".game-canvas").click();
    await page.waitForFunction(() => window.__controlsTest.getSfxAudio()?.state === "running");
    await page.evaluate(() => {
      window.__controlsTest.heal();
      window.__controlsTest.setKeys(["Space"]);
    });
    await page.waitForFunction(() => window.__controlsTest.getAmmo() === 0, null, {
      timeout: 15000,
    });
    await page.waitForFunction(
      () => window.__controlsTest.getSfxAudio().recent.includes("empty"),
      null,
      { timeout: 10000 },
    );
    const emptyCount = () =>
      page.evaluate(
        () => window.__controlsTest.getSfxAudio().recent.filter((id) => id === "empty").length,
      );
    const once = await emptyCount();
    assert.ok(once >= 1);
    await page.waitForFunction(
      (before) =>
        window.__controlsTest.getSfxAudio().recent.filter((id) => id === "empty").length > before,
      once,
      { timeout: 5000 },
    );
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    await page.waitForTimeout(300);
    const released = await emptyCount();
    await page.waitForTimeout(500);
    assert.equal(await emptyCount(), released, "clicks stop when the trigger is released");
    assert.equal(await page.evaluate(() => window.__controlsTest.getAmmo()), 0);
    await page.evaluate(() => window.__controlsTest.setKeys(["KeyW", "ArrowRight"]));
    const hint = page.getByRole("status").filter({ hasText: "Press R to reload" });
    await hint.waitFor({ timeout: 7000 });
    assert.equal(
      await page.evaluate(() => localStorage.getItem("blacksite-reload-hint-seen")),
      "1",
    );
    await page.screenshot({ path: "screenshots/gameplay-reload-hint.png" });
    await hint.waitFor({ state: "detached", timeout: 4500 });
    await page.waitForTimeout(3500);
    assert.equal(await hint.count(), 0, "tutorial only shows once");
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    assert.deepEqual(errors, []);
  } finally {
    await browser.close();
  }
});
