//! The transient effect queue: sound cues, impact sprites, shell casings
//! and pickups.

// `crate::*` rather than `super::*`: these modules need the crate root
// imports (`consts::*`, `mod tactical`, the `Engine` type) as well as `sim`.
use crate::*;

impl Engine {
    pub(crate) fn spawn_timed(&mut self, kind: u8, x: f32, y: f32, life: f32, zoff: f32) {
        self.queue_fx(kind, x, y, 0.0, 0.0, life, zoff);
    }

    pub(crate) fn burst_fx(&mut self, x: f32, y: f32, kind: u8, n: i32, life: f32) {
        for _ in 0..n {
            let a = self.rnd() * core::f32::consts::TAU;
            let sp = 0.8 + self.rnd() * 2.2;
            let life_j = life * (0.6 + self.rnd() * 0.6);
            let z = -4.0 + self.rnd() * 18.0;
            self.queue_fx(kind, x, y, a.cos() * sp, a.sin() * sp, life_j, z);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn queue_fx(&mut self, kind: u8, x: f32, y: f32, vx: f32, vy: f32, timer: f32, zoff: f32) {
        if self.fx_n >= FX_CAP {
            return;
        }
        self.fx_q[self.fx_n] = FxCmd {
            projectile_visual: 0,
            variant: if kind == EK_SPARK { 4 } else { 0 },
            kind,
            x,
            y,
            vx,
            vy,
            timer,
            zoff,
        };
        self.fx_n += 1;
    }

    pub(crate) fn flush_fx(&mut self) {
        if self.fx_n == 0 {
            return;
        }
        let mut q = 0usize;
        for e in self.ents.iter_mut() {
            if q >= self.fx_n {
                break;
            }
            if e.kind != 0 {
                continue;
            }
            let f = self.fx_q[q];
            q += 1;
            let (hp, radius, zdef) = match f.kind {
                EK_GIB => (1, 0.08, 0.0),
                EK_IMPACT => (1, 0.1, -6.0),
                EK_SPARK => (1, 0.06, 0.0),
                EK_SMOKE => (1, 0.1, -4.0),
                EK_FLAME => (1, 0.14, 10.0),
                _ => (1, 0.1, 0.0),
            };
            *e = Ent {
                aim: if f.kind == EK_SPARK && f.variant == 5 { 1.6 } else { 0.0 },
                kind: f.kind,
                x: f.x,
                y: f.y,
                vx: f.vx,
                vy: f.vy,
                hp,
                timer: f.timer,
                frame: 0.0,
                anim: ANIM_IDLE,
                anim_time: 0.0,
                anim_lock: 0.0,
                skin: if f.kind == EK_SPARK && f.variant == 5 && self.weapon == 1 { 1 } else { SKIN_NONE },
                projectile_visual: f.projectile_visual,
                radius,
                flash: 0.0,
                stun: if f.kind == EK_SPARK && f.variant == 5 { 0.74 } else { 0.0 },
                effect_tick: f.variant as f32,
                shield: 0,
                face: 0.0,
                zoff: if f.zoff != 0.0 { f.zoff } else { zdef },
                bar_t: 0.0,
                armor_hp: 0,
                shield_hp: 0,
            };
        }
        self.fx_n = 0;
    }

    pub(crate) fn sound(&mut self, kind: u8, variant: u8, x: f32, y: f32) {
        if self.sound_cues.len() < sound::CAP {
            self.sound_cues.push(sound::SoundCue {kind: kind as f32, variant: variant as f32, x, y});
        }
    }

    pub(crate) fn effect(&mut self, kind: u8, variant: u8, x: f32, y: f32, life: f32, zoff: f32) {
        if kind == EK_IMPACT && variant != 2 { self.sound(13, variant, x, y); }
        let slot = self.fx_n;
        self.spawn_timed(kind, x, y, life, zoff);
        if self.fx_n > slot { self.fx_q[slot].variant = variant; }
    }

    pub(crate) fn pickup(&mut self, kind: u8) {
        self.sound(match kind { EK_MED => 8, EK_ARMOR => 9, EK_AMMO => 10, _ => 11 }, 0, self.px, self.py);
        if kind == EK_AMMO {
            for slot in 19..WEP_N {
                if self.owns_slot(slot) { self.ammo[slot] = (self.ammo[slot] + MAG_SZ[slot] * 2).min(MAG_SZ[slot] * 6); }
            }
        }
        match kind {
            EK_MED => {
                self.health = (self.health + 35).min(100);
                self.events |= EV_PICK_SILVER;
            }
            EK_AMMO => {
                for slot in 0..WEP_N {
                    if slot != 0 && !self.has_w(slot - 1) { continue; }
                    self.ammo[slot] = (self.ammo[slot] + AMMO_PICKUP[slot]).min(RESERVE_CAP[slot]);
                }
                self.events |= EV_PICK_SILVER;
            }
            EK_ARMOR => {
                self.armor = (self.armor + 50).min(100);
                self.events |= EV_PICK_SILVER;
            }
            // Kinds are not contiguous (EK_OVERRIDE_CONSOLE, EK_NODE and
            // EK_TERMINAL sit between them), so the seven guns are named.
            EK_GUN2 | EK_GUN3 | EK_GUN4 | EK_GUN5 | EK_GUN6 | EK_GUN7 | EK_GUN8 => {
                let slot = match kind {
                    EK_GUN2 => 1,
    EK_GUN3 => 2,
    EK_GUN4 => 3,
    EK_GUN5 => 4,
                    EK_GUN6 => 5,
    EK_GUN7 => 6,
    _ => 7,
                };
                self.set_w(slot - 1);
                self.ammo[slot] = (self.ammo[slot] + GROUND_GUN_PICKUP[slot]).min(RESERVE_CAP[slot]);
                if self.mag[slot] <= 0 { self.mag[slot] = MAG_SZ[slot]; }
                self.weapon = slot as i32;
                self.reload_t = 0.0;
                self.pickup_t = 0.6;self.pickup_dur=0.6;
                self.events |= EV_PICK_GOLD;
            }
            k if field::is_boss_case(k) => {
                self.grant_slot(field::boss_slot(kind));
                self.state = 2;
            }
            EK_POWER => {
                let boost = field::power_kind(self.wave);
                self.power = boost;
                self.power_t = if boost == field::POWER_AEGIS { 14.0 } else if boost == field::POWER_FEED { 10.0 } else { 12.0 };
                self.shield_pool = if boost == field::POWER_AEGIS { 96 } else { 0 };
                self.say(field::RADIO_POWER);
                self.events |= EV_PICK_GOLD;
            }
            _ => {}
        }
    }

    pub(crate) fn needs_pickup(&self, kind: u8) -> bool {
        match kind {
            EK_MED => self.health < 100,
            EK_ARMOR => self.armor < 100,
            EK_AMMO => (0..WEP_N).any(|slot| {
                self.owns_slot(slot) && self.ammo[slot] < RESERVE_CAP[slot]
            }),
            _ => true,
        }
    }
}
