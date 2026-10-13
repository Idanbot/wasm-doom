import { createHash } from "node:crypto";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Plugin } from "vite";

/** Content identity is stable across builds and includes the engine/art/audio. */
export async function generateAssetManifest(root = "public") {
  const files: string[] = [];
  async function walk(dir: string) {
    for (const entry of await readdir(path.join(root, dir), { withFileTypes: true })) {
      const name = path.posix.join(dir, entry.name);
      if (entry.isDirectory()) await walk(name);
      else if (/\.(png|webp|svg|mp3|ogg|wav|woff2?|json|wasm)$/.test(name) && name !== "game/asset-manifest.json") files.push(name);
    }
  }
  await walk("game");
  files.push("blacksite.wasm");
  const assets: Record<string, { bytes: number; sha256: string }> = {};
  for (const name of files.sort()) {
    const data = await readFile(path.join(root, name));
    assets[`/${name}`] = { bytes: data.length, sha256: createHash("sha256").update(data).digest("hex") };
  }
  const build = createHash("sha256").update(JSON.stringify(assets)).digest("hex");
  const output = JSON.stringify({ version: 1, build, assets });
  const target = path.join(root, "game/asset-manifest.json");
  if (await readFile(target, "utf8").catch(() => "") !== output + "\n") await writeFile(target, output + "\n");
  return { version: 1, build, assets };
}

export function assetManifestPlugin(): Plugin {
  return {
    name: "blacksite-asset-manifest",
    async buildStart() { await generateAssetManifest(); },
    configureServer(server) {
      let timer: ReturnType<typeof setTimeout> | undefined;
      let pending = Promise.resolve();
      const changed = (file: string) => {
        if (!file.startsWith(path.resolve("public") + path.sep) || file.endsWith("asset-manifest.json")) return;
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => {
          pending = pending.then(async () => { await generateAssetManifest(); })
            .catch(error => server.config.logger.error(`Asset manifest: ${error.message}`));
        }, 150);
      };
      for (const event of ["add", "change", "unlink"] as const) server.watcher.on(event, changed);
      server.httpServer?.once("close", () => {
        if (timer) clearTimeout(timer);
        for (const event of ["add", "change", "unlink"] as const) server.watcher.off(event, changed);
      });
    },
  };
}
