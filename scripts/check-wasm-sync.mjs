#!/usr/bin/env node
/**
 * WASM sync check — ensures public/hellscan.wasm matches engine/src.
 *
 * - Compares mtimes of engine src Rust files against public hellscan wasm.
 * - Exits 1 with a rebuild hint when the wasm is missing or stale.
 * - Used in CI and as `npm run check:wasm`.
 */
import { statSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

export function newestSourceMtime(dir) {
  let newest = 0;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      newest = Math.max(newest, newestSourceMtime(full));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      newest = Math.max(newest, statSync(full).mtimeMs);
    }
  }
  return newest;
}

export function checkWasmSync(wasmPath, srcDir) {
  let wasmMtime = 0;
  try {
    wasmMtime = statSync(wasmPath).mtimeMs;
  } catch {
    return { ok: false, reason: "missing" };
  }
  const srcMtime = newestSourceMtime(srcDir);
  if (srcMtime > wasmMtime) {
    return { ok: false, reason: "stale", wasmMtime, srcMtime };
  }
  return { ok: true, reason: "in-sync", wasmMtime, srcMtime };
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url);
if (isMain) {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const result = checkWasmSync(join(root, "public", "hellscan.wasm"), join(root, "engine", "src"));
  if (!result.ok) {
    if (result.reason === "missing") {
      console.error("[check:wasm] public/hellscan.wasm is missing.");
    } else {
      console.error("[check:wasm] public/hellscan.wasm is older than engine/src.");
    }
    console.error("[check:wasm] Rebuild with: npm run build:wasm");
    process.exit(1);
  }
  console.log("[check:wasm] public/hellscan.wasm is in sync with engine/src.");
}
