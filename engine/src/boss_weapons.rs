//! Boss reward identities and bounded, deterministic delayed attacks.
//!
//! Pressure belongs to a weapon, never to the currently selected slot. Pending
//! damage retains its originating shot/role so switching guns cannot change it.
use crate::*;

pub(crate) const ARCHIVE: usize = 11;
pub(crate) const PREDICTOR: usize = 12;
pub(crate) const SINK: usize = 13;
pub(crate) const RIME: usize = 14;
pub(crate) const RELAY: usize = 15;
pub(crate) const KEST: usize = 17;
pub(crate) const UNDERTOW: usize = 20;
pub(crate) const POLARITY: usize = 22;
pub(crate) const HELIOS: usize = 28;
pub(crate) const EPOCH: usize = 30;
const PENDING_CAP: usize = 32;
const VENT_TIME: f32 = 0.9;
pub(crate) const RELEASE_MIN: f32 = 0.2;

pub(crate) fn pressure_gain(slot: usize) -> f32 {
    match slot {
        SINK => 0.22,
        UNDERTOW => 0.26,
        HELIOS => 0.20,
        _ => 0.0,
    }
}

#[derive(Clone, Copy, Default)]
struct Mark {
    seconds: f32,
    shot: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct Vent {
    pub slot: usize,
    pub seconds: f32,
    start: f32,
}

#[derive(Clone, Copy)]
enum Attack {
    Pulse {
        x: f32,
        y: f32,
        radius: f32,
        damage: i32,
    },
    Burst,
}

#[derive(Clone, Copy)]
pub(crate) struct Pending {
    seconds: f32,
    slot: usize,
    shot: usize,
    attack: Attack,
}

pub(crate) struct Arsenal {
    pub pressure: [f32; WEP_N],
    pub vent: Option<Vent>,
    pub release_flash: f32,
    pub fire_held: bool,
    archive: [Mark; ENT_N],
    frost: [Mark; ENT_N],
    pending: Vec<Pending>,
}

impl Arsenal {
    pub fn new() -> Self {
        Self {
            pressure: [0.0; WEP_N],
            vent: None,
            release_flash: 0.0,
            fire_held: false,
            archive: [Mark::default(); ENT_N],
            frost: [Mark::default(); ENT_N],
            pending: Vec::with_capacity(PENDING_CAP),
        }
    }

    pub fn reset_entity(&mut self, i: usize) {
        self.archive[i] = Mark::default();
        self.frost[i] = Mark::default();
    }

    fn schedule(&mut self, seconds: f32, slot: usize, shot: usize, attack: Attack) {
        if self.pending.len() < PENDING_CAP {
            self.pending.push(Pending {
                seconds,
                slot,
                shot,
                attack,
            });
        }
    }

    pub fn cancel_handling(&mut self) {
        self.vent = None;
        self.pending.retain(|p| !matches!(p.attack, Attack::Burst));
    }
}

impl Engine {
    /// A successful discharge is the only path which spends ammunition or
    /// creates a shot record. Delayed echoes reuse that record; burst bullets
    /// each create their own, matching the actual rounds consumed.
    pub(crate) fn commit_round(&mut self, slot: usize) {
        self.run_shots.push(false);
        self.current_shot = self.run_shots.len();
        if self.power != field::POWER_FEED {
            self.mag[slot] -= 1;
            self.run_ammo = self.run_ammo.saturating_add(1);
        }
        self.events |= EV_FIRE;
        self.ev_weapon = slot as i32;
    }

    pub(crate) fn begin_pressure_vent(&mut self) -> bool {
        let slot = self.weapon as usize;
        if pressure_gain(slot) == 0.0 || self.mechanics.pressure[slot] <= 0.001 {
            return false;
        }
        self.mechanics.cancel_handling();
        self.mechanics.vent = Some(Vent {
            slot,
            seconds: VENT_TIME,
            start: self.mechanics.pressure[slot],
        });
        self.reload_t = VENT_TIME;
        self.reload_dur = VENT_TIME;
        self.muzzle = 0.0;
        self.sound(25, slot as u8, self.px, self.py);
        true
    }

