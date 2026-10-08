//! Behavioural tests for the engine core: weapons, waves, pickups,
//! field objectives, saves and the presentation contract.

use crate::render::*;
use crate::testutil::*;
use crate::*;
use crate::abi::*;

#[test]
fn placed_objects_keep_their_kind_texture_and_height() {
    let mut e = arena();
    for def in enemies::ENEMY_DEFS.iter().filter(|d| !d.hostile && d.role != "fx") {
        map::place_item(&mut e, def.kind, 0, 10.5, 10.5);
        let ent = e.ents.iter().find(|ent| ent.kind == def.kind).unwrap();
        let expected_texture = if matches!(def.kind, EK_OVERRIDE_CONSOLE | EK_NODE | EK_TERMINAL) {
            T_CONSOLE_UPPER
        } else { def.texture };
        assert_eq!(sprite_style(ent).0, expected_texture, "{} rendered as an enemy", def.name);
        assert_eq!(ent.zoff, def.zoff, "{} inherited enemy height", def.name);
        let mut cues = [voices::EnemyCue::default(); 8];
        assert_eq!(voices::snapshot(&e, &mut cues), 0, "{} emitted an enemy voice cue", def.name);
        for ent in &mut e.ents { ent.kind = EK_NONE; }
    }
}

#[test]
fn authored_sector_objects_never_select_enemy_animation_layers() {
    let mut e = arena();
    for wave in 1..=25 {
        e.wave = wave;
        e.door.fill(0.0);
        e.build_map();
        e.place_ents();
        for ent in e.ents.iter().filter(|ent| ent.kind != EK_NONE && !is_hostile_kind(ent.kind)) {
            let texture = sprite_style(ent).0;
            assert!(!(ENEMY_TEX_BASE..T_ORDNANCE).contains(&texture),
                "sector {wave} object kind {} skin {} selected enemy texture {texture}", ent.kind, ent.skin);
        }
    }
}

#[test]
fn held_weapon_key_does_not_cancel_reload() {
    let mut e = arena();
    e.set_w(0);
    e.mag[1] = MAG_SZ[1];
    e.ammo[1] = 6;
    e.bits = IN_W2;
    e.tick(1.0 / 60.0);
    assert_eq!(e.weapon, 1);
    e.mag[1] = 1;
    e.begin_reload();
    e.bits = IN_W2;
    e.tick(1.0 / 60.0);
    assert_eq!(e.weapon, 1);
    assert!(e.reload_t > 0.0, "a held weapon key must not restart the swap");
    e.bits = 0;
    e.tick(1.0 / 60.0);
    e.bits = IN_W1;
    e.tick(1.0 / 60.0);
    assert_eq!(e.weapon, 0);
    assert_eq!(e.reload_t, 0.0, "an actual swap cancels the reload");
    e.bits = 0;
    e.tick(1.0 / 60.0);
    e.bits = IN_W3;
    e.tick(1.0 / 60.0);
    assert_eq!(e.weapon, 0, "a locked gun must not switch");
}

#[test]
fn number_keys_select_every_available_weapon_once_per_press() {
    let mut e = arena();
    e.set_w(0);
    e.set_w(1);
    e.set_w(2);
    e.set_w(3);
    e.set_w(4);
    e.set_w(5);
    e.set_w(6);
    for (expected, bit) in [IN_W1, IN_W2, IN_W3, IN_W4, IN_W5, IN_W6, IN_W7, IN_W8]
        .into_iter()
        .enumerate()
    {
        e.bits = 0;
        e.tick(1.0 / 60.0);
        e.bits = bit;
        e.tick(1.0 / 60.0);
        assert_eq!(e.weapon, expected as i32, "key {} selects its weapon", expected + 1);
    }
}

#[test]
fn arc_and_rotary_stay_locked_until_their_cases_are_picked_up() {
    let mut e = arena();
    for bit in [IN_W6, IN_W7, IN_W8] {
        e.bits = bit;
        e.tick(1.0 / 60.0);
        assert_eq!(e.weapon, 0, "locked late-game weapons must not switch");
        e.bits = 0;
        e.tick(1.0 / 60.0);
    }
    e.pickup(EK_GUN6);
    assert!(e.has_w(4));
    assert_eq!(e.weapon, 5);
    e.pickup(EK_GUN7);
    assert!(e.has_w(5));
    assert_eq!(e.weapon, 6);
    e.pickup(EK_GUN8);
    assert!(e.has_w(6));
    assert_eq!(e.weapon, 7);
}

#[test]
fn approach_opens_doors_but_secrets_need_use() {
    let mut e = arena();
    e.pa = 0.0;
    e.set_cell(5, 4, 8);
    e.set_cell(5, 3, 9);
    e.tick(1.0 / 60.0);
    assert!(e.door[4 * MAP_W + 5] > 0.0, "a door opens on approach");
    assert_eq!(e.door[3 * MAP_W + 5], 0.0, "a secret stays shut");
    assert_eq!(e.secrets, 0);
    e.bits = IN_USE;
    e.tick(1.0 / 60.0);
    assert!(e.door[3 * MAP_W + 5] > 0.0);
    assert_eq!(e.secrets, 1);
    e.tick(1.0 / 60.0);
    assert_eq!(e.secrets, 1, "holding use must not recount the secret");
    assert_eq!(e.events & EV_BOSS_VOICE, 0, "doors never play a boss voice");
    assert_ne!(e.radio_line, 5, "secret doors never announce a voice line");
}

