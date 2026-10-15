//! Integration coverage through the same fire/reload/tick paths used by WASM.
use crate::{boss_weapons::*, *};

fn gun(slot: usize) -> Engine {
    let mut e = testutil::arena();
    e.pa = 0.0;
    e.grant_slot(slot);
    e.events = 0;
    e.pickup_t = 0.0;
    e
}

fn target(e: &mut Engine, x: f32, y: f32) -> usize {
    let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, x, y).unwrap();
    e.ents[i].hp = 2000;
    e.ents[i].stun = 100.0;
    e.ents[i].armor_hp = 0;
    e.ents[i].shield_hp = 0;
    i
}

fn shoot(e: &mut Engine) {
    e.cooldown = 0.0;
    e.tactical.cooldowns.fill(0.0);
    e.fire();
}

fn advance(e: &mut Engine, seconds: f32) {
    for _ in 0..(seconds / 0.02).ceil() as usize {
        e.tick(0.02);
    }
}

#[test]
fn pressure_is_bounded_per_weapon_and_only_successful_shots_charge_it() {
    for slot in [SINK, UNDERTOW, HELIOS] {
        let mut e = gun(slot);
        e.fire();
        assert_eq!(e.mechanics.pressure[slot], pressure_gain(slot));
        let bank = e.mechanics.pressure[slot];
        e.fire();
        assert_eq!(
            e.mechanics.pressure[slot], bank,
            "cooldown must not charge pressure"
        );
        e.cooldown = 0.0;
        e.tactical.cooldowns.fill(0.0);
        e.mag[slot] = 0;
        e.fire();
        assert_eq!(
            e.mechanics.pressure[slot], bank,
            "dry fire must not charge pressure"
        );
        e.mag[slot] = 20;
        for _ in 0..8 {
            shoot(&mut e);
        }
        assert_eq!(e.mechanics.pressure[slot], 1.0);
        let ammo = e.mag[slot];
        shoot(&mut e);
        assert_eq!(
            e.mag[slot], ammo,
            "full pressure requires deliberate venting"
        );
        assert_eq!(e.mechanics.pressure[0], 0.0);
        assert!(e
            .mechanics
            .pressure
            .iter()
            .all(|p| p.is_finite() && (0.0..=1.0).contains(p)));
    }
}

#[test]
fn safe_vent_has_no_fire_effects_or_ammo_transfer_until_real_reload_completes() {
    for slot in [SINK, UNDERTOW, HELIOS] {
        let mut e = gun(slot);
        e.mag[slot] = 1;
        e.ammo[slot] = 2;
        e.mechanics.pressure[slot] = 0.8;
        e.begin_reload();
        for _ in 0..20 {
            let previous = e.mechanics.pressure[slot];
            e.tick(0.02);
            assert!(e.mechanics.pressure[slot] < previous);
            assert_eq!((e.mag[slot], e.ammo[slot]), (1, 2));
            assert_eq!(e.events & EV_FIRE, 0);
            assert_eq!(e.muzzle, 0.0);
            assert!(!e
                .ents
                .iter()
                .any(|e| matches!(e.kind, EK_PROJ | EK_SMOKE | EK_FLAME)));
            assert!(
                !(10..=24).contains(&e.hud.weap_frame),
                "venting must not show magazine removal or firing art"
            );
        }
        advance(&mut e, 0.6);
        assert!(e.mechanics.vent.is_none());
        assert!(e.reload_t > 0.0, "normal magazine sequence follows cooling");
        assert_eq!((e.mag[slot], e.ammo[slot]), (1, 2));
        advance(&mut e, 3.0);
        assert_eq!((e.mag[slot], e.ammo[slot]), (3, 0));
        assert_eq!(e.run_ammo, 0);
        assert!(e.run_shots.is_empty());
    }
}

#[test]
fn vent_discharge_requires_a_fresh_trigger_spends_one_round_and_cancels_reload() {
    for slot in [SINK, UNDERTOW, HELIOS] {
        let mut e = gun(slot);
        e.mechanics.pressure[slot] = 0.9;
        e.mechanics.fire_held = true;
        e.begin_reload();
        let before = (e.mag[slot], e.ammo[slot]);
        e.fire();
        assert!(
            e.mechanics.vent.is_some(),
            "holding fire while pressing R must not discharge"
        );
        assert_eq!((e.mag[slot], e.ammo[slot]), before);
        e.mechanics.fire_held = false;
        e.fire();
        assert!(e.mechanics.vent.is_none());
        assert_eq!(e.reload_t, 0.0);
        assert_eq!(e.mechanics.pressure[slot], 0.0);
        assert_eq!((e.mag[slot], e.ammo[slot]), (before.0 - 1, before.1));
        assert_eq!(e.run_ammo, 1);
        assert_eq!(e.run_shots.len(), 1);
        assert!(e.events & EV_FIRE != 0 && e.muzzle > 0.0);
        assert!(e.mechanics.release_flash > 0.0);
        assert!(e.cooldown > 0.9);
        e.fire();
        assert_eq!(e.run_ammo, 1, "release recovery cannot be bypassed");
    }
}

