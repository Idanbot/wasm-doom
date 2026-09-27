import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const plan = JSON.parse(readFileSync("art/blacksite-voices.json", "utf8"));
const manifest = JSON.parse(readFileSync("public/game/voices/manifest.json", "utf8"));
assert.equal(manifest.model, "@cf/deepgram/aura-2-en");
assert.equal(manifest.enemies.length, plan.enemies.length);
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
const expectedLines = plan.enemies.reduce((count, enemy) => count + enemy.lines.length, 0);
assert.equal(all.length, expectedLines);
assert.equal(new Set(all.map((line) => line.id)).size, expectedLines);
assert.equal(new Set(all.map((line) => line.text)).size, expectedLines);
assert.equal(manifest.enemies.filter((e) => !e.lines.length).length, 4);
console.log(
  `[check:voices] ${expectedLines} Cloudflare clips, ${plan.enemies.length - 4} speaking profiles and 4 nonverbal profiles verified.`,
);

// Boss-kill lines live outside the enemy manifest (art/boss-voices.json).
const bossPlan = JSON.parse(readFileSync("art/boss-voices.json", "utf8"));
assert.equal(bossPlan.bosses.length, 20);
const bossIds = ["veyran", "hecate", "chimera", "oracle", "gravemind", "archivist", "halcyon", "relay", "titan", "kest"];
for (const [i, boss] of bossPlan.bosses.entries()) {
  const name = i < 6 ? bossIds[i] : i < 12 ? bossIds[i - 6] : bossIds[6 + Math.floor((i - 12) / 2)];
  const variant = i >= 6 && i < 12 || i >= 12 && (i - 12) % 2 === 1;
  assert.equal(boss.id, `boss-${name}${variant ? "-v2" : ""}`);
  assert.ok(boss.text.length > 10);
  const bytes = readFileSync(`public/game/voices/${boss.id}.mp3`);
  assert.ok(
    bytes.length > 1000 && (bytes.subarray(0, 3).toString() === "ID3" || bytes[0] === 255),
    `${boss.id}.mp3 must be a real MP3`,
  );
}
console.log(`[check:voices] ${bossPlan.bosses.length} boss-kill lines verified.`);