    pub(crate) fn tick_boss_weapons(&mut self, dt: f32) -> Vec<Pending> {
        self.mechanics.release_flash = (self.mechanics.release_flash - dt).max(0.0);
        if self.state != 0 {
            self.mechanics.pending.clear();
            self.mechanics.vent = None;
            return Vec::new();
        }
        for mark in self
            .mechanics
            .archive
            .iter_mut()
            .chain(self.mechanics.frost.iter_mut())
        {
            mark.seconds = (mark.seconds - dt).max(0.0);
        }
        if let Some(mut vent) = self.mechanics.vent {
            vent.seconds = (vent.seconds - dt).max(0.0);
            self.mechanics.pressure[vent.slot] = vent.start * vent.seconds / VENT_TIME;
            self.reload_t = vent.seconds;
            if vent.seconds <= 0.0 {
                self.mechanics.vent = None;
                // Cooling never grants ammo. Only the subsequent real magazine
                // animation may transfer rounds from reserve.
                self.begin_reload();
            } else {
                self.mechanics.vent = Some(vent);
            }
        }
        let mut due = Vec::new();
        self.mechanics.pending.retain_mut(|p| {
            p.seconds -= dt;
            if p.seconds <= 0.0 {
                due.push(*p);
                false
            } else {
                true
            }
        });
        due
    }

    /// Resolve after this tick's input: a reload/switch on the exact due frame
    /// must cancel a burst before it spends or fires the next bullet.
    pub(crate) fn resolve_boss_weapon_actions(&mut self, due: Vec<Pending>) {
        for p in due {
            // Input/pickups can complete the sector after the timers were
            // collected. A pulse can also finish it itself when reward spawn
            // falls back to an immediate grant, so recheck each action.
            if self.state != 0 {
                self.mechanics.pending.clear();
                break;
            }
            self.current_shot = p.shot;
            match p.attack {
                Attack::Pulse {
                    x,
                    y,
                    radius,
                    damage,
                } => {
                    self.boss_area(x, y, radius, damage, p.slot, None);
                    self.weapon_ring(x, y, radius * 0.65, p.slot, 0.16);
                    self.sound(13, if p.slot == PREDICTOR { 1 } else { 2 }, x, y);
                }
                Attack::Burst
                    if self.weapon as usize == p.slot
                        && self.reload_t <= 0.0
                        && self.mag[p.slot] > 0 =>
                {
                    self.commit_round(p.slot);
                    self.muzzle = 1.0;
                    self.kick = 0.8;
                    self.reward_beam(self.pa, 24, 24.0, 0.02, 1, p.slot);
                    self.eject_casing();
                }
                Attack::Burst => {}
            }
        }
        self.current_shot = 0;
    }

    /// Ordered contacts clipped to solid geometry, shared by the new weapons.
    fn reward_contacts(
        &self,
        angle: f32,
        range: f32,
        width: f32,
        limit: usize,
    ) -> (f32, Vec<(f32, usize)>) {
        let (dx, dy) = (angle.cos(), angle.sin());
        let end = self.wall_distance(self.px, self.py, dx, dy, range);
        let mut hits = Vec::new();
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !target_kind(e.kind) {
                continue;
            }
            let (x, y) = (e.x - self.px, e.y - self.py);
            let along = x * dx + y * dy;
            if along > 0.1
                && along < end
                && (x * dy - y * dx).abs() < e.radius + width
                && self.los(self.px, self.py, e.x, e.y)
            {
                hits.push((along, i));
            }
        }
        hits.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        hits.truncate(limit);
        (end, hits)
    }