#[test]
fn exhausted_weapons_can_cool_but_cannot_create_free_discharge_ammo() {
    let mut e = gun(HELIOS);
    e.mag[HELIOS] = 0;
    e.ammo[HELIOS] = 0;
    e.mechanics.pressure[HELIOS] = 1.0;
    e.begin_reload();
    e.fire();
    assert!(e.mechanics.vent.is_some());
    assert!(e.run_shots.is_empty());
    advance(&mut e, 1.0);
    assert_eq!(e.mechanics.pressure[HELIOS], 0.0);
    assert_eq!((e.mag[HELIOS], e.ammo[HELIOS]), (0, 0));
    assert_eq!(e.reload_t, 0.0);
    assert_eq!(e.hud.weap_frame, 4);
}

#[test]
fn swap_cancels_vent_without_transferring_pressure_or_ammo_and_same_slot_does_not() {
    let mut e = gun(UNDERTOW);
    e.mechanics.pressure[UNDERTOW] = 0.7;
    e.begin_reload();
    e.select_weapon(UNDERTOW);
    assert!(e.mechanics.vent.is_some());
    let bank = e.mechanics.pressure[UNDERTOW];
    e.select_weapon(0);
    assert!(e.mechanics.vent.is_none());
    assert_eq!(e.reload_t, 0.0);
    assert_eq!(e.mechanics.pressure[UNDERTOW], bank);
    assert_eq!(e.mechanics.pressure[0], 0.0);
    assert_eq!(e.mag[UNDERTOW], MAG_SZ[UNDERTOW]);
}

#[test]
fn sector_transition_and_checkpoint_clear_transient_attacks_and_refill_owned_expansion_guns() {
    let mut e = gun(HELIOS);
    e.mechanics.pressure[HELIOS] = 1.0;
    e.begin_reload();
    e.mag[HELIOS] = 0;
    e.ammo[HELIOS] = 2;
    let save = abi::capture_save(&e);
    abi::apply_save(&mut e, &save);
    assert!(e.mechanics.vent.is_none());
    assert_eq!(e.mechanics.pressure[HELIOS], 0.0);
    assert_eq!(
        (e.mag[HELIOS], e.ammo[HELIOS]),
        (0, 2),
        "checkpoint restore must not secretly refill boss ammunition"
    );
    e.mechanics.pressure[HELIOS] = 1.0;
    e.mag[HELIOS] = 0;
    e.ammo[HELIOS] = 0;
    e.next_wave();
    assert_eq!(
        (e.mag[HELIOS], e.ammo[HELIOS]),
        (MAG_SZ[HELIOS], RESERVE_CAP[HELIOS])
    );
    assert_eq!(e.mechanics.pressure[HELIOS], 0.0);
}

#[test]
fn archive_rewards_repeated_target_and_expires_without_leaking_to_reused_entities() {
    let mut e = gun(ARCHIVE);
    let i = target(&mut e, 8.0, 4.5);
    e.fire();
    assert_eq!(e.ents[i].hp, 1958);
    shoot(&mut e);
    assert_eq!(e.ents[i].hp, 1862);
    assert_eq!(e.run_hits, 2);
    shoot(&mut e); // new dossier
    advance(&mut e, 2.6);
    e.ents[i].x = 8.0;
    e.ents[i].y = 4.5;
    e.ents[i].vx = 0.0;
    e.ents[i].vy = 0.0;
    let hp = e.ents[i].hp;
    shoot(&mut e);
    assert_eq!(hp - e.ents[i].hp, 42);
    e.ents[i].kind = EK_NONE;
    let next = target(&mut e, 8.0, 4.5);
    assert_eq!(i, next);
    shoot(&mut e);
    assert_eq!(
        e.ents[next].hp, 1958,
        "new enemy must not inherit the old dossier"
    );
}

#[test]
fn predictor_second_strike_is_delayed_and_committed_to_a_world_position() {
    let mut e = gun(PREDICTOR);
    let i = target(&mut e, 8.0, 4.5);
    e.fire();
    assert_eq!(e.ents[i].hp, 1948);
    advance(&mut e, 0.1);
    assert_eq!(e.ents[i].hp, 1948);
    e.ents[i].y = 8.0;
    advance(&mut e, 0.14);
    assert_eq!(e.ents[i].hp, 1948, "moving away dodges the committed pulse");
}

