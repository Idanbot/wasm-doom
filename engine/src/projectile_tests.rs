//! Swept collision coverage through real simulation ticks, without rendering.
use crate::*;

fn target(e: &mut Engine, x: f32, y: f32) -> usize {
    let i = e.spawn(EK_HUSK, x, y).unwrap();
    e.ents[i].hp = 2000;
    e.ents[i].armor_hp = 0;
    e.ents[i].shield_hp = 0;
    e.ents[i].stun = 100.0;
    i
}

fn projectile(e: &mut Engine, kind: u8, x: f32, vx: f32, friendly: bool) {
    let i = e.spawn(kind, x, 4.5).unwrap();
    e.ents[i].vx = vx;
    e.ents[i].timer = 2.0;
    e.ents[i].hp = if kind == EK_BOLT { 1 } else { 40 };
    e.ents[i].effect_tick = if friendly && kind == EK_PROJ { 5.0 } else { 0.0 };
}

#[test]
fn friendly_projectiles_hit_the_nearest_contact_regardless_of_entity_order() {
    for reverse in [false, true] {
        let mut e = testutil::arena();
        let (near, far) = if reverse {
            let near = target(&mut e, 8.0, 4.5);
            (near, target(&mut e, 11.0, 4.5))
        } else {
            let far = target(&mut e, 11.0, 4.5);
            (target(&mut e, 8.0, 4.5), far)
        };
        projectile(&mut e, EK_PROJ, 5.5, 100.0, true);
        e.tick(0.08);
        assert_eq!(e.ents[near].hp, 1960, "near target must intercept the shot");
        assert_eq!(e.ents[far].hp, 2000, "shot must not pass through the near target");
    }
}

#[test]
fn contact_uses_body_entry_instead_of_center_distance() {
    let mut e = testutil::arena();
    let small = target(&mut e, 8.0, 4.5);
    let large = target(&mut e, 8.3, 4.5);
    e.ents[small].radius = 0.1;
    e.ents[large].radius = 0.8;
    projectile(&mut e, EK_PROJ, 5.5, 100.0, true);
    e.tick(0.08);
    assert_eq!(e.ents[large].hp, 1960);
    assert_eq!(e.ents[small].hp, 2000);
}

#[test]
fn a_wall_later_in_the_tick_does_not_discard_a_nearer_target() {
    let mut e = testutil::arena();
    let near = target(&mut e, 8.0, 4.5);
    e.set_cell(10, 4, 1);
    projectile(&mut e, EK_PROJ, 5.5, 100.0, true);
    e.tick(0.08);
    assert_eq!(e.ents[near].hp, 1960);
}

#[test]
fn enemy_projectile_hits_the_player_before_a_later_wall() {
    let mut e = testutil::arena();
    e.px = 8.5;
    e.armor = 0;
    e.set_cell(6, 4, 1);
    projectile(&mut e, EK_PROJ, 12.5, -100.0, false);
    let before = e.health;
    e.tick(0.08);
    assert!(e.health < before, "a wall behind the player cannot shield them");
}

#[test]
fn walls_stop_friendly_and_enemy_projectiles_before_targets() {
    for friendly in [false, true] {
        let mut e = testutil::arena();
        e.px = 11.5;
        e.armor = 0;
        let behind = target(&mut e, 11.0, 4.5);
        e.set_cell(8, 4, 1);
        projectile(&mut e, EK_PROJ, 5.5, 100.0, friendly);
        let health = e.health;
        e.tick(0.08);
        assert_eq!(e.ents[behind].hp, 2000);
        assert_eq!(e.health, health);
        assert!(!e.ents.iter().any(|ent| ent.kind == EK_PROJ));
    }
}

#[test]
fn rockets_detonate_at_the_nearest_body_instead_of_the_frame_endpoint() {
    for reverse in [false, true] {
        let mut e = testutil::arena();
        let (near, far) = if reverse {
            let near = target(&mut e, 8.0, 4.5);
            (near, target(&mut e, 11.0, 4.5))
        } else {
            let far = target(&mut e, 11.0, 4.5);
            (target(&mut e, 8.0, 4.5), far)
        };
        projectile(&mut e, EK_BOLT, 5.5, 100.0, true);
        e.tick(0.08);
        assert!(e.ents[near].hp < e.ents[far].hp, "near target must receive direct hit and stronger blast");
        let blast = e.ents.iter().find(|ent| ent.kind == EK_IMPACT && ent.effect_tick == 2.0).unwrap();
        assert!((7.0..8.0).contains(&blast.x), "blast belongs at body contact, got {}", blast.x);
    }
}

#[test]
fn rocket_wall_blast_is_at_the_wall_instead_of_its_previous_position() {
    let mut e = testutil::arena();
    e.set_cell(10, 4, 1);
    projectile(&mut e, EK_BOLT, 5.5, 100.0, true);
    e.tick(0.08);
    let blast = e.ents.iter().find(|ent| ent.kind == EK_IMPACT && ent.effect_tick == 2.0).unwrap();
    assert!((blast.x - 10.0).abs() < 0.01, "blast must reach the wall, got {}", blast.x);
    assert!(!e.blocked(blast.x.floor() as i32, blast.y.floor() as i32), "wall blasts originate just outside solid geometry");
}

#[test]
fn rockets_hit_a_body_before_a_wall_and_do_not_blast_through_it() {
    let mut e = testutil::arena();
    let near = target(&mut e, 8.0, 4.5);
    let behind = target(&mut e, 11.0, 4.5);
    e.set_cell(10, 4, 1);
    projectile(&mut e, EK_BOLT, 5.5, 100.0, true);
    e.tick(0.08);
    assert!(e.ents[near].hp < 1976, "near body must receive direct damage plus splash");
    assert_eq!(e.ents[behind].hp, 2000, "wall must still block splash");
}