    fn weapon_tracer(&mut self, from: (f32, f32), to: (f32, f32), slot: usize, life: f32) {
        let distance = (to.0 - from.0).hypot(to.1 - from.1);
        // A scatter discharge has nine lanes; budget their visual dots so it
        // cannot consume the entity slots reserved for live combat.
        let max_steps = if slot == SINK { 3 } else { 18 };
        let steps = ((distance / 0.65).ceil() as usize).clamp(1, max_steps);
        for n in 1..=steps {
            let t = n as f32 / (steps + 1) as f32;
            if let Some(i) = self.spawn(
                EK_RAY,
                from.0 + (to.0 - from.0) * t,
                from.1 + (to.1 - from.1) * t,
            ) {
                self.ents[i].timer = life;
                self.ents[i].projectile_visual = (slot - 7) as u8;
                self.ents[i].zoff = 8.0;
            }
        }
    }

    fn weapon_ring(&mut self, x: f32, y: f32, radius: f32, slot: usize, life: f32) {
        for n in 0..8 {
            let a = n as f32 * core::f32::consts::TAU / 8.0;
            let (rx, ry) = (x + a.cos() * radius, y + a.sin() * radius);
            if self.blocked(rx.floor() as i32, ry.floor() as i32) || !self.los(x, y, rx, ry) {
                continue;
            }
            if let Some(i) = self.spawn(EK_RAY, rx, ry) {
                self.ents[i].timer = life;
                self.ents[i].projectile_visual = (slot - 7) as u8;
                self.ents[i].zoff = 24.0;
            }
        }
    }

    fn reward_beam(
        &mut self,
        angle: f32,
        damage: i32,
        range: f32,
        width: f32,
        limit: usize,
        slot: usize,
    ) -> Vec<usize> {
        let (mut end, hits) = self.reward_contacts(angle, range, width, limit);
        if hits.len() == limit {
            end = hits.last().unwrap().0;
        }
        let mut contacts = Vec::new();
        for (rank, &(_, i)) in hits.iter().enumerate() {
            self.player_hit(
                i,
                (damage as f32 * 0.8f32.powi(rank as i32)).round() as i32,
                self.px,
                self.py,
                slot,
            );
            contacts.push(i);
        }
        self.weapon_tracer(
            (self.px, self.py),
            (self.px + angle.cos() * end, self.py + angle.sin() * end),
            slot,
            0.12,
        );
        contacts
    }

    /// Enemy-only area damage. Secondary effects never inherit another weapon's
    /// role, hit the shooter, or reach through walls. No recursive proc chains.
    fn boss_area(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        damage: i32,
        slot: usize,
        exclude: Option<usize>,
    ) {
        let targets: Vec<_> = self
            .ents
            .iter()
            .enumerate()
            .filter(|(i, e)| {
                Some(*i) != exclude
                    && e.hp > 0
                    && is_hostile_kind(e.kind)
                    && (e.x - x).hypot(e.y - y) < radius
                    && self.los(x, y, e.x, e.y)
            })
            .map(|(i, _)| i)
            .collect();
        for i in targets {
            let falloff = (1.0 - (self.ents[i].x - x).hypot(self.ents[i].y - y) / radius).max(0.2);
            self.hurt_ent_role(
                i,
                (damage as f32 * falloff).round().max(1.0) as i32,
                x,
                y,
                Some(tactical::role(slot)),
            );
        }
    }

    fn pull_enemy(&mut self, i: usize, x: f32, y: f32, strength: f32) {
        if !is_hostile_kind(self.ents[i].kind) || self.ents[i].hp <= 0 {
            return;
        }
        let e = &mut self.ents[i];
        let (dx, dy) = (x - e.x, y - e.y);
        let distance = dx.hypot(dy).max(0.1);
        let strength = strength * if e.kind == EK_BOSS { 0.22 } else { 1.0 };
        // Replace pellet knockback rather than summing against nine impulses;
        // otherwise the "gravity pull" actually pushes its victim away.
        e.vx = (dx / distance * strength).clamp(-6.0, 6.0);
        e.vy = (dy / distance * strength).clamp(-6.0, 6.0);
    }

