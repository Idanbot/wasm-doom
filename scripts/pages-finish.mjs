import { existsSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const out = join(process.cwd(), "dist-pages");
const nested = join(out, "pages", "index.html");
const index = join(out, "index.html");
if (!existsSync(index) && existsSync(nested)) renameSync(nested, index);
if (!existsSync(index)) {
  throw new Error("dist-pages/index.html is missing — GitHub Pages would serve a blank site");
}
if (!existsSync(join(out, "blacksite.wasm"))) {
  throw new Error("dist-pages/blacksite.wasm is missing");
}
const pagesDir = join(out, "pages");
if (existsSync(pagesDir)) rmSync(pagesDir, { recursive: true, force: true });
writeFileSync(join(out, ".nojekyll"), "");