#[test]
fn node_and_terminal_announcements_fire_once_per_interactable() {
    let mut e = arena();
    let (nx, ny) = field::node_point(e.wave);
    (e.px, e.py) = (nx, ny);
    e.use_field();
    let node_seq = e.radio_seq;
    assert_eq!(e.events & EV_BOSS_VOICE, 0, "the node cannot trigger a boss death voice");
    e.use_field();
    assert_eq!(e.radio_seq, node_seq, "the same node cannot speak twice");

    let (tx, ty) = field::terminals(e.wave)[0];
    let terminal = e.spawn(EK_TERMINAL, tx, ty).unwrap();
    (e.px, e.py) = (tx, ty);
    e.use_field();
    let terminal_seq = e.radio_seq;
    assert_eq!(e.events & EV_BOSS_VOICE, 0, "the terminal cannot trigger a boss death voice");
    assert!(terminal_seq > node_seq);
    assert!(e.ents[terminal].timer < 0.0);
    e.use_field();
    assert_eq!(e.radio_seq, terminal_seq, "the same terminal cannot speak twice");
}

#[test]
fn hitscan_stops_at_cover() {
    let mut e = arena();
    e.pa = 0.0;
    let behind = e.spawn(EK_HUSK, 8.5, 4.5).unwrap();
    let hp = e.ents[behind].hp;
    e.set_cell(6, 4, 1);
    assert!(!e.hitscan(0.0, 15, 22.0));
    assert_eq!(e.ents[behind].hp, hp);
    e.set_cell(6, 4, 0);
    assert!(e.hitscan(0.0, 15, 22.0));
    assert_eq!(e.ents[behind].hp, hp - 15);
}

#[test]
fn boss_death_wins_and_barrels_chain_without_kills() {
    let mut e = arena();
    let boss = e.spawn(EK_BOSS, 6.5, 4.5).unwrap();
    e.ents[boss].hp = 1;
    e.hurt_ent(boss, 5, e.px, e.py);
    assert_ne!(e.events & EV_BOSS_VOICE, 0, "boss death emits one voice event");
    assert_eq!(e.ents[boss].anim, ANIM_DEAD);
    assert_eq!(e.state, 0, "the boss drops a weapon instead of ending the sector");
    assert!(e.ents.iter().any(|en| field::is_boss_case(en.kind)));
    assert_eq!(e.kills, 1);
    e.tick(1.0 / 60.0);
    assert_eq!(e.events & EV_BOSS_VOICE, 0, "voice event clears on the next tick");
    for _ in 1..90 { e.tick(1.0 / 60.0); }
    assert_eq!(e.ents[boss].kind, EK_NONE);

    let mut e = arena();
    let a = e.spawn(EK_BARREL, 5.0, 4.5).unwrap();
    let b = e.spawn(EK_BARREL, 6.2, 4.5).unwrap();
    e.ents[a].hp = 1;
    e.ents[b].hp = 1;
    e.hurt_ent(a, 5, 4.0, 4.5);
    assert_eq!(e.ents[a].anim, ANIM_DEAD);
    assert_eq!(e.ents[b].anim, ANIM_DEAD, "a barrel blast chains");
    assert_eq!(e.kills, 0, "props are not kills");
    assert_eq!(e.state, 0);
    assert!(e.health > 0, "a single chain must not delete the player");
    for _ in 0..45 { e.tick(1.0 / 60.0); }
    assert_eq!(e.ents[a].kind, EK_NONE);
    assert_eq!(e.ents[b].kind, EK_NONE);
}

#[test]
fn boss_kill_ends_the_level_and_next_wave_restarts_the_hunt() {
    let mut e = arena();
    let boss = e.spawn(EK_BOSS, 6.5, 4.5).unwrap();
    e.ents[boss].hp = 1;
    e.hurt_ent(boss, 5, e.px, e.py);
    assert_eq!(e.state, 0, "killing the boss drops the case");
    let case = e.ents.iter().position(|en| field::is_boss_case(en.kind)).unwrap();
    e.pickup(e.ents[case].kind);
    assert_eq!(e.state, 2, "taking the boss weapon ends the level");
    assert!(e.has_w(7));
    e.next_wave();
    assert_eq!((e.state, e.wave), (0, 2));
    assert!(!e.boss_spawned && !e.hell);
    assert!(map::living_hostiles(&e) > 0, "the next level spawns its cast");
}

#[test]
fn lockdown_seal_ejects_player_from_doorway() {
    let mut e = Engine::new(160, 100);
    assert_eq!(e.wave, 1);
    let (dx, dy) = field::lockdown_doors(1)[0];
    assert_eq!(e.cell(dx, dy), 8, "the vault leaf must be a door cell");
    let idx = dy as usize * MAP_W + dx as usize;
    e.door[idx] = 1.0;
    e.px = dx as f32 + 0.5;
    e.py = dy as f32 + 0.5;
    assert!(!e.circle_blocked(e.px, e.py, e.pr), "open doorway starts walkable");
    e.seal_lockdown();
    assert!(!e.circle_blocked(e.px, e.py, e.pr), "the seal must eject, not trap");
}

#[test]
fn embedded_player_is_relocated_by_tick() {
    let mut e = arena();
    e.set_cell(5, 4, 1);
    e.px = 5.5;
    e.py = 4.5;
    assert!(e.circle_blocked(e.px, e.py, e.pr));
    e.tick(1.0 / 60.0);
    assert!(!e.circle_blocked(e.px, e.py, e.pr), "a tick must unstick the player");
}

