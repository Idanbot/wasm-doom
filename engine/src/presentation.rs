//! Accessibility preferences affect drawing, never combat or the random stream.
use crate::*;

#[derive(Clone, Copy)]
pub(crate) struct Effects { pub flash: f32, pub shake: f32, pub debris: f32 }
impl Default for Effects {
    fn default() -> Self { Self { flash: 1.0, shake: 1.0, debris: 1.0 } }
}
impl Effects {
    pub(crate) fn new(flash: f32, shake: f32, debris: f32) -> Self {
        let level = |v: f32| if v.is_finite() { v.clamp(0.0, 1.0) } else { 1.0 };
        Self { flash: level(flash), shake: level(shake), debris: level(debris) }
    }
    pub(crate) fn show_sprite(&self, index: usize, ent: &Ent) -> bool {
        if ent.kind != EK_GIB && !(ent.kind == EK_SPARK && ent.effect_tick == 5.0) { return true; }
        // Stable while a fragment moves. No random draws and no entity deletion.
        let sample = ((index as u32).wrapping_mul(2654435761).rotate_left(13) & 65535) as f32 / 65536.0;
        sample < self.debris
    }
}
impl Engine {
    pub(crate) fn set_effects(&mut self, flash: f32, shake: f32, debris: f32) {
        self.effects = Effects::new(flash, shake, debris);
        self.hud.muzzle = self.muzzle * self.effects.flash;
        self.hud.hurt = self.hurt * self.effects.flash;
        self.hud.shake = self.shake * self.shake * self.effects.shake;
    }
}
