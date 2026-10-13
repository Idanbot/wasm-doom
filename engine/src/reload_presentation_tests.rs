use crate::*;

fn feeds(e:&Engine)->usize {e.ents.iter().filter(|p|p.kind==EK_SPARK&&p.effect_tick==5.0).count()}
#[test]
fn feed_removal_crosses_its_pose_once_at_every_weapon_and_tick_rate() {
    let mut e=Engine::new(160,100);e.qa=true;e.state=0;
    for slot in 0..WEP_N { for dt in [1.0/120.0,1.0/30.0,0.08] {
        for p in &mut e.ents {p.kind=EK_NONE;}e.fx_n=0;e.weapon=slot as i32;e.reload_t=0.0;
        e.mag[slot]=0;e.ammo[slot]=MAG_SZ[slot]*2;e.begin_reload();e.flush_fx();assert_eq!(feeds(&e),0);
        let rng=e.rng;
        while e.reload_t>e.reload_dur*(1.0-RELOAD_EJECT_PROGRESS) {e.tick(dt);}
        assert_eq!(feeds(&e),usize::from(slot!=1),"slot {slot}, dt {dt}");
        if slot!=1 {assert_eq!(e.hud.weap_frame,10,"removal must stay inside its authored pose");}
        for _ in 0..4 {e.tick(dt);}
        assert_eq!(feeds(&e),usize::from(slot!=1),"duplicate removal in slot {slot}");
        assert_eq!(e.rng,rng,"reload cosmetics consume combat RNG");
    }}
}
#[test]
fn swap_before_removal_cancels_the_feed_and_after_removal_preserves_identity() {
    let mut e=Engine::new(160,100);e.qa=true;e.state=0;e.owned[6]=true;
    for before in [true,false] {
        for p in &mut e.ents {p.kind=EK_NONE;}e.weapon=7;e.mag[7]=0;e.ammo[7]=10;e.begin_reload();
        if !before {while e.reload_t>e.reload_dur*(1.0-RELOAD_EJECT_PROGRESS) {e.tick(0.01);}}
        e.select_weapon(0);e.tick(0.08);assert_eq!(feeds(&e),usize::from(!before));
        if !before {assert!(e.ents.iter().any(|p|p.kind==EK_SPARK&&p.skin==2));}
    }
}

#[test]
fn reload_never_emits_port_smoke_or_keeps_a_muzzle_flash() {
    let mut e=Engine::new(160,100);e.qa=true;e.state=0;
    for slot in 0..WEP_N {
        for ent in &mut e.ents {ent.kind=EK_NONE;}e.fx_n=0;e.weapon=slot as i32;
        e.reload_t=0.0;e.mag[slot]=0;e.ammo[slot]=MAG_SZ[slot]*2;e.muzzle=1.0;
        e.begin_reload();
        assert_eq!(e.muzzle,0.0,"shot flash leaked into reload for slot {slot}");
        for _ in 0..20 {
            e.tick(0.02);
            assert!(!e.ents.iter().any(|ent|ent.kind==EK_SMOKE),"reload emitted port smoke for slot {slot}");
        }
        assert_eq!(e.muzzle,0.0);
    }
}
#[test]
fn firing_casing_ejections_still_emit_port_smoke_but_removed_feeds_do_not() {
    let mut e=Engine::new(160,100);
    for material in 0..=38 {
        for ent in &mut e.ents {ent.kind=EK_NONE;}e.fx_n=0;
        e.eject_debris(material);e.flush_fx();
        let smoke=e.ents.iter().filter(|ent|ent.kind==EK_SMOKE).count();
        assert_eq!(smoke,usize::from(material<2),"material {material}");
    }
}