#[test]
fn next_wave_restores_the_body_and_keeps_found_gear() {
    let mut e = arena();
    e.health = 11;
    e.state = 1;
    e.set_w(2);
    e.weapon = 3;
    e.ammo[3] = 1;
    e.mag[3] = 0;
    let _med = e.spawn(EK_MED, 8.5, 8.5).unwrap();
    e.spawn(EK_HUSK, 9.5, 8.5).unwrap();
    e.next_wave();
    assert_eq!(e.wave, 2);
    assert_eq!(e.health, 100);
    assert_eq!(e.state, 0);
    assert!(e.has_w(2));
    assert_eq!(e.weapon, 3, "a found gun stays in hand");
    assert_eq!(e.mag[3], MAG_SZ[3]);
    assert!(e.ammo[3] >= 8);
    assert!(e.ents.iter().any(|en| en.kind == EK_MED), "the new sector supplies are placed");
    assert_eq!(
        e.ents.iter().filter(|en| is_hostile_kind(en.kind)).count(),
        map::hostiles(2).len() * 2,
        "the previous cast is cleared, then two copies of the sector roster spawn",
    );
    e.wave = 12;
    e.next_wave();
    assert_eq!(e.wave, 13, "endless waves climb past 12");
    assert!(e.ents.iter().any(|en| en.kind == EK_MED));
}

/// Pulls every `reserve: <int>` out of the TypeScript arsenal table. Kept
/// dependency-free (no regex crate) because this only runs in tests.
fn ts_reserve_caps(ts: &str) -> Vec<i32> {
    let mut out = Vec::new();
    let mut rest = ts;
    while let Some(at) = rest.find("reserve: ") {
        rest = &rest[at + "reserve: ".len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() { continue; }
        out.push(digits.parse().expect("numeric reserve cap"));
    }
    out
}

#[test]
fn damage_records_the_bearing_of_its_source() {
    let mut e = arena();
    e.state = 0;
    e.pa = 0.0;
    e.iframes = 0.0;
    // Facing +x, so a source at +y sits 90 degrees to the left and one at
    // -y sits to the right.
    e.damage_player_from(10, e.px, e.py + 4.0);
    assert!((e.hurt_dir - core::f32::consts::FRAC_PI_2).abs() < 1e-4,
        "a source to the left should read +pi/2, got {}", e.hurt_dir);
    e.iframes = 0.0;
    e.health = 100;
    e.damage_player_from(10, e.px, e.py - 4.0);
    assert!((e.hurt_dir + core::f32::consts::FRAC_PI_2).abs() < 1e-4,
        "a source to the right should read -pi/2, got {}", e.hurt_dir);
    e.iframes = 0.0;
    e.health = 100;
    e.damage_player_from(10, e.px + 4.0, e.py);
    assert!(e.hurt_dir.abs() < 1e-4, "a source dead ahead should read 0, got {}", e.hurt_dir);
    e.iframes = 0.0;
    e.health = 100;
    e.damage_player_from(10, e.px - 4.0, e.py);
    assert!((e.hurt_dir.abs() - core::f32::consts::PI).abs() < 1e-4,
        "a source behind should read +-pi, got {}", e.hurt_dir);
    // The bearing stays wrapped into [-pi, pi] so the shader's sin/cos
    // can consume it directly.
    e.iframes = 0.0;
    e.health = 100;
    e.pa = 2.9;
    e.damage_player_from(10, e.px - 4.0, e.py + 4.0);
    assert!((-core::f32::consts::PI..=core::f32::consts::PI).contains(&e.hurt_dir),
        "hurt_dir escaped [-pi, pi]: {}", e.hurt_dir);
    assert!(e.hurt_dir.is_finite());
}

#[test]
fn low_health_strain_only_appears_when_it_should() {
    let mut e = arena();
    e.state = 0;
    e.health = 100;
    for _ in 0..30 { e.tick(1.0 / 60.0); }
    assert_eq!(e.strain, 0.0, "a healthy player has no strain");

    // Cross the threshold: strain rises but stays bounded.
    e.health = 20;
    e.strain = 0.0;
    let mut peak = 0.0f32;
    for _ in 0..240 { e.tick(1.0 / 60.0); peak = peak.max(e.strain); }
    assert!(peak > 0.4, "low health must be felt, peak was {peak}");
    assert!(peak <= 1.0, "strain must stay in 0..1, peak was {peak}");
    // It breathes rather than sitting still.
    let a = e.strain;
    for _ in 0..12 { e.tick(1.0 / 60.0); }
    assert!((e.strain - a).abs() > 1e-4, "strain must pulse, it was static at {a}");

    // Healing clears it.
    e.health = 100;
    for _ in 0..180 { e.tick(1.0 / 60.0); }
    assert!(e.strain < 0.02, "strain should decay after healing, got {}", e.strain);
    // Dying clears it too, so the death card is not tinted.
    e.state = 1;
    e.health = 10;
    for _ in 0..180 { e.tick(1.0 / 60.0); }
    assert!(e.strain < 0.02, "strain must not persist into the death state");
}

#[test]
fn reserve_caps_match_the_arsenal_table() {
    // `WEAPONS[].reserve` in src/components/game/data.ts documents itself as
    // mirroring the engine pickup caps. Parsed from the TS source so a
    // data-side retune cannot silently diverge from the engine.
    let ts = include_str!("../../src/components/game/data.ts");
    let expected = ts_reserve_caps(ts);
    assert_eq!(expected.len(), 19, "could not read the first 19 arsenal reserves");
    assert_eq!(&RESERVE_CAP[..19], &expected[..], "engine and TS reserve caps diverged");
    for (slot, cap) in RESERVE_CAP.iter().enumerate() {
        assert!(*cap > 0 && *cap <= MAG_SZ[slot] * 12, "slot {slot} reserve {cap} is implausible");
    }
}

#[test]
fn weapon_ownership_survives_a_save_round_trip() {
    let fresh = arena();
    assert!(!fresh.has_w(0), "a fresh run owns only the starting sidearm");
    assert!(fresh.owns_slot(0), "slot 0 is always carried");
    assert!(!fresh.owns_slot(1));
    assert_eq!(capture_save(&fresh).flags, 0, "nothing outside slot 0 starts owned");
    // Every tracked flag must survive the flag word, including the last one.
    for i in 0..OWNED_GUNS {
        let mut r = arena();
        r.owned = [false; OWNED_GUNS];
        r.set_w(i);
        assert!(r.owns_slot(i + 1), "set_w({i}) did not grant slot {}", i + 1);
        let save = capture_save(&r);
        assert!(save.flags & (1 << i) != 0, "flag {i} lost its bit");
        assert_eq!(save.flags, 1 << i, "flag {i} leaked into other bits");
    }
}

#[test]
fn a_hitch_cannot_outrun_the_step_cap() {
    let mut e = arena();
    e.pa = 0.0;
    e.bits = IN_W;
    e.tick(5.0);
    let dist = ((e.px - 4.5).powi(2) + (e.py - 4.5).powi(2)).sqrt();
    assert!((dist - 3.35 * 0.08).abs() < 0.001);
    assert!((e.time - 0.08).abs() < 1e-5);
    let x = e.px;
    e.tick(-3.0);
    assert!((e.time - 0.08).abs() < 1e-5, "a negative step must not rewind");
    assert!((e.px - x).abs() < 0.001);
}

#[test]
fn non_finite_abi_input_cannot_poison_the_simulation() {
    // Feeding sanitised look deltas keeps every downstream transform finite.
    // `finite()` is exactly what `hs_input` applies before storing them.
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut e = arena();
        e.px = 4.5;
        e.py = 4.5;
        e.pa = 0.0;
        e.mx = finite(bad);
        e.my = finite(bad);
        e.tick(1.0 / 60.0);
        assert!(e.time.is_finite(), "time must stay finite after {bad}");
        assert!(e.px.is_finite() && e.py.is_finite(), "position must stay finite after {bad}");
        assert!(e.pa.is_finite() && e.pitch.is_finite(), "facing must stay finite after {bad}");
    }

    // A NaN step advances nothing rather than rewinding or exploding.
    let mut n = arena();
    n.pa = 0.0;
    n.tick(f32::NAN);
    assert!((n.time - 0.0).abs() < 1e-6, "a NaN step must not advance time");
    assert!(n.time.is_finite());

    // The sanitisers themselves: `hs_input` and `hs_tick` are the only doors
    // in and both route through these.
    assert_eq!(finite(f32::NAN), 0.0);
    assert_eq!(finite(f32::INFINITY), 0.0);
    assert_eq!(finite(f32::NEG_INFINITY), 0.0);
    assert_eq!(finite(0.5), 0.5);
    assert_eq!(finite(-2.0), -2.0);
    assert_eq!(safe_dt(f32::NAN), 0.0);
    // A non-finite step becomes a no-op rather than a full-length jump.
    assert_eq!(safe_dt(f32::INFINITY), 0.0);
    assert_eq!(safe_dt(f32::NEG_INFINITY), 0.0);
    assert_eq!(safe_dt(-1.0), 0.0);
    assert_eq!(safe_dt(0.5), 0.08);
    assert_eq!(safe_dt(1.0 / 60.0), 1.0 / 60.0);
}

