//! Simulation core.
//!
//! `Engine::tick` is the one place that advances time. Everything it touches is
//! defined in a sibling module: [`movement`] for collision, [`combat`] for
//! damage and weapon fire, [`world`] for spawning and wave progression,
//! [`fx`] for the transient effect queue, [`geometry`] for map queries,
//! [`weapons`] for ownership, [`lighting`] for the light grid and
//! [`textures`] for the atlas.
//!
//! The split is by responsibility, not by call order: `tick` orchestrates and
//! delegates rather than reimplementing.

pub mod combat;
pub mod fx;
pub mod geometry;
pub mod movement;
pub mod weapons;
pub mod world;
