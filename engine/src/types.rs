//! Core entity types used by the simulation.

#[derive(Clone, Copy)]
pub(crate) struct Ent {
    pub(crate) kind: u8,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) vx: f32,
    pub(crate) vy: f32,
    pub(crate) hp: i32,
    pub(crate) timer: f32,
    pub(crate) frame: f32,
    /// Animation group and elapsed time for the BLACKSITE enemy sheets.
    pub(crate) anim: u8,
    pub(crate) anim_time: f32,
    pub(crate) anim_lock: f32,
    /// Presentation skin. `SKIN_NONE` keeps the legacy effect/prop texture.
    pub(crate) skin: u8,
    pub(crate) radius: f32,
    pub(crate) flash: f32,
    pub(crate) stun: f32,
    pub(crate) effect_tick: f32,
    /// Target angle committed at the start of an attack windup.
    pub(crate) aim: f32,
    /// Front-shield flag. 1 blocks damage from the facing arc.
    pub(crate) shield: u8,
    /// Direction the shield and the body are facing.
    pub(crate) face: f32,
    pub(crate) zoff: f32,
    /// Seconds left to show the damage bar. Set on any hit, including a shield clang.
    pub(crate) bar_t: f32,
    /// Amber pool that depletes before health.
    pub(crate) armor_hp: i32,
    /// Cyan front plate. Depletes before armor. 0 means the plate is gone.
    pub(crate) shield_hp: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct FxCmd {
    pub(crate) variant: u8,
    pub(crate) kind: u8,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) vx: f32,
    pub(crate) vy: f32,
    pub(crate) timer: f32,
    pub(crate) zoff: f32,
}

/// A one-shot ambush zone. When the player steps inside the rect, the
/// matching group from `map::AMBUSH_DEFS` spawns and `fired` latches.
#[derive(Clone, Copy)]
pub(crate) struct AmbushTrigger {
    pub(crate) x0: f32,
    pub(crate) y0: f32,
    pub(crate) x1: f32,
    pub(crate) y1: f32,
    pub(crate) fired: bool,
}
