#!/usr/bin/env node
/** One kill line per boss via Cloudflare Aura-2. Never ships credentials. */
import { readFile, writeFile, mkdir, access } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

const plan = JSON.parse(await readFile("art/boss-voices.json", "utf8"));
const envText = await readFile(".env", "utf8");
const env = Object.fromEntries(
  envText.split(/\r?\n/).flatMap((line) => {
    const m = line.match(/^\s*(\w+)\s*=\s*(.*?)\s*$/);
    return m ? [[m[1], m[2].replace(/^['"]|['"]$/g, "")]] : [];
  }),
);
const key = process.env.CF_API_KEY || env.CF_API_KEY;
const account = process.env.CF_ACCOUNT_ID || env.CF_ACCOUNT_ID;
if (!key || !account || !/^[a-f0-9]{32}$/i.test(account))
  throw new Error("CF_API_KEY and a valid Cloudflare Account ID are required.");
// Same voice-shaping chain as generate-blacksite-voices.mjs.
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
const report = [];
for (const boss of plan.bosses) {
  const raw = `art/source_hd/voices/${boss.id}.wav`;
  const path = `public/game/voices/${boss.id}.mp3`;
  const signature = createHash("sha256")
    .update(JSON.stringify([plan.model, boss.speaker, boss.text, effects[boss.effect]]))
    .digest("hex");
  let cached = false;
  try {
    cached = JSON.parse(await readFile(`${raw}.json`, "utf8")).signature === signature;
  } catch {}
  if (!cached) {
    const response = await fetch(
      `https://api.cloudflare.com/client/v4/accounts/${account}/ai/run/${plan.model}`,
      {
        method: "POST",
        headers: { Authorization: `Bearer ${key}`, "Content-Type": "application/json" },
        body: JSON.stringify({
          text: boss.text,
          speaker: boss.speaker,
          encoding: "linear16",
          container: "wav",
          sample_rate: 24000,
        }),
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
        { signature, model: plan.model, speaker: boss.speaker, effect: effects[boss.effect], generatedAt: new Date().toISOString() },
        null,
        2,
      ) + "\n",
    );
  }
  await access(raw);
  const encode = spawnSync(
    "ffmpeg",
    ["-hide_banner", "-loglevel", "error", "-y", "-i", raw, "-af",
      `${effects[boss.effect]},loudnorm=I=-18:TP=-2:LRA=7,afade=t=in:d=0.015`,
      "-ac", "1", "-ar", "24000", "-codec:a", "libmp3lame", "-b:a", "64k", path],
    { encoding: "utf8" },
  );
  if (encode.status !== 0) throw new Error(encode.stderr);
  const probe = spawnSync(
    "ffprobe",
    ["-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1", path],
    { encoding: "utf8" },
  );
  const duration = Number(probe.stdout.trim());
  if (!(duration > 0.2 && duration < 12)) throw new Error(`Invalid duration for ${boss.id}`);
  report.push({ id: boss.id, url: `/game/voices/${boss.id}.mp3`, duration });
  console.log(`${cached ? "cached" : "generated"} ${boss.id}: ${duration.toFixed(2)}s`);
}
await writeFile("art/source_hd/voices/boss-usage.json", JSON.stringify({ report }, null, 2) + "\n");
