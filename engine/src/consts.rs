//! Shared engine constants: map, textures, entities, input, weapons.

pub(crate) const MAP_W: usize = 48;
pub(crate) const MAP_H: usize = 32;
pub(crate) const MAX_W: usize = 3840;
pub(crate) const MAX_H: usize = 2160;
pub(crate) const TEX: usize = 256;
pub(crate) const TEXM: i32 = (TEX as i32) - 1;
/// Atlas layers: the original 29 world layers plus seven animation sheets
/// for each of the thirteen BLACKSITE enemy skins.
pub(crate) const ENEMY_ANIM_COUNT: usize = 7;
pub(crate) const ENEMY_SKIN_COUNT: usize = 15;
pub(crate) const ENEMY_TEX_BASE: usize = 29;
pub(crate) const T_ORDNANCE: usize = ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * ENEMY_SKIN_COUNT;
pub(crate) const T_GUN3: usize = T_ORDNANCE + 1;
pub(crate) const T_GUN4: usize = T_ORDNANCE + 2;
pub(crate) const T_GUN5: usize = T_ORDNANCE + 3;
pub(crate) const T_GUN6: usize = T_ORDNANCE + 4;
pub(crate) const T_GUN7: usize = T_ORDNANCE + 5;
pub(crate) const T_CONSOLE_UPPER: usize = T_GUN7 + 1;
pub(crate) const T_CONSOLE_FOUNDRY: usize = T_GUN7 + 2;
pub(crate) const T_CONSOLE_BIOFORGE: usize = T_GUN7 + 3;
pub(crate) const T_GUN8: usize = T_CONSOLE_BIOFORGE + 1;
pub(crate) const T_GUN9: usize = T_GUN8 + 1;
pub(crate) const T_GUN10: usize = T_GUN8 + 2;
pub(crate) const T_GUN11: usize = T_GUN8 + 3;
pub(crate) const TEX_N: usize = T_GUN11 + 1;
pub(crate) const ENT_N: usize = 192;
pub(crate) const T_BRICK: usize = 0;
pub(crate) const T_METAL: usize = 1;
pub(crate) const T_FLESH: usize = 2;
pub(crate) const T_SKULL: usize = 3;
pub(crate) const T_DOOR: usize = 4;
pub(crate) const T_GRATE: usize = 5;
pub(crate) const T_CONC: usize = 6;
pub(crate) const T_CEIL: usize = 7;
pub(crate) const T_HUSK: usize = 8;
pub(crate) const T_BRUTE: usize = 9;
pub(crate) const T_WRAITH: usize = 10;
pub(crate) const T_MED: usize = 11;
pub(crate) const T_AMMO: usize = 12;
pub(crate) const T_ARMOR: usize = 13;
pub(crate) const T_BARREL: usize = 14;
pub(crate) const T_BALL: usize = 15;
pub(crate) const T_SPLAT: usize = 16;
pub(crate) const T_SECRET: usize = 17;
pub(crate) const T_TECH: usize = 18;
pub(crate) const T_HAZARD: usize = 19;
pub(crate) const T_LAMP: usize = 20;
pub(crate) const T_CRATE: usize = 21;
pub(crate) const T_IMPACT: usize = 22;
pub(crate) const T_MUZZLEFX: usize = 23;
pub(crate) const T_FLAME: usize = 24;
pub(crate) const T_CHAIN: usize = 25;
pub(crate) const T_PIPES: usize = 26;
pub(crate) const T_GUN2: usize = 27;
/// Sector override floor beacon. This replaced the occult boss seal: the
/// player now reaches a physical control station and deliberately uses it.
pub(crate) const T_OVERRIDE: usize = 28;

pub(crate) const EK_NONE: u8 = 0;
pub(crate) const EK_HUSK: u8 = 1;
pub(crate) const EK_BRUTE: u8 = 2;
pub(crate) const EK_WRAITH: u8 = 3;
pub(crate) const EK_PROJ: u8 = 4;
pub(crate) const EK_MED: u8 = 5;
pub(crate) const EK_AMMO: u8 = 6;
pub(crate) const EK_ARMOR: u8 = 7;
pub(crate) const EK_GUN2: u8 = 8;
pub(crate) const EK_GUN3: u8 = 9;
pub(crate) const EK_GUN4: u8 = 19;
pub(crate) const EK_GUN5: u8 = 20;
pub(crate) const EK_BARREL: u8 = 10;
pub(crate) const EK_GIB: u8 = 11;
pub(crate) const EK_LAMP: u8 = 12;
pub(crate) const EK_CRATE: u8 = 13;
pub(crate) const EK_CHAIN: u8 = 14;
pub(crate) const EK_IMPACT: u8 = 15;
pub(crate) const EK_SPARK: u8 = 16;
pub(crate) const EK_SMOKE: u8 = 17;
pub(crate) const EK_FLAME: u8 = 18;
pub(crate) const EK_RAY: u8 = 21;
pub(crate) const EK_BOLT: u8 = 22;
pub(crate) const EK_BOSS: u8 = 23;
pub(crate) const EK_FIREPATCH: u8 = 24;
/// Suicide-chaser drone. Floats at the player, runs a visible detonation
/// windup (`ANIM_SPECIAL`), then explodes instead of dealing melee damage.
pub(crate) const EK_MARTYR: u8 = 25;
pub(crate) const EK_GUN6: u8 = 26;
pub(crate) const EK_GUN7: u8 = 27;
pub(crate) const EK_OVERRIDE_CONSOLE: u8 = 28;
pub(crate) const EK_GUN8: u8 = 29;
pub(crate) const EK_NODE: u8 = 30;
pub(crate) const EK_TERMINAL: u8 = 31;
pub(crate) const EK_GUN9: u8 = 32;
pub(crate) const EK_GUN10: u8 = 33;
pub(crate) const EK_GUN11: u8 = 34;
pub(crate) const EK_POWER: u8 = 35;
pub(crate) const SKIN_CONSOLE_UPPER: u8 = 240;
pub(crate) const SKIN_CONSOLE_FOUNDRY: u8 = 241;
pub(crate) const SKIN_CONSOLE_BIOFORGE: u8 = 242;

