#!/usr/bin/env node
/** Generate one 1024px design render per BLACKSITE enemy from the locked plan. */
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 1) {
  const key = process.argv[i];
  if (!key.startsWith("--")) continue;
  args.set(key.slice(2), process.argv[i + 1] ?? "");
  i += 1;
}

function dotEnv(text) {
  const values = {};
  for (const line of text.split(/\r?\n/)) {
    const match = line.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)\s*$/);
    if (match) values[match[1]] = match[2].replace(/^['"]|['"]$/g, "");
  }
  return values;
}

const planPath = resolve(args.get("plan") || "art/blacksite-enemy-generation-plan.json");
const plan = JSON.parse(await readFile(planPath, "utf8"));
const env = dotEnv(await readFile(resolve(".env"), "utf8"));
const apiKey = process.env.CF_API_KEY || env.CF_API_KEY;
const account = process.env.CF_ACCOUNT_ID || args.get("account");
const model = args.get("model") || plan.model;
const steps = Number(args.get("steps") || plan.steps);
const only = args.get("spec") ? new Set(args.get("spec").split(",")) : null;
const outputDir = resolve(args.get("output-dir") || "art/source_hd/enemies");

if (!apiKey) throw new Error("CF_API_KEY is missing from .env");
if (!account) throw new Error("Pass the Cloudflare account ID with --account");
if (!Number.isInteger(steps) || steps < 1 || steps > 8) throw new Error("steps must be an integer from 1 to 8");

await mkdir(outputDir, { recursive: true });
for (const enemy of plan.enemies) {
  if (only && !only.has(enemy.specId) && !only.has(enemy.slug)) continue;
  const prompt = [
    "Create one complete isolated game character render.",
    enemy.description + ".",
    "dark military sci-fi, industrial realism, retro FPS pre-rendered sprite, neutral studio light, straight-on camera, clear silhouette.",
    "Use a completely flat neon magenta background #FF00FF with no other background detail.",
    "No room, no floor, no scenery, no cast shadow, no text, no logo, no border. Keep the subject fully inside the square frame, with magenta visible in every open area between limbs and equipment.",
  ].join(" ");
  const response = await fetch(`https://api.cloudflare.com/client/v4/accounts/${account}/ai/run/${model}`, {
    method: "POST",
    headers: { Authorization: `Bearer ${apiKey}`, "Content-Type": "application/json" },
    body: JSON.stringify({ prompt, steps }),
  });
  const body = await response.json();
  if (!response.ok || !body.success || typeof body.result?.image !== "string") {
    throw new Error(`${enemy.specId}: Cloudflare ${response.status} ${JSON.stringify(body.errors ?? [])}`);
  }
  const outputPath = resolve(outputDir, `${enemy.specId}.jpg`);
  await writeFile(outputPath, Buffer.from(body.result.image, "base64"));
  await writeFile(
    outputPath.replace(/\.jpg$/, ".json"),
    `${JSON.stringify({ specId: enemy.specId, slug: enemy.slug, model, steps, requestedSize: plan.sourceSize, prompt, usage: body.result.usage ?? null }, null, 2)}\n`,
  );
  console.log(JSON.stringify({ generated: outputPath, usage: body.result.usage ?? null }));
}
