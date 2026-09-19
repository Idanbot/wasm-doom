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