#[test]
fn full_armor_is_left_on_the_floor() {
    let mut e = arena();
    e.armor = 100;
    let item = e.spawn(EK_ARMOR, e.px, e.py).unwrap();
    e.tick(1.0 / 60.0);
    assert_eq!(e.ents[item].kind, EK_ARMOR);
    assert_eq!(e.armor, 100);
    e.armor = 40;
    e.tick(1.0 / 60.0);
    assert_eq!(e.armor, 90);
    assert_eq!(e.ents[item].kind, EK_NONE);
}

#[test]
fn ammo_crates_restock_the_complete_eight_weapon_arsenal() {
    let mut e = arena();
    e.set_w(0);
    e.set_w(1);
    e.set_w(2);
    e.set_w(3);
    e.set_w(4);
    e.set_w(5);
    e.set_w(6);
    e.ammo = [0; WEP_N];
    let item = e.spawn(EK_AMMO, e.px, e.py).unwrap();
    e.tick(1.0 / 60.0);
    assert_eq!(e.ents[item].kind, EK_NONE, "a useful ammo crate is collected");
    assert_eq!(&e.ammo[..19], &[18, 10, 45, 5, 2, 15, 90, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert!(e.ammo[19..].iter().all(|ammo| *ammo == 0));

    e.ammo[..19].copy_from_slice(&[120, 48, 216, 20, 16, 80, 450, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let full = e.spawn(EK_AMMO, e.px, e.py).unwrap();
    e.tick(1.0 / 60.0);
    assert_eq!(e.ents[full].kind, EK_AMMO, "a full arsenal leaves supplies available");
}

#[test]
fn override_requires_use_then_spawns_the_sector_boss() {
    let mut e = arena();
    e.node_done = true;
    e.wave = 2;
    (e.px, e.py) = map::override_point(e.wave);
    e.tick(1.0 / 60.0);
    assert_eq!(e.boss_intro, 0.0, "standing on the beacon is not enough");
    e.bits = IN_USE;
    e.tick(1.0 / 60.0);
    assert!(e.boss_intro > 5.0, "USE begins the override countdown");
    e.bits = 0;
    for _ in 0..370 { e.tick(1.0 / 60.0); }
    let bosses: Vec<_> = e.ents.iter().filter(|en| en.kind == EK_BOSS).collect();
    assert_eq!(bosses.len(), 1);
    assert_eq!(bosses[0].hp, campaign::boss_health(2));
    assert_eq!(bosses[0].skin, SKIN_HECATE);
    assert!(e.boss_spawned);
    assert!(e.hell);
    e.tick(1.0 / 60.0);
    assert_eq!(e.ents.iter().filter(|en| en.kind == EK_BOSS).count(), 1);
}

#[test]
fn override_use_requires_a_clear_path_to_the_console() {
    let mut e = arena();
    e.node_done = true;
    e.wave = 1;
    e.px = 38.9;
    e.py = 21.5;
    e.set_cell(39, 21, 1);
    e.bits = IN_USE;
    e.tick(1.0 / 60.0);
    assert_eq!(e.boss_intro, 0.0);
    e.set_cell(39, 21, 0);
    e.bits = 0;
    e.tick(1.0 / 60.0);
    e.bits = IN_USE;
    e.tick(1.0 / 60.0);
    assert!(e.boss_intro > 0.0);
}

#[test]
fn boss_sequences_have_distinct_timing_and_entry_effects() {
    let mut e = arena();
    assert_eq!(e.boss_intro_duration(), 4.0);
    e.wave = 2;
    assert_eq!(e.boss_intro_duration(), 5.2);
    e.boss_intro_effect(0);
    assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.kind == EK_SPARK));
    e.fx_n = 0;
    e.wave = 3;
    assert_eq!(e.boss_intro_duration(), 4.6);
    e.boss_intro_effect(0);
    assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.kind == EK_SPARK));
    e.fx_n = 0;
    e.wave = 4;
    assert_eq!(e.boss_intro_duration(), 4.8);
    e.boss_intro_effect(0);
    assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.kind == EK_SPARK));
    e.fx_n = 0;
    e.wave = 5;
    assert_eq!(e.boss_intro_duration(), 5.0);
    e.boss_intro_effect(0);
    assert!(e.fx_q[..e.fx_n].iter().any(|fx| fx.variant == 20));
}

