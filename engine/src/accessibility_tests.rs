use crate::*;

#[test]
fn effects_validate_each_preference_independently() {
    let value = presentation::Effects::new(-1.0, 2.0, 0.4);
    assert_eq!((value.flash,value.shake,value.debris),(0.0,1.0,0.4));
    for bad in [f32::NAN,f32::INFINITY,f32::NEG_INFINITY] {
        let value = presentation::Effects::new(bad,bad,bad);
        assert_eq!((value.flash,value.shake,value.debris),(1.0,1.0,1.0));
    }
}

#[test]
fn effects_update_paused_feedback_without_advancing_or_changing_combat() {
    let mut e = testutil::arena();
    e.muzzle=0.8;e.hurt=0.6;e.shake=0.5;
    let (rng,mag,elapsed)=(e.rng,e.mag,e.elapsed);
    e.set_effects(0.25,0.5,0.0);
    assert_eq!((e.hud.muzzle,e.hud.hurt,e.hud.shake),(0.2,0.15,0.125));
    assert_eq!((e.muzzle,e.hurt,e.shake),(0.8,0.6,0.5));
    e.set_effects(1.0,1.0,1.0);
    assert_eq!((e.hud.muzzle,e.hud.hurt,e.hud.shake),(0.8,0.6,0.25));
    assert_eq!((e.rng,e.mag,e.elapsed),(rng,mag,elapsed));
}

#[test]
fn debris_setting_preserves_dangerous_projectiles_and_telegraphs() {
    let e = testutil::arena();
    let hidden = presentation::Effects::new(0.0,0.0,0.0);
    let mut ent = e.ents[0];
    for kind in [EK_HUSK,EK_BOSS,EK_PROJ,EK_BOLT,EK_FLAME,EK_FIREPATCH,EK_RAY,EK_IMPACT,EK_LAMP,EK_NODE] {
        ent.kind=kind;ent.effect_tick=5.0;
        assert!(hidden.show_sprite(0,&ent),"essential sprite {kind} was hidden");
    }
    ent.kind=EK_SPARK;ent.effect_tick=4.0;
    assert!(hidden.show_sprite(0,&ent));
    for (kind,tick) in [(EK_GIB,0.0),(EK_SPARK,5.0)] {
        ent.kind=kind;ent.effect_tick=tick;
        for index in 0..ENT_N {assert!(!hidden.show_sprite(index,&ent));}
    }
}

#[test]
fn reduced_debris_selects_a_stable_subset_while_fragments_move() {
    let e = testutil::arena();
    let mut ent=e.ents[0];ent.kind=EK_GIB;
    let half=presentation::Effects::new(1.0,1.0,0.5);
    let mut count=0;
    for index in 0..ENT_N {
        let visible=half.show_sprite(index,&ent);
        ent.x+=0.1;ent.y-=0.2;ent.timer-=0.01;
        assert_eq!(visible,half.show_sprite(index,&ent));
        assert!(presentation::Effects::default().show_sprite(index,&ent));
        count+=usize::from(visible);
    }
    assert!(count>ENT_N/3 && count<ENT_N*2/3,"unexpected debris density {count}");
}

#[test]
fn cosmetic_preferences_do_not_change_deterministic_firing_or_reload_results() {
    let mut a=testutil::arena();let mut b=testutil::arena();
    b.set_effects(0.0,0.0,0.0);
    for frame in 0..180 {
        let bits=if !(70..=140).contains(&frame) {IN_FIRE|IN_W}else{IN_RELOAD};
        a.bits=bits;b.bits=bits;
        a.tick(1.0/60.0);b.tick(1.0/60.0);
        assert_eq!((a.rng,a.mag,a.ammo,a.px,a.py,a.health,a.reload_t),(b.rng,b.mag,b.ammo,b.px,b.py,b.health,b.reload_t));
        assert_eq!((a.muzzle,a.kick,a.shake),(b.muzzle,b.kick,b.shake));
        assert_eq!((b.hud.muzzle,b.hud.hurt,b.hud.shake),(0.0,0.0,0.0));
    }
}

#[test]
fn debris_filter_affects_cpu_drawing_without_deleting_entities() {
    let mut e=testutil::arena();
    e.pa=0.0;e.eject_casing();e.flush_fx();
    let index=e.ents.iter().position(|ent|ent.kind==EK_SPARK&&ent.effect_tick==5.0).unwrap();
    e.ents[index].x=7.5;e.ents[index].y=4.5;e.ents[index].zoff=0.0;
    let (texture,_,_,_)=sprite_style(&e.ents[index]);
    e.tex[texture*TEX*TEX..(texture+1)*TEX*TEX].fill(Engine::pack(240,190,40,255));
    e.rebuild_mipmaps();
    let count=e.ents.iter().filter(|ent|ent.kind!=EK_NONE).count();
    e.render();let full=e.fb.clone();
    e.set_effects(1.0,1.0,0.0);let rng=e.rng;e.render();
    assert!(e.fb!=full,"debris filter did not change CPU drawing");
    assert_eq!(e.rng,rng);
    assert_eq!(e.ents.iter().filter(|ent|ent.kind!=EK_NONE).count(),count);
}

#[test]
fn visual_preferences_survive_sector_and_checkpoint_changes() {
    let mut e=testutil::arena();let save=abi::capture_save(&e);
    e.set_effects(0.2,0.3,0.4);e.next_wave();abi::apply_save(&mut e,&save);
    assert_eq!((e.effects.flash,e.effects.shake,e.effects.debris),(0.2,0.3,0.4));
}
