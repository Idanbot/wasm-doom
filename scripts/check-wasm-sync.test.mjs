import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync, utimesSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { checkWasmSync, newestSourceMtime } from "./check-wasm-sync.mjs";

const execFileAsync = promisify(execFile);
const scriptPath = join(dirname(fileURLToPath(import.meta.url)), "check-wasm-sync.mjs");

function makeTree({ withWasm = true, stale = false } = {}) {
  const root = mkdtempSync(join(tmpdir(), "wasm-sync-"));
  const srcDir = join(root, "engine", "src");
  mkdirSync(srcDir, { recursive: true });
  const src = join(srcDir, "lib.rs");
  writeFileSync(src, "fn main() {}\n");
  const wasmPath = join(root, "public", "hellscan.wasm");
  if (withWasm) {
    mkdirSync(join(root, "public"), { recursive: true });
    writeFileSync(wasmPath, "wasm");
  }
  const now = Date.now() / 1000;
  // src older than wasm by default; flip for the stale case.
  utimesSync(src, now - 100, now - 100);
  if (withWasm) utimesSync(wasmPath, now, now);
  if (stale && withWasm) {
    utimesSync(src, now + 100, now + 100);
  }
  return { root, wasmPath, srcDir };
}

test("newestSourceMtime finds nested Rust files and ignores others", () => {
  const root = mkdtempSync(join(tmpdir(), "wasm-sync-"));
  mkdirSync(join(root, "sub"), { recursive: true });
  writeFileSync(join(root, "top.rs"), "x");
  writeFileSync(join(root, "sub", "nested.rs"), "x");
  writeFileSync(join(root, "notes.txt"), "x");
  const mtime = newestSourceMtime(root);
  assert.ok(mtime > 0);
});

test("checkWasmSync reports in-sync trees as ok", () => {
  const { wasmPath, srcDir } = makeTree();
  assert.equal(checkWasmSync(wasmPath, srcDir).ok, true);
});

test("checkWasmSync reports a missing wasm", () => {
  const { wasmPath, srcDir } = makeTree({ withWasm: false });
  const result = checkWasmSync(wasmPath, srcDir);
  assert.equal(result.ok, false);
  assert.equal(result.reason, "missing");
});

test("checkWasmSync reports a stale wasm", () => {
  const { wasmPath, srcDir } = makeTree({ stale: true });
  const result = checkWasmSync(wasmPath, srcDir);
  assert.equal(result.ok, false);
  assert.equal(result.reason, "stale");
});

test("cli passes against the real repo tree", async () => {
  const { stdout } = await execFileAsync(process.execPath, [scriptPath]);
  assert.match(stdout, /in sync/);
});
