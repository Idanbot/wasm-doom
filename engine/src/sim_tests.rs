//! Behavioural tests for combat and simulation: damage, fire, movement,
//! enemy AI and the effect queue.

use crate::testutil::*;
use crate::*;
use crate::abi::*;

#[test]
fn one_shot_enemy_states_reach_all_four_frames_without_looping() {
    let mut e = Engine::new(320, 200);
    let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
    for state in [ANIM_PAIN, ANIM_FIRE, ANIM_RELOAD, ANIM_DEAD, ANIM_SPECIAL] {
        e.ents[i].anim = ANIM_IDLE;
        e.ents[i].anim_lock = 0.0;
        set_anim(&mut e.ents[i], state, 0.24);
        let duration = e.ents[i].anim_lock;
        for frame in 0..4 {
            e.ents[i].anim_time = (frame as f32 + 0.1) / animation_fps(state);
            assert_eq!(anim_frame(&e.ents[i]), frame);
            assert!(e.ents[i].anim_time < duration);
        }
        e.ents[i].anim_time = duration + 0.5;
        assert_eq!(anim_frame(&e.ents[i]), 3);
    }
}

#[test]
fn each_sector_spawns_its_exclusive_enemy_with_the_correct_role() {
    let mut e = Engine::new(320, 200);
    for wave in 1..=50 {
        e.wave = wave;
        map::build_level(&mut e);
        for ent in &mut e.ents { ent.kind = EK_NONE; ent.hp = 0; }
        e.spawn_hostiles(1);
        let skin = 37 + map::level_index(wave) as u8;
        assert!(e.ents.iter().any(|ent| ent.skin == skin && ent.hp > 0));
        assert!(e.ents.iter().filter(|ent| ent.hp > 0 && (37..62).contains(&ent.skin)).all(|ent| ent.skin == skin));
        let base = enemies::combat_skin(skin);
        let ent = e.ents.iter().find(|ent| ent.skin == skin).unwrap();
        assert_eq!(combat::profile(skin, ent.kind).damage, combat::profile(base, ent.kind).damage);
        assert_eq!(enemies::armor_cap(ent.kind, skin), enemies::armor_cap(ent.kind, base));
    }
}

#[test]
fn enemy_cues_project_above_sprites_and_respect_cover() {
    let mut e = arena();
    e.pa = 0.0;
    e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5);
    let mut cues = [voices::EnemyCue::default(); ENT_N];
    assert_eq!(voices::snapshot(&e, &mut cues), 1);
    assert_eq!(std::mem::size_of::<voices::EnemyCue>(), 40);
    assert!((cues[0].screen_x - 0.5).abs() < 0.001);
    assert!(cues[0].screen_y < 0.5);
    assert_eq!(cues[0].sight, 1.0);
    e.set_cell(6, 4, 1);
    voices::snapshot(&e, &mut cues);
    assert_eq!(cues[0].sight, 0.0);
    e.pa = std::f32::consts::PI;
    voices::snapshot(&e, &mut cues);
    assert!(cues[0].screen_x < 0.0);
}

#[test]
fn ranged_windup_commits_aim_and_damage() {
    let mut e = arena();
    let i = e.spawn_with_skin(EK_HUSK, SKIN_MARKSMAN, 9.5, 4.5).unwrap();
    e.ents[i].timer = 0.0;
    e.tick(1.0 / 60.0);
    assert!(e.ents[i].effect_tick > 0.8);
    assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ));
    // Strafe after the warning: the marksman must shoot at the old aim.
    e.py = 7.5;
    for _ in 0..56 { e.tick(1.0 / 60.0); }
    let shot = e.ents.iter().find(|p| p.kind == EK_PROJ).expect("windup releases a shot");
    assert!(shot.vy.abs() < 0.01, "shots must not track dodges during windup");
    assert_eq!(shot.hp, 24);
}

#[test]
fn damage_interrupts_windup_and_cover_cancels_shot() {
    let mut e = arena();
    let i = e.spawn_with_skin(EK_HUSK, SKIN_RIFLEMAN, 8.5, 4.5).unwrap();
    e.ents[i].timer = 0.0;
    e.tick(1.0 / 60.0);
    e.hurt_ent(i, 1, e.px, e.py);
    assert_eq!(e.ents[i].effect_tick, 0.0);
    for _ in 0..20 { e.tick(1.0 / 60.0); }
    assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ));
    e.ents[i].timer = 0.0;
    e.tick(1.0 / 60.0);
    assert!(e.ents[i].effect_tick > 0.0);
    for y in 0..MAP_H { e.set_cell(6, y as i32, 1); }
    for _ in 0..30 { e.tick(1.0 / 60.0); }
    assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ), "cover cancels attack");
}

#[test]
fn hound_melee_can_be_dodged_and_does_not_fire() {
    let mut e = arena();
    let i = e.spawn_with_skin(EK_WRAITH, SKIN_HOUND, 5.3, 4.5).unwrap();
    e.ents[i].timer = 0.0;
    e.tick(1.0 / 60.0);
    assert!(e.ents[i].effect_tick > 0.0);
    e.px = 2.5;
    for _ in 0..20 { e.tick(1.0 / 60.0); }
    assert_eq!(e.health, 100);
    assert!(!e.ents.iter().any(|p| p.kind == EK_PROJ));
}

