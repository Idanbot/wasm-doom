//! Boss gun-inspired attacks, locked telegraphs and punishable recovery.
use crate::*;
use tactical::Role;

#[derive(Clone, Copy)]
pub(crate) struct Profile {
    pub windup: f32,
    pub recovery: f32,
    pub weak: Role,
}
// Keep the level order and timings synchronized with src/game/boss-attacks.ts.
pub(crate) const PROFILES: [Profile; 25] = {
    use Role::*;
    [
        Profile {
            windup: 1.0,
            recovery: 1.5,
            weak: Precision,
        },
        Profile {
            windup: 1.2,
            recovery: 1.8,
            weak: Shock,
        },
        Profile {
            windup: 1.1,
            recovery: 1.6,
            weak: Burn,
        },
        Profile {
            windup: 1.0,
            recovery: 1.5,
            weak: Control,
        },
        Profile {
            windup: 1.3,
            recovery: 1.9,
            weak: Blast,
        },
        Profile {
            windup: 1.1,
            recovery: 1.6,
            weak: Precision,
        },
        Profile {
            windup: 1.2,
            recovery: 1.7,
            weak: Burn,
        },
        Profile {
            windup: 0.9,
            recovery: 1.5,
            weak: Stagger,
        },
        Profile {
            windup: 1.4,
            recovery: 2.0,
            weak: Precision,
        },
        Profile {
            windup: 1.0,
            recovery: 1.5,
            weak: Suppress,
        },
        Profile {
            windup: 1.3,
            recovery: 1.9,
            weak: Shock,
        },
        Profile {
            windup: 1.2,
            recovery: 1.7,
            weak: Freeze,
        },
        Profile {
            windup: 1.2,
            recovery: 1.8,
            weak: Shock,
        },
        Profile {
            windup: 1.1,
            recovery: 1.6,
            weak: Stagger,
        },
        Profile {
            windup: 1.3,
            recovery: 1.9,
            weak: Acid,
        },
        Profile {
            windup: 1.4,
            recovery: 2.0,
            weak: Blast,
        },
        Profile {
            windup: 1.3,
            recovery: 1.8,
            weak: Burn,
        },
        Profile {
            windup: 1.4,
            recovery: 2.0,
            weak: Precision,
        },
        Profile {
            windup: 1.1,
            recovery: 1.6,
            weak: Shock,
        },
        Profile {
            windup: 1.2,
            recovery: 1.8,
            weak: Control,
        },
        Profile {
            windup: 1.4,
            recovery: 2.0,
            weak: Freeze,
        },
        Profile {
            windup: 1.3,
            recovery: 1.9,
            weak: Precision,
        },
        Profile {
            windup: 1.3,
            recovery: 1.8,
            weak: Suppress,
        },
        Profile {
            windup: 1.4,
            recovery: 2.0,
            weak: Burn,
        },
        Profile {
            windup: 1.5,
            recovery: 2.1,
            weak: Shock,
        },
    ]
};

