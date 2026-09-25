#!/usr/bin/env node
/** Generate fallback first-person concept masters with Cloudflare Workers AI. */
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const MODEL = "@cf/black-forest-labs/flux-1-schnell";
const WEAPONS = [
  [
    "mk23s",
    "compact suppressed military pistol, squared slide, long integral suppressor, amber status diode",
  ],
  [
    "br12",
    "short brutal pump shotgun, thick barrel shroud, visible pump grip, industrial breaching teeth",
  ],
  ["kx9", "compact bullpup machine carbine, short barrel, box magazine, cyan ammunition counter"],
  [
    "mr4",
    "heavy electromagnetic precision rifle, twin acceleration rails, narrow cyan charge channel, angular stock",
  ],
  [
    "vlk6",
    "unmistakable shoulder-fired guided missile launcher, large rectangular launch tube, wide hollow muzzle, side targeting optic, top carry rail; absolutely not a flamethrower",
  ],
  [
    "ax12",
    "experimental arc caster, split copper induction prongs around a glowing cyan capacitor, insulated gunmetal body",
  ],
  [
    "m91",
    "heavy rotary cannon with a clearly visible cluster of six barrels, armored motor housing, belt-feed box and amber heat vents",
  ],
];
const envText = await readFile(resolve(".env"), "utf8");
const key = envText.match(/^CF_API_KEY=(.+)$/m)?.[1]?.trim();
if (!key) throw new Error("CF_API_KEY is missing from the ignored .env file");
const account =
  process.env.CF_ACCOUNT_ID ?? envText.match(/^CF_ACCOUNT_ID=(.+)$/m)?.[1]?.trim();
if (!account || !/^[a-f0-9]{32}$/i.test(account))
  throw new Error("CF_ACCOUNT_ID is missing from the ignored .env file");
const only = process.argv.includes("--spec")
  ? process.argv[process.argv.indexOf("--spec") + 1]
  : "";
// Keep lossy Cloudflare concepts separate from the transparent imagegen masters
// and their eight-frame reload grids consumed by the runtime processor.
const outputDir = resolve("art/source_hd/weapons_cloudflare");
await mkdir(outputDir, { recursive: true });

for (const [slug, description] of WEAPONS) {
  if (only && slug !== only) continue;
  const prompt = [
    `First-person FPS viewmodel of a ${description}.`,
    "Centered lower-screen weapon held in two black tactical gloves, barrel points toward the exact top center, strong readable silhouette and believable mechanical construction.",
    "BLACKSITE industrial military science-fiction style, worn gunmetal, restrained amber and cyan indicators, realistic hard-surface game render, orthographic game asset lighting.",
    "Solid flat neon magenta #FF00FF background including gaps around hands and weapon. No room, floor, smoke, flame, muzzle flash, projectile, text, logo, border, cropped muzzle or cropped hands.",
  ].join(" ");
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${account}/ai/run/${MODEL}`,
    {
      method: "POST",
      headers: { Authorization: `Bearer ${key}`, "Content-Type": "application/json" },
      body: JSON.stringify({ prompt, steps: 8 }),
    },
  );
  const body = await response.json();
  if (!response.ok || !body.success || typeof body.result?.image !== "string") {
    throw new Error(`${slug}: Cloudflare ${response.status} ${JSON.stringify(body.errors ?? [])}`);
  }
  const output = resolve(outputDir, `${slug}.jpg`);
  await writeFile(output, Buffer.from(body.result.image, "base64"));
  await writeFile(
    output.replace(/\.jpg$/, ".json"),
    `${JSON.stringify({ model: MODEL, steps: 8, prompt, usage: body.result.usage ?? null }, null, 2)}\n`,
  );
  console.log(JSON.stringify({ generated: output, usage: body.result.usage ?? null }));
}