#[test]
fn martyr_chases_with_move_anim_then_detonates() {
    let mut e = arena();
    let i = e.spawn(EK_MARTYR, 6.5, 4.5).unwrap();
    assert_eq!(e.ents[i].skin, SKIN_MARTYR);
    for _ in 0..10 { e.tick(1.0 / 60.0); }
    assert_eq!(e.ents[i].anim, ANIM_MOVE, "chaser must play its move sheet");
    assert!(e.ents[i].x < 6.5, "martyr must float toward the player");
    e.ents[i].timer = 0.0;
    for _ in 0..120 {
        e.tick(1.0 / 60.0);
        // Break on the detonation tick itself: the blast FX reuses the
        // drone's freed slot, so slot state alone cannot mark the moment.
        if e.health < 100 { break; }
    }
    assert!(e.health < 100, "point-blank detonation must hurt");
    assert!(e.ents.iter().any(|p| p.kind == EK_IMPACT), "detonation leaves a blast mark");
    assert_ne!(e.ents[i].kind, EK_MARTYR, "detonation consumes the drone");
}

#[test]
fn martyr_gunfire_death_explodes_like_a_barrel() {
    let mut e = arena();
    let i = e.spawn(EK_MARTYR, 6.5, 4.5).unwrap();
    e.hurt_ent(i, 500, e.px, e.py);
    e.tick(1.0 / 60.0);
    assert!(e.ents.iter().any(|p| p.kind == EK_IMPACT));
}

#[test]
fn barrels_render_as_props_not_martyr_sheets() {
    let mut e = arena();
    let i = e.spawn(EK_BARREL, 6.5, 4.5).unwrap();
    assert_eq!(e.ents[i].skin, SKIN_NONE);
    let (tex, _, sheet4, _) = sprite_style(&e.ents[i]);
    assert_eq!((tex, sheet4), (T_BARREL, false));
}

#[test]
fn chimera_direct_splash_and_pool_damage_are_reduced_by_a_third() {
    let mut e=arena();
    e.fire_chimera();
    assert_eq!(e.ents.iter().filter(|v| v.kind==EK_PROJ).count(),5);
    assert!(e.ents.iter().filter(|v| v.kind==EK_PROJ).all(|v| v.hp==28));
    for ent in &mut e.ents {ent.kind=EK_NONE;}
    let target=e.spawn(EK_HUSK,8.5,4.5).unwrap();
    e.ents[target].hp=1000; e.ents[target].armor_hp=0;
    e.chimera_burst(8.5,4.5);
    assert_eq!(e.ents[target].hp,947);
    let pool=e.ents.iter().position(|v| v.kind==EK_FIREPATCH).unwrap();
    assert_eq!(e.ents[pool].hp,100);
    e.ents[pool].timer=8.0;
    for _ in 0..100 {
        e.ents[target].x=8.5; e.ents[target].y=4.5;
        e.ents[target].stun=100.0;
        e.tick(0.0625);
    }
    assert_eq!(e.ents[target].hp,813, "25 pool pulses deal exactly 134 damage");
}

#[test]
fn chimera_bolts_are_acid_and_pools_are_not_enemies() {
    let mut e = arena();
    let bolt = e.spawn(EK_PROJ, 4.0, 4.0).unwrap();
    e.ents[bolt].effect_tick = 4.0;
    e.ents[bolt].skin = SKIN_SUBJECT;
    let (tex, _, _, frame) = sprite_style(&e.ents[bolt]);
    assert_eq!((tex, frame), (T_PROJECTILE_NEW + 3, 0));
    let pool = e.spawn(EK_FIREPATCH, 5.0, 4.0).unwrap();
    e.ents[pool].skin = 2;
    let (tex, _, sheet, _) = sprite_style(&e.ents[pool]);
    assert_eq!((tex, sheet), (T_FLAME, true));
    assert!(tex < ENEMY_TEX_BASE);
}

#[test]
fn sprint_is_ten_percent_faster_and_caps_overdrive() {
    let mut e = arena();
    e.pa = 0.0;
    e.bits = IN_W | IN_SPRINT;
    e.power = field::POWER_OVERDRIVE;
    e.tick(0.08);
    let dist = ((e.px - 4.5).powi(2) + (e.py - 4.5).powi(2)).sqrt();
    assert!((dist - 3.35 * 1.32 * 0.08).abs() < 0.001);
}

#[test]
fn qa_heal_restores_health_and_revives() {
    let mut e = arena();
    e.qa = true;
    e.health = 12;
    e.state = 1;
    e.spawn(EK_HUSK, 6.5, 4.5).unwrap();
    e.qa_heal();
    assert_eq!((e.health, e.state), (100, 0));
    assert!(
        e.ents.iter().all(|en| !is_hostile_kind(en.kind)),
        "heal clears the converged crowd"
    );
    e.qa = false;
    e.health = 5;
    e.qa_heal();
    assert_eq!(e.health, 5, "heal is QA-only");
}

#[test]
fn splash_warns_when_wall_inside_blast_radius() {
    let mut e = arena();
    e.set_cell(6, 4, 1);
    e.px = 4.5;
    e.py = 4.5;
    e.pa = 0.0;
    e.weapon = 4;
    e.tick(1.0 / 60.0);
    assert_eq!(e.hud.splash, 1.0, "VLK-6 at a 2m wall must warn");
    e.pa = core::f32::consts::FRAC_PI_2;
    e.tick(1.0 / 60.0);
    assert_eq!(e.hud.splash, 0.0, "open lane must not warn");
    e.weapon = 0;
    e.pa = 0.0;
    e.tick(1.0 / 60.0);
    assert_eq!(e.hud.splash, 0.0, "hitscan guns never warn");
}

#[test]
fn theme_swap_follows_sector_cycle() {
    let mut e = Engine::new(160, 100);
    for slot in 0..16 {
        for px in e.theme_tex[slot * TEX * TEX..(slot + 1) * TEX * TEX].iter_mut() {
            *px = 0xFF000000 | (slot as u32);
        }
    }
    e.apply_theme(1);
    let tech = e.tex[T_TECH * TEX * TEX];
    let door = e.tex[T_DOOR * TEX * TEX];
    assert_eq!((tech, door), (0xFF000000, 0xFF000008));
    e.apply_theme(26);
    assert_eq!(e.tex[T_TECH * TEX * TEX], tech, "wave 26 reuses wave 1 theme");
    e.apply_theme(6);
    assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000007);
    e.apply_theme(2);
    assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000004);
    assert_eq!(e.tex[T_DOOR * TEX * TEX], 0xFF00000C);
    e.apply_theme(4);
    assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000003);
    e.apply_theme(5);
    assert_eq!(e.tex[T_TECH * TEX * TEX], 0xFF000006);
}