    pub(crate) fn boss_hit_modifier(&mut self, i: usize, mut damage: i32, slot: usize) -> i32 {
        if !is_hostile_kind(self.ents[i].kind) || self.ents[i].hp <= 0 {
            return damage;
        }
        let (x, y) = (self.ents[i].x, self.ents[i].y);
        if slot == ARCHIVE {
            let mark = self.mechanics.archive[i];
            if mark.seconds > 0.0 && mark.shot != self.current_shot {
                damage += 54;
                self.mechanics.archive[i] = Mark::default();
                self.boss_area(x, y, 1.7, 30, slot, Some(i));
                self.weapon_ring(x, y, 0.6, slot, 0.25);
            } else {
                self.mechanics.archive[i] = Mark {
                    seconds: 2.5,
                    shot: self.current_shot,
                };
                self.weapon_ring(x, y, 0.35, slot, 0.35);
            }
        } else if slot == RIME {
            let mark = self.mechanics.frost[i];
            // A slower slug from an older volley cannot rearm frost after a
            // newer volley already shattered it, then proc that volley twice.
            if self.current_shot <= mark.shot {
                return damage;
            }
            if mark.seconds > 0.0 && mark.shot != self.current_shot {
                damage += 32;
                // Remember the triggering volley without rearming its other slugs.
                self.mechanics.frost[i] = Mark {
                    seconds: 0.0,
                    shot: self.current_shot,
                };
                self.boss_area(x, y, 2.0, 42, slot, Some(i));
                self.weapon_ring(x, y, 0.9, slot, 0.25);
                self.sound(5, 0, x, y);
            } else if mark.shot != self.current_shot {
                self.mechanics.frost[i] = Mark {
                    seconds: 3.0,
                    shot: self.current_shot,
                };
            }
        }
        damage
    }

