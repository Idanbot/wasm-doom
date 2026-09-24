#!/usr/bin/env node
/** Generate one readable world-pickup case for every BLACKSITE weapon. */
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const ACCOUNT = "CF_ACCOUNT_ID_FROM_ENV";
const MODEL = "@cf/black-forest-labs/flux-1-schnell";
const WEAPONS = {
  mk23s: "compact suppressed black tactical pistol in a short narrow charcoal pistol case",
  m870k: "compact breaching shotgun in a medium wide dark steel shotgun case",
  vx9: "angular compact submachine gun in a medium rectangular graphite weapon case",
  shrike: "long slim cyan-accented electromagnetic marksman rifle in an elongated narrow gun case",
  raven:
    "bulky short guided-missile launcher with a large round optic in a deep heavy weapons case",
  arc12:
    "wide futuristic electric arc shotgun with restrained cyan coils in a broad reinforced case",
};

const envText = await readFile(resolve(".env"), "utf8");
const key = envText.match(/^CF_API_KEY=(.+)$/m)?.[1]?.trim();
if (!key) throw new Error("CF_API_KEY is missing from the ignored .env file");
const only = process.argv.includes("--spec")
  ? process.argv[process.argv.indexOf("--spec") + 1]
  : "";
const outputDir = resolve("art/source_hd/weapon_cases");
await mkdir(outputDir, { recursive: true });

for (const [slug, weapon] of Object.entries(WEAPONS)) {
  if (only && slug !== only) continue;
  const prompt = [
    `One isolated open military science-fiction gun case containing a ${weapon}.`,
    "Three-quarter overhead game pickup view. The foam insert precisely follows the weapon silhouette; the case proportions and thickness visibly match the weapon size. Worn dark gunmetal shell, practical hinges, small amber status lamps, realistic hard-surface detail, no loose ammunition.",
    "Entire open case fully visible, centered, generous clearance on all sides, strong readable silhouette at icon size.",
    "Solid perfectly flat neon magenta #FF00FF background including every gap and hole. No floor, room, scenery, people, hands, text, lettering, logo, border, shadow outside the case, or extra objects.",
  ].join(" ");
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${ACCOUNT}/ai/run/${MODEL}`,
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
