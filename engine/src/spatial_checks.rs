//! Regression checks for live placement and the HUD threat total.
use crate::*;
fn assert_clear(e: &Engine) {
    for en in e.ents.iter().filter(|en| en.hp>0 && is_hostile_kind(en.kind)) {
        assert!(!e.circle_blocked(en.x,en.y,en.radius),"sector {} kind {} at {},{} radius {}",e.wave,en.kind,en.x,en.y,en.radius);
    }
}
#[test]
fn all_sectors_boss_phases_and_support_stay_outside_walls() {
    let mut e=Engine::new(160,100);
    for wave in 1..=25 {
        e.wave=wave;e.tactical=tactical::Tactical::new();for en in &mut e.ents {en.kind=EK_NONE;}e.door.fill(0.0);e.build_map();e.place_ents();
        e.state=0;e.health=100;e.qa=true;e.boss_spawned=false;e.boss_phase=0;
        assert_clear(&e);e.maybe_spawn_boss();assert_clear(&e);
        for phase in 1..=2 {
            let hp=e.boss_max_health();
            for en in &mut e.ents {if en.kind==EK_BOSS {en.hp=if phase==1 {hp*2/3}else{hp/3};}}
            e.tick(0.01);assert_clear(&e);
            assert_eq!(e.hud.living as usize,map::living_hostiles(&e)+e.pending_hostiles,"stale counter in sector {wave} phase {phase}");
            for step in 0..90 {
                if step%30==0 {let (x,y,_)=map::player_start(wave);e.px=x;e.py=y;e.health=100;}
                e.tick(0.05);assert_clear(&e);
                assert_eq!(e.hud.living as usize,map::living_hostiles(&e)+e.pending_hostiles);
            }
        }
    }
}
#[test]
fn a_closed_lockdown_door_ejects_overlapping_hostiles() {
    let mut e=Engine::new(160,100);let &(x,y)=field::lockdown_doors(1).first().unwrap();
    e.door[y as usize*MAP_W+x as usize]=1.0;
    let i=e.spawn(EK_BRUTE,x as f32+0.5,y as f32+0.5).unwrap();e.ents[i].stun=1.0;
    e.seal_lockdown();e.tick(0.01);assert_clear(&e);
}
#[test]
fn endless_arrivals_and_live_counts_agree() {
    let mut e=Engine::new(160,100);
    for wave in [26,50,100,1000] {
        e.wave=wave;for en in &mut e.ents {en.kind=EK_NONE;}e.door.fill(0.0);e.tactical=tactical::Tactical::new();e.build_map();e.place_ents();e.qa=true;e.state=0;e.boss_spawned=false;e.boss_intro=0.0;
        for _ in 0..5 {e.tick(0.01);assert_clear(&e);assert_eq!(e.hud.living as usize,map::living_hostiles(&e)+e.pending_hostiles);}
    }
}

#[test]
fn threat_total_excludes_props_corpses_and_projectiles_and_includes_incoming_boss() {
    let mut e=Engine::new(160,100);e.map.fill(0);for en in &mut e.ents {en.kind=EK_NONE;}
    e.qa=true;e.state=0;e.pending_hostiles=0;e.boss_spawned=false;e.boss_intro=0.0;
    let i=e.spawn(EK_HUSK,10.5,10.5).unwrap();
    e.spawn(EK_CRATE,11.5,10.5);e.spawn(EK_PROJ,12.5,10.5);
    e.tick(0.01);assert_eq!(e.hud.living,1);
    e.ents[i].hp=0;e.tick(0.01);assert_eq!(e.hud.living,0);
    e.boss_intro=2.0;e.tick(0.01);assert_eq!(e.hud.living,1);
}

#[test]
fn hostile_spawn_relocates_deep_wall_points_and_rejects_fully_solid_maps() {
    let mut e=Engine::new(160,100);e.map.fill(1);for en in &mut e.ents {en.kind=EK_NONE;}
    assert!(e.spawn(EK_HUSK,4.5,4.5).is_none());
    e.map[20*MAP_W+30]=0;
    let i=e.spawn(EK_HUSK,4.5,4.5).unwrap();
    assert_eq!((e.ents[i].x,e.ents[i].y),(30.5,20.5));assert_clear(&e);
}