#[test]
fn boss_health_phases_call_support_with_electrical_entry_fx() {
    let mut e = arena();
    e.boss_spawned = true;
    let boss = e.spawn_with_skin(EK_BOSS, SKIN_VEYRAN, 9.5, 4.5).unwrap();
    e.ents[boss].hp = 300;
    e.tick(1.0 / 60.0);
    assert_eq!(e.boss_phase, 1);
    e.tick(1.0 / 60.0);
    assert!(e.ents.iter().any(|x| x.kind == EK_SPARK));
    assert!(e.ents.iter().any(|x| x.skin == SKIN_HORNET && x.hp > 0));
    assert!(e.ents.iter().any(|x| x.skin == SKIN_RIFLEMAN && x.hp > 0));

    let phase_one_support = e.ents.iter().filter(|x| is_hostile_kind(x.kind) && x.kind != EK_BOSS).count();
    e.ents[boss].hp = 150;
    e.tick(1.0 / 60.0);
    assert_eq!(e.boss_phase, 2);
    assert!(e.ents.iter().any(|x| x.skin == SKIN_MARTYR && x.hp > 0));
    assert!(e.ents.iter().any(|x| x.skin == SKIN_LOADER && x.hp > 0));
    let phase_two_support = e.ents.iter().filter(|x| is_hostile_kind(x.kind) && x.kind != EK_BOSS).count();
    assert!(phase_two_support > phase_one_support);
    e.tick(1.0 / 60.0);
    assert_eq!(
        e.ents.iter().filter(|x| is_hostile_kind(x.kind) && x.kind != EK_BOSS).count(),
        phase_two_support,
        "a phase summons its support squad only once",
    );
}

#[test]
fn phase_changes_schedule_warned_boss_attacks_instead_of_instant_barrages() {
    let mut e=arena();
    for wave in 1..=25 {
        for ent in &mut e.ents {ent.kind=EK_NONE;}
        e.boss_attack=boss_attacks::AttackState::new();e.boss_phase=0;
        e.wave=wave;e.state=0;e.boss_spawned=true;e.boss_intro=0.0;
        e.clear_boss_arena();e.px=4.5;e.py=4.5;
        let boss=e.spawn_with_skin(EK_BOSS,map::boss_skin(wave),9.5,4.5).unwrap();
        e.ents[boss].hp=e.boss_max_health()/2;
        e.tick(1.0/60.0);
        assert_eq!(e.boss_phase,1);
        assert!(!e.ents.iter().any(|en|en.kind==EK_PROJ),"instant phase damage in sector {wave}");
        // Clear support to isolate the boss's attack path.
        for ent in &mut e.ents {if is_hostile_kind(ent.kind)&&ent.kind!=EK_BOSS {ent.kind=EK_NONE;}}
        e.px=e.ents[boss].x-3.0;e.py=e.ents[boss].y;
        e.ents[boss].timer=0.0;
        e.tick(1.0/60.0);
        assert_eq!(e.hud.boss_attack_state,1,"sector {wave} should warn");
        assert!(e.hud.boss_attack_t>=0.8);
        assert!(!e.ents.iter().any(|en|en.kind==EK_PROJ));
    }
}

