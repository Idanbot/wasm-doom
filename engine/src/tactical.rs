//! Weapon combinations, readable secrets and physical machinery responses.
use crate::{consts::*, field, map, Engine, is_hostile_kind};
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Role { Precision, Stagger, Control, Blast, Shock, Suppress, Burn, Acid, Freeze }
pub(crate) fn role(slot: usize) -> Role {
    use Role::*;
    [Precision,Stagger,Control,Precision,Blast,Shock,Suppress,Burn,Precision,Burn,Acid,Precision,Precision,Stagger,Freeze,Shock,Blast,Precision,Precision,Blast,Stagger,Precision,Shock,Control,Acid,Precision,Suppress,Precision,Freeze,Blast,Control,Precision,Shock][slot.min(32)]
}
pub(crate) struct Tactical {
    pub acid: [f32;ENT_N],
    pub slow: [f32;ENT_N],
    pub shock_lock: [f32;ENT_N],
    pub cooldowns: [f32;WEP_N],
    pub outages: Vec<(f32,f32,u8)>,
    pub secrets_paid: Vec<(f32,f32)>,
    pub secret_rooms: Vec<(i32,i32,f32,f32)>,
}
impl Tactical {
    pub fn new()->Self { Self { acid:[0.0;ENT_N],slow:[0.0;ENT_N],shock_lock:[0.0;ENT_N],cooldowns:[0.0;WEP_N],outages:Vec::new(),secrets_paid:Vec::new(),secret_rooms:Vec::new() } }
    pub fn reset_entity(&mut self,i:usize) {self.acid[i]=0.0;self.slow[i]=0.0;self.shock_lock[i]=0.0;}
    pub fn tick(&mut self,dt:f32) {
        for i in 0..ENT_N {self.acid[i]=(self.acid[i]-dt).max(0.0);self.slow[i]=(self.slow[i]-dt).max(0.0);self.shock_lock[i]=(self.shock_lock[i]-dt).max(0.0);}
        for t in &mut self.cooldowns {*t=(*t-dt).max(0.0);}
    }
}
impl Engine {
    pub(crate) fn outage_at(&self,x:f32,y:f32,kind:u8)->bool {
        self.tactical.outages.iter().any(|&(a,b,k)| k==kind && (x-a).hypot(y-b)<12.0 && self.los(a,b,x,y))
    }
    pub(crate) fn player_hit(&mut self,i:usize,dmg:i32,hx:f32,hy:f32,slot:usize) {
        let effect=if slot==1 && (self.ents[i].x-hx).hypot(self.ents[i].y-hy)>=4.0 {None}else{Some(role(slot))};
        self.hurt_ent_role(i,dmg,hx,hy,effect);
    }
    pub(crate) fn machinery_destroyed(&mut self,skin:u8,x:f32,y:f32) {
        let kind=crate::tactical_roles::MACHINERY[((skin-150)/3) as usize][((skin-150)%3) as usize];
        self.say(200+kind as i32);
        if kind==1 || kind==2 {self.tactical.outages.push((x,y,kind));}
        let targets:Vec<usize>=self.ents.iter().enumerate().filter(|(_,e)|is_hostile_kind(e.kind)&&e.hp>0&&(e.x-x).hypot(e.y-y)<if kind==1 {12.0}else{6.0}&&self.los(x,y,e.x,e.y)).map(|(i,_)|i).collect();
        for i in targets {match kind {
            0=>{self.tactical.slow[i]=6.0;},
            1=>{self.ents[i].shield_hp=0;self.ents[i].shield=0;},
            3=>{self.ents[i].stun=if self.ents[i].kind==EK_BOSS {0.35}else{2.0};},
            4=>{self.tactical.acid[i]=4.0;}, _=>{}
        }}
        if kind==0 {self.spawn_smoke_cloud(x,y);}
        else if kind==5 {self.explode(x,y,3.2,45.0);}
        else if kind==6 {self.ensure_drop(EK_AMMO,x,y+0.35);self.ensure_drop(EK_ARMOR,x+0.5,y);}
        else {self.burst_fx(x,y,EK_SPARK,8,0.3);}
        self.light_dirty=true;
    }
    pub(crate) fn secret_room(&self,x:i32,y:i32)->Option<(f32,f32)> {
        self.tactical.secret_rooms.iter().find(|&&(a,b,_,_)|a==x&&b==y).map(|&(_,_,a,b)|(a,b)).or_else(||field::secret_interior(x,y))
    }
    pub(crate) fn install_secret_cache(&mut self) {
        self.tactical.secret_rooms.clear();
        if self.map.contains(&9) {return;}
        let start=map::player_start(self.wave);let objective=map::override_point(self.wave);
        let offset=map::level_index(self.wave)*53;
        for j in 0..MAP_CELLS {
            let index=(j+offset)%MAP_CELLS;let (x,y)=((index%MAP_W) as i32,(index/MAP_W) as i32);
            if x<3||y<3||x>=MAP_W as i32-3||y>=MAP_H as i32-3 {continue;}
            if (x as f32-start.0).hypot(y as f32-start.1)<6.0||(x as f32-objective.0).hypot(y as f32-objective.1)<6.0 {continue;}
            for (dx,dy) in [(1,0),(-1,0),(0,1),(0,-1)] {
                if self.cell(x-dx,y-dy)!=0 {continue;}
                let solid=(0..=2).all(|step|(-1..=1).all(|side| {
                    let c=self.cell(x+dx*step-dy*side,y+dy*step+dx*side);
                    c!=0&&c!=8&&c!=9&&c!=10
                }));
                if !solid {continue;}
                self.set_cell(x,y,9);
                for step in 1..=2 {self.set_cell(x+dx*step,y+dy*step,0);}
                self.tactical.secret_rooms.push((x,y,(x+dx*2) as f32+0.5,(y+dy*2) as f32+0.5));
                return;
            }
        }
    }
    pub(crate) fn place_secret_clues(&mut self) {
        // A short worn track leads toward each concealed service panel.
        for y in 1..MAP_H-1 {for x in 1..MAP_W-1 {
            if self.map[y*MAP_W+x]!=9 {continue;}
            for (dx,dy) in [(1i32,0i32),(-1,0),(0,1),(0,-1)] {
                // Mark only the outside of the cache, never its supply interior.
                let interior=self.secret_room(x as i32,y as i32);
                let (nx,ny)=(x as i32+dx,y as i32+dy);
                if self.blocked(nx,ny) {continue;}
                if interior.is_some_and(|(a,b)| (nx as f32+0.5-a).hypot(ny as f32+0.5-b)<1.4) {continue;}
                for step in 1..=2 {
                    let (a,b)=(x as i32+dx*step,y as i32+dy*step);
                    if self.blocked(a,b) {break;}
                    let i=b as usize*MAP_W+a as usize;
                    if self.floor[i]!=2 {self.floor[i]=5+((map::level_index(self.wave)+x+y)%3) as u8;}
                }
            }
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn arena()->Engine {let mut e=Engine::new(160,100);e.map.fill(0);for en in &mut e.ents {en.kind=EK_NONE;}e.px=4.5;e.py=4.5;e}
    #[test]
    fn fast_equip_still_presents_all_four_handling_frames() {
        let mut e=arena();e.has_w6=true;e.mag[5]=10;e.select_weapon(5);
        assert_eq!(e.pickup_dur,0.28);
        let mut frames=std::collections::HashSet::new();
        for _ in 0..8 {e.tick(0.04);frames.insert(e.hud.weap_frame);}
        for frame in 6..=9 {assert!(frames.contains(&frame));}
        assert_eq!(e.hud.weap_frame,0);
    }
    #[test]
    fn tactical_hints_override_general_objectives_but_do_not_show_through_walls() {
        let mut e=arena();e.map[4*MAP_W+5]=9;
        let i=e.spawn_with_skin(EK_CRATE,159,6.5,4.5).unwrap();e.ents[i].hp=35;
        e.tick(0.01);assert_eq!(e.hud.prompt,123);assert!(e.facing_prop(EK_CRATE,4.5).is_none());
        e.door[4*MAP_W+5]=0.8;e.tick(0.01);assert_ne!(e.hud.prompt,123);
        e.map[4*MAP_W+5]=0;e.tick(0.01);assert_eq!(e.hud.prompt,120);
    }
    #[test]
    fn precision_bypasses_armor_and_consumes_one_acid_combo() {
        let mut e=arena();let i=e.spawn(EK_HUSK,8.5,4.5).unwrap();e.ents[i].hp=200;e.ents[i].armor_hp=100;
        e.player_hit(i,50,4.5,4.5,3);assert_eq!(e.ents[i].hp,180);assert_eq!(e.ents[i].armor_hp,70);
        e.ents[i].armor_hp=0;e.player_hit(i,20,4.5,4.5,10);assert!(e.tactical.acid[i]>0.0);
        e.player_hit(i,40,4.5,4.5,3);assert_eq!(e.ents[i].hp,110);assert_eq!(e.tactical.acid[i],0.0);
        e.player_hit(i,40,4.5,4.5,3);assert_eq!(e.ents[i].hp,70);
    }
    #[test]
    fn shock_freeze_and_suppression_have_bounded_distinct_effects() {
        let mut e=arena();let i=e.spawn(EK_HUSK,8.5,4.5).unwrap();e.ents[i].hp=1000;
        e.player_hit(i,1,4.5,4.5,5);assert!(e.ents[i].stun>=0.65);assert!(e.tactical.shock_lock[i]>0.0);
        e.ents[i].stun=0.0;e.player_hit(i,1,4.5,4.5,5);assert_eq!(e.ents[i].stun,0.0);
        e.player_hit(i,1,4.5,4.5,14);assert_eq!(e.tactical.slow[i],3.0);
        e.player_hit(i,1,4.5,4.5,6);assert!(e.ents[i].timer>=0.85);
        e.tactical.tick(4.0);assert_eq!(e.tactical.slow[i],0.0);assert_eq!(e.tactical.shock_lock[i],0.0);
    }
    #[test]
    fn switching_enables_combos_without_bypassing_weapon_cadence() {
        let mut e=arena();e.has_w6=true;e.mag[0]=12;e.mag[5]=10;e.fire();assert_eq!(e.mag[0],11);
        e.select_weapon(5);e.cooldown=0.0;e.fire();assert_eq!(e.mag[5],9);
        e.select_weapon(0);e.cooldown=0.0;e.fire();assert_eq!(e.mag[0],11);
        e.tactical.tick(1.0);e.cooldown=0.0;e.fire();assert_eq!(e.mag[0],10);
    }
    #[test]
    fn coolant_outages_and_networks_respect_cover_and_reset_each_sector() {
        let mut e=arena();e.wave=2;let i=e.spawn(EK_HUSK,8.5,4.5).unwrap();e.arm_shield(i);
        e.machinery_destroyed(153,7.5,4.5);assert_eq!(e.tactical.slow[i],6.0);
        e.machinery_destroyed(154,7.5,4.5);assert_eq!(e.ents[i].shield_hp,0);e.arm_shield(i);assert_eq!(e.ents[i].shield_hp,0);
        e.wave=4;e.machinery_destroyed(159,7.5,4.5);assert!(e.outage_at(9.5,4.5,2));
        e.map[4*MAP_W+8]=1;assert!(!e.outage_at(9.5,4.5,2));
        e.next_wave();assert!(e.tactical.outages.is_empty());
    }
    #[test]
    fn every_sector_has_visible_secret_tracks_without_overwriting_objectives() {
        let mut e=arena();
        for wave in 1..=25 {e.wave=wave;e.build_map();e.place_secret_clues();
            assert!(e.floor.iter().any(|&v| (5..=7).contains(&v)),"missing clues in {wave}");
            let (x,y)=map::override_point(wave);assert_eq!(e.floor[y as usize*MAP_W+x as usize],2);
        }
    }
    #[test]
    fn paired_secret_panels_pay_and_show_a_log_only_once() {
        let mut e=arena();e.pay_secret(4,19);let sequence=e.radio_seq;let count=e.ents.iter().filter(|en|en.kind!=0).count();
        e.pay_secret(5,19);assert_eq!(e.radio_seq,sequence);assert_eq!(e.ents.iter().filter(|en|en.kind!=0).count(),count);
        assert_eq!(e.radio_line,210);
    }
    #[test]
    fn networks_cancel_queued_arrivals_without_softlocking_the_objective() {
        let mut e=arena();e.wave=4;let (_,_,x,y)=map::hostiles(4)[0];e.px=1.5;e.py=1.5;e.pending_hostiles=1;
        e.tactical.outages.push((x,y,2));assert!(e.spawn_reinforcement(true));assert_eq!(e.pending_hostiles,0);
    }
}
