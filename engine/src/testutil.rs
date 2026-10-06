//! Shared fixtures for the engine's unit tests.
//!
//! `arena()` is an empty map with the player parked in a corner, which is what
//! most behavioural tests want: no geometry to trip over and a known pose to
//! assert movement against.

#![cfg(test)]

use crate::*;

pub(crate) fn arena() -> Engine {
    let mut e = Engine::new(160, 100);
    e.map.fill(0);
    for ent in &mut e.ents {
        ent.kind = 0;
    }
    e.px = 4.5;
    e.py = 4.5;
    e
}
