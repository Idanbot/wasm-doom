import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import {
  checkWasmSync,
  collectSourceFiles,
  hashSources,
  updateWasmHash,
} from "./check-wasm-sync.mjs";

const execFileAsync = promisify(execFile);
const scriptPath = join(dirname(fileURLToPath(import.meta.url)), "check-wasm-sync.mjs");

function makeTree({ withWasm = true, withHash = true, drifted = false } = {}) {
  const root = mkdtempSync(join(tmpdir(), "wasm-sync-"));
  const srcDir = join(root, "engine", "src");
  mkdirSync(srcDir, { recursive: true });
  writeFileSync(join(srcDir, "lib.rs"), "fn main() {}\n");
  const wasmPath = join(root, "public", "hellscan.wasm");
  const hashPath = join(root, "public", "hellscan.sha256");
  if (withWasm || withHash) mkdirSync(join(root, "public"), { recursive: true });
  if (withWasm) writeFileSync(wasmPath, "wasm");
  if (withHash) updateWasmHash(hashPath, srcDir);
  if (drifted) writeFileSync(join(srcDir, "lib.rs"), "fn main() { /* changed */ }\n");
  return { root, wasmPath, hashPath, srcDir };
}

test("collectSourceFiles finds nested Rust files and ignores others", () => {
  const root = mkdtempSync(join(tmpdir(), "wasm-sync-"));
  mkdirSync(join(root, "sub"), { recursive: true });
  writeFileSync(join(root, "top.rs"), "x");
  writeFileSync(join(root, "sub", "nested.rs"), "x");
  writeFileSync(join(root, "notes.txt"), "x");
  assert.deepEqual(
    collectSourceFiles(root).map((f) => f.slice(root.length)),
    ["/sub/nested.rs", "/top.rs"],
  );
});

test("hashSources is stable and content-sensitive", () => {
  const { srcDir } = makeTree();
  const before = hashSources(srcDir);
  assert.equal(hashSources(srcDir), before);
  writeFileSync(join(srcDir, "extra.rs"), "y");
  assert.notEqual(hashSources(srcDir), before);
});

test("checkWasmSync reports in-sync trees as ok", () => {
  const { wasmPath, hashPath, srcDir } = makeTree();
  assert.equal(checkWasmSync(wasmPath, hashPath, srcDir).ok, true);
});

test("checkWasmSync reports a missing wasm", () => {
  const { wasmPath, hashPath, srcDir } = makeTree({ withWasm: false });
  const result = checkWasmSync(wasmPath, hashPath, srcDir);
  assert.equal(result.ok, false);
  assert.equal(result.reason, "missing");
});

test("checkWasmSync reports a missing hash file", () => {
  const { wasmPath, hashPath, srcDir } = makeTree({ withHash: false });
  const result = checkWasmSync(wasmPath, hashPath, srcDir);
  assert.equal(result.ok, false);
  assert.equal(result.reason, "missing");
});

test("checkWasmSync reports drifted sources as stale", () => {
  const { wasmPath, hashPath, srcDir } = makeTree({ drifted: true });
  const result = checkWasmSync(wasmPath, hashPath, srcDir);
  assert.equal(result.ok, false);
  assert.equal(result.reason, "stale");
});

test("cli passes against the real repo tree", async () => {
  const { stdout } = await execFileAsync(process.execPath, [scriptPath]);
  assert.match(stdout, /in sync/);
});

test("updateWasmHash writes a 64-char hex digest", () => {
  const { hashPath } = makeTree();
  assert.match(readFileSync(hashPath, "utf8").trim(), /^[0-9a-f]{64}$/);
});