#[test]
fn boss_fight_reports_health_and_phase_to_the_hud() {
    let mut e = arena();
    e.boss_spawned = true;
    let boss = e.spawn_with_skin(EK_BOSS, SKIN_VEYRAN, 9.5, 4.5).unwrap();
    e.ents[boss].hp = 300;
    e.tick(1.0 / 60.0);
    assert_eq!(e.hud.boss_health, 300);
    assert_eq!(e.hud.boss_max_health, 480);
    assert_eq!(e.hud.boss_phase, 1);

    e.ents[boss].hp = 0;
    e.tick(1.0 / 60.0);
    assert_eq!((e.hud.boss_health, e.hud.boss_max_health, e.hud.boss_phase), (0, 0, 0));
}

#[test]
fn hell_swaps_walls_and_keeps_override_doors_legible() {
    let mut e = arena();
    assert_eq!(e.wall_tex(2, 1, 1), T_SECTOR_SURFACE);
    assert_eq!(e.wall_tex(8, 36, 18), T_SECTOR_SURFACE + 2);
    e.hell = true;
    assert_eq!(e.wall_tex(2, 1, 1), T_BOSS_ARENA);
    assert_eq!(e.wall_tex(6, 1, 1), T_BOSS_ARENA);
    assert_eq!(e.wall_tex(7, 1, 1), T_BOSS_ARENA);
    assert_eq!(e.wall_tex(9, 1, 1), T_SECTOR_SURFACE + 1);
    assert_eq!(e.wall_tex(8, 36, 18), T_SECTOR_SURFACE + 2);
    assert_eq!(e.wall_tex(9, 36, 19), T_SECTOR_SURFACE + 1);
}

#[test]
fn shade_and_mip_blend_match_the_scalar_formula() {
    let color = Engine::pack(200, 40, 10, 255);
    let gain = [(1.8 * 65536.0) as i32, (0.5 * 65536.0) as i32, (0.18 * 65536.0) as i32];
    let shaded = Engine::shade_texel(color, gain, 0x3c00_0000);
    assert_eq!(shaded & 255, 255, "channel clamp");
    assert_eq!((shaded >> 8) & 255, 20);
    assert_eq!((shaded >> 16) & 255, 1);
    assert_eq!(shaded >> 24, 0x3c);
    let batch = Engine::shade_texels4([color, 0, color, 0xff], [gain, [0; 3], gain, [65536; 3]], 0);
    assert_eq!(batch[0], Engine::shade_texel(color, gain, 0));
    assert_eq!(batch[2], batch[0]);
    assert_eq!(batch[3], Engine::shade_texel(0xff, [65536; 3], 0));

    let mut e = arena();
    e.tex[3 + 5 * TEX] = 0xff11_2233;
    e.rebuild_mipmaps();
    assert_eq!(e.sample_mip(0, 3, 5, 0, 0), e.sample(0, 3, 5));
    let a = e.sample_mip_at(0, 3, 5, 0);
    let b = e.sample_mip_at(0, 3, 5, 1);
    assert_eq!(Engine::blend_mips4([a, 0, a, 1], [b, 0, b, 2], 128)[0], Engine::blend_mips(a, b, 128));
    assert_eq!(Engine::blend_mips4([a, 0, a, 1], [b, 0, b, 2], 128)[2], Engine::blend_mips(a, b, 128));
}

#[test]
fn wall_lod_stays_sharp_up_close() {
    let (level, mix) = Engine::column_lod(0.15, 500, 0.72, 640);
    assert_eq!(level, 0);
    assert_eq!(mix, 0, "a near column must match an unfiltered sample");
    let (far, _) = Engine::column_lod(16.0, 12, 0.72, 640);
    assert!(far >= 3, "a distant column must use the mip chain, got {far}");
    let (capped, _) = Engine::column_lod(80.0, 1, 0.72, 320);
    assert!(capped <= 8);
}

#[test]
fn wall_shade_matches_one_pixel_at_a_time() {
    let colors = [Engine::pack(200, 10, 255, 128), 0x0102_0304, 0x00ff_ffff, 0];
    let light = [0.55, -0.4, 2.0];
    let batch = Engine::shade_rgb4(colors, 0.82, light);
    for i in 0..4 {
        assert_eq!(batch[i], Engine::shade_rgb(colors[i], 0.82, light));
    }
    let shared = [(3, 5), (259, 5), (-253, 261), (3, 5 + 512)];
    let mut e = arena();
    e.rebuild_mipmaps();
    assert_eq!(
        e.sample_mip4(0, shared, 0, 0),
        [e.sample(0, 3, 5); 4],
        "wrapped uvs that land on one texel must splat, not resample",
    );
}

