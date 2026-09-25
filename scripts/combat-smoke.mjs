#!/usr/bin/env node
// Interactive fallback for environments without agent-browser.
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";
import { checkedUrl } from "./browser-guard.mjs";

const base = checkedUrl(process.argv[2] ?? "http://127.0.0.1:8080/");
const url = new URL(base);
url.searchParams.set("qa", "1");
await mkdir("screenshots", { recursive: true });
const browser = await chromium.launch({ headless: true });
const errors = [];
const results = [];
const assertHudTextContained = async (page, label) => {
  const escaped = await page.evaluate(() =>
    [...document.querySelectorAll(".hud-plate")].flatMap((plate) => {
      const outer = plate.getBoundingClientRect();
      return [...plate.querySelectorAll("span, strong, small, b, p")]
        .filter((node) => node.textContent?.trim() && node.getClientRects().length)
        .flatMap((node) => {
          const inner = node.getBoundingClientRect();
          const outside =
            inner.left < outer.left + 12 ||
            inner.right > outer.right - 12 ||
            inner.top < outer.top + 12 ||
            inner.bottom > outer.bottom - 16;
          return outside
            ? [{ text: node.textContent.trim(), outer: outer.toJSON(), inner: inner.toJSON() }]
            : [];
        });
    }),
  );
  assert.deepEqual(escaped, [], `${label} HUD text escaped its panel`);
};
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  for (const [slot, name, magazine] of [
    [0, "MK23-S", 12],
    [1, "BR-12 BREAKER", 8],
    [2, "KX-9 VECTOR", 36],
    [3, "MR-4 LONGBOW", 5],
    [4, "VLK-6 WARDEN", 4],
    [5, "AX-12 VOLT", 10],
    [6, "M91 CYCLONE", 90],
  ]) {
    await page.goto(url.href);
    await page.waitForFunction(
      () => !!window.__controlsTest && document.body.innerText.includes("HEALTH"),
    );
    if (slot === 0) {
      const locked = await page.locator(".weapon-slot.locked").allTextContents();
      assert.ok(locked.some((text) => text.includes("6") && text.includes("AX-12 VOLT")));
      assert.ok(locked.some((text) => text.includes("7") && text.includes("M91 CYCLONE")));
      await page.evaluate(() => window.__controlsTest.setKeys(["Digit6"]));
      await page.waitForTimeout(80);
      assert.equal(await page.evaluate(() => window.__controlsTest.getWeapon()), 0);
      await page.evaluate(() => window.__controlsTest.setKeys([]));
    }
    await page.evaluate((slot) => {
      const t = window.__controlsTest;
      t.grantWeapons();
      t.setKeys([`Digit${slot + 1}`]);
    }, slot);
    await page.waitForFunction((slot) => window.__controlsTest.getWeapon() === slot, slot);
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    assert.equal(await page.evaluate(() => window.__controlsTest.getAmmo()), magazine);
    await page.waitForFunction((name) => document.body.innerText.includes(name), name);
    await page.evaluate(() => window.__controlsTest.setKeys(["Space"]));
    await page.waitForFunction(() => {
      const weapon = document.querySelector(".weapon-view");
      return (
        weapon?.style.backgroundImage.includes("_fire.png") &&
        ["100% 0%", "0% 100%"].includes(weapon.style.backgroundPosition)
      );
    });
    // Capture an actual recoil/flash frame rather than the settled tail of
    // the shot, which can already be back on the idle sheet after ammo drops.
    await page.screenshot({ path: `screenshots/combat-weapon-${slot}.png` });
    await page.waitForFunction((mag) => window.__controlsTest.getAmmo() < mag, magazine);
    await page.evaluate(() => window.__controlsTest.setKeys(["KeyR"]));
    await page.waitForFunction(() => window.__controlsTest.getReloading() > 0);
    await page.waitForFunction(
      (slot) => {
        const weapon = document.querySelector(".weapon-view");
        if (!weapon?.style.backgroundImage.includes("_reload.png")) return false;
        return slot === 0
          ? weapon.style.backgroundSize === "200% 200%"
          : weapon.style.backgroundSize === "400% 200%";
      },
      slot,
    );
    await page.waitForTimeout(220);
    await page.screenshot({ path: `screenshots/combat-weapon-${slot}-reload.png` });
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    await page.waitForFunction(
      (mag) =>
        window.__controlsTest.getReloading() === 0 && window.__controlsTest.getAmmo() === mag,
      magazine,
    );
    results.push({ name, fire: true, reload: true });
  }
  // Camera basis in this raycaster: right=(-sin(yaw), cos(yaw)).
  await page.goto(url.href);
  await page.waitForFunction(
    () => !!window.__controlsTest && document.body.innerText.includes("HEALTH"),
  );
  const controls = await page.evaluate(async () => {
    const t = window.__controlsTest;
    const move = async (key) => {
      const x = t.getX(),
        y = t.getY(),
        yaw = t.getYaw();
      t.setKeys([key]);
      await new Promise((resolve) => setTimeout(resolve, 180));
      t.setKeys([]);
      return (t.getX() - x) * -Math.sin(yaw) + (t.getY() - y) * Math.cos(yaw);
    };
    const left = await move("KeyA"),
      right = await move("KeyD");
    return { left, right };
  });
  assert.ok(controls.left < -0.03 && controls.right > 0.03, JSON.stringify(controls));

  await page.evaluate(() => {
    const t = window.__controlsTest;
    t.setKeys(["KeyW"]);
    t.grantWeapons();
    t.triggerBoss(0);
    t.setKeys([]);
  });
  await page.waitForFunction(
    () =>
      document.body.innerText.includes("MALIK VEYRAN") &&
      document.body.innerText.includes("PHASE 1 / 3"),
  );
  const boss = await page.evaluate(() => ({
    label: document.querySelector(".hud-boss")?.textContent ?? "",
    width: document.querySelector(".hud-boss-track i")?.getBoundingClientRect().width ?? 0,
  }));
  assert.match(boss.label, /480\s*\/\s*480/);
  assert.ok(boss.width > 200, "the full boss health track should be visible");
  await page.screenshot({ path: "screenshots/combat-boss-hud.png" });

  await page.evaluate(() => window.__controlsTest.nextWave());
  await page.waitForFunction(() => document.body.innerText.includes("CRYOGENIC FOUNDRY"));
  await page.evaluate(() => {
    window.__controlsTest.setKeys(["KeyW"]);
    window.__controlsTest.triggerBoss(0);
    window.__controlsTest.setKeys([]);
  });
  await page.waitForFunction(() => document.body.innerText.includes("HECATE–9"));
  await page.screenshot({ path: "screenshots/sector-foundry.png" });
  await page.evaluate(() => window.__controlsTest.nextWave());
  await page.waitForFunction(() => document.body.innerText.includes("BIOFORGE DEPTHS"));
  await page.evaluate(() => {
    window.__controlsTest.setKeys(["KeyW"]);
    window.__controlsTest.triggerBoss(0);
    window.__controlsTest.setKeys([]);
  });
  await page.waitForFunction(() => document.body.innerText.includes("CHIMERA–9"));
  await page.screenshot({ path: "screenshots/sector-bioforge.png" });

  await page.goto(url.href);
  await page.waitForFunction(
    () => !!window.__controlsTest && document.body.innerText.includes("HEALTH"),
  );
  await page.evaluate(() => {
    const t = window.__controlsTest;
    t.grantWeapons();
    t.setKeys(["Digit1"]);
  });
  await page.waitForFunction(() => window.__controlsTest.getWeapon() === 0);
  await page.evaluate(() => window.__controlsTest.setKeys(["Space"]));
  await page.waitForFunction(() => window.__controlsTest.getAmmo() <= 3);
  assert.match(await page.locator(".hud-ammo-heading").innerText(), /LOW AMMO/);
  await page.evaluate(() => window.__controlsTest.setKeys([]));
  await page.screenshot({ path: "screenshots/combat-low-ammo.png" });
  await assertHudTextContained(page, "desktop");

  const mobile = await browser.newPage({
    viewport: { width: 390, height: 844 },
    isMobile: true,
    hasTouch: true,
  });
  mobile.on("pageerror", (error) => errors.push(error.message));
  mobile.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await mobile.goto(url.href);
  await mobile.waitForFunction(
    () => !!window.__controlsTest && document.body.innerText.includes("HEALTH"),
  );
  await mobile.evaluate(() => {
    const t = window.__controlsTest;
    t.setKeys(["KeyW"]);
    t.triggerBoss(0);
    t.setKeys([]);
  });
  await mobile.waitForFunction(() => document.body.innerText.includes("PHASE 1 / 3"));
  assert.ok(await mobile.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await assertHudTextContained(mobile, "mobile");
  await mobile.screenshot({ path: "screenshots/combat-boss-hud-mobile.png" });

  const narrow = await browser.newPage({ viewport: { width: 320, height: 700 } });
  narrow.on("pageerror", (error) => errors.push(error.message));
  narrow.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await narrow.goto(url.href);
  await narrow.waitForFunction(
    () => !!window.__controlsTest && document.body.innerText.includes("HEALTH"),
  );
  assert.ok(await narrow.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await assertHudTextContained(narrow, "320px");
  await narrow.screenshot({ path: "screenshots/combat-hud-320.png" });
  assert.deepEqual(errors, []);
  console.log(
    JSON.stringify(
      { weapons: results, controls, boss: true, mobileBoss: true, lowAmmo: true, errors },
      null,
      2,
    ),
  );
} finally {
  await browser.close();
}
