use crate::*;

#[test]
fn vertical_look_has_a_wider_symmetric_range() {
    let mut e = testutil::arena();
    for (input, expected) in [(-1000.0, 0.85), (1000.0, -0.85)] {
        e.my = input;
        e.tick(0.0);
        assert!((e.pitch - expected).abs() < 1e-6, "vertical limit was {}", e.pitch);
    }
}

#[test]
fn immediate_look_updates_aim_without_advancing_gameplay() {
    let mut e = testutil::arena();
    let (yaw, elapsed, ammo, cooldown, position) = (e.pa, e.elapsed, e.mag, e.cooldown, (e.px, e.py));
    e.look(2.0, -3.0);
    assert!((e.pa - yaw - 0.0124).abs() < 1e-6);
    assert!((e.pitch - 0.0162).abs() < 1e-6);
    assert_eq!(e.hud.yaw, e.pa);
    assert_eq!((e.elapsed, e.mag, e.cooldown, (e.px, e.py)), (elapsed, ammo, cooldown, position));
    let aim = (e.pa, e.pitch);
    e.tick(1.0 / 60.0);
    assert_eq!((e.pa, e.pitch), aim, "the next physics step must not apply mouse input twice");
}

#[test]
fn look_displacement_is_independent_of_mouse_packet_count() {
    let mut e = testutil::arena();
    let start = e.pa;
    e.look(24.0, -12.0);
    let expected = (e.pa, e.pitch);
    e.pa = start; e.pitch = 0.0;
    for _ in 0..240 { e.look(0.1, -0.05); }
    assert!((e.pa - expected.0).abs() < 1e-5);
    assert!((e.pitch - expected.1).abs() < 1e-5);
}

#[test]
fn look_rejects_invalid_deltas_and_cannot_turn_a_finished_game() {
    let mut e = testutil::arena();
    let yaw = e.pa;
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] { e.look(bad, bad); }
    assert_eq!((e.pa, e.pitch), (yaw, 0.0));
    for state in [1, 2] {
        e.state = state;
        e.look(20.0, 30.0);
        assert_eq!((e.pa, e.pitch), (yaw, 0.0));
    }
}

#[test]
fn camera_motion_interpolates_without_moving_the_physics_player() {
    let mut e = testutil::arena();
    e.reset_camera();
    e.bits = IN_W;
    e.tick(1.0 / 60.0);
    let actual = (e.px, e.py);
    let start = (4.5, 4.5);
    for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
        e.prepare_camera(alpha);
        let view = e.view_position();
        assert!((view.0 - (start.0 + (actual.0 - start.0) * alpha)).abs() < 1e-6);
        assert!((view.1 - (start.1 + (actual.1 - start.1) * alpha)).abs() < 1e-6);
        assert_eq!((e.px, e.py), actual);
    }
    e.tick(1.0 / 60.0);
    e.prepare_camera(0.0);
    assert_eq!(e.view_position(), actual, "each tick retains only its immediate predecessor");
}

#[test]
fn render_camera_snaps_for_teleports_stops_and_invalid_fractions() {
    let mut e = testutil::arena();
    e.reset_camera();
    e.px += 8.0;
    e.prepare_camera(0.1);
    assert_eq!(e.view_position(), (e.px, e.py));
    e.reset_camera();
    e.px += 0.1;
    for alpha in [f32::NAN, f32::INFINITY, 2.0] {
        e.prepare_camera(alpha);
        assert_eq!(e.view_position(), (e.px, e.py));
    }
    e.state = 1;
    e.prepare_camera(0.0);
    assert_eq!(e.view_position(), (e.px, e.py));
}

#[test]
fn sector_and_checkpoint_changes_reset_the_camera_history() {
    let mut e = testutil::arena();
    let save = abi::capture_save(&e);
    e.bits = IN_W; e.tick(1.0 / 60.0); e.prepare_camera(0.0);
    e.next_wave();
    e.prepare_camera(0.0);
    assert_eq!(e.view_position(), (e.px, e.py));
    abi::apply_save(&mut e, &save);
    e.prepare_camera(0.0);
    assert_eq!(e.view_position(), (e.px, e.py));
}

#[test]
fn wider_pitch_and_interpolated_camera_keep_cpu_render_and_enemy_overlays_valid() {
    let mut e = testutil::arena();
    e.map[4 * MAP_W + 9] = 1;
    let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 7.5, 4.5).unwrap();
    e.ents[i].bar_t = 1.0;
    e.ents[i].stun = 100.0;
    e.bits = IN_W; e.tick(1.0 / 60.0); e.prepare_camera(0.5);
    let actual = (e.px, e.py);
    for pitch in [-0.85, 0.0, 0.85] {
        e.pitch = pitch;
        let mut cues = [voices::EnemyCue::default(); 1];
        let mut bars = [voices::BarCue::default(); 1];
        assert_eq!(voices::snapshot(&e, &mut cues), 1);
        assert_eq!(voices::snapshot_bars(&e, &mut bars), 1);
        assert!((cues[0].screen_x - bars[0].screen_x).abs() < 1e-6);
        assert!(cues[0].screen_y.is_finite() && bars[0].screen_y.is_finite());
        e.render();
        assert!(e.fb.iter().any(|pixel| pixel & 0xffffff != 0));
        assert_eq!((e.px, e.py), actual);
    }
}
