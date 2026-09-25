#!/usr/bin/env node
/** Generate isolated first-person weapon and player VFX masters with Workers AI. */
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const MODEL = "@cf/black-forest-labs/flux-1-schnell";
const SPECS = {
  missile_rear: [
    "Isolated compact guided missile already in flight, viewed from directly behind as it travels away from the player toward the upper-center vanishing point.",
    "Circular rear engine nozzle faces the viewer, four readable stabilizer fins around a foreshortened dark gunmetal missile body, small hot amber-white exhaust core with a tight short blue-orange plume trailing downward toward the viewer.",
    "Military science-fiction projectile, realistic hard-surface game VFX, centered, compact silhouette, no launcher and no fireball.",
  ],
  muzzle_flash_front: [
    "Isolated compact radial amber light-burst VFX icon, centered and seen straight-on.",
    "Small bright ivory core, six short irregular amber rays, a few tiny sparks and one faint grey smoke ring. Tight restrained silhouette suitable for a modern action game.",
    "Crisp realistic game effect, no weapon, no characters, no giant circular glow.",
  ],
  missile_impact: [
    "Isolated guided missile impact explosion at the instant of detonation.",
    "Dense compact orange-white blast core, angular metal fragments, pressure ring, dark grey smoke and restrained cyan electrical sparks. Radial groundless game VFX with a clear edge and no lingering flames.",
    "Realistic military science-fiction game effect, centered, no environment, no weapon and no text.",
  ],
};

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
const outputDir = resolve("art/source_hd/combat_v3");
await mkdir(outputDir, { recursive: true });

for (const [slug, lines] of Object.entries(SPECS)) {
  if (only && slug !== only) continue;
  const prompt = [
    ...lines,
    "Solid perfectly flat neon magenta #FF00FF background across the entire image including all holes and gaps. No room, floor, scenery, text, logo, frame, border, cropped subject, or extra objects.",
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
