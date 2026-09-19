#!/usr/bin/env node
/**
 * WASM sync check — ensures public/hellscan.wasm matches engine/src.
 *
 * Content-based (not mtimes, which are unreliable after a fresh git
 * checkout): `npm run build:wasm` records a SHA-256 over every
 * engine/src Rust file into public/hellscan.sha256. This script recomputes
 * the hash and exits 1 with a rebuild hint when the wasm/hash is missing
 * or the sources drifted.
 *
 * Used in CI and as `npm run check:wasm`. Pass `--update` (used by
 * `npm run build:wasm`) to write the hash file instead of checking it.
 */
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

export function collectSourceFiles(dir) {
  const out = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...collectSourceFiles(full));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      out.push(full);
    }
  }
  out.sort();
  return out;
}

export function hashSources(srcDir) {
  const hash = createHash("sha256");
  for (const file of collectSourceFiles(srcDir)) {
    // Relative path: absolute checkouts differ per machine (notably CI).
    hash.update(file.slice(srcDir.length));
    hash.update("\0");
    hash.update(readFileSync(file));
    hash.update("\0");
  }
  return hash.digest("hex");
}

export function checkWasmSync(wasmPath, hashPath, srcDir) {
  try {
    statSync(wasmPath);
  } catch {
    return { ok: false, reason: "missing", detail: "wasm" };
  }
  let recorded;
  try {
    recorded = readFileSync(hashPath, "utf8").trim();
  } catch {
    return { ok: false, reason: "missing", detail: "hash" };
  }
  const current = hashSources(srcDir);
  if (recorded !== current) {
    return { ok: false, reason: "stale", recorded, current };
  }
  return { ok: true, reason: "in-sync", current };
}

export function updateWasmHash(hashPath, srcDir) {
  writeFileSync(hashPath, `${hashSources(srcDir)}\n`);
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url);
if (isMain) {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const wasmPath = join(root, "public", "hellscan.wasm");
  const hashPath = join(root, "public", "hellscan.sha256");
  const srcDir = join(root, "engine", "src");
  if (process.argv.includes("--update")) {
    updateWasmHash(hashPath, srcDir);
    console.log("[check:wasm] recorded engine sources hash.");
  } else {
    const result = checkWasmSync(wasmPath, hashPath, srcDir);
    if (!result.ok) {
      if (result.reason === "missing") {
        console.error(`[check:wasm] public/hellscan.${result.detail === "wasm" ? "wasm" : "sha256"} is missing.`);
      } else {
        console.error("[check:wasm] engine/src drifted from public/hellscan.wasm.");
      }
      console.error("[check:wasm] Rebuild with: npm run build:wasm");
      process.exit(1);
    }
    console.log("[check:wasm] public/hellscan.wasm is in sync with engine/src.");
  }
}