#[test]
fn every_sector_boss_entry_and_phase_is_safe() {
    let mut e = Engine::new(160, 100);
    for wave in 1..=map::LEVEL_COUNT as i32 {
        e.wave = wave;
        map::build_level(&mut e);
        map::place_level(&mut e);
        e.boss_spawned = false;
        e.boss_phase = 0;
        e.state = 0;
        e.maybe_spawn_boss();
        let i = e.ents.iter().position(|x| x.kind == EK_BOSS).expect("boss spawns");
        assert_eq!(e.ents[i].skin, map::boss_skin(wave));
        e.ents[i].hp = e.boss_max_health() / 2;
        e.tick(0.016);
        assert_eq!(e.boss_phase, 1);
        if wave == 11 {
            assert!(e.ents.iter().any(|x| x.kind == EK_MARTYR && x.hp > 0));
        }
    }
}

#[test]
fn every_boss_case_is_collected_by_world_overlap() {
    let mut e = arena();
    for wave in 1..=map::LEVEL_COUNT as i32 {
        e.state = 0;
        e.wave = wave;
        let kind = field::boss_case(wave);
        let slot = field::boss_slot(kind);
        assert!(is_weapon_item(kind), "wave {wave} reward isn't a weapon pickup");
        let index = e.spawn(kind, e.px, e.py).unwrap();
        e.tick(0.016);
        assert_eq!(e.ents[index].kind, EK_NONE, "wave {wave} case wasn't collected");
        assert_eq!(e.weapon as usize, slot, "wave {wave} equipped wrong reward");
        assert_eq!(e.state, 2, "wave {wave} didn't complete after collection");
    }
}

#[test]
fn expansion_weapons_fire_reload_and_last_reward_survives_endless_transition() {
    let mut e = arena();
    e.select_weapon(32);
    assert_eq!(e.weapon, 0, "locked rewards cannot be selected");
    for slot in 19..WEP_N {
        e.ents.iter_mut().for_each(|ent| ent.kind = EK_NONE);
        e.grant_slot(slot);
        e.state = 0; e.pickup_t = 0.0; e.cooldown = 0.0;
        e.fire();
        assert_eq!(e.mag[slot], MAG_SZ[slot] - 1);
        assert!(e.muzzle > 0.0 && e.cooldown > 0.0);
        e.mag[slot] = 0; e.ammo[slot] = 2; e.begin_reload();
        assert!(e.reload_t > 0.0);
        for _ in 0..60 { e.tick(0.08); }
        assert_eq!(e.mag[slot], 2, "slot {slot} must reload from reserve");
        assert_eq!(e.ammo[slot], 0);
    }
    e.wave = 25;
    e.weapon = 32;
    let save = capture_save(&e);
    e.extra_weapons = 0;
    apply_save(&mut e, &save);
    assert_eq!(e.weapon, 32);
    assert_eq!(e.extra_weapons, 16383);
    e.next_wave();
    assert_eq!(e.wave, 26);
    assert_eq!(map::level_index(e.wave), 0);
    assert_eq!(e.weapon, 32);
    assert_eq!(e.mag[32], MAG_SZ[32]);
}

#[test]
fn reinforcement_queue_grows_and_replenishes_without_spawning_on_player() {
    let mut e = arena();
    e.wave = 1000;
    e.spawn_hostiles(4);
    let remaining = e.pending_hostiles;
    assert!(remaining > 100);
    assert_eq!(map::living_hostiles(&e), campaign::ACTIVE_HOSTILES);
    assert!(!e.spawn_reinforcement(true));
    assert_eq!(e.pending_hostiles, remaining);
    for ent in &mut e.ents { if is_hostile_kind(ent.kind) { ent.kind = EK_NONE; break; } }
    assert!(e.spawn_reinforcement(true));
    assert_eq!(e.pending_hostiles, remaining - 1);
    let arrival = e.ents.iter().find(|ent| is_hostile_kind(ent.kind) && ent.stun > 0.0).unwrap();
    assert!((arrival.x - e.px).powi(2) + (arrival.y - e.py).powi(2) >= 49.0);
    assert!(arrival.timer >= 1.0);
}

#[test]
fn rocket_art_keeps_outgoing_model_when_camera_turns() {
    let mut e = arena();
    let i = e.spawn(EK_BOLT, 7.5, 4.5).unwrap();
    e.ents[i].hp = 50; e.ents[i].timer = 2.0; e.ents[i].vx = 5.0;
    e.tick(0.016);
    assert_eq!(sprite_style(&e.ents[i]).0, T_PLAYER_MISSILE);
    assert!(!sprite_style(&e.ents[i]).2);
    e.ents[i].hp = 95;
    assert_eq!(sprite_style(&e.ents[i]).0, T_PLAYER_MISSILE + 1);
    e.ents[i].vx = -5.0;
    e.tick(0.016);
    assert_eq!(sprite_style(&e.ents[i]).0, T_PLAYER_MISSILE + 1);
}

