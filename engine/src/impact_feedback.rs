//! Contact feedback uses the surface actually struck, before absorption.
use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Contact { Flesh, Armor, Shield, Machine }

pub(crate) fn mechanical_body(skin: u8) -> bool {
    matches!(enemies::combat_skin(skin), 5 | 8 | 11 | 13 | 15..=20 | 22..=27 | 29..=36)
}

impl Engine {
    pub(crate) fn contact_feedback(&mut self, i: usize, hx: f32, hy: f32) -> Contact {
        let e = self.ents[i];
        let contact = if e.shield_hp > 0 && field::shield_blocks(e.face,e.x,e.y,hx,hy) { Contact::Shield }
            else if e.armor_hp > 0 { Contact::Armor }
            else if !is_hostile_kind(e.kind) || mechanical_body(e.skin) { Contact::Machine }
            else { Contact::Flesh };
        self.sound(if contact==Contact::Flesh {4}else{3}, u8::from(contact==Contact::Shield), e.x,e.y);
        if !is_hostile_kind(e.kind) { return contact; }
        let (dx,dy)=(hx-e.x,hy-e.y); let d=dx.hypot(dy).max(0.01);
        let (x,y)=(e.x+dx/d*e.radius*0.7,e.y+dy/d*e.radius*0.7);
        // No combat RNG, world stains or secondary impact audio for these cosmetics.
        if contact!=Contact::Flesh {
            let slot=self.fx_n;
            self.queue_fx(EK_IMPACT,x,y,0.0,0.0,0.19,-8.0);
            if self.fx_n>slot {self.fx_q[slot].variant=u8::from(contact==Contact::Shield);}
        }
        for n in 0..3 {
            let angle=(hy-e.y).atan2(hx-e.x)+(n as f32-1.0)*0.7;
            let (kind,variant)=match contact {Contact::Flesh=>(EK_GIB,0),Contact::Shield=>(EK_SPARK,1),_=>(EK_SPARK,4)};
            let slot=self.fx_n;
            self.queue_fx(kind,x,y,angle.cos()*1.1,angle.sin()*1.1,0.22,-6.0);
            if self.fx_n>slot {self.fx_q[slot].variant=variant;}
        }
        contact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn target(skin:u8)->(Engine,usize) {
        let mut e=Engine::new(160,100);e.map.fill(0);for p in &mut e.ents {p.kind=EK_NONE;}
        let i=e.spawn_with_skin(EK_HUSK,skin,10.5,10.5).unwrap();e.ents[i].hp=1000;e.ents[i].armor_hp=0;
        (e,i)
    }
    #[test]
    fn shield_front_armor_and_unprotected_flesh_have_distinct_feedback() {
        let (mut e,i)=target(SKIN_RIFLEMAN);e.ents[i].shield_hp=100;e.ents[i].face=0.0;
        let rng=e.rng;assert_eq!(e.contact_feedback(i,12.5,10.5),Contact::Shield);assert_eq!(e.rng,rng);
        assert_eq!(e.sound_cues.last().unwrap().variant,1.0);
        assert!(e.fx_q[..e.fx_n].iter().all(|p|p.kind!=EK_GIB));e.fx_n=0;
        assert_eq!(e.contact_feedback(i,8.5,10.5),Contact::Flesh);
        assert_eq!(e.fx_q[..e.fx_n].iter().filter(|p|p.kind==EK_GIB).count(),3);
        e.fx_n=0;e.ents[i].shield_hp=0;e.ents[i].armor_hp=8;
        e.hurt_ent(i,8,8.5,10.5);assert_eq!(e.ents[i].hp,1000);assert!(e.fx_n>0);
        assert_eq!(e.hitmarker,0.7);assert_eq!(e.ents[i].armor_hp,0);e.fx_n=0;
        e.hurt_ent(i,10,8.5,10.5);assert_eq!(e.ents[i].hp,990);
        assert!(e.fx_q[..e.fx_n].iter().any(|p|p.kind==EK_GIB));
    }
    #[test]
    fn machines_and_sector_variants_shed_sparks_without_blood() {
        for skin in [SKIN_LOADER,SKIN_HORNET,45] {
            let (mut e,i)=target(skin);
            // Sector 9 inherits gunner armor; its unarmored body remains human.
            if skin==45 {e.ents[i].armor_hp=10;}
            e.hurt_ent(i,5,8.5,10.5);
            assert!(e.fx_q[..e.fx_n].iter().any(|p|p.kind==EK_SPARK));
            assert!(!e.fx_q[..e.fx_n].iter().any(|p|p.kind==EK_GIB));
        }
    }
}
