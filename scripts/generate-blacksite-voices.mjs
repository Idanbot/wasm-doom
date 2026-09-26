/** Bounded, resumable Cloudflare Aura-2 asset generation. Never ships credentials. */
import { readFile, writeFile, mkdir, access } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    account: { type: "string" },
    limit: { type: "string", default: "54" },
    "dry-run": { type: "boolean" },
  },
});
const plan = JSON.parse(await readFile("art/blacksite-voices.json", "utf8"));
const characters = plan.enemies.flatMap((e) => e.lines).reduce((n, [, text]) => n + text.length, 0);
const estimate = {
  model: plan.model,
  clips: plan.enemies.reduce((n, enemy) => n + enemy.lines.length, 0),
  characters,
  estimatedUsd: (characters * 0.03) / 1000,
  estimatedNeurons: Math.ceil(((characters * 0.03) / 1000 / 0.011) * 1000),
};
console.log(JSON.stringify(estimate));
if (values["dry-run"]) process.exit(0);
const envText = await readFile(".env", "utf8");
const env = Object.fromEntries(
  envText.split(/\r?\n/).flatMap((line) => {
    const m = line.match(/^\s*(\w+)\s*=\s*(.*?)\s*$/);
    return m ? [[m[1], m[2].replace(/^['"]|['"]$/g, "")]] : [];
  }),
);
const key = process.env.CF_API_KEY || env.CF_API_KEY;
const account = values.account || process.env.CF_ACCOUNT_ID || env.CF_ACCOUNT_ID;
if (!key || !account || !/^[a-f0-9]{32}$/i.test(account))
  throw new Error("CF_API_KEY and a valid Cloudflare Account ID are required.");
const limit = Number(values.limit);
if (!Number.isInteger(limit) || limit < 1 || limit > 54) throw new Error("limit must be 1–54");
const effects = {
  radio: "highpass=f=240,lowpass=f=4400",
  respirator: "highpass=f=380,lowpass=f=2600,aecho=0.8:0.6:18:0.16",
  heavy: "asetrate=22080,aresample=24000,atempo=1.04,highpass=f=110,lowpass=f=5200",
  comms: "highpass=f=190,lowpass=f=4800",
  "robot-low":
    "asetrate=20400,aresample=24000,atempo=1.12,aeval=val(0)*(0.72+0.28*sin(2*PI*48*t)),highpass=f=120,lowpass=f=3600",
  "robot-high": "aeval=val(0)*(0.7+0.3*sin(2*PI*76*t)),highpass=f=350,lowpass=f=6000",
  overlord:
    "asetrate=21120,aresample=24000,atempo=1.05,aecho=0.8:0.6:44:0.18,highpass=f=100,lowpass=f=5200",
};
await mkdir("art/source_hd/voices", { recursive: true });
await mkdir("public/game/voices", { recursive: true });
let generated = 0;
const enemies = [];
for (const enemy of plan.enemies) {
  const lines = [];
  for (const [index, [cue, text]] of enemy.lines.entries()) {
    const id = `${enemy.id}-${index + 1}`;
    const raw = `art/source_hd/voices/${id}.wav`;
    const path = `public/game/voices/${id}.mp3`;
    const signature = createHash("sha256")
      .update(JSON.stringify([plan.model, enemy.speaker, text, effects[enemy.effect]]))
      .digest("hex");
    let cached = false;
    try {
      cached = JSON.parse(await readFile(`${raw}.json`, "utf8")).signature === signature;
    } catch {}
    if (!cached) {
      if (generated >= limit) throw new Error("Generation limit reached; rerun to resume.");
      const request = {
        text,
        speaker: enemy.speaker,
        encoding: "linear16",
        container: "wav",
        sample_rate: 24000,
      };
      const response = await fetch(
        `https://api.cloudflare.com/client/v4/accounts/${account}/ai/run/${plan.model}`,
        {
          method: "POST",
          headers: { Authorization: `Bearer ${key}`, "Content-Type": "application/json" },
          body: JSON.stringify(request),
          signal: AbortSignal.timeout(90000),
        },
      );
      const bytes = Buffer.from(await response.arrayBuffer());
      if (!response.ok || bytes.subarray(0, 4).toString() !== "RIFF") {
        throw new Error(`Cloudflare ${response.status}: ${bytes.toString("utf8").slice(0, 700)}`);
      }
      await writeFile(raw, bytes);
      await writeFile(
        `${raw}.json`,
        JSON.stringify(
          {
            signature,
            model: plan.model,
            request,
            style: enemy.style,
            effect: effects[enemy.effect],
            generatedAt: new Date().toISOString(),
          },
          null,
          2,
        ) + "\n",
      );
      generated++;
    }
    await access(raw);
    const filter = `${effects[enemy.effect]},loudnorm=I=-18:TP=-2:LRA=7,afade=t=in:d=0.015`;
    const encode = spawnSync(
      "ffmpeg",
      [
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        raw,
        "-af",
        filter,
        "-ac",
        "1",
        "-ar",
        "24000",
        "-codec:a",
        "libmp3lame",
        "-b:a",
        "64k",
        path,
      ],
      { encoding: "utf8" },
    );
    if (encode.status !== 0) throw new Error(encode.stderr);
    const probe = spawnSync(
      "ffprobe",
      ["-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1", path],
      { encoding: "utf8" },
    );
    const duration = Number(probe.stdout.trim());
    if (!(duration > 0.2 && duration < 12)) throw new Error(`Invalid duration for ${id}`);
    lines.push({ id, cue, text, url: `/game/voices/${id}.mp3`, duration });
    console.log(`${cached ? "cached" : "generated"} ${id}: ${duration.toFixed(2)}s`);
  }
  enemies.push({
    skin: enemy.skin,
    id: enemy.id,
    name: enemy.name,
    style: enemy.style,
    speaker: enemy.speaker ?? null,
    lines,
  });
}
await writeFile(
  "public/game/voices/manifest.json",
  JSON.stringify({ model: plan.model, enemies }, null, 2) + "\n",
);
await writeFile(
  "art/source_hd/voices/usage.json",
  JSON.stringify(
    {
      ...estimate,
      generatedThisRun: generated,
      note: "Estimated character billing, not measured account usage. Cached assets do not call Cloudflare.",
    },
    null,
    2,
  ) + "\n",
);