#[test]
fn every_sector_has_exclusive_surfaces_and_three_breakable_types() {
    let mut e = arena();
    for wave in 1..=25 {
        e.wave = wave;
        e.build_map();
        for ent in &mut e.ents { ent.kind = EK_NONE; }
        map::place_level(&mut e);
        let base = T_SECTOR_SURFACE + (wave as usize - 1) * 5;
        assert_eq!(e.wall_tex(1, 1, 1), base);
        assert_eq!(e.wall_tex(6, 1, 1), base + 1);
        assert_eq!(e.wall_tex(8, 1, 1), base + 2);
        assert_eq!(e.plane_sample(true, 4.5, 4.5, 0).0, base + 3);
        assert_eq!(e.plane_sample(false, 4.5, 4.5, 0).0, base + 4);
        for ty in 0..3 {
            let skin = 150 + ((wave - 1) * 3 + ty) as u8;
            let i = e.ents.iter().position(|ent| ent.kind == EK_CRATE && ent.skin == skin).expect("missing exclusive prop type");
            assert_eq!(sprite_style(&e.ents[i]).0, T_SECTOR_PROP + ((wave - 1) * 3 + ty) as usize);
            assert!(!e.blocked(e.ents[i].x as i32, e.ents[i].y as i32));
            let kills = e.kills;
            e.hurt_ent(i, 1000, e.px, e.py);
            assert_eq!(e.ents[i].hp, 0);
            assert_eq!(e.kills, kills, "breaking scenery must not count as an enemy kill");
        }
    }
    e.wave = 26;
    assert_eq!(e.wall_tex(1, 1, 1), T_SECTOR_SURFACE);
}

#[test]
fn all_boss_reward_guns_emit_unique_player_visuals_without_changing_element_semantics() {
    let mut e=arena();
    for slot in 8..WEP_N {
        for en in &mut e.ents {en.kind=EK_NONE;}
        e.weapon=slot as i32; e.mag[slot]=MAG_SZ[slot]; e.cooldown=0.0; e.reload_t=0.0;
        e.fire(); e.flush_fx();
        let travel: Vec<_>=e.ents.iter().filter(|en|matches!(en.kind,EK_RAY|EK_PROJ|EK_BOLT|EK_FLAME)).collect();
        assert!(!travel.is_empty(),"slot {slot} needs its own travel presentation");
        for en in travel {assert_eq!(sprite_style(en).0,T_BOSS_PROJECTILE+slot-8);}
    }
}

#[test]
fn swarm_launches_five_visible_homing_drones_and_keeps_allied_collision() {
    let mut e=arena();
    let target=e.spawn_with_skin(EK_HUSK,SKIN_RIFLEMAN,12.5,6.5).unwrap();
    e.ents[target].timer=10.0;
    e.weapon=23; e.mag[23]=12; e.fire();
    let drones:Vec<_>=e.ents.iter().enumerate().filter(|(_,en)|en.kind==EK_PROJ).map(|(i,_)|i).collect();
    assert_eq!(drones.len(),5);
    assert_eq!(drones.iter().map(|&i|e.ents[i].hp).sum::<i32>(),66,"more visible drones preserve volley damage");
    let before=e.ents[drones[0]].vy; let hp=e.health;
    for &i in &drones { assert_eq!(e.ents[i].effect_tick,6.0); assert_eq!(sprite_style(&e.ents[i]).0,T_BOSS_PROJECTILE+15); }
    e.tick(0.05);
    assert_ne!(e.ents[drones[0]].vy,before,"swarm must steer toward a hostile");
    assert_eq!(e.health,hp,"allied swarm cannot damage its shooter");
}

#[test]
fn enemy_attacks_use_isolated_type_specific_projectiles() {
    let mut e = arena();
    for skin in [0, 1, 3, 4, 7, 8, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 28, 29, 30, 31, 32, 33, 34, 35, 36] {
        for ent in &mut e.ents { ent.kind = EK_NONE; }
        let wave=(1..=25).find(|&wave|map::boss_skin(wave)==skin);
        e.boss_attack=boss_attacks::AttackState::new();
        let shooter = e.spawn_with_skin(if wave.is_some() {EK_BOSS}else{EK_WRAITH}, skin, 9.5, 4.5).unwrap();
        if let Some(wave)=wave {e.wave=wave;e.ents[shooter].timer=0.0;e.tick_boss_attack(shooter,0.01);e.tick_boss_attack(shooter,2.0);}
        e.ents[shooter].aim = core::f32::consts::PI;
        e.enemy_shoot(shooter);
        let shots: Vec<_> = e.ents.iter().filter(|ent| ent.kind == EK_PROJ).collect();
        assert!(!shots.is_empty(), "skin {skin} did not shoot");
        for shot in shots {
            let (texture, _, sheet, _) = sprite_style(shot);
            assert_eq!(texture, T_ENEMY_PROJECTILE + skin as usize, "skin {skin} shares or mixes another projectile");
            assert!(!sheet, "enemy shots must sample one isolated complete image");
        }
    }
}

#[test]
fn new_boss_attacks_have_distinct_summon_phase_and_shield_behaviors() {
    for wave in [16,20,24] {
        let mut e=arena();e.wave=wave;e.px=11.5;e.py=4.5;
        let boss=e.spawn_with_skin(EK_BOSS,map::boss_skin(wave),7.5,4.5).unwrap();
        e.ents[boss].timer=0.0;e.tick_boss_attack(boss,0.01);
        assert!(!e.ents.iter().any(|ent|ent.kind==EK_PROJ));
        if wave==24 {assert!(e.ents[boss].shield_hp>0);}
        e.py=7.5;e.tick_boss_attack(boss,2.0);
        if wave==16 {assert_eq!(e.ents.iter().filter(|ent|ent.kind==EK_MARTYR).count(),1);}
        if wave==20 {assert!((e.ents[boss].x-11.5).hypot(e.ents[boss].y-4.5)<0.6);}
        assert!(e.ents.iter().any(|ent|ent.kind==EK_PROJ&&ent.skin==100+map::boss_skin(wave)));
        assert_eq!(e.ents[boss].shield_hp,0,"recovery opens shield");
    }
}

