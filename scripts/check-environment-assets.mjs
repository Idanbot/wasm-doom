#!/usr/bin/env node
/** Validate the generated 1024px environment masters and 256px runtime cells. */

import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { readPngInfo } from "./check-assets.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const planPath = join(root, "art", "blacksite-environment-generation-plan.json");
const reportPath = join(root, "art", "source_hd", "environment", "processing-report.json");
const errors = [];

function readPlan() {
  try {
    return JSON.parse(readFileSync(planPath, "utf8"));
  } catch (error) {
    errors.push(`cannot read environment plan: ${error.message}`);
    return null;
  }
}

function checkPng(relative, size, alphaRequired = false) {
  const full = join(root, relative);
  if (!existsSync(full)) {
    errors.push(`missing ${relative}`);
    return;
  }
  try {
    const info = readPngInfo(readFileSync(full));
    if (info.width !== size || info.height !== size) errors.push(`${relative}: expected ${size}x${size}, got ${info.width}x${info.height}`);
    if (alphaRequired && !info.hasAlpha) errors.push(`${relative}: expected an alpha channel`);
  } catch (error) {
    errors.push(`${relative}: ${error.message}`);
  }
}

const plan = readPlan();
if (plan) {
  if (plan.sourceSize !== 1024 || plan.runtimeTileSize !== 256) errors.push("environment plan size contract must be 1024 source / 256 runtime");
  if (!Array.isArray(plan.objectFamilies) || plan.objectFamilies.length < 12) errors.push("environment plan needs at least 12 object families");
  for (const theme of plan.themes ?? []) {
    if (!Array.isArray(theme.objects) || theme.objects.length <= 11) errors.push(`${theme.id}: needs more than 11 environmental objects`);
    checkPng(theme.texture?.source, 1024);
    checkPng(theme.texture?.runtime, 256, true);
    checkPng(theme.objectSheet, 1024, true);
    for (const objectId of theme.objects ?? []) {
      checkPng(`${theme.runtimeObjectPrefix}${objectId}.png`, 256, true);
    }
  }
  for (const sheet of plan.itemsAndCollectibles?.sheets ?? []) {
    checkPng(sheet.source, 1024, true);
    // Cell names are deterministic and are checked against the four source
    // sheet groups instead of duplicating the full catalog in this gate.
    const names = {
      combat: ["pistol_magazine", "shotgun_shell_bundle", "rifle_magazine", "energy_cell", "frag_grenade", "incendiary_charge", "proximity_mine", "explosive_charge", "armor_plate", "trauma_injector", "compact_medkit", "stimulant_ampoule", "repair_tool", "welding_torch", "ration_pack", "flare"],
      security: ["security_keycard", "access_badge", "encrypted_data_drive", "map_tablet", "master_access_token", "biometric_chip", "evidence_cassette", "blacksite_logbook", "radio_handset", "signal_beacon", "camera_module", "cipher_coin", "evidence_pouch", "drone_core", "command_seal", "data_crystal"],
      science: ["janus_specimen_vial", "bio_sample_canister", "coolant_cartridge", "radiation_filter", "reactor_fuel_cell", "isotope_capsule", "green_medgel_syringe", "culture_tube", "pressure_gauge", "coolant_valve", "hazard_dosimeter", "lead_sample_case", "frozen_tissue_sample", "microchip_tray", "cryo_tag", "emergency_oxygen"],
      occult: ["bone_seal_shard", "iron_reliquary", "crimson_sigil_token", "skull_medallion", "chapel_key", "blood_glass_vial", "prayer_strip", "demon_horn_fragment", "ivory_talisman", "chained_heart_relic", "obsidian_coin", "candle_bundle", "vault_crown_fragment", "bone_die", "red_crystal", "sealed_skull_capsule"],
    }[sheet.id] ?? [];
    for (const name of names) checkPng(`${sheet.runtimePrefix}${name}.png`, 256, true);
  }
  if (existsSync(reportPath)) {
    const report = JSON.parse(readFileSync(reportPath, "utf8"));
    const maxDelta = Math.max(...Object.values(report.textures ?? {}).map((entry) => entry.maxWrapEdgeDelta ?? 999));
    if (maxDelta !== 0) errors.push(`texture wrap report has a non-zero edge delta (${maxDelta})`);
  } else {
    errors.push("missing texture processing report");
  }
}

function checkDraftV2() {
  const ids = ["ax12", "br12", "cm9", "hc9", "hx8", "kx9", "m91", "mk23s", "mr4", "vlk6", "vr9"];
  const themes = ["hangar", "plaza", "security", "datacenter", "foundry", "biotech", "nuclear", "vault"];
  const dim = (relative, w, h, alpha = false) => {
    const full = join(root, relative);
    if (!existsSync(full)) {
      errors.push(`missing ${relative}`);
      return;
    }
    try {
      const info = readPngInfo(readFileSync(full));
      if (info.width !== w || info.height !== h) errors.push(`${relative}: expected ${w}x${h}, got ${info.width}x${info.height}`);
      if (alpha && !info.hasAlpha) errors.push(`${relative}: expected an alpha channel`);
    } catch (error) {
      errors.push(`${relative}: ${error.message}`);
    }
  };
  for (const id of ids) {
    dim(`public/game/draft/v2/weap_${id}_5x5.png`, 2560, 1920, true);
    dim(`public/game/draft/v2/weap_${id}_aim.png`, 512, 384, true);
    dim(`public/game/draft/v2/cases/case_${id}.png`, 1024, 768, true);
  }
  for (const name of ["ac_power_unit", "ammo_cache", "beacon_warning", "lantern_amber", "lantern_red", "medkit", "reactor_unit", "server_rack", "ventilation_array", "worklight_cyan", "worklight_white"]) {
    dim(`public/game/draft/v2/items/${name}.png`, 768, 1024, true);
  }
  for (const theme of themes) {
    dim(`public/game/theme/wall_${theme}.png`, 256, 256);
    dim(`public/game/theme/door_${theme}.png`, 256, 256);
  }
  for (const name of ["spr_med.png", "spr_ammo.png", "spr_lamp.png", "spr_prop_reactor.png", "spr_prop_server.png", "spr_prop_ac.png", "spr_prop_vent.png", "spr_prop_worklight_cyan.png", "spr_prop_worklight_white.png", "spr_prop_beacon.png"]) {
    dim(`public/game/${name}`, 256, 256, true);
  }
  for (const name of ["wall_brick.png", "wall_metal.png", "floor_grate.png", "floor_concrete.png", "wall_hazard_tile2x2.png", "ceil_pipes.png"]) {
    // Opaque world surfaces: no alpha channel by design.
    dim(`public/game/${name}`, 256, 256, false);
  }
}
checkDraftV2();

if (errors.length) {
  for (const error of errors) console.error(`[check:environment] ${error}`);
  process.exit(1);
}

console.log(`[check:environment] ${plan.themes.length} themes, ${plan.objectFamilies.length} object families, 128 theme object cells, and 64 collectible cells are valid.`);