#[test]
fn gpu_cast_hits_a_wall_and_packs_sprites() {
    hs_init(160, 100);
    hs_prepare_gpu();
    let cols = unsafe { std::slice::from_raw_parts(hs_gpu_cols(), 160) };
    assert!(cols.iter().any(|c| c.hit > 0.5), "the enclosed map must hit a wall");
    assert!(cols.iter().all(|c| c.perp.is_finite() && c.z.is_finite()));
    assert!(hs_gpu_sprite_count() > 0);
    for angle in [0.0, 1.2, core::f32::consts::PI] {
        eng().pa = angle;
        hs_prepare_gpu();
        let sprites = unsafe { std::slice::from_raw_parts(hs_gpu_sprites(), hs_gpu_sprite_count() as usize) };
        let view = gpu_scratch().view;
        let depth = |s: &GpuSprite| (s.x - view.px) * view.dir_x + (s.y - view.py) * view.dir_y;
        assert!(sprites.windows(2).all(|pair| depth(&pair[0]) >= depth(&pair[1])),
            "GPU sprites must paint far to near regardless of camera yaw");
    }
    assert_eq!(std::mem::size_of::<GpuCol>() / 4, 16);
    assert_eq!(std::mem::size_of::<GpuView>() / 4, 16);
    assert_eq!(std::mem::size_of::<GpuSprite>() / 4, 8);
}

#[test]
fn override_stays_dark_until_the_sector_node_is_used() {
    let mut e = arena();
    (e.px, e.py) = map::override_point(e.wave);
    e.bits = IN_USE;
    e.tick(1.0 / 60.0);
    assert_eq!(e.boss_intro, 0.0, "the console ignores USE before the node");
    let (x, y) = field::node_point(e.wave);
    e.px = x;
    e.py = y;
    e.bits = 0;
    e.tick(1.0 / 60.0);
    e.bits = IN_USE;
    e.tick(1.0 / 60.0);
    assert!(e.node_done);
    assert_eq!(e.hud.objective, 1);
}

#[test]
fn shield_blocks_the_front_and_takes_the_flank() {
    let mut e = arena();
    let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
    e.arm_shield(i);
    let hp = e.ents[i].hp;
    e.ents[i].face = 0.0;
    e.hurt_ent(i, 12, 10.0, 4.5);
    assert_eq!(e.ents[i].hp, hp, "the first front hit stays on the plate");
    assert!(e.ents[i].shield_hp < enemies::SHIELD_CAP, "the plate takes the hit");
    assert!(e.ents[i].bar_t > 1.0, "a clang shows the bar");
    e.hurt_ent(i, 12, 6.0, 4.5);
    assert!(e.ents[i].hp < hp, "a rear hit gets through");
}

#[test]
fn pyre_leaves_a_fire_patch() {
    let mut e = arena();
    e.set_w(6);
    e.weapon = 7;
    e.mag[7] = 6;
    e.fire();
    assert!(e.ents.iter().any(|en| en.kind == EK_FIREPATCH));
    assert_eq!(e.mag[7], 5);
}

#[test]
fn rifleman_and_gunner_die_from_the_front() {
    let mut e = arena();
    let rifle = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
    e.hurt_ent(rifle, 80, 6.0, 4.5);
    assert_eq!(e.ents[rifle].hp, 0, "an unshielded rifleman dies");
    let gunner = e.spawn_with_skin(EK_BRUTE, SKIN_GUNNER, 12.5, 4.5).unwrap();
    e.arm_shield(gunner);
    e.ents[gunner].face = 0.0;
    for _ in 0..40 {
        e.hurt_ent(gunner, 40, 14.0, 4.5);
    }
    assert_eq!(e.ents[gunner].hp, 0, "a shield gunner dies once the plate is gone");
}

#[test]
fn floor_props_drop_onto_the_floor() {
    let mut e = arena();
    e.h = 480;
    let console = e.spawn(EK_OVERRIDE_CONSOLE, 6.5, 4.5).unwrap();
    let kit = e.spawn(EK_MED, 7.5, 4.5).unwrap();
    e.tick(1.0 / 60.0);
    assert!(e.ents[console].zoff > 50.0, "console zoff {}", e.ents[console].zoff);
    assert!(e.ents[kit].zoff > 100.0, "pickup zoff {}", e.ents[kit].zoff);
}

#[test]
fn use_shuts_a_lamp_and_a_crate_drops_loot() {
    let mut e = arena();
    e.px = 4.5;
    e.py = 4.5;
    e.pa = 0.0;
    e.spawn(EK_LAMP, 6.0, 4.5);
    e.bits = 0;
    e.tick(1.0 / 60.0);
    e.bits = IN_USE;
    e.tick(1.0 / 60.0);
    let lamp = e.ents.iter().find(|en| en.kind == EK_LAMP).unwrap();
    assert!(lamp.timer < 0.0, "USE facing a lamp shuts it");
    let crate_i = e.spawn(EK_CRATE, 6.2, 4.5).unwrap();
    e.hurt_ent(crate_i, 20, 4.5, 4.5);
    assert!(e.ents.iter().any(|en| matches!(en.kind, EK_MED | EK_AMMO | EK_ARMOR)));
}

#[test]
fn a_fuel_barrel_leaves_fire() {
    let mut e = arena();
    let i = e.spawn_with_skin(EK_BARREL, 1, 8.0, 4.5).unwrap();
    e.hurt_ent(i, 40, 4.5, 4.5);
    assert!(e.ents.iter().any(|en| en.kind == EK_FIREPATCH || en.kind == EK_FLAME));
}

#[test]
fn hostiles_spawn_in_open_space_on_every_sector() {
    let mut e = Engine::new(160, 100);
    for wave in 1..=map::LEVEL_COUNT as i32 {
        e.wave = wave;
        e.build_map();
        e.place_ents();
        for ent in e.ents.iter().filter(|en| is_hostile_kind(en.kind) && en.hp > 0) {
            assert!(
                !e.circle_blocked(ent.x, ent.y, ent.radius),
                "wave {wave} {} stuck at {}, {}",
                ent.kind, ent.x, ent.y
            );
        }
    }
}

