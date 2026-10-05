import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
const base = process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/";
async function fixture(browser, mobile = false) {
  const context = await browser.newContext(
    mobile ? { viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true } : {},
  );
  const page = await context.newPage();
  await page.route("**/__sfx-test.html", (route) =>
    route.fulfill({ contentType: "text/html", body: '<button id="unlock">Unlock audio</button>' }),
  );
  await page.goto(new URL("/__sfx-test.html", base).href);
  await page.evaluate(async () => {
    const [{ SfxPlayer }, { VoiceGate }] = await Promise.all([
      import("/src/game/sfx-player.ts"),
      import("/src/game/voice-gate.ts"),
    ]);
    const ctx = new AudioContext({ latencyHint: "interactive" }),
      bus = ctx.createGain(),
      master = ctx.createGain(),
      analyser = ctx.createAnalyser();
    analyser.fftSize = 2048;
    bus.connect(master);
    master.connect(analyser);
    analyser.connect(ctx.destination);
    const gate = new VoiceGate(),
      player = new SfxPlayer(ctx, bus, gate);
    Object.assign(window, { soundFixture: { ctx, bus, master, analyser, gate, player } });
    document.querySelector("#unlock").onclick = () => ctx.resume();
  });
  if (mobile) await page.locator("#unlock").tap();
  else await page.locator("#unlock").click();
  await page.waitForFunction(() => window.soundFixture.ctx.state === "running");
  return { context, page };
}
for (const mobile of [false, true])
  test(`${mobile ? "mobile" : "desktop"} recorded SFX decode, produce audio, respect volume/mute, spatial loops and voice exclusivity`, async () => {
    const browser = await chromium.launch({ headless: true });
    try {
      const { page } = await fixture(browser, mobile);
      const results = await page.evaluate(async () => {
        const f = window.soundFixture;
        await f.player.load();
        // Verify both compatibility formats contain non-silent mono signals, not merely valid URLs.
        const manifest = await (await fetch("/game/sfx/v2/manifest.json")).json();
        let decoded = 0,
          peak = 0,
          minEnergy = 1;
        for (const c of manifest.clips)
          for (const url of [c.url, c.fallback]) {
            const b = await f.ctx.decodeAudioData(await (await fetch(url)).arrayBuffer());
            if (b.numberOfChannels !== 1 || b.duration <= 0 || b.duration > 1.9)
              throw Error("Invalid sound: " + url);
            const data = b.getChannelData(0);
            let energy = 0,
              p = 0;
            for (const x of data) {
              energy += x * x;
              p = Math.max(p, Math.abs(x));
            }
            peak = Math.max(peak, p);
            minEnergy = Math.min(minEnergy, energy / data.length);
            decoded++;
          }
        f.player.updateLoops([{ id: 1, kind: 14, x: 0, y: 0 }]);
        async function measure() {
          await new Promise((r) => setTimeout(r, 100));
          let peak = 0;
          const data = new Float32Array(2048);
          const until = f.ctx.currentTime + 0.85;
          while (f.ctx.currentTime < until) {
            f.analyser.getFloatTimeDomainData(data);
            for (const x of data) peak = Math.max(peak, Math.abs(x));
            await new Promise(requestAnimationFrame);
          }
          return peak;
        }
        const loud = await measure();
        f.bus.gain.value = 0.25;
        const quiet = await measure();
        f.master.gain.value = 0;
        const zero = await measure();
        f.master.gain.value = 1;
        f.bus.gain.value = 1;
        f.player.updateLoops([]);
        const stopped = f.player.diagnostics().active;
        f.player.play({ id: "pain-human0" });
        const occupied = f.gate.busy;
        const secondVocal = f.player.play({ id: "pain-creature0" });
        let dialogue = false;
        f.gate.enqueue((done) => {
          dialogue = true;
          window.releaseDialogue = done;
        });
        f.player.stop(); // Releases the vocal lane, so pending dialogue can start.
        const vocalDuringSpeech = f.player.play({ id: "death-human0" });
        window.releaseDialogue();
        f.player.updateLoops(
          Array.from({ length: 30 }, (_, id) => ({ id, kind: 14, x: id * 0.1, y: 0 })),
        );
        const bounded = f.player.diagnostics().active;
        f.player.setMuted(true);
        const mutedPlay = f.player.play({ id: "fire0" });
        const mutedActive = f.player.diagnostics().active;
        f.player.setMuted(false);
        f.player.stop();
        f.player.close();
        const afterClose = f.player.play({ id: "fire0" });
        await f.ctx.close();
        return {
          decoded,
          peak,
          minEnergy,
          loud,
          quiet,
          zero,
          stopped,
          occupied,
          secondVocal,
          dialogue,
          vocalDuringSpeech,
          bounded,
          mutedPlay,
          mutedActive,
          afterClose,
        };
      });
      console.log(mobile ? "mobile" : "desktop", results);
      assert.equal(results.decoded, 260);
      assert.ok(results.peak < 0.8, JSON.stringify(results));
      assert.ok(results.minEnergy > 1e-8);
      assert.ok(results.loud > 0.0001, JSON.stringify(results));
      assert.ok(results.quiet < results.loud * 0.6, JSON.stringify(results));
      assert.equal(results.zero, 0);
      assert.equal(results.stopped, 0);
      assert.equal(results.occupied, true);
      assert.equal(results.secondVocal, false);
      assert.equal(results.dialogue, true);
      assert.equal(results.vocalDuringSpeech, false);
      assert.ok(results.bounded <= 6);
      assert.equal(results.mutedPlay, false);
      assert.equal(results.mutedActive, 0);
      assert.equal(results.afterClose, false);
    } finally {
      await browser.close();
    }
  });
