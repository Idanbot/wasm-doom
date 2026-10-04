//! Telegraph-first boss environments. Floor styles 3/4 are warning/active.
use crate::{consts::*, map, Engine};

#[derive(Clone, Copy)]
pub(crate) struct Profile {
    pub color: [f32; 3],
    pub damage: i32,
    pub intro: f32,
}

pub(crate) struct ArenaState {
    pub armed: bool,
    pub elapsed: f32,
    pub cycle: i32,
    pub stage: u8,
    pub base: [u8; MAP_CELLS],
    pub mask: [u8; MAP_CELLS],
    pub anchor: (f32, f32),
    pub damage_t: f32,
    pub fx_t: f32,
}
impl ArenaState {
    pub fn new() -> Self {
        Self { armed: false, elapsed: 0.0, cycle: -1, stage: 0, base: [0; MAP_CELLS], mask: [0; MAP_CELLS], anchor: (0.0,0.0), damage_t: 0.0, fx_t: 0.0 }
    }
}

/// Authored spatial rules. The pattern stays fixed throughout its warning/impact.
pub(crate) fn pattern(level: usize, x: f32, y: f32, cycle: i32, anchor: (f32,f32)) -> bool {
    let c = cycle.rem_euclid(6) as f32;
    let r = x.hypot(y);
    let angle = y.atan2(x);
    let spoke = |n: f32, offset: f32, width: f32| ((angle + offset) * n).sin().abs() < width;
    match level {
        0 => (x - (c - 2.5) * 1.3).abs() < 0.55 && y.abs() > 1.0,
        1 => r > 2.0 && spoke(3.0, c * 0.25, 0.22),
        2 => (r - (1.8 + c * 0.75)).abs() < 0.55,
        3 => (x-anchor.0).abs() < 0.45 || (y-anchor.1).abs() < 0.45,
        4 => (r - (6.0 - c * 0.7)).abs() < 0.6,
        5 => ((x.floor() as i32 + y.floor() as i32 + cycle).rem_euclid(4)) == 0,
        6 => (y - (c-2.5)*1.3).abs()<0.6 && x.abs()>1.2,
        7 => r>1.6 && spoke(1.0,c*0.53,0.2),
        8 => ((x+7.0).floor() as i32/2 + (y+7.0).floor() as i32/2 + cycle).rem_euclid(5)==0,
        9 => (x-anchor.0).hypot(y-anchor.1)<1.6,
        10 => (r-2.2).abs()<0.45 || (r-5.0).abs()<0.45,
        11 => (x-c+2.5).abs()<0.38 || (x+c-2.5).abs()<0.38,
        12 => (y-(c-2.5)*1.4).abs()<0.7 && x> -3.0,
        13 => (x+y-(c-2.5)*1.5).abs()<0.65,
        14 => ((x+7.0).floor() as i32/2-cycle).rem_euclid(3)==0 && (y.floor() as i32).rem_euclid(2)==0,
        15 => r>2.0 && spoke(2.0,c*0.4,0.32) && r<5.4,
        16 => (x.sin()*y.cos()+c*0.17).sin()>0.86,
        17 => (y-c+2.5).abs()<0.35 || (y+c-2.5).abs()<0.35,
        18 => x.abs()>3.0+c*0.25 && y.abs()<2.5,
        19 => (x-y-c+2.5).abs()<0.55 || (x-y+c-2.5).abs()<0.55,
        20 => (r-(2.0+(cycle.rem_euclid(3) as f32)*1.8)).abs()<0.65,
        21 => ((angle+r*0.75+c*0.6).sin()).abs()<0.18 && r>1.8,
        22 => (((x+7.0)/2.0).floor() as i32+2*((y+7.0)/2.0).floor() as i32+cycle).rem_euclid(7)==0,
        23 => r>2.5 && spoke(4.0,c*0.19,0.17),
        _ => (r-(6.4-(cycle.rem_euclid(4) as f32))).abs()<0.7 && (angle+c*0.7).cos()<0.65,
    }
}

impl Engine {
    pub(crate) fn clear_boss_arena(&mut self) {
        if !self.boss_arena.armed { return; }
        self.floor.copy_from_slice(&self.boss_arena.base);
        self.boss_arena = ArenaState::new();
    }

