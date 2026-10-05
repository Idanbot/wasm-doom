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

#[test]
fn authored_maps_are_geometrically_unique_and_have_safe_arrivals_and_spread_theme_enemies() {
    let mut e=Engine::new(160,100);
    let mut layouts=std::collections::HashSet::new();
    for wave in 1..=25 {
        e.wave=wave;e.door.fill(0.0);e.build_map();e.place_ents();
        // Ignore wall materials; doors are traversable geometry.
        let occupancy:Vec<bool>=e.map.iter().map(|&v|v!=0&&v!=8).collect();
        assert!(layouts.insert(occupancy),"duplicate room/wall geometry in sector {wave}");
        let themed:Vec<_>=e.ents.iter().filter(|en|en.hp>0&&en.skin==36+wave as u8).collect();
        assert!(themed.len()>=3,"sector {wave} needs distributed dedicated enemies: {}",themed.len());
        assert!(themed.iter().any(|a|themed.iter().any(|b|(a.x-b.x).powi(2)+(a.y-b.y).powi(2)>25.0)),"theme enemies clustered in {wave}");
        for en in e.ents.iter().filter(|en|en.hp>0&&is_hostile_kind(en.kind)) {
            assert!(!map::in_spawn_room(wave,en.x,en.y),"arrival room contains hostile in sector {wave}");
        }
        for en in &mut e.ents {en.kind=EK_NONE;}
        e.boss_spawned=false;e.maybe_spawn_boss();
        assert!(e.ents.iter().filter(|en|en.skin==36+wave as u8&&en.hp>0).count()>=2,"boss {wave} missing dedicated escorts");
    }
}
#[test]
fn missed_basic_weapons_reappear_randomly_but_owned_weapons_never_drop() {
    let mut e=Engine::new(160,100);
    let kinds=[EK_GUN2,EK_GUN3,EK_GUN4,EK_GUN5,EK_GUN6,EK_GUN7];
    for wave in 2..=25 {
        e.wave=wave;e.door.fill(0.0);e.build_map();e.place_ents();
        for kind in kinds {assert_eq!(e.ents.iter().filter(|en|en.kind==kind).count(),1,"missing recovery weapon {kind} in {wave}");}
        e.has_w2=true;e.has_w3=true;e.has_w4=true;e.has_w5=true;e.has_w6=true;e.has_w7=true;
        e.place_ents();assert!(!e.ents.iter().any(|en|kinds.contains(&en.kind)));
        e.has_w2=false;e.has_w3=false;e.has_w4=false;e.has_w5=false;e.has_w6=false;e.has_w7=false;
    }
}

#[test]
fn boss_entrances_have_25_distinct_choreographies_and_do_not_cross_walls() {
    let mut e=Engine::new(160,100);let mut signatures=std::collections::HashSet::new();
    for wave in 1..=25 {
        e.wave=wave;e.door.fill(0.0);e.build_map();let anchor=map::boss_spots(wave)[0];
        let mut signature=Vec::new();
        for stage in 0..5 {
            e.fx_n=0;e.boss_intro_effect(stage);
            assert!(e.fx_n>0,"missing boss entrance particles in {wave}/{stage}");
            for fx in &e.fx_q[..e.fx_n] {
                assert!(!e.blocked(fx.x.floor() as i32,fx.y.floor() as i32));
                assert!(e.los(anchor.0,anchor.1,fx.x,fx.y));
                assert!((16..24).contains(&fx.variant));
                signature.push((stage,fx.variant,((fx.x-anchor.0)*100.0) as i32,((fx.y-anchor.1)*100.0) as i32));
            }
        }
        assert!(signatures.insert(signature),"duplicate boss sequence in {wave}");
    }
}
#[test]
fn casing_gravity_uses_world_units_at_all_render_resolutions() {
    let mut e=Engine::new(160,100);e.map.fill(0);e.px=20.5;e.py=15.5;e.state=0;e.qa=true;
    let mut heights=Vec::new();
    for h in [100,200,400] {
        e.h=h;for ent in &mut e.ents {ent.kind=EK_NONE;}
        e.eject_casing();e.flush_fx();let i=e.ents.iter().position(|p|p.kind==EK_SPARK&&p.effect_tick==5.0).unwrap();
        for _ in 0..12 {e.tick(1.0/60.0);}
        heights.push((e.ents[i].stun,e.ents[i].zoff/h as f32));
    }
    for height in &heights {assert!((height.0-heights[0].0).abs()<0.001);assert!((height.1-heights[0].1).abs()<0.001);}
}