#[test]
fn echo_reward_unlocks_fires_and_survives_save() {
    let mut e = arena();
    assert!(!e.has_w(17));
    e.pickup(EK_GUN19);
    assert!(e.has_w(17));
    assert_eq!(e.weapon, 18);
    assert_eq!(e.mag[18], MAG_SZ[18]);
    e.pa = 0.0;
    e.state = 0;
    e.cooldown = 0.0;
    let target = e.spawn(EK_BRUTE, 6.5, 4.5).unwrap();
    let before = e.ents[target].hp;
    e.fire();
    assert!(e.ents[target].hp < before);
    assert_eq!(e.mag[18], MAG_SZ[18] - 1);
    let save = capture_save(&e);
    let mut restored = arena();
    apply_save(&mut restored, &save);
    assert!(restored.has_w(17));
    assert_eq!(restored.mag[18], e.mag[18]);
    assert_eq!(restored.weapon, 18);
}

#[test]
fn run_save_layout_matches_ts_side() {
    // Pinned against SAVE_SIZE/SAVE_AMMO_BASE/SAVE_MAG_BASE in
    // src/game/save-abi.ts. Update both files together when WEP_N changes.
    assert_eq!(core::mem::size_of::<RunSave>(), 300);
    let s = RunSave {
        wave: 0, health: 0, armor: 0, weapon: 0, flags: 0, kills: 0,
        secrets: 0, elapsed_ms: 0, ammo: [0; WEP_N], mag: [0; WEP_N], extra_weapons: 0,
    };
    let base = &s as *const RunSave as usize;
    assert_eq!(&s.ammo as *const _ as usize - base, 32);
    assert_eq!(&s.mag as *const _ as usize - base, 164);
}

#[test]
fn smoke_cloud_fills_then_clears_after_ten_seconds() {
    let mut e = arena();
    e.spawn_smoke_cloud(e.px, e.py);
    e.rebuild_smoke_grid();
    assert!(e.smoke_grid.iter().any(|d| *d > 0.2), "a fresh cloud must occupy the cell");
    assert!(e.smoke_grid.iter().all(|d| *d <= 0.56), "thin smoke must never block the view");
    for _ in 0..700 { e.age_smoke(1.0 / 60.0); }
    assert!(e.smokes.iter().all(|s| s.age < 0.0), "smoke must be gone after 10 seconds");
}

#[test]
fn next_wave_stays_within_entity_budget() {
    let mut e = Engine::new(160, 100);
    for _ in 0..10 { e.next_wave(); }
    assert_eq!(e.wave, 11);
    assert!(map::living_hostiles(&e) <= 60, "wave spawn must leave FX slots free");
    e.maybe_spawn_boss();
    let boss = e.ents.iter().find(|x| x.kind == EK_BOSS).expect("boss spawns");
    assert!(boss.hp <= 6000, "late-wave boss must stay killable");
}

#[test]
fn deep_waves_scale_hostile_health() {
    let mut e = arena();
    e.wave = 25;
    let base = e.spawn(EK_HUSK, 6.5, 4.5).unwrap();
    let base_hp = e.ents[base].hp;
    e.wave = 26;
    let next = e.spawn(EK_HUSK, 7.5, 4.5).unwrap();
    assert!(e.ents[next].hp > base_hp);
    assert!(e.ents[next].hp <= (base_hp as f32 * 1.06).ceil() as i32);
    e.wave = 1000;
    let deep = e.spawn(EK_HUSK, 8.5, 4.5).unwrap();
    assert!(e.ents[deep].hp > base_hp * 20);
}

#[test]
fn waves_are_endless_and_keep_scaling() {
    let mut e = Engine::new(160, 100);
    e.wave = 25;
    e.next_wave();
    assert_eq!(e.wave, 26);
    assert_eq!(map::level_index(e.wave), 0);
    assert!(map::living_hostiles(&e) <= campaign::ACTIVE_HOSTILES);
    let hp = e.boss_max_health();
    e.wave = 999;
    e.next_wave();
    assert_eq!(e.wave, 1000);
    assert!(e.boss_max_health() > 6000 && e.boss_max_health() > hp);
    assert!(e.pending_hostiles > 0, "uncapped total is queued within the active budget");
    assert!(map::living_hostiles(&e) <= campaign::ACTIVE_HOSTILES);
}

#[test]
fn warden_fires_one_round_and_blast_respects_wall() {
    let mut e = arena();
    e.weapon = 4;
    e.mag[4] = MAG_SZ[4];
    e.pa = 0.0;
    e.set_cell(7, 4, 1);
    let exposed = e.spawn(EK_BRUTE, 6.5, 5.5).unwrap();
    let covered = e.spawn(EK_BRUTE, 8.5, 4.5).unwrap();
    for i in [exposed, covered] { e.ents[i].stun = 5.0; }
    e.fire();
    e.fire();
    assert_eq!(e.mag[4], MAG_SZ[4] - 1, "cooldown must prevent a second shot");
    assert_eq!(e.ents.iter().filter(|p| p.kind == EK_BOLT).count(), 1);
    for _ in 0..24 { e.tick(1.0 / 60.0); }
    assert!(e.ents[exposed].hp < 78);
    assert_eq!(e.ents[covered].hp, 78);
    assert!(!e.ents.iter().any(|p| p.kind == EK_FIREPATCH), "missiles no longer leave floating flame patches");
    assert!(e.ents.iter().any(|p| p.kind == EK_IMPACT && p.effect_tick == 2.0));
}

