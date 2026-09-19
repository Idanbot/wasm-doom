//! Audio/event bit flags shared between the sim and the TS audio layer.

pub(crate) const EV_FIRE: u32 = 1;
pub(crate) const EV_EMPTY: u32 = 2;
pub(crate) const EV_RELOAD: u32 = 4;
pub(crate) const EV_HIT: u32 = 8;
pub(crate) const EV_HURT: u32 = 16;
pub(crate) const EV_PICK_SILVER: u32 = 32;
pub(crate) const EV_PICK_GOLD: u32 = 64;
pub(crate) const EV_DIE: u32 = 128;
pub(crate) const EV_EXPLODE: u32 = 256;
pub(crate) const EV_FOOT: u32 = 512;
pub(crate) const EV_DOOR: u32 = 1024;
pub(crate) const EV_BOSS: u32 = 2048;
pub(crate) const EV_KILL: u32 = 4096;
pub(crate) const EV_BOSS_HUSH: u32 = 8192;
pub(crate) const EV_BOSS_DROP: u32 = 16384;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_flags_are_unique_powers_of_two() {
        let flags = [
            EV_FIRE, EV_EMPTY, EV_RELOAD, EV_HIT, EV_HURT, EV_PICK_SILVER, EV_PICK_GOLD,
            EV_DIE, EV_EXPLODE, EV_FOOT, EV_DOOR, EV_BOSS, EV_KILL, EV_BOSS_HUSH,
            EV_BOSS_DROP,
        ];
        assert_eq!(flags.len(), 15);
        for (i, a) in flags.iter().enumerate() {
            assert_ne!(*a, 0);
            assert_eq!(*a & (a - 1), 0, "event flag {i} is not a power of two");
            for b in &flags[i + 1..] {
                assert_eq!(a & b, 0, "event flags overlap");
            }
        }
    }
}