// BLACKSITE animation groups. Each group is one 2x2 atlas layer with four
// distinct motion frames.
pub(crate) const ANIM_IDLE: u8 = 0;
pub(crate) const ANIM_MOVE: u8 = 1;
pub(crate) const ANIM_PAIN: u8 = 2;
pub(crate) const ANIM_FIRE: u8 = 3;
pub(crate) const ANIM_RELOAD: u8 = 4;
pub(crate) const ANIM_DEAD: u8 = 5;
pub(crate) const ANIM_SPECIAL: u8 = 6;
pub(crate) const ANIM_FRAME_COUNTS: [u8; ENEMY_ANIM_COUNT] = [4, 4, 4, 4, 4, 4, 4];
pub(crate) const SKIN_NONE: u8 = 255;

pub(crate) const SKIN_RIFLEMAN: u8 = 0;
pub(crate) const SKIN_BREACHER: u8 = 1;
pub(crate) const SKIN_SUBJECT: u8 = 2;
pub(crate) const SKIN_HAZMAT: u8 = 3;
pub(crate) const SKIN_GUNNER: u8 = 4;
pub(crate) const SKIN_LOADER: u8 = 5;
pub(crate) const SKIN_VATBRUTE: u8 = 6;
pub(crate) const SKIN_MARKSMAN: u8 = 7;
pub(crate) const SKIN_HORNET: u8 = 8;
pub(crate) const SKIN_HOUND: u8 = 9;
pub(crate) const SKIN_SPITTER: u8 = 10;
pub(crate) const SKIN_MARTYR: u8 = 11;
pub(crate) const SKIN_VEYRAN: u8 = 12;
pub(crate) const SKIN_HECATE: u8 = 13;
pub(crate) const SKIN_CHIMERA: u8 = 14;

pub(crate) const IN_W: u32 = 1;
pub(crate) const IN_S: u32 = 2;
pub(crate) const IN_A: u32 = 4;
pub(crate) const IN_D: u32 = 8;
pub(crate) const IN_FIRE: u32 = 16;
pub(crate) const IN_SPRINT: u32 = 32;
pub(crate) const IN_USE: u32 = 64;
pub(crate) const IN_W1: u32 = 128;
pub(crate) const IN_W2: u32 = 256;
pub(crate) const IN_W3: u32 = 512;
pub(crate) const IN_TURNL: u32 = 1024;
pub(crate) const IN_TURNR: u32 = 2048;
pub(crate) const IN_RELOAD: u32 = 4096;
pub(crate) const IN_W4: u32 = 8192;
pub(crate) const IN_W5: u32 = 16384;
pub(crate) const IN_W6: u32 = 32768;
pub(crate) const IN_W7: u32 = 65536;
pub(crate) const IN_W8: u32 = 131072;
pub(crate) const IN_W9: u32 = 262144;
pub(crate) const IN_W10: u32 = 524288;
pub(crate) const IN_W11: u32 = 1048576;

pub(crate) const WEP_N: usize = 11;
pub(crate) const MAG_SZ: [i32; WEP_N] = [12, 8, 36, 5, 4, 10, 90, 6, 4, 14, 5];
pub(crate) const RELOAD_T: [f32; WEP_N] = [0.95, 1.75, 1.30, 1.60, 1.95, 1.55, 2.45, 1.85, 1.70, 1.35, 1.55];
pub(crate) const MAP_CELLS: usize = MAP_W * MAP_H;
pub(crate) const FX_CAP: usize = 64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_derived_sizes_stay_consistent() {
        assert_eq!(MAP_CELLS, MAP_W * MAP_H);
        assert_eq!(MAP_CELLS, 48 * 32);
    }

    #[test]
    fn weapon_tables_cover_all_eleven_guns() {
        assert_eq!(MAG_SZ.len(), WEP_N);
        assert_eq!(RELOAD_T.len(), WEP_N);
        for m in MAG_SZ {
            assert!(m > 0);
        }
        for t in RELOAD_T {
            assert!(t > 0.0);
        }
    }

    #[test]
    fn input_bits_are_unique_powers_of_two() {
        let bits = [
            IN_W, IN_S, IN_A, IN_D, IN_FIRE, IN_SPRINT, IN_USE, IN_W1, IN_W2, IN_W3, IN_TURNL,
            IN_TURNR, IN_RELOAD, IN_W4, IN_W5, IN_W6, IN_W7, IN_W8, IN_W9, IN_W10, IN_W11,
        ];
        for (i, a) in bits.iter().enumerate() {
            assert_ne!(*a, 0);
            assert_eq!(*a & (a - 1), 0, "input bit {i} is not a power of two");
            for b in &bits[i + 1..] {
                assert_eq!(a & b, 0, "input bits overlap");
            }
        }
    }

    #[test]
    fn texture_slots_cover_the_known_atlas() {
        assert_eq!(
            TEX_N,
            ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * ENEMY_SKIN_COUNT + 13
        );
        assert_eq!(TEX, 256);
        assert_eq!(TEXM, 255);
        assert_eq!(T_OVERRIDE, 28);
    }
}
