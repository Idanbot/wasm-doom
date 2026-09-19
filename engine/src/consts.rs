//! Shared engine constants: map, textures, entities, input, weapons.

pub(crate) const MAP_W: usize = 48;
pub(crate) const MAP_H: usize = 32;
pub(crate) const MAX_W: usize = 1920;
pub(crate) const MAX_H: usize = 1200;
pub(crate) const TEX: usize = 256;
pub(crate) const TEXM: i32 = (TEX as i32) - 1;
pub(crate) const TEX_N: usize = 29;
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
pub(crate) const T_GUN: usize = 27;
pub(crate) const T_SEAL: usize = 28;
pub(crate) const SEAL_X: i32 = 38;
pub(crate) const SEAL_Y: i32 = 19;
pub(crate) const SEAL_W: i32 = 8;
pub(crate) const SEAL_H: i32 = 9;

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

/// Opening hostile cast for the hub-and-spoke map, west to east then
/// south: hangar duo, plaza pair, lab guards, chapel line, vault
/// honor guard, pit pair. Every coordinate must be an open cell —
/// `map::tests::hub_spoke_zones_are_all_connected` enforces reachability
/// and `opening_cast_matches_the_zone_playlist` pins the count.
pub(crate) const HOSTILES: [(u8, f32, f32); 14] = [
    (EK_HUSK, 7.5, 14.5),
    (EK_HUSK, 14.5, 15.5),
    (EK_HUSK, 20.5, 13.5),
    (EK_BRUTE, 25.5, 17.5),
    (EK_HUSK, 22.5, 9.5),
    (EK_BRUTE, 26.5, 3.5),
    (EK_WRAITH, 19.5, 3.5),
    (EK_HUSK, 32.5, 15.5),
    (EK_BRUTE, 37.5, 13.5),
    (EK_WRAITH, 36.5, 21.5),
    (EK_BRUTE, 41.5, 25.5),
    (EK_WRAITH, 38.5, 26.5),
    (EK_WRAITH, 22.5, 26.5),
    (EK_BRUTE, 25.5, 25.5),
];
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

pub(crate) const MAG_SZ: [i32; 5] = [12, 6, 32, 4, 40];
pub(crate) const RELOAD_T: [f32; 5] = [0.95, 1.55, 1.35, 1.45, 1.8];
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
    fn weapon_tables_cover_all_five_guns() {
        assert_eq!(MAG_SZ.len(), 5);
        assert_eq!(RELOAD_T.len(), 5);
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
            IN_W, IN_S, IN_A, IN_D, IN_FIRE, IN_SPRINT, IN_USE, IN_W1, IN_W2, IN_W3,
            IN_TURNL, IN_TURNR, IN_RELOAD, IN_W4, IN_W5,
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
        assert_eq!(TEX_N, 29);
        assert_eq!(TEX, 256);
        assert_eq!(TEXM, 255);
        assert_eq!(T_SEAL, TEX_N - 1);
    }
}