    pub(crate) fn tick_boss_arena(&mut self, dt: f32) {
        let intro = self.boss_intro > 0.0;
        let fighting = self.ents.iter().any(|e| e.kind==EK_BOSS && e.hp>0);
        if self.state!=0 || (!intro && !fighting) { self.clear_boss_arena(); return; }
        let level=map::level_index(self.wave);
        let (bx,by)=map::boss_spots(self.wave)[0];
        let profile=PROFILES[level];
        if !self.boss_arena.armed {
            self.boss_arena.armed=true;
            self.boss_arena.base.copy_from_slice(&self.floor);
        }
        if !intro { self.boss_arena.elapsed += dt; }
        let t=self.boss_arena.elapsed;
        let period=5.0+(level%3) as f32*0.25;
        let cycle=(t/period) as i32;
        let within=t%period;
        let stage=if intro || within<1.6 {3} else if within<2.8 {4} else {0};
        if cycle!=self.boss_arena.cycle {
            self.boss_arena.cycle=cycle;
            self.boss_arena.anchor=(self.px-bx,self.py-by);
            self.boss_arena.mask.fill(0);
            let refuge=map::override_point(self.wave);
            for y in 0..MAP_H {for x in 0..MAP_W {
                let (wx,wy)=(x as f32+0.5,y as f32+0.5);
                if self.blocked(x as i32,y as i32) || (wx-bx).hypot(wy-by)>7.5 || (wx-refuge.0).hypot(wy-refuge.1)<1.65 || !self.los(bx,by,wx,wy) {continue;}
                if pattern(level,wx-bx,wy-by,cycle,self.boss_arena.anchor) {self.boss_arena.mask[y*MAP_W+x]=1;}
            }}
            if !self.boss_arena.mask.iter().any(|v| *v != 0) {
                let candidate=(0..MAP_CELLS).filter(|&i| {
                    let (x,y)=((i%MAP_W) as f32+0.5,(i/MAP_W) as f32+0.5);
                    !self.blocked((i%MAP_W) as i32,(i/MAP_W) as i32) && (x-bx).hypot(y-by)<7.5 && (x-refuge.0).hypot(y-refuge.1)>1.65 && self.los(bx,by,x,y)
                }).min_by(|&a,&b| {
                    let distance=|i:usize| ((i%MAP_W) as f32+0.5-self.px).powi(2)+((i/MAP_W) as f32+0.5-self.py).powi(2);
                    distance(a).total_cmp(&distance(b))
                });
                if let Some(i)=candidate { self.boss_arena.mask[i]=1; }
            }
        }
        let changed=stage!=self.boss_arena.stage;
        self.boss_arena.stage=stage;
        self.floor.copy_from_slice(&self.boss_arena.base);
        if stage!=0 {
            for i in 0..MAP_CELLS {if self.boss_arena.mask[i]!=0 {self.floor[i]=stage;}}
        }
        if changed && stage==4 {
            self.sound(2, if matches!(level,2|16){3}else{2},bx,by);
            if matches!(level,0|8|9|18|24) {self.shake=(self.shake+0.2).min(0.6);}
        }
        self.boss_arena.damage_t=(self.boss_arena.damage_t-dt).max(0.0);
        let (cx,cy)=(self.px.floor() as i32,self.py.floor() as i32);
        if stage==4 && self.boss_arena.damage_t<=0.0 && cx>=0 && cy>=0 && cx<MAP_W as i32 && cy<MAP_H as i32 && self.boss_arena.mask[cy as usize*MAP_W+cx as usize]!=0 {
            self.damage_player(profile.damage);
            self.boss_arena.damage_t=0.6;
        }
        self.boss_arena.fx_t-=dt;
        if stage==4 && self.boss_arena.fx_t<=0.0 {
            self.boss_arena.fx_t=0.35;
            // Bounded physical effects: debris, steam or electrical discharges.
            let offset=(cycle as usize*31+(self.time*4.0) as usize)%MAP_CELLS;
            if let Some(i)=(0..MAP_CELLS).map(|j|(j+offset)%MAP_CELLS).find(|&i|self.boss_arena.mask[i]!=0) {
                let (x,y)=((i%MAP_W) as f32+0.5,(i/MAP_W) as f32+0.5);
                if matches!(level,6|12|21|23) {self.spawn_smoke_cloud(x,y);}
                else if matches!(level,2|16) {self.effect(EK_IMPACT,3,x,y,0.4,4.0);}
                else {self.spawn_timed(EK_SPARK,x,y,0.45,10.0);}
            }
        }
    }
}

