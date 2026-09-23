#!/usr/bin/env node
/**
 * Asset pipeline gate for the BLACKSITE IFRIT conversion.
 *
 * Validates `art/blacksite-manifest.json` structurally (unique ids, files,
 * atlas slots, known engine kinds/slots) and checks every `ready` entry's
 * delivered file: exists under public/game/, valid PNG, expected dimensions,
 * alpha-capable color type, snake_case name, plus the spec §35 metadata
 * JSON schema when an `art/atlases/<specId>.json` sidecar exists.
 *
 * Passes vacuously until the first asset batch lands (all `planned`).
 * Used in CI and as `npm run check:assets`.
 */
import { readFileSync, existsSync } from "node:fs";
import { join, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";

const KNOWN_KINDS = new Set(["EK_HUSK", "EK_BRUTE", "EK_WRAITH", "EK_BOSS", "EK_BARREL"]);
const ATLAS_SLOTS = 121; // World + 13 × 7 enemy animation layers + ordnance
const STATUSES = new Set(["planned", "ready"]);

export function isSnakePng(name) {
  return /^[a-z0-9_]+\.png$/.test(name);
}

/** Minimal PNG header parse (signature + IHDR). Throws on invalid data. */
export function readPngInfo(buf) {
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  if (buf.length < 33 || !buf.subarray(0, 8).equals(sig)) {
    throw new Error("not a PNG");
  }
  const type = buf.subarray(12, 16).toString("ascii");
  if (type !== "IHDR") throw new Error("missing IHDR");
  const width = buf.readUInt32BE(16);
  const height = buf.readUInt32BE(20);
  const colorType = buf[25];
  // 3 = palette (alpha via tRNS), 4 = gray+alpha, 6 = RGBA.
  return { width, height, colorType, hasAlpha: colorType === 3 || colorType === 4 || colorType === 6 };
}

export function validateMetadataSchema(meta) {
  const errors = [];
  if (typeof meta !== "object" || meta === null) return ["metadata is not an object"];
  if (typeof meta.id !== "string") errors.push("metadata.id must be a string");
  for (const k of ["frameWidth", "frameHeight"]) {
    if (!Number.isInteger(meta[k]) || meta[k] <= 0) errors.push(`metadata.${k} must be a positive int`);
  }
  if (typeof meta.origin?.x !== "number" || typeof meta.origin?.y !== "number") {
    errors.push("metadata.origin must be {x, y} numbers");
  }
  if (typeof meta.animations !== "object" || meta.animations === null) {
    errors.push("metadata.animations must be an object");
  } else {
    for (const [name, a] of Object.entries(meta.animations)) {
      if (!Number.isInteger(a?.start) || a.start < 0) errors.push(`animations.${name}.start must be >= 0`);
      if (!Number.isInteger(a?.frames) || a.frames <= 0) errors.push(`animations.${name}.frames must be > 0`);
      if (typeof a?.fps !== "number" || a.fps <= 0) errors.push(`animations.${name}.fps must be > 0`);
    }
  }
  return errors;
}

function eachEntry(manifest) {
  const out = [];
  for (const [section, kind] of [["enemies", "enemy"], ["weapons", "weapon"], ["textures", "texture"], ["props", "prop"]]) {
    for (const e of manifest[section] ?? []) out.push({ section, kind, ...e });
  }
  return out;
}

export function validateManifest(manifest) {
  const errors = [];
  const warnings = [];
  const seenId = new Set();
  const seenFile = new Set();
  const seenSlot = new Set();
  const entries = eachEntry(manifest);
  if (entries.length === 0) errors.push("manifest has no entries");
  for (const e of entries) {
    const tag = `${e.section}/${e.specId}`;
    if (typeof e.specId !== "string" || !e.specId) errors.push(`${tag}: missing specId`);
    if (seenId.has(e.specId)) errors.push(`duplicate specId: ${e.specId}`);
    seenId.add(e.specId);
    if (!STATUSES.has(e.status)) errors.push(`${tag}: bad status ${e.status}`);
    const files = e.animationFiles ?? (e.file ? [e.file] : (e.files ?? []));
    if (files.length === 0) errors.push(`${tag}: no files listed`);
    for (const f of files) {
      if (!isSnakePng(basename(f))) errors.push(`${tag}: file not snake_case PNG: ${f}`);
      if (seenFile.has(f)) errors.push(`duplicate file: ${f}`);
      seenFile.add(f);
    }
    if (e.section === "weapons") {
      if (e.engineSlot == null) errors.push(`${tag}: weapons need engineSlot 0-4`);
    } else if (e.textureSlot != null) {
      if (!Number.isInteger(e.textureSlot) || e.textureSlot < 0) {
        errors.push(`${tag}: bad textureSlot ${e.textureSlot}`);
      } else {
        if (seenSlot.has(e.textureSlot)) errors.push(`duplicate textureSlot: ${e.textureSlot}`);
        seenSlot.add(e.textureSlot);
        if (e.textureSlot >= ATLAS_SLOTS && !e.needsSlotExtension) {
          warnings.push(`${tag}: slot ${e.textureSlot} needs a TEX_N extension first`);
        }
      }
    } else if (!e.needsSlotExtension) {
      errors.push(`${tag}: null textureSlot without needsSlotExtension`);
    }
    if (e.engineKind && !KNOWN_KINDS.has(e.engineKind)) errors.push(`${tag}: unknown engineKind ${e.engineKind}`);
    if (e.engineSlot != null && (!Number.isInteger(e.engineSlot) || e.engineSlot < 0 || e.engineSlot > 4)) {
      errors.push(`${tag}: bad engineSlot ${e.engineSlot}`);
    }
    if (e.sheet && e.sheet !== "1x1" && e.sheet !== "2x2") errors.push(`${tag}: bad sheet ${e.sheet}`);
  }
  // Engine weapon slots 0-4 must be covered exactly once when any weapon is ready.
  const readyWeapons = entries.filter((e) => e.section === "weapons" && e.status === "ready");
  if (readyWeapons.length > 0) {
    const slots = readyWeapons.map((e) => e.engineSlot).sort();
    if (JSON.stringify(slots) !== JSON.stringify([0, 1, 2, 3, 4].slice(0, slots.length))) {
      warnings.push("ready weapons should fill engine slots 0..4 in order");
    }
  }
  return { errors, warnings };
}

export function validateReadyFiles(manifest, gameDir, atlasesDir) {
  const errors = [];
  const warnings = [];
  let checked = 0;
  for (const e of eachEntry(manifest)) {
    if (e.status !== "ready") continue;
    const files = e.animationFiles
      ? e.animationFiles.map((name) => ({ name, min: e.size ?? 256, exact: true }))
      : e.file
        ? [{ name: e.file, min: e.size ?? 256, exact: e.kind !== "weapon" }]
        : (e.files ?? []).map((name) => ({ name, min: 512, exact: false }));
    for (const { name, min, exact } of files) {
      const full = join(gameDir, basename(name));
      if (!existsSync(full)) {
        errors.push(`${e.specId}: missing delivered file ${name}`);
        continue;
      }
      let info;
      try {
        info = readPngInfo(readFileSync(full));
      } catch (err) {
        errors.push(`${e.specId}: ${name} is not a valid PNG (${err.message})`);
        continue;
      }
      checked += 1;
      if (info.width !== info.height) errors.push(`${e.specId}: ${name} must be square, got ${info.width}x${info.height}`);
      if (exact && info.width !== min) errors.push(`${e.specId}: ${name} must be ${min}x${min}, got ${info.width}x${info.height}`);
      if (!exact && info.width < min) errors.push(`${e.specId}: ${name} must be at least ${min}px, got ${info.width}`);
      if (!info.hasAlpha) errors.push(`${e.specId}: ${name} has no alpha channel (spec §11)`);
    }
    const metaPath = join(atlasesDir, `${e.specId}.json`);
    if (existsSync(metaPath)) {
      let meta;
      try {
        meta = JSON.parse(readFileSync(metaPath, "utf8"));
      } catch {
        errors.push(`${e.specId}: metadata JSON is invalid`);
        continue;
      }
      for (const err of validateMetadataSchema(meta)) errors.push(`${e.specId}: ${err}`);
      if (meta.id !== e.specId) errors.push(`${e.specId}: metadata id mismatch (${meta.id})`);
    } else {
      warnings.push(`${e.specId}: ready without art/atlases metadata sidecar`);
    }
  }
  return { errors, warnings, checked };
}

export function checkAssets(root) {
  const manifestPath = join(root, "art", "blacksite-manifest.json");
  let manifest;
  try {
    manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  } catch (err) {
    return { ok: false, errors: [`cannot read manifest: ${err.message}`], warnings: [], checked: 0 };
  }
  const m = validateManifest(manifest);
  const f = validateReadyFiles(manifest, join(root, "public", "game"), join(root, "art", "atlases"));
  return {
    ok: m.errors.length === 0 && f.errors.length === 0,
    errors: [...m.errors, ...f.errors],
    warnings: [...m.warnings, ...f.warnings],
    checked: f.checked,
  };
}

const isMain = process.argv[1] === fileURLToPath(import.meta.url);
if (isMain) {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const result = checkAssets(root);
  for (const w of result.warnings) console.warn(`[check:assets] warn: ${w}`);
  if (!result.ok) {
    for (const e of result.errors) console.error(`[check:assets] ${e}`);
    console.error(`[check:assets] ${result.errors.length} error(s). See art/blacksite-manifest.json + docs/blacksite-ifrit/.`);
    process.exit(1);
  }
  console.log(`[check:assets] manifest ok (${result.checked} delivered file(s) checked).`);
}
