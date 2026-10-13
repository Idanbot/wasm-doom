//! Bounded, cosmetic floor effects. Rendering never advances simulation RNG.
use crate::*;

pub(crate) const RESIDUE_CAP: usize = 64;
#[derive(Clone, Copy)]
pub(crate) struct Residue { pub x: f32, pub y: f32, pub kind: u8, pub born: f32 }

impl Engine {
    /// Material IDs: concrete, metal, grate, liquid, soft. Same wet mask as GPU art.
    pub(crate) fn floor_material(&self, x: f32, y: f32) -> u8 {
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 || x >= MAP_W as f32 || y >= MAP_H as f32 { return 0; }
        let level = map::level_index(self.wave);
        let style = self.floor[y as usize * MAP_W + x as usize];
        if style == 1 { return 0; }
        if (5..=7).contains(&style) { return 2; }
        const FAMILIES: [u8; 25] = [0,1,2,3,4,5,1,3,0,6,7,0,6,7,4,3,2,4,6,5,4,6,5,1,7];
        let family = FAMILIES[level];
        let seed = (x.floor() * 0.1031 + y.floor() * 0.11369 + level as f32 * 0.137).fract();
        let qx = x.fract() - (0.48 + seed * 0.1);
        let qy = (y.fract() - (0.48 - seed * 0.1)) * 1.7;
        if style < 2 && matches!(family, 6 | 7) && seed > 0.72 && qx*qx+qy*qy < 0.12 { return 3; }
        if family == 2 { 4 } else if matches!(family, 0 | 5) { 0 } else { 1 }
    }

    pub(crate) fn leave_residue(&mut self, x: f32, y: f32, kind: u8) {
        if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 || x >= MAP_W as f32 || y >= MAP_H as f32 { return; }
        // Replace a cell's recent stain, preventing automatic fire from flooding the budget.
        self.residues.retain(|r| r.x.floor() != x.floor() || r.y.floor() != y.floor());
        if self.residues.len() >= RESIDUE_CAP { self.residues.remove(0); }
        self.residues.push(Residue { x, y, kind: kind.clamp(1,3), born: self.time });
    }

    pub(crate) fn update_surface_fx(&mut self) {
        self.surface_fx.fill([0.0; 4]);
        self.decal.fill(0);
        self.residues.retain(|r| self.time >= r.born && self.time-r.born < 24.0);
        for r in &self.residues {
            // Put the same stain on the closest adjoining wall face, if one was hit.
            let (x,y)=(r.x.floor() as i32,r.y.floor() as i32);
            let neighbors=[(x-1,y,r.x.fract()),(x+1,y,1.0-r.x.fract()),(x,y-1,r.y.fract()),(x,y+1,1.0-r.y.fract())];
            if let Some(&(wx,wy,_))=neighbors.iter().filter(|&&(wx,wy,d)|d<0.22 && wx>=0 && wy>=0 && wx<MAP_W as i32 && wy<MAP_H as i32 && self.blocked(wx,wy)).min_by(|a,b|a.2.total_cmp(&b.2)) {
                self.decal[wy as usize*MAP_W+wx as usize]=r.kind;
            }
            let i = r.y as usize * MAP_W + r.x as usize;
            let fade = ((24.0-(self.time-r.born))/6.0).clamp(0.0,1.0);
            self.surface_fx[i][1] = r.kind as f32 + fade * 0.9;
            self.surface_fx[i][2] = r.x.fract(); self.surface_fx[i][3] = r.y.fract();
        }
        for e in &self.ents {
            if e.kind == 0 || matches!(e.kind, EK_PROJ | EK_BOLT | EK_RAY | EK_IMPACT | EK_SMOKE | EK_FIREPATCH | EK_FLAME)
                || (e.kind == EK_SPARK && e.effect_tick != 5.0) { continue; }
            if (e.x-self.px).powi(2)+(e.y-self.py).powi(2)>144.0 { continue; }
            let height = if e.kind == EK_SPARK { ((e.zoff-68.0).abs()/100.0).min(1.0) } else if is_hostile_kind(e.kind) && enemies::combat_skin(e.skin)==8 {0.7} else { 0.0 };
            let radius = if e.kind == EK_SPARK { 0.32+height*0.2 } else if e.kind == EK_BOSS { 1.0 } else { 0.62 };
            let left = self.light_at(e.x-0.5,e.y).iter().sum::<f32>();
            let right = self.light_at(e.x+0.5,e.y).iter().sum::<f32>();
            let above = self.light_at(e.x,e.y-0.5).iter().sum::<f32>();
            let below = self.light_at(e.x,e.y+0.5).iter().sum::<f32>();
            let cx=e.x+(left-right).clamp(-0.35,0.35); let cy=e.y+(above-below).clamp(-0.35,0.35);
            for y in (cy-radius).floor().max(0.0) as usize..=((cy+radius).ceil() as usize).min(MAP_H-1) {
                for x in (cx-radius).floor().max(0.0) as usize..=((cx+radius).ceil() as usize).min(MAP_W-1) {
                    if self.blocked(x as i32,y as i32) { continue; }
                    let d2=(x as f32+0.5-cx).powi(2)+(y as f32+0.5-cy).powi(2);
                    let shadow=(1.0-d2/(radius+0.45).powi(2)).max(0.0).powi(2)*0.32*(1.0-height*0.8);
                    let i=y*MAP_W+x; self.surface_fx[i][0]=self.surface_fx[i][0].max(shadow);
                }
            }
        }
    }

