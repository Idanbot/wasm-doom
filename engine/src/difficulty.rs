//! Fixed run presets. Authored health, ammo and readable windups stay unchanged.
pub(crate) struct Preset { pub damage: f32, pub speed: f32, pub recovery: f32 }
pub(crate) const PRESETS: [Preset; 4] = [
    Preset { damage: 0.65, speed: 0.85, recovery: 1.25 },
    Preset { damage: 1.0, speed: 1.0, recovery: 1.0 },
    Preset { damage: 1.25, speed: 1.10, recovery: 0.90 },
    Preset { damage: 1.50, speed: 1.20, recovery: 0.80 },
];

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::testutil::arena;

    #[test]
    fn presets_change_real_damage_and_preserve_the_original_balance() {
        for (preset, expected) in [13, 20, 25, 30].into_iter().enumerate() {
            let mut e = arena(); e.difficulty = preset; e.iframes = 0.0;
            e.damage_player(20);
            assert_eq!(e.health, 100 - expected);
            assert_eq!(e.run_damage, expected as u32);
        }
        assert_eq!(Engine::new(160, 100).difficulty, 1);
    }

    #[test]
    fn damage_totals_do_not_count_invulnerability_or_health_overkill() {
        let mut e = arena(); e.iframes = 0.0; e.health = 5;
        e.damage_player(100); assert_eq!(e.run_damage, 5);
        e.damage_player(100); assert_eq!(e.run_damage, 5);
    }

    #[test]
    fn one_shot_counts_once_across_pellets_targets_and_shield_hits() {
        let mut e = arena();
        let a = e.spawn(EK_HUSK, 7.0, 4.5).unwrap();
        let b = e.spawn(EK_HUSK, 8.0, 4.5).unwrap();
        e.run_shots.push(false); e.current_shot = 1;
        e.hurt_ent(a, 1, 4.5, 4.5); e.hurt_ent(a, 1, 4.5, 4.5); e.hurt_ent(b, 1, 4.5, 4.5);
        assert_eq!(e.run_hits, 1);
        e.current_shot = 0; e.hurt_ent(b, 1, 4.5, 4.5); assert_eq!(e.run_hits, 1);
    }

    #[test]
    fn empty_fire_does_not_count_and_a_fired_missile_keeps_its_shot_id() {
        let mut e = arena(); e.mag[0] = 0; e.fire();
        assert!(e.run_shots.is_empty()); assert_eq!(e.run_ammo, 0);
        e.cooldown = 0.0; e.weapon = 4; e.mag[4] = 2; e.fire();
        assert_eq!(e.run_shots.len(), 1); assert_eq!(e.run_ammo, 1);
        assert_eq!(e.current_shot, 0);
        let missile = e.ents.iter().find(|ent| ent.kind == EK_BOLT).unwrap();
        assert_eq!(missile.shot_id, 1);
        e.next_wave(); assert_eq!(e.run_shots.len(), 1); assert_eq!(e.difficulty, 1);
    }
}