#[test]
fn sink_pulls_visible_front_targets_but_not_targets_behind_cover_or_player() {
    let mut e = gun(SINK);
    let front = target(&mut e, 8.0, 4.5);
    let rear = target(&mut e, 2.0, 4.5);
    let hidden = target(&mut e, 10.0, 5.5);
    e.map[5 * MAP_W + 9] = 1;
    e.fire();
    assert!(e.ents[front].vx < 0.0);
    assert_eq!(e.ents[rear].vx, 0.0);
    assert_eq!(e.ents[hidden].vx, 0.0);
}

#[test]
fn rime_shatters_between_volleys_never_between_slugs_of_the_same_volley() {
    let mut e = gun(RIME);
    let i = target(&mut e, 8.0, 4.5);
    let adjacent = target(&mut e, 8.0, 5.4);
    e.commit_round(RIME);
    for _ in 0..3 {
        e.player_hit(i, 26, 7.5, 4.5, RIME);
    }
    assert_eq!(e.ents[i].hp, 2000 - 78);
    assert_eq!(e.ents[adjacent].hp, 2000);
    assert!(e.tactical.slow[i] > 0.0);
    e.commit_round(RIME);
    for _ in 0..3 {
        e.player_hit(i, 26, 7.5, 4.5, RIME);
    }
    assert_eq!(e.ents[i].hp, 2000 - 78 - 78 - 32);
    assert!(e.ents[adjacent].hp < 2000);
    assert!(e.tactical.slow[adjacent] > 0.0);
}

#[test]
fn relay_chains_distinct_targets_with_falloff_and_cannot_arc_through_walls() {
    let mut e = gun(RELAY);
    let first = target(&mut e, 7.5, 4.5);
    let second = target(&mut e, 7.5, 6.0);
    let third = target(&mut e, 7.5, 7.5);
    let hidden = target(&mut e, 9.5, 7.5);
    for y in 0..MAP_H {
        e.map[y * MAP_W + 8] = 1;
    }
    e.fire();
    assert_eq!(e.ents[first].hp, 1966);
    assert_eq!(e.ents[second].hp, 1974);
    assert_eq!(e.ents[third].hp, 1980);
    assert_eq!(e.ents[hidden].hp, 2000);
    assert_eq!(e.run_hits, 1, "arcs belong to one ammunition unit");
}

#[test]
fn kest_burst_uses_real_timed_rounds_and_follows_current_aim() {
    let mut e = gun(KEST);
    let a = target(&mut e, 8.0, 4.5);
    let b = target(&mut e, 4.5, 8.0);
    e.fire();
    assert_eq!(e.mag[KEST], MAG_SZ[KEST] - 1);
    assert_eq!(e.ents[a].hp, 1976);
    e.pa = core::f32::consts::FRAC_PI_2;
    advance(&mut e, 0.2);
    assert_eq!(e.mag[KEST], MAG_SZ[KEST] - 3);
    assert_eq!(e.ents[a].hp, 1976);
    assert!(e.ents[b].hp < 2000);
    assert_eq!(e.run_ammo, 3);
    assert_eq!(e.run_shots.len(), 3);
}

#[test]
fn kest_burst_stops_on_reload_switch_empty_magazine_and_death() {
    for cancel in 0..4 {
        let mut e = gun(KEST);
        e.fire();
        match cancel {
            0 => e.begin_reload(),
            1 => e.select_weapon(0),
            2 => e.mag[KEST] = 0,
            _ => e.state = 1,
        }
        advance(&mut e, 0.24);
        assert_eq!(e.run_ammo, 1, "cancellation {cancel} fired queued rounds");
    }
}

#[test]
fn polarity_pierces_three_targets_strips_armor_and_stops_at_walls() {
    let mut e = gun(POLARITY);
    let mut targets = Vec::new();
    for x in [7.0, 9.0, 11.0, 13.0] {
        let i = target(&mut e, x, 4.5);
        e.ents[i].armor_hp = 100;
        targets.push(i);
    }
    e.fire();
    for &i in &targets[..3] {
        assert!(e.ents[i].armor_hp < 76);
    }
    assert_eq!(e.ents[targets[3]].armor_hp, 100);
    for y in 0..MAP_H {
        e.map[y * MAP_W + 8] = 1;
    }
    let armor = e.ents[targets[1]].armor_hp;
    shoot(&mut e);
    assert_eq!(e.ents[targets[1]].armor_hp, armor);
}

