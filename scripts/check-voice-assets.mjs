import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const plan = JSON.parse(readFileSync("art/blacksite-voices.json", "utf8"));
const manifest = JSON.parse(readFileSync("public/game/voices/manifest.json", "utf8"));
assert.equal(manifest.model, "@cf/deepgram/aura-2-en");
assert.equal(manifest.enemies.length, 13);
const all = [];
for (const enemy of manifest.enemies) {
  const source = plan.enemies.find((p) => p.skin === enemy.skin);
  assert.ok(source);
  assert.equal(enemy.lines.length, source.lines.length);
  assert.ok(enemy.lines.length === 0 || (enemy.lines.length >= 5 && enemy.lines.length <= 7));
  for (const [index, line] of enemy.lines.entries()) {
    assert.equal(line.text, source.lines[index][1]);
    assert.equal(line.cue, source.lines[index][0]);
    assert.match(line.url, /^\/game\/voices\/[a-z]+-\d\.mp3$/);
    assert.ok(line.duration > 0.2 && line.duration < 12);
    const bytes = readFileSync(`public${line.url}`);
    assert.ok(
      bytes.length > 1000 && (bytes.subarray(0, 3).toString() === "ID3" || bytes[0] === 255),
    );
    all.push(line);
  }
}
assert.equal(all.length, 54);
assert.equal(new Set(all.map((line) => line.id)).size, 54);
assert.equal(new Set(all.map((line) => line.text)).size, 54);
assert.equal(manifest.enemies.filter((e) => !e.lines.length).length, 4);
console.log(
  "[check:voices] 54 Cloudflare clips, 9 speaking profiles and 4 nonverbal profiles verified.",
);