#[test]
fn equidistant_contacts_break_ties_deterministically_and_misses_keep_flying() {
    let mut e = testutil::arena();
    let first = target(&mut e, 8.0, 4.5);
    let second = target(&mut e, 8.0, 4.5);
    projectile(&mut e, EK_PROJ, 5.5, 100.0, true);
    e.tick(0.08);
    assert_eq!(e.ents[first].hp, 1960);
    assert_eq!(e.ents[second].hp, 2000);

    projectile(&mut e, EK_PROJ, 5.5, 100.0, true);
    let missile = e.ents.iter_mut().find(|ent| ent.kind == EK_PROJ).unwrap();
    missile.y = 6.5;
    e.tick(0.08);
    let missile = e.ents.iter().find(|ent| ent.kind == EK_PROJ).expect("an unobstructed miss stays alive");
    assert_eq!((missile.x, missile.y), (13.5, 6.5));
    assert!(missile.timer > 0.0);
}

#[test]
fn stationary_projectiles_only_hit_overlapping_bodies_and_expire_normally() {
    let mut e = testutil::arena();
    let body = target(&mut e, 8.0, 4.5);
    projectile(&mut e, EK_PROJ, 8.0, 0.0, true);
    e.tick(0.08);
    assert_eq!(e.ents[body].hp, 1960);

    projectile(&mut e, EK_PROJ, 5.5, 0.0, true);
    let missile = e.ents.iter_mut().find(|ent| ent.kind == EK_PROJ).unwrap();
    missile.timer = 0.01;
    e.tick(0.08);
    assert!(!e.ents.iter().any(|ent| ent.kind == EK_PROJ));
    assert_eq!(e.ents[body].hp, 1960);
}

#[test]
fn friendly_rounds_and_rockets_respect_both_door_types() {
    for door_kind in [8, 9] {
        for open in [false, true] {
            for kind in [EK_PROJ, EK_BOLT] {
                let mut e = testutil::arena();
                let behind = target(&mut e, 11.0, 4.5);
                e.set_cell(8, 4, door_kind);
                e.door[4 * MAP_W + 8] = if open { 1.0 } else { 0.0 };
                projectile(&mut e, kind, 5.5, 100.0, true);
                e.tick(0.08);
                assert_eq!(e.ents[behind].hp < 2000, open,
                    "door {door_kind}, open={open}, projectile {kind}");
                assert!(!e.ents.iter().any(|ent| ent.kind == kind));
            }
        }
    }
}

#[test]
fn enemy_rounds_cannot_damage_the_player_through_closed_doors() {
    for door_kind in [8, 9] {
        for open in [false, true] {
            let mut e = testutil::arena();
            e.px = 12.5;
            e.armor = 0;
            e.set_cell(8, 4, door_kind);
            e.door[4 * MAP_W + 8] = if open { 1.0 } else { 0.0 };
            projectile(&mut e, EK_PROJ, 5.5, 100.0, false);
            let health = e.health;
            e.tick(0.08);
            assert_eq!(e.health < health, open, "door {door_kind}, open={open}");
        }
    }
}

#[test]
fn projectile_geometry_includes_endpoint_contacts_and_the_exact_door_threshold() {
    let mut e = testutil::arena();
    e.set_cell(8, 4, 8);
    let (start, end) = ((5.5, 4.5), (8.0, 4.5));
    assert_eq!(e.projectile_wall_contact(start, end), Some(1.0));
    assert_eq!(e.projectile_wall_contact(end, end), Some(0.0));
    assert_eq!(e.projectile_wall_contact(end, (9.0, 4.5)), Some(0.0));
    e.door[4 * MAP_W + 8] = 0.979;
    assert_eq!(e.projectile_wall_contact(start, end), Some(1.0));
    e.door[4 * MAP_W + 8] = 0.98;
    assert_eq!(e.projectile_wall_contact(start, end), None);
    assert_eq!(e.projectile_wall_contact(end, end), None);
}

#[test]
fn body_contacts_handle_diagonals_tangents_endpoints_and_reverse_motion() {
    use sim::geometry::sweep_circle_contact;
    let cases = [
        ((2.0, 2.0), (8.0, 8.0), (5.0, 5.0), 2.0_f32.sqrt(), Some(1.0 / 3.0)),
        ((2.0, 2.0), (8.0, 2.0), (5.0, 3.0), 1.0, Some(0.5)),
        ((2.0, 2.0), (8.0, 2.0), (5.0, 3.01), 1.0, None),
        ((2.0, 2.0), (8.0, 2.0), (9.0, 2.0), 1.0, Some(1.0)),
        ((2.0, 2.0), (-4.0, 2.0), (5.0, 2.0), 1.0, None),
        ((2.0, 2.0), (2.0, 2.0), (2.0, 2.0), 1.0, Some(0.0)),
        ((2.0, 2.0), (2.0, 2.0), (4.0, 2.0), 1.0, None),
    ];
    for (start, end, center, radius, expected) in cases {
        let actual = sweep_circle_contact(start, end, center, radius);
        match (actual, expected) {
            (Some(a), Some(b)) => assert!((a - b).abs() < 1e-5, "{start:?} -> {end:?}: {a} != {b}"),
            (None, None) => {},
            _ => panic!("{start:?} -> {end:?}: {actual:?} != {expected:?}"),
        }
    }
}