#[test]
fn empty_magazine_waits_for_manual_reload_without_dry_fire_animation() {
    let mut e = arena();
    e.mag[0] = 0;
    e.ammo[0] = 9;
    e.tick(1.0 / 60.0);
    assert_eq!(e.hud.weap_frame, 3, "reserve ammo keeps the empty-mag pose");
    e.fire();
    assert_eq!(e.reload_t, 0.0);
    e.tick(1.0 / 60.0);
    assert_eq!(e.hud.weap_frame, 3, "an empty trigger pull keeps the empty-mag pose");
    assert_eq!(e.events & EV_EMPTY, 0);
    e.begin_reload();
    assert!(e.reload_t > 0.0, "reload remains available on explicit input");

    e.reload_t = 0.0;
    e.ammo[0] = 0;
    e.tick(1.0 / 60.0);
    assert_eq!(e.hud.weap_frame, 4, "zero magazine and reserve use no-ammo pose");
}

#[test]
fn empty_trigger_repeats_at_bounded_cadence_without_firing_animation() {
    let mut e = arena();
    e.mag[0] = 0;
    e.bits = IN_FIRE;
    let mut clicks = 0;
    for _ in 0..60 {
        e.tick(1.0 / 60.0);
        clicks += e.sound_cues.iter().filter(|c| c.kind == 21.0).count();
        assert_eq!(e.events & EV_FIRE, 0);
        assert_eq!(e.hud.weap_frame, 3);
    }
    assert!((4..=5).contains(&clicks), "empty trigger should click about 4.5 times/sec: {clicks}");
    e.bits = 0;
    for _ in 0..30 {
        e.tick(1.0 / 60.0);
        assert!(!e.sound_cues.iter().any(|c| c.kind == 21.0));
    }
    assert_eq!(e.mag[0], 0);
}

#[test]
fn casings_tumble_bounce_sound_then_settle_and_expire() {
    let mut e = arena();
    e.eject_casing(); e.flush_fx();
    let i = e.ents.iter().position(|p| p.kind == EK_SPARK && p.effect_tick == 5.0).unwrap();
    assert_eq!(sprite_style(&e.ents[i]).0, T_CASING);
    let initial = e.ents[i].zoff;
    e.tick(1.0/60.0);
    assert!(e.ents[i].zoff < initial, "initial ejection rises");
    let mut clicks = 0;
    for _ in 0..180 {
        e.tick(1.0/60.0);
        clicks += e.sound_cues.iter().filter(|c| c.kind == 23.0).count();
        assert!(!e.circle_blocked(e.ents[i].x, e.ents[i].y, 0.04));
    }
    assert!((1..=3).contains(&clicks), "landing cues are physical and bounded: {clicks}");
    assert_eq!(e.ents[i].shield, 3);
    assert_eq!(e.ents[i].vx, 0.0); assert_eq!(e.ents[i].vy, 0.0);
    assert_eq!(sprite_style(&e.ents[i]).3, 0, "settled casing stops tumbling");
    for _ in 0..200 { e.tick(1.0/60.0); }
    assert_eq!(e.ents[i].kind, EK_NONE);
}

#[test]
fn cosmetic_casings_are_capped_and_shotgun_uses_shell_landing_cue() {
    let mut e = arena(); e.weapon = 1;
    for _ in 0..40 { e.eject_casing(); e.flush_fx(); }
    assert_eq!(e.ents.iter().filter(|p| p.kind == EK_SPARK && p.effect_tick == 5.0).count(), 24);
    let mut shell_sound = false;
    for _ in 0..120 {
        e.tick(1.0/60.0);
        shell_sound |= e.sound_cues.iter().any(|c| c.kind == 23.0 && c.variant == 1.0);
    }
    assert!(shell_sound);
}

#[test]
fn br12_reload_feeds_one_shell_per_step() {
    let mut e = arena();
    e.weapon = 1;
    e.set_w(0);
    e.mag[1] = 2;
    e.ammo[1] = 3;
    e.begin_reload();
    assert!((e.reload_t - 0.42).abs() < 0.001);
    for expected in 3..=5 {
        while e.mag[1] < expected { e.tick(1.0 / 60.0); }
        assert_eq!(e.mag[1], expected);
        assert_eq!(e.ammo[1], 5 - expected);
        assert_eq!(e.reload_t > 0.0, expected < 5);
    }
}

#[test]
fn br12_trigger_interrupts_shell_feed_after_a_shell_is_loaded() {
    let mut e = arena();
    e.weapon = 1;
    e.set_w(0);
    e.mag[1] = 0;
    e.ammo[1] = 3;
    e.begin_reload();
    e.fire();
    assert_eq!(e.mag[1], 0, "empty tube cannot interrupt the first shell");
    assert!(e.reload_t > 0.0);
    while e.mag[1] == 0 { e.tick(1.0 / 60.0); }
    assert!(e.reload_t > 0.0, "the next shell is being loaded");
    e.fire();
    assert_eq!(e.mag[1], 0, "loaded shell fires");
    assert_eq!(e.reload_t, 0.0, "trigger cancels the shell-feed sequence");
    assert_ne!(e.events & EV_FIRE, 0);
}

#[test]
fn player_missile_smoke_trail_expires_within_one_second() {
    let mut e = arena();
    e.weapon = 4;
    e.mag[4] = 1;
    e.fire();
    e.tick(0.1);
    assert!(e.ents.iter().any(|en| en.kind == EK_SMOKE), "missile leaves a small puff");
    for en in &mut e.ents { if en.kind == EK_BOLT { en.kind = EK_NONE; } }
    for _ in 0..70 { e.tick(1.0 / 60.0); }
    assert!(!e.ents.iter().any(|en| en.kind == EK_SMOKE), "trail disappears after flight");
}

#[test]
fn projectile_sweep_hits_between_steps() {
    let mut e = arena();
    let i = e.spawn(EK_PROJ, 3.5, 4.5).unwrap();
    e.ents[i].vx = 30.0;
    e.ents[i].hp = 24;
    e.tick(0.08);
    assert_eq!(e.health, 76, "swept hit must use projectile damage");
    assert_eq!(e.ents[i].kind, EK_NONE);
}

