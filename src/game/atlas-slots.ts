/**
 * GPU atlas slot layout — the single source of truth for texture-array indices.
 *
 * Kept dependency-free (no JSON imports, no browser globals) so that asset
 * checks can `import` it directly under `node --experimental-strip-types`
 * instead of duplicating the arithmetic. `gpu-world.ts` re-exports every name.
 */

export const VIEW_FLOATS = 16;
export const COL_FLOATS = 16;
export const SPR_FLOATS = 8;
export const MAP_W = 48;
export const MAP_H = 32;
export const ENEMY_ANIM_COUNT = 7;
export const ENEMY_SKIN_COUNT = 62;
export const ENEMY_PROJECTILE_COUNT = 37;
export const ENEMY_TEX_BASE = 29;
export const SECTOR_COUNT = 25;
export const EXPANSION_CASE_COUNT = 14;
export const T_ORDNANCE = ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * ENEMY_SKIN_COUNT;
export const T_GUN3 = T_ORDNANCE + 1;
export const T_GUN4 = T_ORDNANCE + 2;
export const T_GUN5 = T_ORDNANCE + 3;
export const T_GUN6 = T_ORDNANCE + 4;
export const T_GUN7 = T_ORDNANCE + 5;
export const T_GUN8 = T_GUN7 + 4;
export const T_GUN9 = T_GUN8 + 1;
export const T_GUN10 = T_GUN8 + 2;
export const T_TECH = 18;
export const T_DOOR = 4;
export const T_GUN11 = T_GUN8 + 3;
export const T_GUN12 = T_GUN11 + 1;
export const T_GUN13 = T_GUN12 + 1;
export const T_GUN14 = T_GUN13 + 1;
export const T_GUN15 = T_GUN14 + 1;
export const T_GUN16 = T_GUN15 + 1;
export const T_GUN17 = T_GUN16 + 1;
export const T_GUN18 = T_GUN17 + 1;
export const T_GUN19 = T_GUN18 + 1;
export const T_PROP_REACTOR = T_GUN19 + 1;
export const T_PROP_SERVER = T_GUN19 + 2;
export const T_PROP_AC = T_GUN19 + 3;
export const T_PROP_VENT = T_GUN19 + 4;
export const T_PROP_WLIGHT_C = T_GUN19 + 5;
export const T_PROP_WLIGHT_W = T_GUN19 + 6;
export const T_PROP_BEACON = T_GUN19 + 7;
export const T_EXPANSION_CASE = T_PROP_BEACON + 1;
export const T_PROJECTILE_NEW = T_EXPANSION_CASE + EXPANSION_CASE_COUNT;
export const T_IMPACT_NEW = T_PROJECTILE_NEW + 5;
export const T_ENEMY_PROJECTILE = T_IMPACT_NEW + 4;
export const T_PLAYER_MISSILE = T_ENEMY_PROJECTILE + ENEMY_PROJECTILE_COUNT;
export const T_SECTOR_SURFACE = T_PLAYER_MISSILE + 3;
export const T_SECTOR_PROP = T_SECTOR_SURFACE + SECTOR_COUNT * 5;
export const T_BOSS_PROJECTILE = T_SECTOR_PROP + SECTOR_COUNT * 3;
export const T_BOSS_ARENA = T_BOSS_PROJECTILE + SECTOR_COUNT;
export const T_CASING = T_BOSS_ARENA + SECTOR_COUNT;
export const T_CASING_SHELL = T_CASING + 1;
export const T_BOSS_ENTRANCE = T_CASING_SHELL + 1;
export const TEX_N = T_BOSS_ENTRANCE + 8;