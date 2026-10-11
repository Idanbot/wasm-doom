//! Deterministic locomotion between attacks. Windups and recovery stay planted.
use core::f32::consts::PI;

#[derive(Clone, Copy)]
enum Pattern {
    Orbit,
    Weave,
    FigureEight,
    Stutter,
    Surge,
    Spiral,
    Flank,
    Siege,
}
#[derive(Clone, Copy)]
struct Profile {
    pattern: Pattern,
    range: f32,
    rate: f32,
    side: f32,
    speed: f32,
}
const fn p(pattern: Pattern, range: f32, rate: f32, side: f32, speed: f32) -> Profile {
    Profile {
        pattern,
        range,
        rate,
        side,
        speed,
    }
}
use Pattern::*;
// Campaign order: each boss owns a distinct rhythm, spacing and handedness.
const PROFILES: [Profile; 25] = [
    p(Weave, 5.0, 1.5, 0.8, 1.0),         // Veyran: rail sidesteps
    p(Orbit, 5.8, 0.8, -0.8, 0.85),       // Hecate: clockwise arc
    p(Surge, 3.5, 1.1, 0.35, 0.95),       // Chimera: corrosive pressure
    p(FigureEight, 6.0, 1.0, 0.9, 1.0),   // Oracle: predictive crossings
    p(Spiral, 4.5, 0.7, -0.7, 0.75),      // Gravemind: reactor spiral
    p(Stutter, 5.4, 1.4, 0.9, 0.95),      // Archivist: stop/start indexing
    p(Siege, 5.0, 0.6, -0.3, 0.65),       // Halcyon: guarded coolant march
    p(Flank, 5.8, 1.7, 1.0, 1.1),         // Relay: alternating signal flanks
    p(Siege, 4.0, 0.5, 0.25, 0.6),        // Titan: deliberate siege advance
    p(Surge, 4.5, 1.8, -0.55, 1.1),       // Kest: command feint
    p(FigureEight, 5.5, 0.7, -0.75, 0.9), // Mnemosyne: memory loop
    p(Weave, 4.2, 2.0, -0.8, 0.9),        // Vulcan: ember-lane zigzag
    p(Surge, 5.8, 0.8, 0.5, 0.85),        // Leviathan: tidal approach
    p(FigureEight, 6.4, 1.6, 1.0, 1.05),  // Prism: refracted figure eight
    p(Spiral, 4.8, 1.2, 0.8, 0.8),        // Magnus: magnetic spiral
    p(Orbit, 6.2, 0.65, 1.0, 0.85),       // Broodmother: swarm perimeter
    p(Stutter, 4.3, 0.9, -0.55, 0.7),     // Mycelium: spore creep
    p(Flank, 6.5, 0.8, -0.9, 0.9),        // Astra: missile repositioning
    p(Weave, 4.8, 2.6, 0.65, 1.0),        // Scrapper: flechette sweep
    p(Stutter, 5.6, 2.2, 1.0, 1.1),       // Umbra: phase-footwork (no teleport)
    p(Orbit, 4.6, 1.3, -0.65, 0.95),      // Sol: solar orbit
    p(Surge, 6.0, 1.3, -0.4, 0.8),        // Nautilus: pressure surge
    p(FigureEight, 5.2, 2.0, -0.85, 1.0), // Chronos: rapid echo loops
    p(Siege, 5.6, 1.1, 0.45, 0.7),        // Boreas: glacial sweep
    p(Spiral, 5.0, 1.6, -1.0, 1.1),       // Sovereign: tightening crown spiral
];

// Give the authored footwork a readable interval before the next telegraph.
pub(crate) fn reposition_time(level: usize) -> f32 {
    (1.0 / PROFILES[level].rate).clamp(0.65, 1.3)
}