test("unsupported or missing OGG uses MP3 once; missing both fails loading clearly without retries", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const { page } = await fixture(browser);
    let attempts = 0;
    await page.route("**/game/sfx/v2/fire3.ogg", (route) => {
      attempts++;
      return route.fulfill({ status: 404, body: "missing" });
    });
    assert.deepEqual(
      await page.evaluate(async () => {
        await window.soundFixture.player.load();
        await window.soundFixture.player.load();
        return window.soundFixture.player.diagnostics().fallback;
      }),
      ["fire3"],
    );
    assert.equal(attempts, 1);
    await page.close();
    const bad = await fixture(browser);
    let failures = 0;
    for (const ext of ["ogg", "mp3"])
      await bad.page.route(`**/game/sfx/v2/fire3.${ext}`, (route) => {
        failures++;
        return route.fulfill({ status: 404, body: "missing" });
      });
    const result = await bad.page.evaluate(async () => {
      let message = "";
      for (let n = 0; n < 2; n++)
        try {
          await window.soundFixture.player.load();
        } catch (e) {
          message = e.message;
        }
      return {
        message,
        failed: window.soundFixture.player.diagnostics().failed,
        play: window.soundFixture.player.play({ id: "fire3" }),
      };
    });
    assert.match(result.message, /Sound effects unavailable: fire3/);
    assert.deepEqual(result.failed, ["fire3"]);
    assert.equal(result.play, false);
    assert.equal(failures, 2);
  } finally {
    await browser.close();
  }
});
test("game loading waits for SFX and real firing/reload/pause events use recorded audio", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } }),
      errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    const url = new URL(base);
    url.searchParams.set("qa", "1");
    await page.goto(url.href, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(
      () =>
        window.__controlsTest?.getSfxAudio()?.loaded === 130 &&
        window.__controlsTest?.getReserve() === 72 &&
        document.body.innerText.includes("HEALTH"),
      null,
      { timeout: 90000 },
    );
    await page.locator(".game-canvas").click({ force: true });
    await page.evaluate(() => {
      window.__controlsTest.heal();
      window.__controlsTest.setKeys(["Digit1", "Space"]);
    });
    await page.waitForFunction(() => window.__controlsTest.getSfxAudio().recent.includes("fire0"));
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    await page.evaluate(() => window.__controlsTest.setKeys(["KeyR"]));
    await page.waitForFunction(() =>
      window.__controlsTest.getSfxAudio().recent.includes("mag-out"),
    );
    await page.evaluate(() => window.__controlsTest.setKeys([]));
    await page.waitForFunction(() =>
      window.__controlsTest.getSfxAudio().recent.includes("chamber"),
    );
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => document.body.innerText.includes("Operation paused"));
    await page.waitForFunction(() => !document.pointerLockElement);
    await page.waitForFunction(() => window.__controlsTest.getSfxAudio().active === 0);
    await page.getByRole("button", { name: "Sound on", exact: true }).click();
    await page.waitForFunction(() => window.__controlsTest.getSfxAudio().levels.master < 0.001);
    await page.getByRole("button", { name: "Sound off", exact: true }).click();
    await page.waitForFunction(() => window.__controlsTest.getSfxAudio().levels.master > 0.1);
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.getByRole("button", { name: "Audio", exact: true }).click();
    await page.getByLabel("Effects", { exact: true }).press("Home");
    await page.waitForFunction(() => window.__controlsTest.getSfxAudio().levels.sfx < 0.001);
    await page.getByLabel("Effects", { exact: true }).press("End");
    await page.waitForFunction(
      () => Math.abs(window.__controlsTest.getSfxAudio().levels.sfx - 1) < 0.001,
    );
    assert.deepEqual(errors, []);
    await page.screenshot({ path: "screenshots/sfx-pause.png" });
  } finally {
    await browser.close();
  }
});
