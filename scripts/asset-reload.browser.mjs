import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

/**
 * Settings > Display > "Reload all assets" repairs the texture atlas after a
 * decode that failed once (flaky connection, or a GPU context loss during a
 * paused resolution change). Without it the world stays blank for the rest of
 * the session with no recovery.
 *
 * The click itself is asserted through the real dialog; the runtime call is
 * driven directly so the assertions are about the outcome (atlas re-uploaded,
 * frame still renders) rather than about a progress animation.
 */
test("asset reload re-uploads every atlas layer and leaves the game rendering", async () => {
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
    // Deliberately no canvas click: that engages pointer lock, which captures
    // the mouse and makes every overlay button unclickable until Escape.

    // The repair action is reachable from the paused Display panel.
    await page.keyboard.press("p");
    await page.getByRole("heading", { name: "Operation paused" }).waitFor({ timeout: 30000 });
    assert.ok(
      await page.evaluate(() =>
        [...document.querySelectorAll(".pause-menu button")].some((b) =>
          /Settings/.test(b.textContent ?? ""),
        ),
      ),
      "the pause screen must expose Settings",
    );
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    // The dialog is portalled and its content is lazy; wait for it to exist.
    await page.locator(".settings-dialog").waitFor({ timeout: 60000 });
    await page.getByRole("button", { name: "Display", exact: true }).click();
    assert.ok(
      await page.evaluate(() =>
        [...document.querySelectorAll(".settings-dialog button")].some((b) =>
          /Reload/.test(b.textContent ?? ""),
        ),
      ),
      "Settings > Display must offer a Reload all assets action",
    );
    await page.keyboard.press("Escape");
    await page.getByRole("button", { name: "Resume operation" }).click({ force: true });
    await page.waitForFunction(() => !document.body.innerText.includes("Operation paused"));

    /** Lit pixel count in the world canvas plus viewmodel coverage. */
    const frame = () =>
      page.evaluate(async () => {
        for (let i = 0; i < 12; i++) await new Promise((r) => requestAnimationFrame(r));
        const gc = document.querySelector(".game-canvas");
        const off = document.createElement("canvas");
        off.width = gc.width;
        off.height = gc.height;
        const octx = off.getContext("2d");
        octx.drawImage(gc, 0, 0);
        const d = octx.getImageData(0, 0, off.width, off.height).data;
        let lit = 0;
        for (let i = 0; i < d.length; i += 4) {
          if (d[i] + d[i + 1] + d[i + 2] > 24) lit++;
        }
        const wv = document.querySelector(".weapon-view canvas");
        let weapon = 0;
        if (wv) {
          const wd = wv.getContext("2d").getImageData(0, 0, wv.width, wv.height).data;
          for (let i = 3; i < wd.length; i += 4) if (wd[i] > 8) weapon++;
        }
        return { lit, weapon };
      });

    const before = await frame();
    assert.ok(before.lit > 1000, `expected a rendered world, got ${before.lit}`);

    const result = await page.evaluate(async () => {
      const rt = window.__blacksiteRuntimeTest;
      let total = 0;
      let calls = 0;
      await rt.reloadAllAssets((done, all) => {
        total = all;
        calls++;
      });
      return { total, calls };
    });

    // Every atlas layer and theme variant is walked, not just the visible ones.
    assert.ok(result.total > 800, `expected the whole atlas, got ${result.total} items`);
    assert.ok(
      result.calls >= result.total - 1,
      `progress should fire per item, got ${result.calls} for ${result.total}`,
    );

    const after = await frame();
    assert.ok(
      after.lit > 1000,
      `the world must still render after a reload, got ${after.lit} lit pixels`,
    );
    assert.ok(
      after.weapon > 1000,
      `the viewmodel must survive a reload, got ${after.weapon} opaque pixels`,
    );
    assert.deepEqual(errors, []);
  } finally {
    await browser.close();
  }
});
