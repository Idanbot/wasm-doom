import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
const base = process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/";

test("rebound keyboard and mouse actions reach the input ABI without stale held actions", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    await page.route("**/__controls.html", (r) =>
      r.fulfill({ contentType: "text/html", body: "<!doctype html><body></body>" }),
    );
    await page.goto(new URL("__controls.html", base).href);
    const value = await page.evaluate(async () => {
      const { BlacksiteRuntime } = await import("/src/game/runtime.ts");
      const { DEFAULT_BINDINGS } = await import("/src/game/controls.ts");
      const rt = new BlacksiteRuntime(document.createElement("canvas"), {
        onHud() {},
        onState() {},
      });
      const selected = [],
        cycles = [];
      rt.wasm = { hs_select_weapon: (n) => selected.push(n) };
      rt.audio = { unlock() {} };
      rt.playing = true;
      rt.cycleWeapon = (n) => cycles.push(n);
      rt.requestLock = () => {};
      rt.setBindings({
        ...DEFAULT_BINDINGS,
        forward: ["KeyI"],
        fire: ["Mouse2"],
        nextWeapon: ["KeyX"],
        weapon33: ["KeyN"],
      });
      const press = (code, repeat = false) =>
        rt.onKeyDown(new KeyboardEvent("keydown", { code, repeat, cancelable: true }));
      press("KeyW");
      const old = rt.bitsFromKeys();
      press("KeyI");
      const move = rt.bitsFromKeys();
      rt.onKeyUp(new KeyboardEvent("keyup", { code: "KeyI" }));
      rt.onMouseDown(new MouseEvent("mousedown", { button: 2 }));
      const fire = rt.bitsFromKeys();
      rt.onMouseUp(new MouseEvent("mouseup", { button: 2 }));
      const released = rt.bitsFromKeys();
      press("KeyN");
      press("KeyN", true);
      press("KeyX");
      press("KeyX", true);
      press("KeyI");
      rt.setBindings(DEFAULT_BINDINGS);
      const reset = rt.bitsFromKeys();
      return { old, move, fire, released, selected, cycles, reset };
    });
    assert.deepEqual(value, {
      old: 0,
      move: 1,
      fire: 16,
      released: 0,
      selected: [32],
      cycles: [1],
      reset: 0,
    });
  } finally {
    await browser.close();
  }
});

