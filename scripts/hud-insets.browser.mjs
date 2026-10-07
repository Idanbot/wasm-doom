import { test } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";

test("all weapon and vitals text stays inside the artwork inset at desktop and mobile sizes", async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on("pageerror", e => errors.push(e.message));
    const url = new URL("/__hud_insets.html", process.env.BLACKSITE_TEST_URL ?? "http://127.0.0.1:8080/");
    await page.route("**/__hud_insets.html", route => route.fulfill({ contentType: "text/html", body: `
      <div id="root"></div><script type="module">
      import React from '/node_modules/.vite/deps/react.js';
      import ReactDOM from '/node_modules/.vite/deps/react-dom_client.js';
      import RefreshRuntime from '/@react-refresh';
      import { DEFAULT_HUD } from '/src/game/types.ts';
      import { WEAPONS } from '/src/components/game/data.ts';
      import '/src/styles.css';
      RefreshRuntime.injectIntoGlobalHook(window);
      window.$RefreshReg$ = () => {};
      window.$RefreshSig$ = () => type => type;
      window.__vite_plugin_react_preamble_installed__ = true;
      const { HudBar } = await import('/src/components/game/HudBar.tsx');
      const root = ReactDOM.createRoot(document.getElementById('root'));
      window.weapons = WEAPONS.length;
      window.renderHud = (weapon, reloading = 0) => root.render(React.createElement(HudBar, {
        hud: {...DEFAULT_HUD, weapon, health:100, armor:100, ammo:100, reserve:450, reloading},
        fps:60, resolution:'1920 × 1080', renderer:'webgl2'
      }));
      window.renderHud(0);
      </script>` }));
    await page.goto(url.href);
    await page.waitForFunction(() => window.renderHud);
    await page.evaluate(() => document.fonts.ready);
    for (const viewport of [{width:1280,height:800},{width:800,height:600},{width:390,height:844},{width:320,height:568}]) {
      await page.setViewportSize(viewport);
      for (let weapon = 0; weapon < await page.evaluate(() => window.weapons); weapon++) {
        for (const reload of [0, .5]) {
          await page.evaluate(([w,r]) => window.renderHud(w,r), [weapon,reload]);
          await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
          const failures = await page.evaluate(() => {
            const failures = [];
            for (const card of document.querySelectorAll('.hud-plate')) {
              const box = card.getBoundingClientRect(), safe = card.querySelector('.hud-content').getBoundingClientRect();
              // The dark interior of the actual panel art occupies y=.17..79.
              if (safe.top < box.top + box.height*.17 + 2 || safe.bottom > box.top + box.height*.79 - 2)
                failures.push('content area overlaps decorative rails');
              const walker = document.createTreeWalker(card, NodeFilter.SHOW_TEXT);
              while(walker.nextNode()) {
                const node = walker.currentNode;
                if(!node.textContent.trim())continue;
                const range=document.createRange();range.selectNodeContents(node);
                const clip=node.parentElement.closest('.hud-ammo-heading > span');
                for(const rect of range.getClientRects()) {
                  // Names may ellipsize horizontally, but all line boxes must fit vertically.
                  if(rect.top < safe.top-.5 || rect.bottom > safe.bottom+.5 || (!clip && (rect.left < safe.left-.5 || rect.right > safe.right+.5)))
                    failures.push({text:node.textContent,rect:{top:rect.top,bottom:rect.bottom},safe:{top:safe.top,bottom:safe.bottom}});
                }
              }
            }
            return failures;
          });
          assert.deepEqual(failures, [], `${viewport.width}×${viewport.height}, weapon ${weapon}, reload ${reload}`);
        }
      }
      await page.screenshot({path:`screenshots/hud-insets-${viewport.width}.png`});
    }
    assert.deepEqual(errors, []);
  } finally { await browser.close(); }
});
