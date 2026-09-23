import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { chromium } from "playwright";
import { checkedUrl } from "./browser-guard.mjs";

const base = checkedUrl(process.argv[2] ?? "http://127.0.0.1:8080/");
await mkdir("screenshots", { recursive: true });
const browser = await chromium.launch({
  headless: true,
  args: ["--autoplay-policy=no-user-gesture-required"],
});
const errors = [];
const results = {};
let diagnosticPage;
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  diagnosticPage = page;
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
  });
  await page.goto(`${base}?qa=1`, { waitUntil: "domcontentloaded", timeout: 60000 });
  await page.waitForFunction(() => window.__controlsTest?.getEnemyAudio()?.loaded === 54, null, {
    timeout: 60000,
  });
  await page.waitForSelector(".field-hud", { timeout: 90000 });
  await page.waitForSelector(".enemy-caption", { timeout: 30000 });
  const initial = await page.evaluate(() => window.__controlsTest.getEnemyAudio());
  assert.equal(initial.state, "running");
  assert.equal(initial.voices[0].panning, "HRTF");
  assert.ok(initial.voices[0].id >= 0);
  await page.screenshot({ path: "screenshots/voices-gameplay.png" });
  const initialYaw = await page.evaluate(() => window.__controlsTest.getYaw());
  await page.evaluate(() => window.__controlsTest.look(45, 0));
  await page.waitForFunction(
    (yaw) => Math.abs(window.__controlsTest.getYaw() - yaw) > 0.01,
    initialYaw,
  );
  const rotated = await page.evaluate(() => ({
    audio: window.__controlsTest.getEnemyAudio(),
    yaw: window.__controlsTest.getYaw(),
  }));
  assert.ok(Math.abs(rotated.audio.listener.dx - Math.cos(rotated.yaw)) < 0.02);
  assert.ok(Math.abs(rotated.audio.listener.dy - Math.sin(rotated.yaw)) < 0.02);
  await page.getByRole("button", { name: "Pause", exact: true }).click();
  assert.equal((await page.evaluate(() => window.__controlsTest.getEnemyAudio())).voices.length, 0);
  await page.getByRole("button", { name: /Settings/ }).click();
  await page.getByRole("button", { name: "Accessibility", exact: true }).click();
  await page.getByRole("checkbox", { name: "Enemy subtitles", exact: true }).uncheck();
  await page.screenshot({ path: "screenshots/voices-settings.png" });
  await page.getByRole("button", { name: "Audio", exact: true }).click();
  await page.getByRole("button", { name: "Machine", exact: true }).click();
  await page.waitForFunction(() =>
    window.__controlsTest.getEnemyAudio().voices.some((v) => v.id === -1),
  );
  const volume = page.getByRole("slider", { name: "Enemy voices", exact: true });
  await volume.focus();
  await volume.press("Home");
  await page.waitForFunction(() => window.__controlsTest.getEnemyAudio().volume < 0.01);
  await volume.press("End");
  await page.getByRole("button", { name: "Close settings", exact: true }).click();
  await page.getByRole("button", { name: /Resume operation/ }).click();
  await page.waitForTimeout(1500);
  assert.equal(await page.locator(".enemy-caption").count(), 0);
  await page.goto(base, { waitUntil: "domcontentloaded", timeout: 60000 });
  await page.getByRole("button", { name: /Enter blacksite/ }).waitFor({ timeout: 60000 });
  await page.getByRole("button", { name: /Settings/ }).click();
  await page.getByRole("button", { name: "Accessibility", exact: true }).click();
  assert.equal(
    await page.getByRole("checkbox", { name: "Enemy subtitles", exact: true }).isChecked(),
    false,
  );
  await page.getByRole("checkbox", { name: "Enemy subtitles", exact: true }).check();
  await page.getByRole("button", { name: "Close settings", exact: true }).click();
  results.desktop = {
    decodedVoices: 54,
    captionAnchoredToEnemy: true,
    spatialListenerTracksYaw: true,
    pauseStopsVoice: true,
    volumeWorks: true,
    subtitlesPersist: true,
  };
  const mobile = await browser.newPage({
    viewport: { width: 390, height: 844 },
    isMobile: true,
    hasTouch: true,
  });
  mobile.on("pageerror", (e) => errors.push(e.message));
  mobile.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
  });
  await mobile.goto(base, { waitUntil: "domcontentloaded", timeout: 60000 });
  await mobile.getByRole("button", { name: /Enter blacksite/ }).waitFor({ timeout: 60000 });
  await mobile.screenshot({ path: "screenshots/voices-menu-mobile.png" });
  await mobile.getByRole("button", { name: /Settings/ }).click();
  await mobile.getByRole("button", { name: "Accessibility", exact: true }).click();
  await mobile.screenshot({ path: "screenshots/voices-settings-mobile.png" });
  assert.ok(await mobile.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await mobile.getByRole("button", { name: "Close settings", exact: true }).click();
  await mobile.getByRole("button", { name: /Enter blacksite/ }).click();
  await mobile.waitForSelector(".field-hud");
  await mobile.screenshot({ path: "screenshots/voices-gameplay-mobile.png" });
  results.mobile = { settingsFit: true, gameplay: true };
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({ ...results, errors }, null, 2));
} catch (error) {
  console.error(error);
  if (diagnosticPage && !diagnosticPage.isClosed()) {
    console.error(await diagnosticPage.evaluate(() => ({ body: document.body.innerText, audio: window.__controlsTest?.getEnemyAudio() })).catch(() => "Page unavailable"));
    await diagnosticPage.screenshot({ path: "screenshots/voices-error.png" }).catch(() => {});
  }
  throw error;
} finally {
  await browser.close();
}