    pub(crate) fn fire_reworked_boss(&mut self, slot: usize) -> bool {
        let cadence = match slot {
            ARCHIVE => 0.38,
            PREDICTOR => 0.68,
            SINK => 0.8,
            RIME => 0.75,
            RELAY => 0.18,
            KEST => 0.32,
            UNDERTOW => 0.8,
            POLARITY => 0.55,
            HELIOS => 0.85,
            EPOCH => 0.65,
            _ => return false,
        };
        self.cooldown = cadence;
        self.muzzle = 1.0;
        self.kick = if pressure_gain(slot) > 0.0 { 1.3 } else { 0.8 };
        match slot {
            ARCHIVE => {
                self.reward_beam(self.pa, 42, 24.0, 0.025, 1, slot);
            }
            PREDICTOR => {
                let (_, hits) = self.reward_contacts(self.pa, 28.0, 0.02, 1);
                let end = self.wall_distance(self.px, self.py, self.pa.cos(), self.pa.sin(), 20.0);
                let (x, y) = hits
                    .first()
                    .map(|&(_, i)| {
                        let e = self.ents[i];
                        let (x, y) = (e.x + e.vx * 0.18, e.y + e.vy * 0.18);
                        if self.los(e.x, e.y, x, y) {
                            (x, y)
                        } else {
                            (e.x, e.y)
                        }
                    })
                    .unwrap_or((
                        self.px + self.pa.cos() * (end - 0.1).max(0.0),
                        self.py + self.pa.sin() * (end - 0.1).max(0.0),
                    ));
                self.reward_beam(self.pa, 52, 28.0, 0.02, 1, slot);
                self.mechanics.schedule(
                    0.18,
                    slot,
                    self.current_shot,
                    Attack::Pulse {
                        x,
                        y,
                        radius: 0.95,
                        damage: 46,
                    },
                );
                self.weapon_ring(x, y, 0.3, slot, 0.18);
            }
            SINK => {
                for n in -4..=4 {
                    self.reward_beam(self.pa + n as f32 * 0.023, 13, 10.0, 0.015, 1, slot);
                }
                let (_, targets) = self.reward_contacts(self.pa, 8.0, 1.8, 5);
                for (_, i) in targets {
                    self.pull_enemy(i, self.px, self.py, 3.8);
                }
                self.eject_casing();
            }
            RIME => {
                for n in -1..=1 {
                    self.reward_projectile(slot, self.pa + n as f32 * 0.05, 26, 9.0, 0.0);
                }
            }
            RELAY => {
                let hits = self.reward_beam(self.pa, 34, 23.0, 0.03, 1, slot);
                if let Some(&first) = hits.first() {
                    let mut visited = vec![first];
                    let mut origin = (self.ents[first].x, self.ents[first].y);
                    for damage in [26, 20, 16] {
                        let next = self
                            .ents
                            .iter()
                            .enumerate()
                            .filter(|(i, e)| {
                                !visited.contains(i)
                                    && e.hp > 0
                                    && is_hostile_kind(e.kind)
                                    && (e.x - origin.0).hypot(e.y - origin.1) < 4.0
                                    && self.los(origin.0, origin.1, e.x, e.y)
                            })
                            .min_by(|(ia, a), (ib, b)| {
                                (a.x - origin.0)
                                    .hypot(a.y - origin.1)
                                    .total_cmp(&(b.x - origin.0).hypot(b.y - origin.1))
                                    .then(ia.cmp(ib))
                            })
                            .map(|(i, _)| i);
                        let Some(i) = next else {
                            break;
                        };
                        let point = (self.ents[i].x, self.ents[i].y);
                        self.weapon_tracer(origin, point, slot, 0.16);
                        self.player_hit(i, damage, origin.0, origin.1, slot);
                        visited.push(i);
                        origin = point;
                    }
                }
            }
            KEST => {
                self.reward_beam(self.pa, 24, 24.0, 0.02, 1, slot);
                self.eject_casing();
                for seconds in [0.085, 0.17] {
                    self.mechanics.schedule(seconds, slot, 0, Attack::Burst);
                }
            }
            UNDERTOW => {
                self.reward_projectile(slot, self.pa, 94, 13.0, 0.0);
            }
            POLARITY => {
                let hits = self.reward_beam(self.pa, 72, 24.0, 0.08, 3, slot);
                for i in hits {
                    self.ents[i].armor_hp = (self.ents[i].armor_hp - 24).max(0);
                    let along = (self.ents[i].x - self.px) * self.pa.cos()
                        + (self.ents[i].y - self.py) * self.pa.sin();
                    self.pull_enemy(
                        i,
                        self.px + self.pa.cos() * along,
                        self.py + self.pa.sin() * along,
                        4.0,
                    );
                }
            }
            HELIOS => {
                self.reward_projectile(slot, self.pa, 82, 10.0, 0.0);
            }
            EPOCH => {
                let (end, hits) = self.reward_contacts(self.pa, 26.0, 0.025, 1);
                let point = hits
                    .first()
                    .map(|&(_, i)| (self.ents[i].x, self.ents[i].y))
                    .unwrap_or((
                        self.px + self.pa.cos() * (end - 0.1).max(0.0),
                        self.py + self.pa.sin() * (end - 0.1).max(0.0),
                    ));
                self.reward_beam(self.pa, 62, 26.0, 0.025, 1, slot);
                for seconds in [0.35, 0.7] {
                    self.mechanics.schedule(
                        seconds,
                        slot,
                        self.current_shot,
                        Attack::Pulse {
                            x: point.0,
                            y: point.1,
                            radius: 1.5,
                            damage: 38,
                        },
                    );
                }
                self.weapon_ring(point.0, point.1, 0.75, slot, 0.7);
            }
            _ => unreachable!(),
        }
        self.mechanics.pressure[slot] =
            (self.mechanics.pressure[slot] + pressure_gain(slot)).min(1.0);
        true
    }

