#!/usr/bin/env node
/** Validate the projectile/effect source master and runtime sheet contract. */

import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { readPngInfo } from "./check-assets.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const errors = [];

function checkPng(relative, width, height, alpha = true) {
  const full = join(root, relative);
  if (!existsSync(full)) {
    errors.push(`missing ${relative}`);
    return;
  }
  try {
    const info = readPngInfo(readFileSync(full));
    if (info.width !== width || info.height !== height) errors.push(`${relative}: expected ${width}x${height}, got ${info.width}x${info.height}`);
    if (alpha && !info.hasAlpha) errors.push(`${relative}: expected an alpha channel`);
  } catch (error) {
    errors.push(`${relative}: ${error.message}`);
  }
}

checkPng("art/source_hd/projectiles/projectile_effects_4x4.png", 1024, 1024);
checkPng("public/game/spr_ball.png", 256, 256);
for (const filename of ["spr_muzzle.png", "spr_impact.png", "spr_flame.png"]) {
  checkPng(`public/game/${filename}`, 512, 512);
}
for (const name of [
  "plasma_bolt", "incendiary_projectile", "lance_beam", "acid_seeker",
  "pistol_muzzle", "shotgun_muzzle", "smg_muzzle", "rifle_muzzle",
  "metal_impact", "plasma_impact", "explosive_impact", "acid_impact",
  "fire_patch", "smoke_puff", "electric_sparks", "casing_burst",
]) checkPng(`art/source_hd/projectiles/cells/${name}.png`, 256, 256);

if (!existsSync(join(root, "art/source_hd/projectiles/processing-report.json"))) {
  errors.push("missing projectile processing report");
}

if (errors.length) {
  for (const error of errors) console.error(`[check:projectiles] ${error}`);
  process.exit(1);
}
console.log("[check:projectiles] 1024px master, 16 cells, and four runtime effect slots are valid.");
