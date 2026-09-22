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
    pub(crate) zoff: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct FxCmd {
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