pub(crate) struct AttackState {
    pub owner: Option<usize>,
    pub stage: i32,
    pub remaining: f32,
    target: (f32, f32),
    origin: (f32, f32),
    aim: f32,
    fx_t: f32,
    pub sequence: u32,
}
impl AttackState {
    pub fn new() -> Self {
        Self {
            owner: None,
            stage: 0,
            remaining: 0.0,
            target: (0.0, 0.0),
            origin: (0.0, 0.0),
            aim: 0.0,
            fx_t: 0.0,
            sequence: 0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Shot {
    forward: f32,
    side: f32,
    angle: f32,
    speed: f32,
}
fn shot(forward: f32, side: f32, angle: f32, speed: f32) -> Shot {
    Shot {
        forward,
        side,
        angle,
        speed,
    }
}
/// Different emission geometry AND arrival rhythms, rather than reskinned fans.
fn pattern(level: usize, cycle: u32) -> Vec<Shot> {
    let flip = if cycle.is_multiple_of(2) { 1.0 } else { -1.0 };
    match level {
        0 => vec![shot(0.0, -0.55, 0.0, 8.5), shot(0.0, 0.55, 0.0, 6.5)], // override rails
        1 => (-2..=2)
            .map(|n| shot(0.0, n as f32 * 0.45, 0.0, 5.0))
            .collect(), // cutter wall
        2 => (0..5)
            .map(|n| shot(0.0, 0.0, (n as f32 - 2.0) * 0.09, 3.4 + n as f32 * 0.35))
            .collect(),
        3 => vec![
            shot(0.0, -1.0, 0.14, 7.0),
            shot(0.0, 1.0, -0.14, 7.0),
            shot(0.0, 0.0, 0.0, 4.0),
        ],
        4 => (-2..=2)
            .map(|n| shot(0.0, n as f32 * 0.7, -n as f32 * 0.1, 4.1))
            .collect(),
        5 => (0..4)
            .map(|n| {
                shot(
                    0.0,
                    if n % 2 == 0 { -0.3 } else { 0.3 },
                    0.0,
                    5.0 + n as f32,
                )
            })
            .collect(),
        6 => (-3i32..=3)
            .map(|n| shot(n.unsigned_abs() as f32 * 0.15, 0.0, n as f32 * 0.14, 3.6))
            .collect(),
        7 => (0..5)
            .map(|n| {
                shot(
                    0.0,
                    0.0,
                    (n as f32 - 2.0) * 0.07 * flip,
                    4.0 + n as f32 * 0.9,
                )
            })
            .collect(),
        8 => vec![
            shot(0.0, -0.8, 0.1, 3.0),
            shot(0.0, 0.8, -0.1, 3.0),
            shot(0.0, 0.0, 0.0, 4.4),
        ],
        9 => vec![
            shot(0.0, -0.35, 0.035, 8.0),
            shot(0.0, 0.0, 0.0, 6.8),
            shot(0.0, 0.35, -0.035, 5.6),
        ],
        10 => (0..6)
            .map(|n| {
                shot(
                    0.0,
                    0.0,
                    if n % 2 == 0 { -0.1 } else { 0.1 },
                    3.3 + n as f32 * 0.7,
                )
            })
            .collect(),
        11 => vec![
            shot(0.0, -0.4, -0.12, 5.4),
            shot(0.0, 0.4, 0.12, 5.4),
            shot(0.0, 0.0, 0.0, 3.3),
        ],
        12 => (-2..=2)
            .map(|n| {
                shot(
                    0.0,
                    0.0,
                    n as f32 * 0.18,
                    if n % 2 == 0 { 4.4 } else { 6.0 },
                )
            })
            .collect(),
        13 => (-2..=2)
            .map(|n| shot(0.0, n as f32 * 0.4, -n as f32 * 0.15, 6.2))
            .collect(),
        14 => (0..6)
            .map(|n| shot(0.0, 0.0, (n as f32 - 2.5) * 0.22, 4.3))
            .collect(),
        15 => (0..4)
            .map(|n| {
                shot(
                    0.0,
                    0.0,
                    n as f32 * core::f32::consts::FRAC_PI_2 + cycle as f32 * 0.25,
                    3.8,
                )
            })
            .collect(),
        16 => (0..7)
            .map(|n| {
                shot(
                    0.0,
                    (n as f32 - 3.0) * 0.22,
                    (n as f32 - 3.0) * 0.06,
                    2.6 + (n % 3) as f32 * 0.45,
                )
            })
            .collect(),
        17 => (-2..=2)
            .map(|n| {
                shot(
                    0.0,
                    n as f32 * 0.65,
                    -n as f32 * 0.08,
                    3.4 + (n + 2) as f32 * 0.45,
                )
            })
            .collect(),
        18 => (-4..=4)
            .map(|n| shot(0.0, n as f32 * 0.24, 0.0, 7.2))
            .collect(),
        19 => vec![shot(0.0, -0.25, 0.0, 7.3), shot(0.0, 0.25, 0.0, 4.8)],
        20 => (1..12)
            .map(|n| shot(0.0, 0.0, n as f32 * core::f32::consts::TAU / 12.0, 4.0))
            .collect(), // gap ahead
        21 => vec![shot(0.0, -0.65, 0.06, 3.2), shot(0.0, 0.65, -0.06, 4.6)],
        22 => vec![
            shot(0.0, 0.0, -0.05, 3.0),
            shot(0.0, 0.0, 0.0, 5.0),
            shot(0.0, 0.0, 0.05, 7.0),
        ],
        23 => (-2..=2)
            .map(|n| shot(0.0, n as f32 * 0.3, n as f32 * 0.12, 3.5))
            .collect(),
        _ => (0..6)
            .map(|n| {
                shot(
                    0.0,
                    if n % 2 == 0 { -0.75 } else { 0.75 },
                    (n as f32 - 2.5) * 0.08,
                    4.2 + (n % 3) as f32,
                )
            })
            .collect(),
    }
}

impl Engine {
    /// Called before ordinary movement AI. Returns true while this boss is busy.
    pub(crate) fn tick_boss_attack(&mut self, i: usize, dt: f32) -> bool {
        let e = self.ents[i];
        let profile = PROFILES[map::level_index(self.wave)];
        if self.state != 0 {
            return true;
        }
        if e.stun > 0.0 && self.boss_attack.stage != 2 {
            if self.boss_attack.owner == Some(i) && self.boss_attack.stage == 1 {
                self.boss_attack.stage = 0;
                self.boss_attack.remaining = 0.0;
                self.ents[i].effect_tick = 0.0;
                self.ents[i].timer = 0.6;
            }
            return false; // existing stun/knockback physics
        }
        if self.boss_attack.owner == Some(i) && self.boss_attack.stage != 0 {
            self.ents[i].stun = (self.ents[i].stun - dt).max(0.0);
            if self.boss_attack.stage == 2 && self.ents[i].anim_lock <= 0.0 {
                set_anim(&mut self.ents[i], ANIM_RELOAD, self.boss_attack.remaining);
            }
            self.boss_attack.remaining = (self.boss_attack.remaining - dt).max(0.0);
            self.boss_attack.fx_t -= dt;
            if self.boss_attack.fx_t <= 0.0 {
                self.boss_attack.fx_t = 0.2;
                self.boss_attack_fx(i);
            }
            if self.boss_attack.remaining <= 0.0 {
                if self.boss_attack.stage == 1 {
                    self.resolve_boss_attack(i);
                } else {
                    self.boss_attack.stage = 0;
                    self.ents[i].timer = 0.55; // reset instead of retaining a stale cooldown
                    set_anim(&mut self.ents[i], ANIM_IDLE, 0.2);
                }
            }
            return true;
        }
        if e.timer <= dt
            && (self.px - e.x).hypot(self.py - e.y) < 12.0
            && self.los(e.x, e.y, self.px, self.py)
        {
            self.boss_attack.owner = Some(i);
            self.boss_attack.stage = 1;
            self.boss_attack.remaining = profile.windup;
            self.boss_attack.fx_t = 0.0;
            self.boss_attack.target = (self.px, self.py);
            self.boss_attack.origin = (e.x, e.y);
            self.boss_attack.aim = (self.py - e.y).atan2(self.px - e.x);
            self.ents[i].aim = self.boss_attack.aim;
            if map::level_index(self.wave) == 23 {
                self.arm_shield(i);
            }
            set_anim(&mut self.ents[i], ANIM_SPECIAL, profile.windup);
            self.boss_attack_fx(i);
            self.sound(13, 1, e.x, e.y);
            return true;
        }
        false
    }

    fn boss_attack_fx(&mut self, i: usize) {
        if self.boss_attack.stage == 2 {
            // A raised core beacon remains attached to the exposed boss.
            self.effect(EK_SPARK, 2, self.ents[i].x, self.ents[i].y, 0.22, -28.0);
            return;
        }
        let level = map::level_index(self.wave);
        let (x, y) = self.boss_attack.origin;
        let a = self.boss_attack.aim;
        // Sample EVERY lane, capped at three markers per lane. Cosmetic only.
        for s in pattern(level, self.boss_attack.sequence) {
            let sx = x + a.cos() * s.forward - a.sin() * s.side;
            let sy = y + a.sin() * s.forward + a.cos() * s.side;
            if self.circle_blocked(sx, sy, 0.12) || !self.los(x, y, sx, sy) {
                continue;
            }
            let angle = a + s.angle;
            let d = self
                .wall_distance(sx, sy, angle.cos(), angle.sin(), 8.0)
                .min(8.0);
            for n in 1..=3 {
                let t = d * n as f32 / 4.0;
                self.spawn_timed(
                    EK_RAY,
                    sx + angle.cos() * t,
                    sy + angle.sin() * t,
                    0.23,
                    14.0,
                );
            }
        }
        if matches!(level, 4 | 14 | 19) {
            let (tx, ty) = self.boss_attack.target;
            for n in 0..6 {
                let r = n as f32 * core::f32::consts::TAU / 6.0;
                let (mx, my) = (tx + r.cos() * 1.2, ty + r.sin() * 1.2);
                if !self.circle_blocked(mx, my, 0.1) {
                    self.effect(EK_SPARK, 1, mx, my, 0.23, 12.0);
                }
            }
        }
    }

    pub(crate) fn resolve_boss_attack(&mut self, i: usize) {
        // Only a completed anticipation can deal damage. No unannounced attacks.
        if self.boss_attack.owner != Some(i) || self.boss_attack.stage != 1 || self.boss_attack.remaining > 0.0 {
            return;
        }
        let level = map::level_index(self.wave);
        let e = self.ents[i];
        let (x, y) = self.boss_attack.origin;
        let a = self.boss_attack.aim;
        let damage = combat::profile(e.skin, EK_BOSS).damage;
        for s in pattern(level, self.boss_attack.sequence) {
            let sx = x + a.cos() * s.forward - a.sin() * s.side;
            let sy = y + a.sin() * s.forward + a.cos() * s.side;
            if self.circle_blocked(sx, sy, 0.12) || !self.los(x, y, sx, sy) {
                continue;
            }
            if let Some(j) = self.spawn(EK_PROJ, sx, sy) {
                let p = &mut self.ents[j];
                let angle = a + s.angle;
                p.vx = angle.cos() * s.speed;
                p.vy = angle.sin() * s.speed;
                p.timer = 3.5;
                p.hp = damage;
                p.skin = 100 + e.skin;
                p.zoff = -8.0;
                p.effect_tick = if matches!(level, 2 | 16) { 3.0 } else { 0.0 };
                if level >= 11 {
                    p.face = (level - 10) as f32;
                } // existing campaign impact identities
            }
        }
        let (tx, ty) = self.boss_attack.target;
        // Gravity only captures players who stayed in its advertised target disk.
        if matches!(level, 4 | 14)
            && (self.px - tx).hypot(self.py - ty) < 1.2
            && self.los(x, y, self.px, self.py)
        {
            let d = (self.px - x).hypot(self.py - y).max(0.01);
            self.try_move(
                self.px + (x - self.px) / d * 0.55,
                self.py + (y - self.py) / d * 0.55,
            );
            self.shake = self.shake.max(0.2);
        }
        if level == 19 {
            // Ambush the warned position, never silently track the dodging player.
            let (nx, ny) = self.nearest_open(tx, ty, e.radius);
            if (nx - tx).hypot(ny - ty) < 0.6
                && (nx - self.px).hypot(ny - self.py) > 1.2
                && self.los(x, y, nx, ny)
            {
                self.effect(EK_IMPACT, 3, x, y, 0.3, -8.0);
                self.ents[i].x = nx;
                self.ents[i].y = ny;
            }
        }
        if matches!(level, 15 | 24) {
            let kind = if level == 15 { EK_MARTYR } else { EK_BRUTE };
            let skin = if level == 15 {
                SKIN_MARTYR
            } else {
                SKIN_GUNNER
            };
            let cap = if level == 15 { 3 } else { 2 };
            let living = self
                .ents
                .iter()
                .filter(|e| e.kind == kind && e.skin == skin && e.hp > 0)
                .count();
            if living < cap && map::living_hostiles(self) < campaign::ACTIVE_HOSTILES {
                let (nx, ny) = self.nearest_open(x - a.sin() * 1.6, y + a.cos() * 1.6, 0.34);
                if (nx - self.px).hypot(ny - self.py) > 1.2 {
                    if let Some(j) = self.spawn_with_skin(kind, skin, nx, ny) {
                        if level == 24 {
                            self.arm_shield(j);
                        }
                    }
                }
            }
        }
        self.boss_attack.sequence = self.boss_attack.sequence.wrapping_add(1);
        self.boss_attack.stage = 2;
        self.boss_attack.remaining = PROFILES[level].recovery;
        self.boss_vuln = self.boss_vuln.max(PROFILES[level].recovery);
        self.ents[i].shield_hp = 0;
        self.ents[i].shield = 0;
        self.ents[i].effect_tick = 0.0;
        set_anim(&mut self.ents[i], ANIM_FIRE, 0.3);
        self.effect(EK_IMPACT, 1, e.x, e.y, 0.25, -8.0);
        self.sound(2, 2, e.x, e.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn arena(wave: i32) -> (Engine, usize) {
        let mut e = Engine::new(160, 100);
        e.map.fill(0);
        for en in &mut e.ents {
            en.kind = EK_NONE;
        }
        e.wave = wave;
        e.state = 0;
        e.px = 12.5;
        e.py = 10.5;
        e.health = 100;
        e.armor = 0;
        let i = e
            .spawn_with_skin(EK_BOSS, map::boss_skin(wave), 6.5, 10.5)
            .unwrap();
        e.ents[i].timer = 0.0;
        e.ents[i].armor_hp = 0;
        e.ents[i].shield_hp = 0;
        e.ents[i].shield = 0;
        (e, i)
    }
    #[test]
    fn twenty_five_patterns_have_distinct_geometry_and_safe_timings() {
        let mut seen = std::collections::HashSet::new();
        for level in 0..25 {
            let fingerprint: Vec<_> = pattern(level, 0)
                .iter()
                .map(|s| {
                    (
                        s.forward.to_bits(),
                        s.side.to_bits(),
                        s.angle.to_bits(),
                        s.speed.to_bits(),
                    )
                })
                .collect();
            assert!(seen.insert(fingerprint), "duplicate level {}", level + 1);
            assert!(PROFILES[level].windup >= 0.9);
            assert!(PROFILES[level].recovery >= 1.5);
        }
    }
    #[test]
    fn all_bosses_warn_then_attack_with_locked_aim_and_recover_in_phase_one() {
        for wave in 1..=25 {
            let (mut e, i) = arena(wave);
            assert!(e.tick_boss_attack(i, 0.01));
            assert_eq!(e.boss_attack.stage, 1);
            assert_eq!(e.health, 100);
            assert_eq!(e.ents.iter().filter(|e| e.kind == EK_PROJ).count(), 0);
            let a = e.boss_attack.aim;
            e.py += 3.0;
            e.tick_boss_attack(i, PROFILES[(wave - 1) as usize].windup - 0.02);
            assert_eq!(e.boss_attack.stage, 1);
            assert_eq!(e.boss_attack.aim, a);
            e.tick_boss_attack(i, 0.03);
            assert_eq!(e.boss_attack.stage, 2);
            assert!(e.boss_vuln >= 1.5);
            let projectiles = e.ents.iter().filter(|e| e.kind == EK_PROJ).count();
            assert!(projectiles > 0, "wave {wave}");
            assert_eq!(e.ents[i].shield_hp, 0);
            let (x, y) = (e.ents[i].x, e.ents[i].y);
            e.tick_boss_attack(i, 0.5);
            assert_eq!(
                e.ents.iter().filter(|e| e.kind == EK_PROJ).count(),
                projectiles
            );
            assert_eq!((e.ents[i].x, e.ents[i].y), (x, y));
            e.tick_boss_attack(i, 3.0);
            assert_eq!(e.boss_attack.stage, 0);
        }
    }
    #[test]
    fn recovery_rewards_the_indicated_weapon_without_requiring_it() {
        for wave in 1..=25 {
            let (mut e, i) = arena(wave);
            e.tick_boss_attack(i, 0.01);
            e.tick_boss_attack(i, 2.0);
            let weak = PROFILES[(wave - 1) as usize].weak;
            let other = if weak == Role::Precision {
                Role::Blast
            } else {
                Role::Precision
            };
            e.ents[i].hp = 1000;
            e.hurt_ent_role(i, 20, 0.0, 0.0, Some(other));
            let regular = 1000 - e.ents[i].hp;
            e.ents[i].hp = 1000;
            e.hurt_ent_role(i, 20, 0.0, 0.0, Some(weak));
            let preferred = 1000 - e.ents[i].hp;
            assert_eq!(regular, 40, "wave {wave}");
            assert_eq!(preferred, 50, "wave {wave}");
        }
    }
    #[test]
    fn ordinary_hits_keep_the_warning_and_stagger_cancels_it_without_firing() {
        let (mut e, i) = arena(1);
        e.ents[i].hp = 1000;
        e.tick_boss_attack(i, 0.01);
        e.hurt_ent_role(i, 10, 0.0, 0.0, Some(Role::Precision));
        e.tick_boss_attack(i, 0.1);
        assert_eq!(e.boss_attack.stage, 1);
        e.hurt_ent_role(i, 10, 0.0, 0.0, Some(Role::Stagger));
        e.tick_boss_attack(i, 0.01);
        assert_eq!(e.boss_attack.stage, 0);
        e.enemy_shoot(i);
        assert!(!e.ents.iter().any(|e| e.kind == EK_PROJ));
    }
    #[test]
    fn lane_origins_never_cross_walls_and_minions_stay_bounded() {
        let (mut e, i) = arena(19);
        e.map[9 * MAP_W + 6] = 1;
        e.map[11 * MAP_W + 6] = 1;
        e.tick_boss_attack(i, 0.01);
        e.tick_boss_attack(i, 2.0);
        for p in e.ents.iter().filter(|p| p.kind == EK_PROJ) {
            assert!(!e.circle_blocked(p.x, p.y, 0.12));
            assert!(e.los(6.5, 10.5, p.x, p.y));
        }
        for wave in [16, 25] {
            let (mut e, i) = arena(wave);
            for _ in 0..15 {
                e.ents[i].timer = 0.0;
                e.tick_boss_attack(i, 0.01);
                e.tick_boss_attack(i, 2.0);
                e.tick_boss_attack(i, 3.0);
            }
            let kind = if wave == 16 { EK_MARTYR } else { EK_BRUTE };
            assert!(
                e.ents
                    .iter()
                    .filter(|en| en.kind == kind && en.hp > 0)
                    .count()
                    <= if wave == 16 { 3 } else { 2 }
            );
        }
    }
    #[test]
    fn gravity_and_ambush_do_not_follow_a_dodging_player() {
        for wave in [5, 15, 20] {
            let (mut e, i) = arena(wave);
            e.tick_boss_attack(i, 0.01);
            e.px = 14.5;
            e.py = 14.5;
            let before = (e.px, e.py);
            e.tick_boss_attack(i, 2.0);
            assert_eq!((e.px, e.py), before);
            if wave == 20 {
                assert!((e.ents[i].x - 12.5).hypot(e.ents[i].y - 10.5) < 0.6);
            }
        }
    }
}