/// Unitless radial/tangential steering transformed into a bounded world vector.
pub(crate) fn steering(level: usize, phase: u8, time: f32, dx: f32, dy: f32) -> (f32, f32) {
    let profile = PROFILES[level];
    let distance = dx.hypot(dy).max(0.01);
    let beat = time * profile.rate + level as f32 * 0.37;
    let mut range = profile.range;
    let mut side = profile.side;
    let mut speed = profile.speed;
    match profile.pattern {
        Orbit => range += beat.sin() * 0.3,
        Weave => side *= beat.sin(),
        FigureEight => {
            side *= beat.sin();
            range += (beat * 2.0).cos();
        }
        Stutter => {
            side *= if beat.sin() >= 0.0 { 1.0 } else { -1.0 };
            speed *= if beat.sin() > 0.35 { 1.0 } else { 0.2 };
        }
        Surge => {
            range += beat.sin() * 1.5;
            side *= beat.cos();
        }
        Spiral => {
            range += beat.sin() * 1.2;
            side *= 0.7 + beat.cos() * 0.3;
        }
        Flank => {
            side *= (beat + PI * 0.25).sin();
            range += beat.cos() * 0.6;
        }
        Siege => {
            side *= beat.sin() * 0.5;
            speed *= 0.8;
        }
    }
    // Later phases tighten space slightly; never introduce an untelegraphed dash.
    range -= phase.min(2) as f32 * 0.25;
    let radial = ((distance - range) * 0.55).clamp(-1.0, 1.0);
    let (ux, uy) = (dx / distance, dy / distance);
    let (x, y) = (ux * radial - uy * side, uy * radial + ux * side);
    let scale = speed / x.hypot(y).max(1.0);
    (x * scale, y * scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn fixture(wave: i32) -> (Engine, usize) {
        let mut e = testutil::arena();
        e.wave = wave;
        e.px = 18.5;
        e.py = 12.5;
        let i = e
            .spawn_with_skin(EK_BOSS, map::boss_skin(wave), 10.5, 12.5)
            .unwrap();
        e.ents[i].timer = 100.0;
        (e, i)
    }

    #[test]
    fn all_profiles_are_distinct_bounded_and_deterministic() {
        let mut signatures = std::collections::HashSet::new();
        for level in 0..25 {
            let mut signature = Vec::new();
            for phase in 0..3 {
                for n in 0..180 {
                    let time = n as f32 / 30.0;
                    let v = steering(level, phase, time, 6.0, 2.0);
                    assert_eq!(v, steering(level, phase, time, 6.0, 2.0));
                    assert!(v.0.is_finite() && v.1.is_finite());
                    assert!(v.0.hypot(v.1) <= 1.101);
                    signature.push((v.0.to_bits(), v.1.to_bits()));
                }
            }
            assert!(signatures.insert(signature), "duplicate boss {}", level + 1);
            assert_eq!(steering(level, 0, 0.0, 0.0, 0.0), (0.0, 0.0));
        }
    }

    #[test]
    fn every_boss_moves_in_real_simulation_and_replays_identically() {
        for wave in 1..=25 {
            let (mut e, i) = fixture(wave);
            let (mut replay, j) = fixture(wave);
            for _ in 0..90 {
                e.tick(1.0 / 60.0);
                replay.tick(1.0 / 60.0);
                assert_eq!(
                    (e.ents[i].x, e.ents[i].y),
                    (replay.ents[j].x, replay.ents[j].y)
                );
                assert!(!e.circle_blocked(e.ents[i].x, e.ents[i].y, e.ents[i].radius));
            }
            assert!(
                (e.ents[i].x - 10.5).hypot(e.ents[i].y - 12.5) > 0.02,
                "stationary boss {wave}"
            );
        }
    }

    #[test]
    fn movement_respects_walls_stun_and_attack_lock() {
        for wave in 1..=25 {
            let (mut e, i) = fixture(wave);
            for x in 0..MAP_W {
                e.map[10 * MAP_W + x] = 1;
                e.map[14 * MAP_W + x] = 1;
            }
            for _ in 0..60 {
                e.tick(1.0 / 60.0);
                assert!(
                    !e.circle_blocked(e.ents[i].x, e.ents[i].y, e.ents[i].radius),
                    "wall crossing boss {wave}"
                );
            }
            for stage in [1, 2] {
                let (mut e, i) = fixture(wave);
                e.boss_attack.owner = Some(i);
                e.boss_attack.stage = stage;
                e.boss_attack.remaining = 2.0;
                e.tick(1.0 / 60.0);
                assert_eq!((e.ents[i].x, e.ents[i].y), (10.5, 12.5));
            }
            let (mut e, i) = fixture(wave);
            e.ents[i].stun = 1.0;
            e.tick(1.0 / 60.0);
            assert_eq!((e.ents[i].x, e.ents[i].y), (10.5, 12.5));
        }
    }
}