// Generated from art/boss-arenas-v1/specs.json.
pub(crate) const PROFILES: [Profile; 25] = [
    Profile { color: [1.0, 0.70196, 0.27843], damage: 8, intro: 4.0 },
    Profile { color: [1.0, 0.50588, 0.24314], damage: 8, intro: 5.2 },
    Profile { color: [0.56863, 0.8549, 0.30196], damage: 8, intro: 4.6 },
    Profile { color: [0.32549, 0.85882, 0.98039], damage: 8, intro: 4.8 },
    Profile { color: [0.67059, 0.48627, 1.0], damage: 8, intro: 5.0 },
    Profile { color: [0.91373, 0.7451, 0.51373], damage: 8, intro: 5.4 },
    Profile { color: [0.54902, 0.90588, 1.0], damage: 8, intro: 5.6 },
    Profile { color: [0.31765, 0.95294, 0.72941], damage: 9, intro: 4.4 },
    Profile { color: [1.0, 0.72941, 0.27843], damage: 9, intro: 6.0 },
    Profile { color: [1.0, 0.41176, 0.39216], damage: 9, intro: 4.2 },
    Profile { color: [0.81176, 0.56863, 1.0], damage: 9, intro: 5.8 },
    Profile { color: [1.0, 0.57255, 0.34118], damage: 9, intro: 5.6 },
    Profile { color: [0.31373, 0.80392, 0.83137], damage: 9, intro: 4.2 },
    Profile { color: [0.93725, 0.56863, 0.96078], damage: 9, intro: 4.48 },
    Profile { color: [0.41569, 0.61961, 1.0], damage: 10, intro: 4.76 },
    Profile { color: [0.70588, 0.83922, 0.33725], damage: 10, intro: 5.04 },
    Profile { color: [0.70588, 0.9098, 0.63529], damage: 10, intro: 5.32 },
    Profile { color: [0.50588, 0.79216, 1.0], damage: 10, intro: 5.6 },
    Profile { color: [0.90196, 0.63922, 0.38824], damage: 10, intro: 4.2 },
    Profile { color: [0.66275, 0.51765, 0.91765], damage: 10, intro: 4.48 },
    Profile { color: [1.0, 0.8, 0.38431], damage: 10, intro: 4.76 },
    Profile { color: [0.42353, 0.8549, 0.70588], damage: 11, intro: 5.04 },
    Profile { color: [0.66667, 0.73333, 1.0], damage: 11, intro: 5.32 },
    Profile { color: [0.70196, 0.94902, 1.0], damage: 11, intro: 5.6 },
    Profile { color: [1.0, 0.83529, 0.52549], damage: 11, intro: 4.2 },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_twenty_five_hazards_have_distinct_patterns_and_safe_ground() {
        let mut fingerprints = std::collections::HashSet::new();
        for level in 0..25 {
            let cells: Vec<bool> = (0..6).flat_map(|cycle| (-7..=7).flat_map(move |y| (-7..=7).map(move |x| pattern(level,x as f32+0.5,y as f32+0.5,cycle,(2.5,-1.5))))).collect();
            assert!(cells.iter().any(|v| *v), "level {} has no hazard", level+1);
            assert!(cells.iter().any(|v| !*v), "level {} has no safe ground", level+1);
            assert!(fingerprints.insert(cells), "duplicate level {}",level+1);
        }
    }
    #[test]
    fn every_arena_warns_before_damage_and_restores_after_death() {
        let mut e = Engine::new(160,100);
        for wave in 1..=25 {
            e.clear_boss_arena(); e.wave=wave; e.state=0;
            map::build_level(&mut e);
            for ent in &mut e.ents { ent.kind=EK_NONE; }
            let original=e.floor.clone();
            let (bx,by)=map::boss_spots(wave)[0];
            let boss=e.spawn(EK_BOSS,bx,by).unwrap();
            let refuge=map::override_point(wave);
            e.px=refuge.0; e.py=refuge.1;
            e.tick_boss_arena(0.01);
            assert_eq!(e.boss_arena.stage,3);
            // Targeted hazards can miss a refuge; test all six authored cycles.
            let mut found=false;
            for cycle in 0..6 {
                e.boss_arena.elapsed=cycle as f32*(5.0+(map::level_index(wave)%3) as f32*0.25);
                e.tick_boss_arena(0.01);
                if let Some(i)=e.boss_arena.mask.iter().position(|v| *v!=0) {
                    found=true; e.px=(i%MAP_W) as f32+0.5; e.py=(i/MAP_W) as f32+0.5;
                    e.health=100; e.armor=0; e.iframes=0.0;
                    e.tick_boss_arena(0.1); assert_eq!(e.health,100);
                    e.tick_boss_arena(1.6); assert!(e.health<100);
                    let hp=e.health; e.tick_boss_arena(0.1); assert_eq!(e.health,hp);
                    break;
                }
            }
            assert!(found,"level {wave} hazard never reaches its arena");
            e.ents[boss].hp=0; e.boss_intro=0.0;
            e.tick_boss_arena(0.01);
            assert!(!e.boss_arena.armed); assert_eq!(e.floor,original);
        }
    }
}
