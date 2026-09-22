import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 1) {
  const key = process.argv[i];
  if (!key.startsWith("--")) continue;
  args.set(key.slice(2), process.argv[i + 1] ?? "");
  i += 1;
}

function readDotEnv(path) {
  return readFile(path, "utf8").then((text) => {
    const values = {};
    for (const line of text.split(/\r?\n/)) {
      const match = line.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)\s*$/);
      if (match) values[match[1]] = match[2].replace(/^['"]|['"]$/g, "");
    }
    return values;
  });
}

const env = await readDotEnv(resolve(".env"));
const apiKey = process.env.CF_API_KEY || env.CF_API_KEY;
const accountId = process.env.CF_ACCOUNT_ID || args.get("account");
const model = args.get("model") || "@cf/black-forest-labs/flux-1-schnell";
const output = args.get("output");
const prompt = args.get("prompt");
const steps = Number(args.get("steps") || 4);
const requestedSeed = Number(args.get("seed") || 4127);

if (!apiKey) throw new Error("CF_API_KEY is missing from .env");
if (!accountId) throw new Error("Pass the Cloudflare account ID with --account");
if (!output) throw new Error("Pass an output path with --output");
if (!prompt) throw new Error("Pass the generation prompt with --prompt");
if (!Number.isInteger(steps) || steps < 1 || steps > 8) throw new Error("steps must be an integer from 1 to 8");

const response = await fetch(
  `https://api.cloudflare.com/client/v4/accounts/${accountId}/ai/run/${model}`,
  {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    // The REST endpoint currently rejects the documented seed field; keep the
    // requested seed in sidecar metadata while letting the service choose it.
    body: JSON.stringify({ prompt, steps }),
  },
);
const body = await response.json();
if (!response.ok || !body.success || typeof body.result?.image !== "string") {
  const errors = Array.isArray(body.errors) ? body.errors : [];
  throw new Error(`Cloudflare image generation failed (${response.status}): ${JSON.stringify(errors)}`);
}

const outputPath = resolve(output);
await mkdir(dirname(outputPath), { recursive: true });
await writeFile(outputPath, Buffer.from(body.result.image, "base64"));
await writeFile(
  outputPath.replace(/\.[^.]+$/, ".json"),
  `${JSON.stringify({
    model,
    steps,
    requestedSeed,
    seedApplied: false,
    requestedReviewSize: "512x512",
    usage: body.result.usage ?? null,
    prompt,
  }, null, 2)}\n`,
);
console.log(JSON.stringify({ generated: outputPath, usage: body.result.usage ?? null }));
