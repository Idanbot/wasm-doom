//! Campaign progression: readable first tour, then gently growing endless waves.
pub(crate) const LEVELS: usize = 25;
pub(crate) const ACTIVE_HOSTILES: usize = 72;

pub(crate) fn health_scale(wave: i32) -> f32 {
    let tour = (wave.clamp(1, 25) - 1) as f32;
    let endless = wave.saturating_sub(25).max(0) as f32;
    (1.0 + tour * 0.012) * (1.0 + endless * 0.025 + endless.powf(1.35) * 0.0001)
}

pub(crate) fn boss_health(wave: i32) -> i32 {
    let tour = (wave.clamp(1, 25) - 1) as f32;
    let base = 480.0 + 65.0 * tour + 12.0 * tour.powf(1.35);
    let endless = wave.saturating_sub(25).max(0) as f32;
    (base * (1.0 + endless * 0.025 + endless.powf(1.35) * 0.0001)).round() as i32
}

pub(crate) fn hostile_total(wave: i32, roster: usize) -> usize {
    let copies = 1usize << (wave.saturating_sub(1).clamp(0, 2) as usize);
    let depth = wave.saturating_sub(25).max(0) as f64;
    let base_roster = if depth > 0.0 { roster.max(16) } else { roster };
    ((base_roster * copies) as f64 * (1.0 + depth * 0.018 + depth.powf(1.35) * 0.00008)).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endless_growth_has_no_gameplay_cap_and_no_cliff() {
        assert_eq!(boss_health(1), 480);
        assert!(boss_health(26) > boss_health(25));
        assert!(boss_health(26) < (boss_health(25) as f32 * 1.03) as i32);
        for depth in [26, 50, 100, 1000, 10000] {
            assert!(boss_health(depth + 25) > boss_health(depth));
            assert!(health_scale(depth + 25) > health_scale(depth));
            assert!(hostile_total(depth + 25, 16) > hostile_total(depth, 16));
        }
        for wave in 26..100 {
            assert!(hostile_total(wave + 1, 6) >= hostile_total(wave, 16), "smaller legacy casts cannot reset endless pressure");
        }
        assert!(boss_health(1000) > 6000);
        assert!(hostile_total(1000, 16) > 192);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Weapon { pub mode: u8, pub damage: i32, pub speed: f32, pub cadence: f32 }
pub(crate) const WEAPONS: [Weapon; 14] = [
    Weapon { mode: 0, damage: 55, speed: 13.0, cadence: 0.55 },
    Weapon { mode: 1, damage: 70, speed: 11.0, cadence: 0.8 },
    Weapon { mode: 2, damage: 28, speed: 20.0, cadence: 0.32 },
    Weapon { mode: 3, damage: 48, speed: 15.0, cadence: 0.5 },
    Weapon { mode: 4, damage: 22, speed: 10.0, cadence: 0.45 },
    Weapon { mode: 5, damage: 38, speed: 8.0, cadence: 0.7 },
    Weapon { mode: 6, damage: 95, speed: 14.0, cadence: 1.05 },
    Weapon { mode: 7, damage: 16, speed: 18.0, cadence: 0.16 },
    Weapon { mode: 8, damage: 45, speed: 17.0, cadence: 0.42 },
    Weapon { mode: 9, damage: 75, speed: 12.0, cadence: 0.75 },
    Weapon { mode: 10, damage: 85, speed: 12.0, cadence: 0.95 },
    Weapon { mode: 11, damage: 62, speed: 22.0, cadence: 0.65 },
    Weapon { mode: 12, damage: 35, speed: 11.0, cadence: 0.6 },
    Weapon { mode: 13, damage: 44, speed: 18.0, cadence: 0.3 },
];

/// Light colors match the authored projectile and boss palettes.
pub(crate) const LIGHTS: [[f32; 3]; 14] = [
    [0.72,0.22,0.04], [0.08,0.48,0.65], [0.08,0.62,0.72], [0.72,0.08,0.05],
    [0.55,0.42,0.03], [0.14,0.62,0.08], [0.40,0.12,0.72], [0.68,0.32,0.04],
    [0.38,0.10,0.62], [0.72,0.54,0.12], [0.05,0.56,0.65], [0.72,0.36,0.06],
    [0.14,0.52,0.72], [0.72,0.52,0.08],
];
