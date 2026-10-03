//! Bounded presentation events. Drained after every simulation tick so positional
//! world sounds are not lost when several ticks run in a single rendered frame.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SoundCue { pub kind: f32, pub variant: f32, pub x: f32, pub y: f32 }
pub const CAP: usize = 64;