    fn reward_projectile(&mut self, slot: usize, angle: f32, damage: i32, speed: f32, charge: f32) {
        // Start at the player rather than teleporting through nearby cover.
        if let Some(i) = self.spawn(EK_PROJ, self.px, self.py) {
            let e = &mut self.ents[i];
            e.vx = angle.cos() * speed;
            e.vy = angle.sin() * speed;
            e.hp = damage;
            e.timer = 3.0;
            e.zoff = 10.0;
            e.effect_tick = 6.0;
            e.skin = if slot == RIME { 232 } else { 231 };
            e.projectile_visual = (slot - 7) as u8;
            e.projectile_charge = charge;
        }
    }

    pub(crate) fn resolve_reward_projectile(
        &mut self,
        projectile: Ent,
        target: Option<usize>,
        x: f32,
        y: f32,
    ) -> bool {
        let slot = projectile.projectile_visual as usize + 7;
        match slot {
            UNDERTOW => {
                if let Some(i) = target {
                    self.player_hit(i, projectile.hp, x, y, slot);
                    self.pull_enemy(
                        i,
                        self.px,
                        self.py,
                        5.5 + projectile.projectile_charge * 2.0,
                    );
                    self.boss_area(
                        x,
                        y,
                        1.8 + projectile.projectile_charge,
                        26 + (projectile.projectile_charge * 45.0) as i32,
                        slot,
                        Some(i),
                    );
                }
                self.weapon_ring(x, y, 0.7, slot, 0.25);
                true
            }
            HELIOS => {
                if let Some(i) = target {
                    self.player_hit(i, projectile.hp / 2, x, y, slot);
                }
                let radius = 2.4 + projectile.projectile_charge;
                self.boss_area(x, y, radius, projectile.hp, slot, None);
                self.weapon_ring(x, y, radius * 0.7, slot, 0.3);
                for seconds in [0.25, 0.5, 0.75] {
                    self.mechanics.schedule(
                        seconds,
                        slot,
                        projectile.shot_id,
                        Attack::Pulse {
                            x,
                            y,
                            radius,
                            damage: 16 + (projectile.projectile_charge * 12.0) as i32,
                        },
                    );
                }
                self.sound(2, 0, x, y);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn release_pressure(&mut self) -> bool {
        let Some(vent) = self.mechanics.vent else {
            return false;
        };
        let bank = self.mechanics.pressure[vent.slot];
        if self.mechanics.fire_held || bank < RELEASE_MIN || self.mag[vent.slot] <= 0 {
            return false;
        }
        self.mechanics.vent = None;
        self.mechanics.pressure[vent.slot] = 0.0;
        self.mechanics.release_flash = 0.32;
        self.reload_t = 0.0;
        self.commit_round(vent.slot);
        self.muzzle = 1.0;
        self.kick = 1.9;
        self.shake = (self.shake + 0.45).min(1.0);
        self.cooldown = 0.95;
        match vent.slot {
            SINK => {
                for _ in 0..9 {
                    let angle = self.pa + (self.rnd() - 0.5) * 0.48;
                    self.reward_beam(angle, 14 + (bank * 22.0) as i32, 12.0, 0.035, 2, vent.slot);
                }
                self.eject_casing();
            }
            UNDERTOW => {
                for n in -1..=1 {
                    let angle = self.pa + n as f32 * 0.12 + (self.rnd() - 0.5) * 0.08;
                    self.reward_projectile(vent.slot, angle, 55 + (bank * 70.0) as i32, 17.0, bank);
                }
            }
            HELIOS => {
                let angle = self.pa + (self.rnd() - 0.5) * 0.34;
                self.reward_projectile(vent.slot, angle, 100 + (bank * 140.0) as i32, 8.0, bank);
            }
            _ => unreachable!(),
        }
        self.sound(26, vent.slot as u8, self.px, self.py);
        self.tactical.cooldowns[vent.slot] = self.cooldown;
        self.current_shot = 0;
        true
    }
}
