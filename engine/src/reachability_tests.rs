//! Player-clear routes, generated placement, ambushes, boss escorts and endless copies.
use crate::*;
const SCALE:usize=4;
const WIDTH:usize=MAP_W*SCALE;
fn routes(e:&Engine,start:(f32,f32))->Vec<bool> {
    let mut seen=vec![false;MAP_CELLS*SCALE*SCALE];
    let mut queue=vec![((start.0*SCALE as f32).round() as usize,(start.1*SCALE as f32).round() as usize)];
    while let Some((x,y))=queue.pop() {
        if x>=WIDTH||y>=MAP_H*SCALE {continue;}
        let i=y*WIDTH+x;if seen[i]||e.circle_blocked(x as f32/SCALE as f32,y as f32/SCALE as f32,e.pr) {continue;}
        seen[i]=true;
        if x>0 {queue.push((x-1,y));}
        if y>0 {queue.push((x,y-1));}
        queue.push((x+1,y));queue.push((x,y+1));
    }
    seen
}
fn connected(e:&Engine,seen:&[bool],x:f32,y:f32,range:f32)->bool {
    if e.circle_blocked(x,y,0.05) {return false;}
    let gx=(x*SCALE as f32).round() as i32;let gy=(y*SCALE as f32).round() as i32;
    let radius=(range*SCALE as f32).ceil() as i32;
    for iy in gy-radius..=gy+radius {for ix in gx-radius..=gx+radius {
        if ix<0||iy<0||ix>=WIDTH as i32||iy>=(MAP_H*SCALE) as i32 {continue;}
        let (nx,ny)=(ix as f32/SCALE as f32,iy as f32/SCALE as f32);
        if seen[iy as usize*WIDTH+ix as usize]&&(nx-x).hypot(ny-y)<=range&&e.los(nx,ny,x,y) {return true;}
    }}false
}
fn entities_reachable(e:&Engine,seen:&[bool],seed:u32,label:&str) {
    for p in e.ents.iter().filter(|p|p.kind!=0&&(is_hostile_kind(p.kind)||floor_prop(p.kind))) {
        assert!(connected(e,seen,p.x,p.y,0.85),"sector {} seed {seed} {label}: kind {} at {},{} unreachable",e.wave,p.kind,p.x,p.y);
    }
}
#[test]
fn every_sector_seed_and_spawn_variant_has_player_clear_routes() {
    let mut e=Engine::new(160,100);
    for wave in (1..=25).chain([26,50,100]) {for seed in [1,0x12345678,0xdeadbeef] {
        e.wave=wave;e.rng=seed;e.door.fill(0.0);e.build_map();e.place_ents();e.state=0;e.boss_spawned=false;
        // Doors can be opened by approach/USE; this checks geometric reachability.
        e.door.fill(1.0);let start=map::player_start(wave);e.px=start.0;e.py=start.1;
        let seen=routes(&e,(e.px,e.py));assert!(seen.iter().any(|&v|v));
        let objective=map::override_point(wave);let node=field::node_point(wave);
        for (x,y) in [objective,node].into_iter().chain(field::terminals(wave).iter().copied()).chain(map::boss_spots(wave).iter().copied()) {
            assert!(connected(&e,&seen,x,y,0.85),"sector {wave} seed {seed}: objective at {x},{y} unreachable");
        }
        entities_reachable(&e,&seen,seed,"initial");
        // Clear slots, preserving the map: repeated copies exercise jitter relocation.
        for p in &mut e.ents {p.kind=EK_NONE;}e.pending_hostiles=map::hostiles(wave).len()*3;e.reinforcement_cursor=map::hostiles(wave).len();
        for _ in 0..map::hostiles(wave).len()*2 {assert!(e.spawn_reinforcement(false));}
        entities_reachable(&e,&seen,seed,"reinforcement copies");
        for p in &mut e.ents {p.kind=EK_NONE;}
        let ambushes=e.ambush.clone();for a in ambushes {e.px=(a.x0+a.x1)*0.5;e.py=(a.y0+a.y1)*0.5;map::check_ambushes(&mut e);}
        entities_reachable(&e,&seen,seed,"ambush");
        e.px=objective.0;e.py=objective.1;e.maybe_spawn_boss();entities_reachable(&e,&seen,seed,"boss and escorts");
        let boss=e.ents.iter().find(|p|p.kind==EK_BOSS).expect("boss must spawn");
        e.drop_boss_case(boss.x,boss.y);entities_reachable(&e,&seen,seed,"boss reward");
    }}
}
#[test]
fn route_check_rejects_disconnected_floor_and_too_narrow_passage() {
    let mut e=Engine::new(160,100);e.map.fill(1);
    for x in 3..8 {e.map[5*MAP_W+x]=0;}e.map[12*MAP_W+12]=0;
    let seen=routes(&e,(3.5,5.5));assert!(connected(&e,&seen,7.5,5.5,0.5));
    assert!(!connected(&e,&seen,12.5,12.5,0.5));
    e.pr=0.51;assert!(!routes(&e,(3.5,5.5)).iter().any(|&v|v));
}