#[test]
fn effects_keep_their_identity_over_lifetime() {
    let mut e = arena();
    for (kind, texture, frame) in [(EK_SMOKE, T_FLAME, 1), (EK_FIREPATCH, T_FLAME, 0), (EK_BOLT, T_PLAYER_MISSILE, 0)] {
        let i = e.spawn(kind, 5.5, 5.5).unwrap();
        for age in [0.0, 0.3, 0.7, 1.1] {
            e.ents[i].frame = age;
            let style = sprite_style(&e.ents[i]);
            assert_eq!((style.0, style.3), (texture, frame));
        }
    }
}

#[test]
fn diagonal_motion_slides_along_wall() {
    let mut e = arena();
    e.set_cell(5, 4, 1);
    e.px = 4.75;
    e.try_move(5.0, 4.6);
    assert!((e.py - 4.6).abs() < 0.001, "wall must not stop tangential motion");
    assert!(!e.circle_blocked(e.px, e.py, e.pr));
}

#[test]
fn blast_respects_cover_and_preserves_pickups() {
    let mut e = arena();
    e.set_cell(5, 4, 1);
    e.px = 6.2;
    let enemy = e.spawn(EK_HUSK, 6.2, 4.5).unwrap();
    let hp = e.ents[enemy].hp;
    let med = e.spawn(EK_MED, 4.4, 4.5).unwrap();
    e.explode(4.5, 4.5, 3.0, 80.0);
    assert_eq!(e.health, 100, "cover must protect player");
    assert_eq!(e.ents[enemy].hp, hp, "cover must protect enemies");
    assert_eq!(e.ents[med].kind, EK_MED, "explosions must not erase supplies");
}

#[test]
fn full_health_does_not_consume_medkit() {
    let mut e = arena();
    let med = e.spawn(EK_MED, e.px, e.py).unwrap();
    e.tick(1.0 / 60.0);
    assert_eq!(e.ents[med].kind, EK_MED);
    e.health = 70;
    e.tick(1.0 / 60.0);
    assert_eq!(e.health, 100);
    assert_eq!(e.ents[med].kind, EK_NONE);
}

#[test]
fn restart_keeps_uploaded_art() {
    hs_init(160, 100);
    eng().tex[123] = 0x12345678;
    eng().health = 12;
    hs_restart();
    assert_eq!(eng().tex[123], 0x12345678);
    assert_eq!(eng().health, 100);
}

#[test]
fn strafing_and_diagonal_speed_match_camera() {
    let mut e = arena();
    e.bits = IN_A;
    e.tick(1.0 / 60.0);
    assert!(e.py < 4.5);
    e.bits = IN_D;
    e.tick(1.0 / 60.0);
    assert!((e.py - 4.5).abs() < 0.001);
    e.bits = IN_W | IN_D;
    e.tick(1.0 / 60.0);
    let distance = ((e.px - 4.5).powi(2) + (e.py - 4.5).powi(2)).sqrt();
    assert!((distance - 3.35 / 60.0).abs() < 0.001);
}

#[test]
fn strafing_enemy_stays_out_of_walls() {
    let mut e = arena();
    e.px = 4.3;
    e.py = 6.5;
    e.set_cell(3, 4, 1);
    let i = e.spawn(EK_WRAITH, 4.3, 4.5).unwrap();
    for _ in 0..20 { e.tick(1.0 / 60.0); }
    let enemy = e.ents[i];
    assert!(!e.circle_blocked(enemy.x, enemy.y, enemy.radius));
}

#[test]
fn mipmaps_average_texture_detail_and_render_at_extreme_pitch() {
    let mut e = arena();
    for y in 0..TEX {
        for x in 0..TEX {
            e.tex[y * TEX + x] = if (x + y) % 2 == 0 { 0xffffffff } else { 0xff000000 };
        }
    }
    e.rebuild_mipmaps();
    assert_eq!(e.sample_lod(0, 0, 0, 2.0) & 0xffffff, 0x7f7f7f);
    for pitch in [-0.42, 0.0, 0.42] {
        e.pitch = pitch;
        e.render();
        assert!(e.fb.iter().any(|c| c & 0xffffff != 0));
    }
}

#[test]
fn shotgun_has_close_range_damage_and_stagger() {
    let mut e = arena();
    e.weapon = 1;
    let near = e.spawn(EK_BRUTE, 6.5, 4.5).unwrap();
    assert!(e.hitscan(0.0, 7, 11.0));
    let close_damage = 78 - e.ents[near].hp;
    assert!(e.ents[near].stun >= 0.3);
    e.ents[near].kind = EK_NONE;
    let far = e.spawn(EK_BRUTE, 12.5, 4.5).unwrap();
    assert!(e.hitscan(0.0, 7, 11.0));
    assert!(78 - e.ents[far].hp < close_damage);
    assert_eq!(e.ents[far].stun, 0.0);
}

#[test]
fn ripper_spread_builds_and_recovers() {
    let mut e = arena();
    e.weapon = 2;
    e.mag[2] = 32;
    e.bits = IN_FIRE;
    for _ in 0..40 { e.tick(1.0 / 60.0); }
    assert!(e.spread > 0.12);
    e.bits = 0;
    for _ in 0..65 { e.tick(1.0 / 60.0); }
    assert_eq!(e.spread, 0.0);
}

#[test]
fn lance_penetrates_enemies_but_not_cover() {
    let mut e = arena();
    let mut ids = Vec::new();
    for x in [6.5, 8.5, 10.5, 12.5] {
        let i = e.spawn(EK_BRUTE, x, 4.5).unwrap();
        e.ents[i].hp = 500;
        ids.push(i);
    }
    e.set_cell(9, 4, 1);
    e.fire_lance();
    assert_eq!(e.ents[ids[0]].hp, 340);
    assert_eq!(e.ents[ids[1]].hp, 388);
    assert_eq!(e.ents[ids[2]].hp, 500);
    e.set_cell(9, 4, 0);
    e.fire_lance();
    assert_eq!(e.ents[ids[2]].hp, 422);
    assert_eq!(e.ents[ids[3]].hp, 500, "three-enemy penetration cap");
}