#[test]
fn chimera_fan_does_not_leave_a_pyre() {
    let mut e = arena();
    e.set_w(9);
    e.weapon = 10;
    e.mag[10] = 5;
    e.fire();
    assert!(e.ents.iter().any(|en| en.kind == EK_PROJ && en.effect_tick == 4.0));
    assert!(!e.ents.iter().any(|en| en.kind == EK_FIREPATCH));
}

#[test]
fn pyre_shot_flames_burn_then_expire() {
    let mut e = arena();
    e.set_w(6);
    e.weapon = 7;
    e.mag[7] = 6;
    let target = e.spawn(EK_HUSK, 6.2, 4.5).unwrap();
    e.ents[target].hp = 200;
    e.fire();
    e.tick(1.0 / 60.0);
    assert!(e.ents.iter().any(|en| en.kind == EK_FLAME));
    for _ in 0..40 { e.tick(1.0 / 60.0); }
    assert!(e.ents[target].hp < 200, "shot flames damage while they burn");
    assert!(e.ents.iter().any(|en| en.kind == EK_FLAME), "flames last more than a blink");
    for _ in 0..140 { e.tick(1.0 / 60.0); }
    assert!(!e.ents.iter().any(|en| en.kind == EK_FLAME), "shot flames are gone after a few seconds");
}

#[test]
fn warden_blast_reaches_twice_as_far() {
    let mut e = arena();
    e.weapon = 4;
    e.mag[4] = 4;
    e.pa = 0.0;
    e.set_cell(8, 4, 1);
    let far = e.spawn(EK_BRUTE, 6.2, 8.2).unwrap();
    e.ents[far].hp = 78;
    e.ents[far].stun = 5.0;
    e.fire();
    for _ in 0..30 { e.tick(1.0 / 60.0); }
    assert!(e.ents[far].hp < 78, "a target about four cells off the impact still takes the blast");
}

#[test]
fn override_pierces_the_lane_for_boss_damage() {
    let mut e = arena();
    e.set_w(7);
    e.weapon = 8;
    e.mag[8] = 4;
    let a = e.spawn(EK_HUSK, 6.5, 4.5).unwrap();
    let b = e.spawn(EK_HUSK, 8.5, 4.5).unwrap();
    e.fire();
    assert!(e.ents[a].hp <= 0 || e.ents[a].hp < 28 - 80);
    assert!(e.ents[b].hp <= 0 || e.ents[b].hp < 28 - 80, "the rail does not stop at the first body");
}

#[test]
fn lockdown_opens_from_outside_and_stays_shut_from_inside() {
    let mut e = Engine::new(160, 100);
    let cases = [
        (1, 36.5, 17.5, 36.5, 20.5, 36i32, 18i32),
        (2, 27.5, 23.5, 29.5, 23.5, 28, 23),
        (3, 29.5, 15.5, 31.5, 15.5, 30, 15),
    ];
    for (wave, ox, oy, ix, iy, dx, dy) in cases {
        e.wave = wave;
        e.build_map();
        e.door.fill(0.0);
        let spot = map::boss_spots(wave)[0];
        let _ = e.spawn(EK_BOSS, spot.0, spot.1);
        e.seal_lockdown();
        assert_eq!(e.cell(dx, dy), 8, "wave {wave} door must stay a door");
        let idx = dy as usize * MAP_W + dx as usize;
        assert_eq!(e.door[idx], 0.0);
        e.px = ix;
        e.py = iy;
        e.bits = 0;
        e.state = 0;
        e.tick(1.0 / 60.0);
        assert_eq!(e.door[idx], 0.0, "wave {wave} stays shut from inside");
        e.door.fill(0.0);
        e.px = ox;
        e.py = oy;
        e.tick(1.0 / 60.0);
        assert!(e.door[idx] > 0.0, "wave {wave} opens from outside");
        for ent in e.ents.iter_mut() { ent.kind = 0; }
    }
}

#[test]
fn owned_map_guns_become_supplies_on_the_next_sector() {
    let mut e = Engine::new(160, 100);
    e.set_w(0);
    e.set_w(6);
    e.wave = 1;
    e.build_map();
    map::place_level(&mut e);
    assert!(!e.ents.iter().any(|en| en.kind == EK_GUN2), "an owned breaker case must not return");
    assert!(!e.ents.iter().any(|en| en.kind == EK_GUN8), "an owned pyre case must not return");
    assert!(e.ents.iter().any(|en| en.kind == EK_GUN4), "an unowned gun still drops");
    assert!(e.ents.iter().any(|en| matches!(en.kind, EK_AMMO | EK_MED)));
}

#[test]
fn chimera_bolts_hurt_enemies_and_not_the_shooter() {
    let mut e = arena();
    e.set_w(9);
    e.weapon = 10;
    e.mag[10] = 5;
    e.health = 100;
    let target = e.spawn(EK_HUSK, 7.5, 4.5).unwrap();
    e.ents[target].hp = 80;
    e.ents[target].stun = 3.0;
    e.fire();
    for _ in 0..40 { e.tick(1.0 / 60.0); }
    assert_eq!(e.health, 100, "the specimen fan must not kill its owner");
    assert!(e.ents[target].hp < 80, "the fan still damages the target");
}
