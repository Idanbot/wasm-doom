use crate::*;

#[test]
fn all_sectors_have_unique_dominant_palettes_and_opposite_temperature_accents() {
    let mut palettes=Vec::new();
    for wave in 1..=25 {
        let tint=Engine::sector_tint(wave); let accent=Engine::sector_contrast(wave);
        assert!(tint.iter().all(|&v|v>0.0 && v<=1.0));
        assert!(!palettes.contains(&tint)); palettes.push(tint);
        assert!((tint[0]>tint[2]) != (accent[0]>accent[2]));
    }
}

#[test]
fn muzzle_light_responds_to_material_and_cannot_illuminate_through_a_sealed_wall() {
    let mut e=testutil::arena(); e.light_dirty=false; e.static_light.fill([0.0;3]);
    e.muzzle=1.0; e.wave=4; e.update_lighting(); let metal=e.light_grid[4*MAP_W+5][0];
    e.wave=3; e.update_lighting(); let soft=e.light_grid[4*MAP_W+5][0];
    assert!(metal>soft*1.5);
    assert!(e.light_grid[4*MAP_W+7][0]>0.0);
    for y in 0..MAP_H {e.map[y*MAP_W+6]=1;}
    let rng=e.rng; e.update_lighting(); assert_eq!(e.light_grid[4*MAP_W+7],[0.0;3]);
    assert_eq!(e.rng,rng); e.muzzle=0.0; e.update_lighting(); assert_eq!(e.light_grid[4*MAP_W+5],[0.0;3]);
}

#[test]
fn every_boss_reveal_builds_light_and_rendering_cannot_advance_it_or_rng() {
    let mut e=testutil::arena(); e.light_dirty=false;
    for wave in 1..=25 {
        e.wave=wave; e.static_light.fill([0.08;3]);
        let profile=boss_arena::PROFILES[map::level_index(wave)];
        let (x,y)=map::boss_spots(wave)[0]; let index=y as usize*MAP_W+x as usize;
        e.boss_intro=profile.intro; e.update_lighting(); let early=e.light_grid[index].iter().sum::<f32>();
        e.boss_intro=profile.intro*0.05; e.update_lighting(); let late=e.light_grid[index].iter().sum::<f32>();
        assert!(late>early,"wave {wave}: late {late} vs early {early}");
        let rng=e.rng; let timer=e.boss_intro; let grid=e.light_grid.clone();
        e.update_lighting(); assert_eq!(e.light_grid,grid); assert_eq!(e.rng,rng); assert_eq!(e.boss_intro,timer);
    }
}
