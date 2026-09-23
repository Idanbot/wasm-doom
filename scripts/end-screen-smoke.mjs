#!/usr/bin/env node
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { checkedUrl } from "./browser-guard.mjs";

const base = checkedUrl(process.argv[2] ?? "http://127.0.0.1:8080/");
const url = new URL(base);
url.searchParams.set("qa", "1");
const browser = await chromium.launch({ headless: true });
const errors = [];
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await page.goto(url.href, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(
    () => !!window.__controlsTest && document.body.innerText.includes("HEALTH"),
    null,
    { timeout: 90000 },
  );
  await page.evaluate(() => window.__controlsTest.triggerEnd(1));
  await page.getByText("FLATLINED", { exact: true }).waitFor();
  await page.screenshot({ path: "screenshots/refine-game-over.png" });
  await page.getByRole("button", { name: /Start again/ }).click();
  await page.waitForFunction(() => document.body.innerText.includes("HEALTH"));
  await page.evaluate(() => window.__controlsTest.triggerEnd(2));
  await page.getByText("SITE CLEARED", { exact: true }).waitFor();
  await page.screenshot({ path: "screenshots/refine-level-won.png" });

  const mobile = await browser.newPage({
    viewport: { width: 390, height: 844 },
    isMobile: true,
    hasTouch: true,
  });
  mobile.on("pageerror", (error) => errors.push(error.message));
  await mobile.goto(url.href, { waitUntil: "domcontentloaded" });
  await mobile.waitForFunction(
    () => !!window.__controlsTest && document.body.innerText.includes("HEALTH"),
    null,
    { timeout: 90000 },
  );
  await mobile.evaluate(() => window.__controlsTest.triggerEnd(2));
  await mobile.getByText("SITE CLEARED", { exact: true }).waitFor();
  assert.ok(await mobile.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await mobile.screenshot({ path: "screenshots/refine-level-won-mobile.png", fullPage: true });
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({ dead: true, win: true, mobile: true, errors }, null, 2));
} finally {
  await browser.close();
}