test(
  "saved key bindings and comfort sliders work in real gameplay and restore defaults",
  { timeout: 300000 },
  async () => {
    const browser = await chromium.launch({ headless: true });
    try {
      const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
      const errors = [];
      page.on("pageerror", (e) => errors.push(e.message));
      await page.addInitScript(() => {
        localStorage.setItem("blacksite-res", "320");
      });
      const url = new URL(base);
      url.searchParams.set("qa", "1");
      await page.goto(url.href);
      const ready = () =>
        page.waitForFunction(() => window.__controlsTest?.isLive(), null, { timeout: 180000 });
      await ready();
      await page.keyboard.press("p");
      await page.getByRole("button", { name: "Settings", exact: true }).click();
      await page.getByRole("button", { name: "Controls", exact: true }).click();
      await page.getByRole("button", { name: "Bind Fire 1", exact: true }).click();
      await page.getByRole("button", { name: "Bind Fire 1", exact: true }).click();
      assert.match(
        await page.locator(".binding-status").innerText(),
        /Reload:/,
        "left mouse capture must finish without reopening",
      );
      const bind = async (name, key) => {
        await page.getByRole("button", { name, exact: true }).click();
        await page.keyboard.press(key);
      };
      await bind("Bind Reload / vent 1", "f");
      await bind("Bind Interact 1", "f");
      await page.getByRole("alert").filter({ hasText: "already assigned" }).waitFor();
      await page.keyboard.press("Escape");
      await bind("Bind Interact 1", "g");
      await bind("Bind Move forward 1", "i");
      await bind("Bind Pause / resume 1", "o");
      await bind("Add Next weapon binding", "x");
      await bind("Add Previous weapon binding", "z");
      await page.getByRole("button", { name: "Bind Fire 1", exact: true }).click();
      await page
        .getByRole("button", { name: "Bind Fire 1", exact: true })
        .click({ button: "right" });
      await page.getByText("Direct weapon bindings · 33 weapons", { exact: true }).click();
      await bind("Add Weapon 33 binding", "n");
      await page.getByText("Direct weapon bindings · 33 weapons", { exact: true }).click();
      await page.locator(".settings-body").evaluate((e) => {
        e.scrollTop = 0;
      });
      await page.screenshot({ path: "screenshots/key-bindings-desktop.png" });
      await page.setViewportSize({ width: 390, height: 844 });
      await page.screenshot({ path: "screenshots/key-bindings-mobile.png" });
      await page.setViewportSize({ width: 1280, height: 800 });
      await page.getByRole("button", { name: "Accessibility", exact: true }).click();
      for (const label of ["Flash intensity", "Shake intensity", "Debris amount"]) {
        await page.getByRole("slider", { name: label, exact: true }).focus();
        await page.keyboard.press("Home");
        assert.equal(
          await page.getByRole("slider", { name: label, exact: true }).inputValue(),
          "0",
        );
      }
      await page.screenshot({ path: "screenshots/accessibility-settings-desktop.png" });
      await page.setViewportSize({ width: 390, height: 844 });
      await page.screenshot({ path: "screenshots/accessibility-settings-mobile.png" });
      assert.ok(
        await page
          .locator("[role=dialog]")
          .evaluate((e) => e.getBoundingClientRect().width <= window.innerWidth),
      );
      await page.setViewportSize({ width: 1280, height: 800 });
      await page.getByRole("button", { name: "Done", exact: true }).click();
      await page.keyboard.press("o");
      await page.waitForFunction(() => !document.body.innerText.includes("Operation paused"));
      const preferences = await page.evaluate(() => window.__controlsTest.getPreferences());
      assert.deepEqual(
        [preferences.gfx.flash, preferences.gfx.shake, preferences.gfx.debris],
        [0, 0, 0],
      );
      assert.deepEqual(preferences.bindings.reload, ["KeyF"]);
      assert.deepEqual(preferences.bindings.interact, ["KeyG"]);
      assert.equal(preferences.bindings.fire[0], "Mouse2");
      const pos = () =>
        page.evaluate(() => [window.__controlsTest.getX(), window.__controlsTest.getY()]);
      const start = await pos();
      await page.keyboard.down("w");
      await page.waitForTimeout(220);
      await page.keyboard.up("w");
      assert.deepEqual(await pos(), start, "old movement key must be inactive");
      await page.keyboard.down("i");
      await page.waitForTimeout(220);
      await page.keyboard.up("i");
      assert.notDeepEqual(await pos(), start, "new movement key must move the real player");
      await page.keyboard.press("n");
      await page.waitForFunction(() => window.__controlsTest.getWeapon() === 32);
      await page.keyboard.press("x");
      await page.waitForFunction(() => window.__controlsTest.getWeapon() === 0);
      await page.keyboard.press("z");
      await page.waitForFunction(() => window.__controlsTest.getWeapon() === 32);
      await page.keyboard.press("1");
      await page.waitForFunction(() => window.__controlsTest.getWeapon() === 0);
      await page.waitForTimeout(700);
      const ammo = await page.evaluate(() => window.__controlsTest.getAmmo());
      await page.locator(".game-canvas").hover();
      await page.mouse.down({ button: "right" });
      await page.waitForFunction((n) => window.__controlsTest.getAmmo() < n, ammo);
      assert.deepEqual(await page.evaluate(() => window.__controlsTest.getFeedback()), {
        muzzle: 0,
        hurt: 0,
        shake: 0,
      });
      assert.ok(
        await page
          .locator(".weapon-view")
          .evaluate((e) => e.style.transform.includes("rotate(0deg)")),
      );
      await page.mouse.up({ button: "right" });
      await page.keyboard.press("r");
      assert.equal(await page.evaluate(() => window.__controlsTest.getReloading()), 0);
      await page.keyboard.press("f");
      await page.waitForFunction(() => window.__controlsTest.getReloading() > 0);
      await page.keyboard.press("p");
      assert.equal(await page.getByText("Operation paused", { exact: true }).count(), 0);
      await page.keyboard.press("o");
      await page.getByText("Operation paused", { exact: true }).waitFor();
      await page.reload();
      await ready();
      const saved = await page.evaluate(() => window.__controlsTest.getPreferences());
      assert.deepEqual(saved.bindings, preferences.bindings);
      assert.deepEqual(saved.gfx, preferences.gfx);
      await page.keyboard.press("Escape");
      await page.getByText("Operation paused", { exact: true }).waitFor();
      await page.getByRole("button", { name: "Settings", exact: true }).click();
      await page.getByRole("button", { name: "Restore defaults", exact: true }).click();
      await page.waitForFunction(() => window.__controlsTest.getPreferences().gfx.flash === 1);
      assert.equal(
        await page.evaluate(() => window.__controlsTest.getPreferences().bindings.fire[0]),
        "Mouse0",
      );
      assert.equal(
        await page.evaluate(() => window.__controlsTest.getPreferences().bindings.reload[0]),
        "KeyR",
      );
      assert.deepEqual(errors, []);
    } finally {
      await browser.close();
    }
  },
);

test(
  "zero cosmetic effects render in Canvas fallback through a sector change",
  { timeout: 180000 },
  async () => {
    const browser = await chromium.launch({ headless: true });
    try {
      const page = await browser.newPage({ viewport: { width: 960, height: 600 } });
      const errors = [];
      page.on("pageerror", (e) => errors.push(e.message));
      await page.addInitScript(() => {
        localStorage.setItem("blacksite-res", "320");
        localStorage.setItem(
          "blacksite-gfx",
          JSON.stringify({ crt: false, bloom: false, fog: false, flash: 0, shake: 0, debris: 0 }),
        );
        Object.defineProperty(navigator, "gpu", { value: undefined });
        const original = HTMLCanvasElement.prototype.getContext;
        HTMLCanvasElement.prototype.getContext = function (type, ...args) {
          return type === "webgl2" || type === "webgpu" ? null : original.call(this, type, ...args);
        };
      });
      const url = new URL(base);
      url.searchParams.set("qa", "1");
      await page.goto(url.href);
      await page.waitForFunction(() => window.__controlsTest?.isLive(), null, { timeout: 120000 });
      const ammo = await page.evaluate(() => window.__controlsTest.getAmmo());
      await page.evaluate(() => window.__controlsTest.setKeys(["Space"]));
      await page.waitForFunction((n) => window.__controlsTest.getAmmo() < n, ammo);
      assert.deepEqual(await page.evaluate(() => window.__controlsTest.getFeedback()), {
        muzzle: 0,
        hurt: 0,
        shake: 0,
      });
      await page.evaluate(() => {
        window.__controlsTest.setKeys([]);
        window.__controlsTest.nextWave();
      });
      await page.waitForFunction(() => window.__controlsTest.getWorldEffects().wave === 2);
      assert.deepEqual(await page.evaluate(() => window.__controlsTest.getFeedback()), {
        muzzle: 0,
        hurt: 0,
        shake: 0,
      });
      await page.screenshot({ path: "screenshots/accessibility-canvas.png" });
      assert.deepEqual(errors, []);
    } finally {
      await browser.close();
    }
  },
);