#[test]
fn pyre_patches_burn_expire_merge_and_stay_bounded() {
    let mut e = arena();
    let target = e.spawn(EK_HUSK, 5.0, 4.5).unwrap();
    e.ents[target].hp = 500;
    e.ignite(4.5, 4.5);
    e.ignite(4.6, 4.5);
    assert_eq!(e.ents.iter().filter(|e| e.kind == EK_FIREPATCH).count(), 1);
    for _ in 0..30 { e.tick(1.0 / 60.0); }
    assert!(e.ents[target].hp <= 490);
    for _ in 0..120 { e.tick(1.0 / 60.0); }
    assert_eq!(e.ents.iter().filter(|en| en.kind == EK_FIREPATCH).count(), 1, "the trail is still burning at two seconds");
    for _ in 0..500 { e.tick(1.0 / 60.0); }
    assert_eq!(e.ents.iter().filter(|en| en.kind == EK_FIREPATCH).count(), 0, "the trail is gone after ten seconds");
    for x in 1..30 { e.ignite(x as f32, 10.5); }
    assert_eq!(e.ents.iter().filter(|e| e.kind == EK_FIREPATCH).count(), 12);
}

#[test]
fn lighting_is_colored_occluded_and_does_not_advance_simulation() {
    let mut e = arena();
    e.spawn(EK_LAMP, 4.5, 4.5);
    for y in 0..MAP_H { e.set_cell(5, y as i32, 1); }
    let rng = e.rng;
    e.update_lighting();
    let warm = e.light_at(4.5, 4.5);
    assert!(warm[0] > warm[1] && warm[1] > warm[2]);
    let covered = e.light_at(6.5, 4.5);
    assert!(covered[0] <= 0.0);
    e.render();
    assert_eq!(e.rng, rng);
    assert_eq!(e.elapsed, 0.0);
}

#[test]
fn horizontal_planes_preserve_wall_pixels() {
    let mut e = arena();
    for y in 0..MAP_H { e.set_cell(8, y as i32, 2); }
    let wall = e.wall_tex(2, 8, 4);
    e.tex[wall * TEX * TEX..(wall + 1) * TEX * TEX].fill(Engine::pack(200, 0, 0, 255));
    e.tex[T_SKULL * TEX * TEX..(T_SKULL + 1) * TEX * TEX].fill(Engine::pack(200, 0, 0, 255));
    e.rebuild_mipmaps();
    e.render();
    let middle = e.fb[e.h / 2 * e.w + e.w / 2];
    assert!(middle & 255 > 40);
    assert_eq!((middle >> 16) & 255, 0);
    assert!(e.fb.iter().all(|c| *c != 0));
}

#[test]
fn armor_soaks_two_thirds_then_iframes_and_death_stick() {
    let mut e = arena();
    e.armor = 30;
    e.damage_player(15);
    assert_eq!(e.armor, 20, "armor soaks two thirds, capped by the vest");
    assert_eq!(e.health, 95);
    assert!(e.events & EV_HURT != 0);
    e.damage_player(40);
    assert_eq!(e.health, 95, "iframes must swallow the follow-up");
    assert_eq!(e.armor, 20);
    e.iframes = 0.0;
    e.armor = 0;
    e.damage_player(200);
    assert_eq!(e.health, 0);
    assert_eq!(e.state, 1);
    assert!(e.events & EV_DIE != 0);
    e.damage_player(20);
    assert_eq!(e.health, 0, "a downed player takes no further damage");
    assert_eq!(e.state, 1);
}

#[test]
fn reload_draws_reserve_and_blocks_the_trigger() {
    let mut e = arena();
    e.mag[0] = MAG_SZ[0];
    e.ammo[0] = 4;
    e.bits = IN_RELOAD;
    e.tick(1.0 / 60.0);
    assert_eq!(e.reload_t, 0.0, "a full magazine must not reload");
    assert_eq!(e.ammo[0], 4);

    e.bits = 0;
    e.tick(1.0 / 60.0);
    e.mag[0] = 2;
    e.ammo[0] = 0;
    e.bits = IN_RELOAD;
    e.tick(1.0 / 60.0);
    assert_eq!(e.reload_t, 0.0, "an empty reserve must not start a reload");

    e.bits = 0;
    e.tick(1.0 / 60.0);
    e.ammo[0] = 5;
    e.bits = IN_RELOAD;
    e.tick(1.0 / 60.0);
    assert!((e.reload_t - RELOAD_T[0]).abs() < 0.001);
    assert!(e.events & EV_RELOAD != 0);
    e.bits = IN_FIRE;
    e.tick(1.0 / 60.0);
    assert_eq!(e.mag[0], 2, "the trigger is dead while reloading");
    e.bits = 0;
    while e.reload_t > 0.0 {
        e.tick(1.0 / 60.0);
    }
    assert_eq!(e.mag[0], 7);
    assert_eq!(e.ammo[0], 0);

    e.mag[0] = 10;
    e.ammo[0] = 1;
    e.begin_reload();
    while e.reload_t > 0.0 {
        e.tick(1.0 / 60.0);
    }
    assert_eq!(e.mag[0], 11, "reload takes only what the reserve holds");
    assert_eq!(e.ammo[0], 0);

    e.mag[0] = 0;
    e.bits = IN_FIRE;
    e.tick(1.0 / 60.0);
    assert_eq!(e.events & EV_EMPTY, 0);
    assert_eq!(e.reload_t, 0.0);
    assert_eq!(e.mag[0], 0);
}