#[test]
fn undertow_projectile_pulls_target_and_respects_near_muzzle_cover() {
    let mut e = gun(UNDERTOW);
    let i = target(&mut e, 8.0, 4.5);
    e.fire();
    advance(&mut e, 0.3);
    assert!(e.ents[i].hp < 2000);
    assert!(e.ents[i].vx < 0.0);
    let mut blocked = gun(UNDERTOW);
    let hidden = target(&mut blocked, 8.0, 4.5);
    for y in 0..MAP_H {
        blocked.map[y * MAP_W + 5] = 1;
    }
    blocked.fire();
    advance(&mut blocked, 0.4);
    assert_eq!(blocked.ents[hidden].hp, 2000);
}

#[test]
fn helios_bursts_and_pulses_hit_nearby_enemies_without_self_damage() {
    let mut e = gun(HELIOS);
    let i = target(&mut e, 7.5, 4.5);
    let adjacent = target(&mut e, 7.5, 5.8);
    e.fire();
    advance(&mut e, 0.4);
    assert!(e.ents[i].hp < 1900);
    assert!(e.ents[adjacent].hp < 2000);
    let hp = e.ents[adjacent].hp;
    advance(&mut e, 0.8);
    assert!(e.ents[adjacent].hp < hp);
    assert_eq!(e.health, 100);
    assert_eq!(e.run_ammo, 1);
    assert_eq!(e.run_hits, 1);
}

#[test]
fn epoch_echoes_are_delayed_keep_their_origin_and_do_not_change_role_on_swap() {
    let mut e = gun(EPOCH);
    let i = target(&mut e, 8.0, 4.5);
    e.fire();
    assert_eq!(e.ents[i].hp, 1938);
    e.select_weapon(0);
    advance(&mut e, 0.3);
    assert_eq!(e.ents[i].hp, 1938);
    advance(&mut e, 0.12);
    let first = e.ents[i].hp;
    assert!(first < 1938);
    assert!(
        e.tactical.slow[i] > 0.0,
        "echo retains control role after swapping to pistol"
    );
    advance(&mut e, 0.4);
    assert!(e.ents[i].hp < first);
    assert_eq!(e.run_ammo, 1);
    assert_eq!(e.run_hits, 1);
}

#[test]
fn delayed_attacks_never_damage_through_newly_closed_geometry() {
    let mut e = gun(EPOCH);
    let i = target(&mut e, 8.0, 4.5);
    e.fire();
    e.ents[i].x = 9.2;
    for y in 0..MAP_H {
        e.map[y * MAP_W + 8] = 1;
    }
    advance(&mut e, 0.8);
    assert_eq!(e.ents[i].hp, 1938);
}

#[test]
fn release_projectiles_are_distinct_and_stronger_than_normal_payloads() {
    for slot in [UNDERTOW, HELIOS] {
        let mut normal = gun(slot);
        normal.fire();
        let damage = normal.ents.iter().find(|e| e.kind == EK_PROJ).unwrap().hp;
        let mut charged = gun(slot);
        charged.mechanics.pressure[slot] = 1.0;
        charged.begin_reload();
        charged.fire();
        let shots: Vec<_> = charged.ents.iter().filter(|e| e.kind == EK_PROJ).collect();
        assert_eq!(shots.len(), if slot == UNDERTOW { 3 } else { 1 });
        assert!(shots
            .iter()
            .all(|e| e.projectile_visual == (slot - 7) as u8 && e.projectile_charge == 1.0));
        assert!(shots.iter().map(|e| e.hp).sum::<i32>() > damage * 2);
    }
}

#[test]
fn fixed_seed_replays_all_ten_mechanics_without_invalid_state() {
    fn play() -> Vec<(i32, u32, u32, Vec<i32>)> {
        [
            ARCHIVE, PREDICTOR, SINK, RIME, RELAY, KEST, UNDERTOW, POLARITY, HELIOS, EPOCH,
        ]
        .into_iter()
        .map(|slot| {
            let mut e = gun(slot);
            e.rng = 12345;
            let a = target(&mut e, 8.0, 4.5);
            let b = target(&mut e, 9.0, 5.0);
            e.fire();
            advance(&mut e, 1.2);
            assert!(e.px.is_finite() && e.py.is_finite());
            (
                e.mag[slot],
                e.run_ammo,
                e.run_hits,
                vec![e.ents[a].hp, e.ents[b].hp],
            )
        })
        .collect()
    }
    assert_eq!(play(), play());
}