    /// Match the eight-pixel interpolation used for CPU floor lighting, keeping
    /// SIMD texel batches intact. GPU surfaces sample the same grid per fragment.
    pub(crate) fn floor_shadow(&self,x:f32,y:f32) -> f32 {
        let sx=(x-0.5).clamp(0.0,(MAP_W-2) as f32); let sy=(y-0.5).clamp(0.0,(MAP_H-2) as f32);
        let i=sy as usize*MAP_W+sx as usize; let u=sx.fract(); let v=sy.fract();
        let a=self.surface_fx[i][0]*(1.0-u)+self.surface_fx[i+1][0]*u;
        let b=self.surface_fx[i+MAP_W][0]*(1.0-u)+self.surface_fx[i+MAP_W+1][0]*u;
        a*(1.0-v)+b*v
    }

    pub(crate) fn finish_floor_fx(&self, base: u32, x: f32, y: f32) -> u32 {
        if self.residues.is_empty() { return base; }
        let ix=x.floor() as i32; let iy=y.floor() as i32;
        if ix<0 || iy<0 || ix>=MAP_W as i32 || iy>=MAP_H as i32 { return base; }
        let data=self.surface_fx[iy as usize*MAP_W+ix as usize];
        let mut result=base;
        let kind=data[1].floor() as u8;
        if kind>0 && self.floor[iy as usize*MAP_W+ix as usize]<2 {
            let q=(x.fract()-data[2]).powi(2)+(y.fract()-data[3]).powi(2);
            let mask=(1.0-q/0.18).max(0.0).powi(2)*data[1].fract()*0.7;
            let color=match kind {2=>Self::pack(61,91,18,255),3=>Self::pack(93,91,86,255),_=>Self::pack(18,13,11,255)};
            result=Self::blend(result,color,mask);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn residues_are_bounded_fade_and_do_not_change_rng() {
        let mut e=testutil::arena(); let seed=e.rng;
        for n in 0..100 { e.leave_residue((n%40) as f32+1.5,(n/40) as f32+1.5,2); }
        assert_eq!(e.residues.len(),RESIDUE_CAP); e.update_surface_fx(); assert_eq!(e.rng,seed);
        e.time=25.0; e.update_surface_fx(); assert!(e.residues.is_empty());
    }
    #[test] fn floor_material_follows_local_floor_and_puddles() {
        let mut e=testutil::arena(); e.wave=4;
        assert_eq!(e.floor_material(4.5,4.5),1);
        e.floor[4*MAP_W+4]=1; assert_eq!(e.floor_material(4.5,4.5),0);
        e.floor[4*MAP_W+4]=6; assert_eq!(e.floor_material(4.5,4.5),2);
        e.floor.fill(0); e.wave=13;
        assert!((1..30).any(|x| e.floor_material(x as f32+0.5,4.5)==3));
    }
    #[test] fn contact_shadow_follows_height_and_does_not_stamp_walls() {
        let mut e=testutil::arena(); let id=e.spawn(EK_SPARK,5.5,5.5).unwrap();
        e.ents[id].effect_tick=5.0; e.ents[id].zoff=68.0;
        e.update_surface_fx(); let grounded=e.surface_fx[5*MAP_W+5][0]; assert!(grounded>0.0);
        e.ents[id].zoff = -32.0; e.update_surface_fx(); assert!(e.surface_fx[5*MAP_W+5][0]<grounded);
        e.map[5*MAP_W+5]=1; e.update_surface_fx(); assert_eq!(e.surface_fx[5*MAP_W+5][0],0.0);
    }
    #[test] fn stains_align_to_adjacent_walls_and_expire_on_both_surfaces() {
        let mut e=testutil::arena(); e.map[4*MAP_W+5]=1;
        e.leave_residue(4.92,4.5,2); e.update_surface_fx();
        assert_eq!(e.decal[4*MAP_W+5],2);
        assert_eq!(e.surface_fx[4*MAP_W+4][1].floor(),2.0);
        assert_eq!(e.decal.iter().filter(|&&v|v!=0).count(),1);
        e.time=25.0; e.update_surface_fx(); assert!(e.decal.iter().all(|&v|v==0));
    }
}
