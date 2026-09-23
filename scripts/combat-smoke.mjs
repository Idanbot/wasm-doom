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
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  page.on("pageerror", error => errors.push(error.message));
  page.on("console", message => { if (message.type() === "error") errors.push(message.text()); });
  for (const [slot, name, magazine] of [[0, "MK23-S", 12], [1, "M870K", 6], [2, "VX-9", 32], [3, "SHRIKE", 4], [4, "RAVEN", 6]]) {
    await page.goto(url.href);
    await page.waitForFunction(() => !!window.__controlsTest && document.body.innerText.includes("HEALTH"));
    await page.evaluate(slot => {
      const t = window.__controlsTest;
      t.grantWeapons();
      t.setKeys([`Digit${slot + 1}`]);
    }, slot);
    await page.waitForFunction(slot => window.__controlsTest.getWeapon() === slot, slot);
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    assert.equal(await page.evaluate(() => window.__controlsTest.getAmmo()), magazine);
    await page.waitForFunction(name => document.body.innerText.includes(name), name);
    await page.evaluate(() => window.__controlsTest.setKeys(["Space"]));
    await page.waitForFunction(mag => window.__controlsTest.getAmmo() < mag, magazine);
    // Capture the live firing sheet before releasing the trigger.
    await page.screenshot({ path: `screenshots/combat-weapon-${slot}.png` });
    await page.evaluate(() => window.__controlsTest.setKeys(["KeyR"]));
    await page.waitForFunction(() => window.__controlsTest.getReloading() > 0);
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    await page.waitForFunction(mag => window.__controlsTest.getReloading() === 0 && window.__controlsTest.getAmmo() === mag, magazine);
    results.push({ name, fire: true, reload: true });
  }
  // Camera basis in this raycaster: right=(-sin(yaw), cos(yaw)).
  await page.goto(url.href);
  await page.waitForFunction(() => !!window.__controlsTest && document.body.innerText.includes("HEALTH"));
  const controls = await page.evaluate(async () => {
    const t = window.__controlsTest;
    const move = async key => {
      const x = t.getX(), y = t.getY(), yaw = t.getYaw();
      t.setKeys([key]);
      await new Promise(resolve => setTimeout(resolve, 180));
      t.setKeys([]);
      return (t.getX() - x) * -Math.sin(yaw) + (t.getY() - y) * Math.cos(yaw);
    };
    const left = await move("KeyA"), right = await move("KeyD");
    return { left, right };
  });
  assert.ok(controls.left < -0.03 && controls.right > 0.03, JSON.stringify(controls));
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({ weapons: results, controls, errors }, null, 2));
} finally {
  await browser.close();
}