#[test]
fn pressure_scatter_preserves_entity_headroom_and_control_has_boss_resistance() {
    let mut e = gun(SINK);
    e.mechanics.pressure[SINK] = 1.0;
    e.begin_reload();
    e.fire();
    assert!(e.ents.iter().filter(|e| e.kind != EK_NONE).count() <= 32);
    let mut ordinary = gun(SINK);
    let a = target(&mut ordinary, 8.0, 4.5);
    ordinary.fire();
    let mut boss = gun(SINK);
    let b = target(&mut boss, 8.0, 4.5);
    boss.ents[b].kind = EK_BOSS;
    boss.fire();
    assert!(boss.ents[b].vx.abs() < ordinary.ents[a].vx.abs() * 0.3);
    boss.player_hit(ENT_N, 50, 4.5, 4.5, ARCHIVE);
    assert!(tactical::role(HELIOS) == tactical::Role::Burn);
}

#[test]
fn older_coolant_slugs_cannot_rearm_a_volley_which_already_shattered() {
    let mut e = gun(RIME);
    let i = target(&mut e, 8.0, 4.5);
    e.commit_round(RIME);
    let older = e.current_shot;
    e.player_hit(i, 26, 7.5, 4.5, RIME);
    e.commit_round(RIME);
    let newer = e.current_shot;
    e.player_hit(i, 26, 7.5, 4.5, RIME);
    let hp = e.ents[i].hp;
    e.current_shot = older;
    e.player_hit(i, 26, 7.5, 4.5, RIME);
    e.current_shot = newer;
    e.player_hit(i, 26, 7.5, 4.5, RIME);
    assert_eq!(
        hp - e.ents[i].hp,
        52,
        "staggered arrivals must not create another shatter"
    );
}

#[test]
fn reload_input_on_the_due_frame_cancels_a_burst_before_it_spends_ammo() {
    let mut e = gun(KEST);
    e.fire();
    e.tick(0.07);
    e.bits = IN_RELOAD;
    e.tick(0.02);
    assert_eq!(e.run_ammo, 1);
    assert!(e.reload_t > 0.0);
}

#[test]
fn collecting_a_basic_weapon_cancels_another_weapons_pressure_vent() {
    let mut e = gun(SINK);
    e.mechanics.pressure[SINK] = 0.8;
    e.begin_reload();
    e.spawn(EK_GUN3, e.px, e.py).unwrap();
    e.tick(0.02);
    assert_eq!(e.weapon, 2);
    assert!(e.mechanics.vent.is_none(), "auto-equip must cancel the previous weapon's vent");
    let ammo = (e.mag[SINK], e.mag[2]);
    e.cooldown = 0.0;
    e.bits = IN_FIRE;
    e.tick(0.02);
    assert_eq!((e.mag[SINK], e.mag[2]), (ammo.0, ammo.1 - 1));
    assert!(e.mechanics.pressure[SINK] > 0.0, "cancellation preserves the old weapon's pressure");
}

#[test]
fn a_delayed_attack_due_on_the_reward_pickup_frame_cannot_fire_after_sector_completion() {
    let mut e = gun(PREDICTOR);
    let i = target(&mut e, 8.0, 4.5);
    e.fire();
    advance(&mut e, 0.16);
    let hp = e.ents[i].hp;
    e.spawn(field::boss_case(e.wave), e.px, e.py).unwrap();
    e.tick(0.04);
    assert_eq!(e.state, 2);
    assert_eq!(e.ents[i].hp, hp, "a committed pulse must stop when the sector ends");
}

#[test]
fn checkpoint_ammunition_is_bounded_before_reload_and_fire() {
    let mut e = gun(HELIOS);
    let mut save = abi::capture_save(&e);
    for slot in 0..WEP_N {
        save.mag[slot] = if slot % 2 == 0 { i32::MIN } else { i32::MAX };
        save.ammo[slot] = if slot % 2 == 0 { i32::MAX } else { i32::MIN };
    }
    abi::apply_save(&mut e, &save);
    for slot in 0..WEP_N {
        assert!((0..=MAG_SZ[slot]).contains(&e.mag[slot]), "slot {slot} magazine escaped its bounds");
        assert!((0..=RESERVE_CAP[slot]).contains(&e.ammo[slot]), "slot {slot} reserve escaped its bounds");
    }
    e.begin_reload();
    advance(&mut e, 4.0);
    assert_eq!(e.mag[HELIOS], MAG_SZ[HELIOS]);
    assert_eq!(e.ammo[HELIOS], RESERVE_CAP[HELIOS] - MAG_SZ[HELIOS]);
}
