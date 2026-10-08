//! Damage resolution and weapon fire. All the maths that decides who is
//! hurt and by how much lives here, so balance changes stay in one place.

// `crate::*` rather than `super::*`: these modules need the crate root
// imports (`consts::*`, `mod tactical`, the `Engine` type) as well as `sim`.
use crate::*;

impl Engine {
    pub(crate) fn campaign_impact(&mut self, x: f32, y: f32, variant: usize) {
        self.sound(13, (variant % 4) as u8, x, y);
        if let Some(i) = self.spawn(EK_IMPACT, x, y) {
            self.ents[i].skin = 200 + (variant % 4) as u8;
            self.ents[i].timer = 0.34;
            self.ents[i].zoff = -8.0;
        }
    }

    pub(crate) fn projectile_impact(&mut self, x: f32, y: f32, visual: u8, palette: i32) {
        if visual == 110 || visual == 114 || visual == 128 || visual == 233 { self.effect(EK_IMPACT, 3, x, y, 0.28, -8.0); return; }
        let effect = if palette == 10 || palette == 12 { 3 }
            else if matches!(palette, 4 | 9 | 14) { 2 }
            else if visual == 230 { 0 }
            else if visual == 234 { 2 } else { 1 };
        self.campaign_impact(x, y, effect);
    }

    pub(crate) fn campaign_projectile(&mut self, x: f32, y: f32, a: f32, weapon: campaign::Weapon, friendly: bool, visual: usize) {
        if let Some(i) = self.spawn(EK_PROJ, x, y) {
            let e = &mut self.ents[i];
            e.vx = a.cos() * weapon.speed;
            e.vy = a.sin() * weapon.speed;
            e.hp = weapon.damage;
            e.timer = 3.0;
            e.effect_tick = if friendly { 6.0 } else { 0.0 };
            e.skin = if friendly { 230 + visual.min(4) as u8 } else { 123 + weapon.mode };
            e.face = weapon.mode as f32 + 1.0;
            e.zoff = 10.0;
            e.aim = if friendly && weapon.mode == 4 { 1.0 } else { 0.0 };
            if e.aim == 1.0 { e.timer=4.0; e.radius=0.12; e.zoff=8.0; }
        }
    }

    pub(crate) fn fire_campaign_weapon(&mut self) {
        let slot = self.weapon as usize;
        let spec = campaign::WEAPONS[slot - 19];
        self.cooldown = spec.cadence;
        self.muzzle = 1.0;
        self.kick = if matches!(spec.mode, 6 | 10) { 1.5 } else { 0.85 };
        let visual = match spec.mode { 0 | 6 | 10 => 0, 1 | 12 => 2, 5 => 3, 8 | 13 => 4, _ => 1 };
        let count = match spec.mode { 2 => 3, 4 => 5, 5 => 4, 7 => 5, 12 => 3, _ => 1 };
        if matches!(spec.mode, 3 | 7 | 11) {
            for n in 0..count {
                let a = self.pa + (n as f32 - (count - 1) as f32 * 0.5) * 0.035;
                self.hitscan(a, spec.damage, 25.0);
            }
            let t = self.wall_distance(self.px, self.py, self.pa.cos(), self.pa.sin(), 25.0);
            self.campaign_impact(self.px + self.pa.cos() * t * 0.96, self.py + self.pa.sin() * t * 0.96, if spec.mode == 3 { 2 } else if spec.mode == 11 { 3 } else { 0 });
            for n in 1..=12 {
                let d = t.min(12.0) * n as f32 / 13.0;
                self.spawn_timed(EK_RAY, self.px + self.pa.cos() * d, self.py + self.pa.sin() * d, 0.10, 14.0);
            }
        } else {
            for n in 0..count {
                let a = self.pa + (n as f32 - (count - 1) as f32 * 0.5) * if spec.mode==4 {0.12} else {0.045};
                let mut shot=spec;
                if spec.mode==4 {shot.damage=(spec.damage*3+n)/count;}
                self.campaign_projectile(self.px + a.cos() * 1.15, self.py + a.sin() * 1.15, a, shot, true, visual);
            }
        }
    }

    pub(crate) fn grant_slot(&mut self, slot: usize) {
        match slot {
            1..=18 => self.owned[slot - 1] = true,
            19..=32 => self.extra_weapons |= 1 << (slot - 19),
            _ => return,
        }
        let grant = MAG_SZ[slot] * 2;
        let cap = RESERVE_CAP[slot];
        self.ammo[slot] = (self.ammo[slot] + grant).min(cap);
        if self.mag[slot] <= 0 {
            self.mag[slot] = MAG_SZ[slot];
        }
        self.weapon = slot as i32;
        self.reload_t = 0.0;
        self.events |= EV_PICK_GOLD;
        self.sound(11, 0, self.px, self.py);
    }

    /// Damage with no known source: used by tests and scripted hazards. The
    /// vignette stays centred because there is nothing to point at.
    pub(crate) fn damage_player(&mut self, dmg: i32) {
        self.damage_player_from(dmg, self.px, self.py);
    }

    /// Damage the player and record where it came from, so the hurt vignette
    /// can point at the shooter instead of ringing the whole screen.
    pub(crate) fn damage_player_from(&mut self, dmg: i32, sx: f32, sy: f32) {
        if self.iframes > 0.0 || self.state != 0 {
            return;
        }
        if self.power == field::POWER_AEGIS && self.power_t > 10.0 {
            self.events |= EV_HIT;
            return;
        }
        let mut d = dmg;
        if self.shield_pool > 0 {
            let take = d.min(self.shield_pool);
            self.shield_pool -= take;
            d -= take;
        }
        if d <= 0 {
            self.events |= EV_HIT;
            return;
        }
        if self.armor > 0 {
            let soak = (d * 2) / 3;
            let take = soak.min(self.armor);
            self.armor -= take;
            d -= take;
        }
        // Bearing of the source relative to facing: 0 dead ahead, +pi behind.
        // `pa` follows `atan2(dy, dx)` (see `pa.cos()/pa.sin()`), and Rust's
        // `y.atan2(x)` is the angle of the point `(x, y)`, so the y delta is the
        // receiver. Stored wrapped to [-pi, pi] so the shader's sin/cos can
        // consume it directly.
        let bearing = (sy - self.py).atan2(sx - self.px);
        self.hurt_dir = (bearing - self.pa + core::f32::consts::PI)
            .rem_euclid(core::f32::consts::TAU)
            - core::f32::consts::PI;
        self.hurt = 1.0;
        self.health -= d.max(1);
        self.iframes = 0.35;
        self.shake = (self.shake + 0.55).min(1.0);
        self.events |= EV_HURT;
        if self.health <= 0 {
            self.health = 0;
            self.state = 1;
            self.events |= EV_DIE;
        }
    }

    pub(crate) fn explode(&mut self, x: f32, y: f32, radius: f32, dmg: f32) {
        self.sound(2, 0, x, y);
        self.effect(EK_IMPACT, 2, x, y, 0.38, 10.0);
        // Blast smoke is what the player sees when firing explosive weapons;
        // keep it half the authored arena size so it reads as a puff.
        self.spawn_smoke_cloud_scaled(x, y, crate::lighting::SMOKE_FIRE_SCALE);
        self.shake = (self.shake + 0.8).min(1.0);
        self.events |= EV_EXPLODE;
        let pd = ((self.px - x).powi(2) + (self.py - y).powi(2)).sqrt();
        if pd < radius && self.los(x, y, self.px, self.py) {
            let fall = 1.0 - pd / radius;
            self.damage_player_from((dmg * fall) as i32, x, y);
        }
        let mut hits: Vec<(usize, i32)> = Vec::new();
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !target_kind(e.kind) {
                continue;
            }
            let d = ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt();
            if d < radius && self.los(x, y, e.x, e.y) {
                let fall = 1.0 - d / radius;
                hits.push((i, (dmg * fall) as i32));
            }
        }
        for (i, d) in hits {
            self.hurt_ent(i, d.max(1), x, y);
        }
        for _ in 0..7 {
            let a = self.rnd() * core::f32::consts::TAU;
            let sp = 1.5 + self.rnd() * 2.5;
            let life = 0.4 + self.rnd() * 0.4;
            self.queue_fx(EK_GIB, x, y, a.cos() * sp, a.sin() * sp, life, 0.0);
        }
    }

    pub(crate) fn hurt_ent(&mut self, i: usize, dmg: i32, hx: f32, hy: f32) {
        self.hurt_ent_role(i,dmg,hx,hy,None);
    }

    pub(crate) fn hurt_ent_role(&mut self, i: usize, mut dmg: i32, hx: f32, hy: f32, role: Option<tactical::Role>) {
        if i >= ENT_N || dmg <= 0 {
            return;
        }
        if self.ents[i].kind == 0 || self.ents[i].hp <= 0 {
            return;
        }
        if is_hostile_kind(self.ents[i].kind) {
            use tactical::Role;
            if let Some(r)=role {
                if r==Role::Precision && self.tactical.acid[i]>0.0 {
                    dmg=(dmg as f32*1.25).round() as i32;
                    self.tactical.acid[i]=0.0;
                    self.effect(EK_IMPACT,3,self.ents[i].x,self.ents[i].y,0.25,4.0);
                }
                match r {
                    Role::Acid=>{if self.tactical.acid[i]<=0.0 {self.effect(EK_IMPACT,3,self.ents[i].x,self.ents[i].y,0.22,4.0);}self.tactical.acid[i]=3.0;},
                    Role::Freeze=>{if self.tactical.slow[i]<=0.0 {self.effect(EK_IMPACT,2,self.ents[i].x,self.ents[i].y,0.22,4.0);}self.tactical.slow[i]=3.0;},
                    Role::Shock if self.tactical.shock_lock[i]<=0.0=>{
                        let boss=self.ents[i].kind==EK_BOSS;
                        self.ents[i].stun=self.ents[i].stun.max(if boss {0.18}else{0.65});
                        self.effect(EK_IMPACT,1,self.ents[i].x,self.ents[i].y,0.22,4.0);
                        self.ents[i].shield_hp=(self.ents[i].shield_hp-20).max(0);
                        self.tactical.shock_lock[i]=if boss {3.0}else{1.2};
                    },
                    Role::Stagger=>self.ents[i].stun=self.ents[i].stun.max(if self.ents[i].kind==EK_BOSS {0.10}else{0.34}),
                    Role::Suppress=>self.ents[i].timer=self.ents[i].timer.max(if self.ents[i].kind==EK_BOSS {0.45}else{0.85}),
                    Role::Control=>self.tactical.slow[i]=self.tactical.slow[i].max(0.6),
                    _=>{}
                }
            }
        }
        let target = self.ents[i];
        let metal = target.armor_hp > 0 || target.shield_hp > 0 ||
            !is_hostile_kind(target.kind) || matches!(enemies::combat_skin(target.skin), 5 | 8 | 11 | 13 | 15..=20 | 22..=27 | 29..=36);
        self.sound(if metal {3} else {4}, 0, target.x, target.y);
        // A shot or a USE shuts the lamp. It stays in the world, dark.
        if self.ents[i].kind == EK_LAMP {
            self.ents[i].timer = -1.0;
            self.ents[i].flash = 0.2;
            self.light_dirty = true;
            self.events |= EV_HIT;
            return;
        }
        if self.ents[i].shield_hp > 0
            && field::shield_blocks(self.ents[i].face, self.ents[i].x, self.ents[i].y, hx, hy)
        {
            let (x, y) = (self.ents[i].x, self.ents[i].y);
            let soak = dmg.min(self.ents[i].shield_hp);
            self.ents[i].shield_hp -= soak;
            dmg -= soak;
            self.ents[i].bar_t = 2.0;
            self.ents[i].flash = 0.2;
            self.events |= EV_HIT;
            self.burst_fx(x, y, EK_SPARK, 4, 0.14);
            if dmg <= 0 {
                return;
            }
        }
        if self.ents[i].armor_hp > 0 {
            let eligible = if role==Some(tactical::Role::Precision) {dmg*3/5}else{dmg};
            let soak = eligible.min(self.ents[i].armor_hp);
            self.ents[i].armor_hp -= soak;
            dmg -= soak;
            self.ents[i].bar_t = 2.0;
            self.ents[i].flash = 0.16;
            if dmg <= 0 {
                // Absorbed by armor: a weaker, shorter confirm than a flesh hit.
                self.hitmarker = 0.7;
                self.events |= EV_HIT;
                return;
            }
        }
        if self.ents[i].kind == EK_BOSS && self.boss_vuln > 0.0 {
            dmg = (dmg * 2).max(2);
            if self.boss_attack.stage==2 && role==Some(boss_attacks::PROFILES[map::level_index(self.wave)].weak) {
                dmg = dmg * 5 / 4;
            }
        }
        let kind;
        let skin;
        let x;
        let y;
        {
            let e = &mut self.ents[i];
            e.hp -= dmg;
            e.bar_t = 2.0;
            e.flash = 0.12;
            let dx = e.x - hx;
            let dy = e.y - hy;
            let l = (dx * dx + dy * dy).sqrt().max(0.01);
            e.vx += dx / l * 1.6;
            e.vy += dy / l * 1.6;
            kind = e.kind;
            skin = e.skin;
            x = e.x;
            y = e.y;
            if e.hp > 0 {
                if is_hostile_kind(kind) && kind != EK_BOSS {
                    e.effect_tick = 0.0;
                    e.timer = e.timer.max(0.45);
                }
                if kind!=EK_BOSS || self.boss_attack.stage!=1 {set_anim(e, ANIM_PAIN, 0.24);}
                self.hitmarker = 1.0;
                self.events |= EV_HIT;
                return;
            }
            e.hp = 0;
            e.vx = 0.0;
            e.vy = 0.0;
            set_anim(e, ANIM_DEAD, 0.62);
        }
        self.hitmarker = 1.7;
        self.events |= EV_KILL;
        if solid_kind(kind) && kind != EK_BARREL {
            self.kills += 1;
            for _ in 0..5 {
                let a = self.rnd() * core::f32::consts::TAU;
                let sp = 1.2 + self.rnd() * 2.0;
                let life = 0.45 + self.rnd() * 0.25;
                self.queue_fx(EK_GIB, x, y, a.cos() * sp, a.sin() * sp, life, 0.0);
            }
        }
        if kind == EK_BARREL {
            self.light_dirty = true;
            if skin == 1 {
                // Fuel drum: the blast is the fire it leaves, not a grenade.
                self.ignite(x, y);
                self.ignite(x + 0.45, y - 0.2);
                self.explode(x, y, 1.15, 12.0);
            } else {
                self.explode(x, y, 2.6, 55.0);
            }
        } else if kind == EK_MARTYR {
            self.light_dirty = true;
            self.explode(x, y, 2.6, 55.0);
        }
        if !is_hostile_kind(kind) && kind != EK_BARREL {
            self.sound(if kind == EK_CRATE && !(150..=224).contains(&skin) {7} else if kind == EK_LAMP || kind == EK_PROP_SERVER {5} else {6}, 0, x, y);
        }
        if matches!(kind,EK_PROP_SERVER|EK_PROP_AC|EK_PROP_REACTOR|EK_PROP_VENT) {
            let role=match kind {EK_PROP_SERVER=>2,EK_PROP_AC=>1,EK_PROP_REACTOR=>0,_=>3};
            let sector=map::level_index(self.wave);
            if let Some(index)=tactical_roles::MACHINERY[sector].iter().position(|&r| r==role) {
                self.machinery_destroyed(150+(sector*3+index) as u8,x,y);
            }
        }
        if kind == EK_CRATE {
            if (150..=224).contains(&skin) {
                self.machinery_destroyed(skin,x,y);
                self.campaign_impact(x, y, if (skin - 150) % 3 == 1 { 1 } else { 2 });
                self.burst_fx(x, y, EK_SPARK, 5, 0.22);
                self.light_dirty = true;
            }
            let roll = (self.rnd() * 3.0) as i32;
            let drop = [EK_AMMO, EK_MED, EK_ARMOR][roll.clamp(0, 2) as usize];
            self.ensure_drop(drop, x, y + 0.35);
        }
        if kind == EK_BOSS {
            self.drop_boss_case(x, y);
        }
    }

    pub(crate) fn hitscan(&mut self, ang: f32, mut dmg: i32, maxd: f32) -> bool {
        if self.power == field::POWER_OVERDRIVE {
            dmg = ((dmg as f32) * 1.65).round() as i32;
        }
        let dx = ang.cos();
        let dy = ang.sin();
        let mut best_t = maxd;
        let mut best_e: Option<usize> = None;
        for (i, e) in self.ents.iter().enumerate() {
            if e.kind == 0 || e.hp <= 0 || !target_kind(e.kind) {
                continue;
            }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            if t < 0.12 || t > best_t {
                continue;
            }
            let px = self.px + dx * t;
            let py = self.py + dy * t;
            let rad = e.radius * 1.35;
            if (px - e.x).powi(2) + (py - e.y).powi(2) < rad * rad {
                best_t = t;
                best_e = Some(i);
            }
        }
        // wall
        let mut t = 0.05;
        let mut wall_t = maxd;
        while t < maxd {
            let x = self.px + dx * t;
            let y = self.py + dy * t;
            if self.blocked(x.floor() as i32, y.floor() as i32) {
                wall_t = t;
                break;
            }
            t += 0.08;
        }
        if let Some(i) = best_e {
            if best_t < wall_t {
                let (ex, ey) = (self.ents[i].x, self.ents[i].y);
                let damage = if self.weapon == 1 {
                    let falloff = (1.0 - (best_t - 2.0).max(0.0) * 0.085).clamp(0.25, 1.0);
                    if best_t < 4.0 {
                        self.ents[i].stun = if self.ents[i].kind == EK_BOSS { 0.10 } else { 0.34 };
                    }
                    (dmg as f32 * falloff).round().max(1.0) as i32
                } else { dmg };
                self.player_hit(i, damage, self.px, self.py,self.weapon as usize);
                self.burst_fx(ex, ey, EK_SPARK, 3, 0.28);
                return true;
            }
        }
        if wall_t < maxd {
            let hx = self.px + dx * wall_t * 0.96;
            let hy = self.py + dy * wall_t * 0.96;
            self.stamp_decal(hx.floor() as i32, hy.floor() as i32);
            self.spawn_timed(EK_IMPACT, hx, hy, 0.28, -8.0);
            self.burst_fx(hx, hy, EK_SPARK, 2, 0.18);
        }
        false
    }

    pub(crate) fn wall_distance(&self, x: f32, y: f32, dx: f32, dy: f32, max_distance: f32) -> f32 {
        let mut cx = x.floor() as i32;
        let mut cy = y.floor() as i32;
        if self.blocked(cx, cy) { return 0.0; }
        let sx = if dx < 0.0 { -1 } else { 1 };
        let sy = if dy < 0.0 { -1 } else { 1 };
        let delta_x = if dx.abs() < 1e-6 { f32::INFINITY } else { dx.recip().abs() };
        let delta_y = if dy.abs() < 1e-6 { f32::INFINITY } else { dy.recip().abs() };
        let mut tx = (if dx < 0.0 { x - cx as f32 } else { cx as f32 + 1.0 - x }) * delta_x;
        let mut ty = (if dy < 0.0 { y - cy as f32 } else { cy as f32 + 1.0 - y }) * delta_y;
        for _ in 0..(MAP_W + MAP_H) {
            let distance;
            if tx < ty { distance = tx; tx += delta_x; cx += sx; }
            else { distance = ty; ty += delta_y; cy += sy; }
            if distance >= max_distance { return max_distance; }
            if self.blocked(cx, cy) { return distance; }
        }
        max_distance
    }

    pub(crate) fn fire_lance(&mut self) {
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let mut end = self.wall_distance(self.px, self.py, dx, dy, 28.0);
        let mut hits = [(0.0f32, 0usize); ENT_N];
        let mut count = 0;
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !target_kind(e.kind) { continue; }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            let side = ex * dy - ey * dx;
            let radius = e.radius + 0.08;
            if t <= 0.0 || side.abs() > radius { continue; }
            let entry = (t - (radius * radius - side * side).sqrt()).max(0.0);
            if entry >= end { continue; }
            hits[count] = (entry, i);
            count += 1;
        }
        // Nearest first, with the entity index as a deterministic tiebreak: two
        // entities at an identical f32 depth must always resolve the same way,
        // because the rank below decides who eats the falloff damage.
        hits[..count].sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (n, &(distance, i)) in hits[..count].iter().take(3).enumerate() {
            if self.ents[i].kind == EK_NONE { continue; }
            let barrel = self.ents[i].kind == EK_BARREL;
            self.player_hit(i, [160, 112, 78][n], self.px, self.py,self.weapon as usize);
            if barrel || n == 2 { end = distance; break; }
        }
        // Short-lived tracer sprites have no gameplay collision; the beam resolves once.
        let segments = ((end / 0.5).ceil() as usize).clamp(1, 24);
        for n in 0..segments {
            let t = 0.25 + (end - 0.25).max(0.0) * n as f32 / segments as f32;
            if t >= end { break; }
            self.spawn_timed(EK_RAY, self.px + dx * t, self.py + dy * t, 0.09, 6.0);
        }
        let t = (end - 0.04).max(0.0);
        self.spawn_timed(EK_IMPACT, self.px + dx * t, self.py + dy * t, 0.22, 0.0);
    }

    pub(crate) fn ignite(&mut self, x: f32, y: f32) {
        if self.blocked(x.floor() as i32, y.floor() as i32) { return; }
        let mut count = 0;
        for e in &mut self.ents {
            if e.kind != EK_FIREPATCH { continue; }
            count += 1;
            if (e.x - x).powi(2) + (e.y - y).powi(2) < 0.36 {
                e.timer = 10.0;
                return;
            }
        }
        if count >= 12 { return; }
        if let Some(i) = self.spawn(EK_FIREPATCH, x, y) {
            self.ents[i].timer = 10.0;
            self.ents[i].radius = 0.7;
            self.ents[i].zoff = 64.0;
        }
    }

    pub(crate) fn begin_reload(&mut self) {
        if self.reload_t > 0.0 || self.state != 0 {
            return;
        }
        let w = self.weapon as usize;
        if self.mag[w] >= MAG_SZ[w] || self.ammo[w] <= 0 {
            return;
        }
        // BR-12 feeds its tube one shell at a time. Other weapons swap a magazine.
        self.reload_dur = if w == 1 { 0.42 } else { RELOAD_T[w] };
        self.reload_t = self.reload_dur;
        self.events |= EV_RELOAD;
    }

    pub(crate) fn finish_reload(&mut self) {
        let w = self.weapon as usize;
        let need = MAG_SZ[w] - self.mag[w];
        let take = need.min(self.ammo[w]).max(0).min(if w == 1 { 1 } else { i32::MAX });
        self.mag[w] += take;
        self.ammo[w] -= take;
        self.reload_t = if w == 1 && self.mag[w] < MAG_SZ[w] && self.ammo[w] > 0 {
            self.events |= EV_RELOAD;
            self.reload_dur
        } else {
            0.0
        };
    }

    pub(crate) fn fire(&mut self) {
        if self.state != 0 || self.cooldown > 0.0 || self.tactical.cooldowns[self.weapon as usize]>0.0 {
            return;
        }
        let w = self.weapon as usize;
        // A tube-fed BR-12 can fire the shells already loaded. Pulling the
        // trigger interrupts the remaining shell-feed sequence.
        if self.reload_t > 0.0 {
            if w != 1 || self.mag[w] <= 0 { return; }
            self.reload_t = 0.0;
        }
        if self.mag[w] <= 0 {
            self.sound(21, w as u8, self.px, self.py);
            self.cooldown = 0.22;
            return;
        }
        self.mag[w] -= 1;
        if self.power == field::POWER_FEED {
            self.mag[w] += 1;
        }
        self.events |= EV_FIRE;
        self.ev_weapon = self.weapon;
        let fx_start=self.fx_n;
        let free_slots: [bool; ENT_N] = core::array::from_fn(|i| self.ents[i].kind == EK_NONE);
        match self.weapon {
            0 => {
                self.cooldown = 0.26;
                self.muzzle = 1.0;
                self.kick = 1.0;
                self.shake = (self.shake + 0.12).min(1.0);
                let a = self.pa + (self.rnd() - 0.5) * 0.02;
                self.hitscan(a, 15, 22.0);
                self.eject_casing();
            }
            1 => {
                self.cooldown = 0.56;
                self.muzzle = 1.0;
                self.kick = 1.4;
                self.shake = (self.shake + 0.38).min(1.0);
                for _ in 0..8 {
                    let a = self.pa + (self.rnd() - 0.5) * 0.19;
                    self.hitscan(a, 7, 11.0);
                }
                self.eject_casing();
            }
            2 => {
                self.cooldown = 0.075;
                self.muzzle = 1.0;
                self.kick = 0.7;
                self.shake = (self.shake + 0.08).min(1.0);
                self.spread = (self.spread + 0.020).min(0.18);
                let a = self.pa + (self.rnd() - 0.5) * (0.03 + self.spread);
                self.hitscan(a, 8, 20.0);
                self.eject_casing();
            }
            3 => {
                self.cooldown = 0.78;
                self.muzzle = 1.0;
                self.kick = 1.1;
                self.shake = (self.shake + 0.22).min(1.0);
                self.fire_lance();
            }
            4 => {
                self.cooldown = 0.78;
                self.muzzle = 1.0;
                self.kick = 1.5;
                self.shake = (self.shake + 0.28).min(1.0);
                let a = self.pa;
                if let Some(i) = self.spawn(EK_BOLT, self.px, self.py) {
                    self.ents[i].vx = a.cos() * 11.0;
                    self.ents[i].vy = a.sin() * 11.0;
                    self.ents[i].timer = 1.25;
                    self.ents[i].zoff = 10.0;
                }
            }
            5 => {
                self.cooldown = 0.42;
                self.muzzle = 1.0;
                self.kick = 0.9;
                self.shake = (self.shake + 0.18).min(1.0);
                let a = self.pa + (self.rnd() - 0.5) * 0.018;
                let hit = self.hitscan(a, 34, 18.0);
                let length = if hit { 9 } else { 15 };
                for n in 1..length {
                    let t = n as f32 * 0.55;
                    self.spawn_timed(EK_RAY, self.px + a.cos() * t, self.py + a.sin() * t, 0.1, 4.0);
                }
            }
            6 => {
                self.cooldown = 0.058;
                self.muzzle = 1.0;
                self.kick = 0.52;
                self.shake = (self.shake + 0.055).min(1.0);
                self.spread = (self.spread + 0.014).min(0.17);
                let a = self.pa + (self.rnd() - 0.5) * (0.035 + self.spread);
                self.hitscan(a, 7, 24.0);
                self.eject_casing();
            }
            7 => {
                self.cooldown = 0.62;
                self.muzzle = 1.0;
                self.kick = 1.15;
                self.shake = (self.shake + 0.24).min(1.0);
                let a = self.pa + (self.rnd() - 0.5) * 0.04;
                let _hit = self.hitscan(a, 22, 12.0);
                let dist = self.wall_distance(self.px, self.py, a.cos(), a.sin(), 11.0);
                let lead = 1.7;
                let reach = (dist * 0.92).clamp(lead, (dist - 0.12).max(lead));
                self.ignite(self.px + a.cos() * reach, self.py + a.sin() * reach);
                for n in 0..6 {
                    let t = lead + n as f32 * 0.65;
                    if t >= dist - 0.05 { break; }
                    self.spawn_timed(EK_FLAME, self.px + a.cos() * t, self.py + a.sin() * t, 2.5, 8.0);
                }
            }
            19..=32 => self.fire_campaign_weapon(),
            8 => self.fire_override(),
            9 => self.fire_forge(),
            10 => self.fire_chimera(),
            11 => {
                // Archivist's carbine is a precise amber pulse: a single
                // instant hit with a brief, narrow afterimage along its path.
                self.cooldown = 0.38;
                self.muzzle = 1.0;
                self.kick = 0.8;
                self.shake = (self.shake + 0.16).min(1.0);
                let a = self.pa;
                self.hitscan(a, 42, 22.0);
                let reach = self.wall_distance(self.px, self.py, a.cos(), a.sin(), 22.0).min(15.0);
                for n in 1..=20 {
                    let t = reach * n as f32 / 21.0;
                    self.spawn_timed(EK_RAY, self.px + a.cos() * t, self.py + a.sin() * t, 0.09, 12.0);
                }
            }
            12 => {
                // Oracle's predictor lands a delayed-looking, accurate double pulse.
                self.cooldown = 0.68; self.muzzle = 1.0; self.kick = 1.1;
                self.hitscan(self.pa, 58, 28.0);
                self.hitscan(self.pa + 0.012, 24, 28.0);
                for n in 1..=18 {
                    let t = n as f32 * 0.7;
                    self.spawn_timed(EK_RAY, self.px + self.pa.cos()*t, self.py + self.pa.sin()*t, 0.12, 13.0);
                }
            }
            13 => {
                // Reactor sink: short, dense copper scatter with a pressure kick.
                self.cooldown = 0.9; self.muzzle = 1.0; self.kick = 1.7;
                self.shake = (self.shake + 0.38).min(1.0);
                for _ in 0..11 {
                    let a = self.pa + (self.rnd()-0.5)*0.22;
                    self.hitscan(a, 11, 10.0);
                }
                self.eject_casing();
            }
            14 => {
                // Rime's three slow coolant slugs force movement through lanes.
                self.cooldown = 0.75; self.muzzle = 1.0; self.kick = 1.2;
                for n in -1..=1 {
                    let a = self.pa + n as f32 * 0.055;
                    if let Some(i) = self.spawn(EK_PROJ, self.px, self.py) {
                        self.ents[i].vx = a.cos()*6.1; self.ents[i].vy = a.sin()*6.1;
                        self.ents[i].timer = 2.4; self.ents[i].hp = 30; self.ents[i].zoff = 12.0;
                        self.ents[i].effect_tick = 5.0;
                        self.ents[i].skin = 232;
                    }
                }
            }
            15 => {
                // Relay discharges a fast, narrow signal burst.
                self.cooldown = 0.11; self.muzzle = 1.0; self.kick = 0.55;
                let a = self.pa + (self.rnd()-0.5)*0.045;
                self.hitscan(a, 14, 23.0);
                for n in 1..=8 { let t = n as f32 * 1.1; self.spawn_timed(EK_RAY, self.px+a.cos()*t, self.py+a.sin()*t, 0.06, 14.0); }
            }
            16 => {
                // Titan's siege shell is a slow, visible heavy explosive.
                self.cooldown = 1.25; self.muzzle = 1.0; self.kick = 1.9;
                self.shake = (self.shake + 0.48).min(1.0);
                let a = self.pa;
                if let Some(i) = self.spawn(EK_BOLT, self.px, self.py) {
                    self.ents[i].vx = a.cos()*8.2; self.ents[i].vy = a.sin()*8.2;
                    self.ents[i].timer = 1.65; self.ents[i].zoff = 10.0; self.ents[i].hp = 85;
                }
            }
            17 => {
                // Kest's assault rifle fires an accurate three-shot burst.
                self.cooldown = 0.32; self.muzzle = 1.0; self.kick = 0.85;
                for n in -1..=1 { self.hitscan(self.pa + n as f32 * 0.025, 14, 22.0); }
                self.eject_casing();
            }
            18 => {
                self.muzzle = 1.0;
                self.fire_lance();
                self.cooldown = 0.52;
                self.kick = 0.95;
                for n in 1..=12 {
                    let t = n as f32 * 0.9;
                    self.spawn_timed(EK_RAY, self.px+self.pa.cos()*t, self.py+self.pa.sin()*t, 0.16, 15.0);
                }
            }
            _ => {}
        }
        if self.weapon >= 8 {
            if !self.fx_q[fx_start..self.fx_n].iter().any(|fx|matches!(fx.kind,EK_RAY|EK_FLAME)) && !self.ents.iter().enumerate().any(|(i,en)| free_slots[i] && matches!(en.kind,EK_PROJ|EK_BOLT|EK_RAY|EK_FLAME)) {
                let reach=self.wall_distance(self.px,self.py,self.pa.cos(),self.pa.sin(),20.0).min(12.0);
                for n in 1..=8 { let d=reach*n as f32/9.0; self.spawn_timed(EK_RAY,self.px+self.pa.cos()*d,self.py+self.pa.sin()*d,0.10,12.0); }
            }
            for fx in &mut self.fx_q[fx_start..self.fx_n] {if matches!(fx.kind,EK_RAY|EK_FLAME) {fx.projectile_visual=1+(self.weapon-8) as u8;}}
            for (i, ent) in self.ents.iter_mut().enumerate() {
                if free_slots[i] && matches!(ent.kind,EK_PROJ|EK_BOLT|EK_RAY|EK_FLAME) { ent.projectile_visual=1+(self.weapon-8) as u8; }
            }
        }
        if self.power == field::POWER_FEED {
            self.cooldown *= 0.45;
        }
        self.tactical.cooldowns[w]=self.cooldown;
    }

    /// Veyran's rail. Pierces every target in the lane, then bursts.
    pub(crate) fn fire_override(&mut self) {
        self.cooldown = 0.8;
        self.muzzle = 1.0;
        self.kick = 1.5;
        self.shake = (self.shake + 0.4).min(1.0);
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let end = self.wall_distance(self.px, self.py, dx, dy, 32.0);
        let mut hits = [(0.0f32, 0usize); ENT_N];
        let mut count = 0;
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !target_kind(e.kind) { continue; }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            let side = ex * dy - ey * dx;
            let radius = e.radius + 0.12;
            if t <= 0.2 || side.abs() > radius || t >= end { continue; }
            hits[count] = (t, i);
            count += 1;
        }
        // Index tiebreak keeps equally-deep targets in a stable damage order.
        hits[..count].sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for &(_, i) in &hits[..count] {
            if self.ents[i].hp <= 0 { continue; }
            self.player_hit(i, 120, self.px, self.py,self.weapon as usize);
        }
        let tip = (end - 0.2).max(0.4);
        let (x, y) = (self.px + dx * tip, self.py + dy * tip);
        self.explode(x, y, 2.8, 90.0);
        for n in 0..12 {
            let t = tip * n as f32 / 12.0;
            self.spawn_timed(EK_RAY, self.px + dx * t, self.py + dy * t, 0.14, 4.0);
        }
    }

    /// HECATE's cutter. A wide beam that splashes at the strike.
    pub(crate) fn fire_forge(&mut self) {
        self.cooldown = 0.48;
        self.muzzle = 1.0;
        self.kick = 0.7;
        self.shake = (self.shake + 0.16).min(1.0);
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let mut strike = self.wall_distance(self.px, self.py, dx, dy, 18.0);
        for (offset, dmg) in [(0.0, 78), (-0.08, 42), (0.08, 42)] {
            let a = self.pa + offset;
            if self.hitscan(a, dmg, 18.0) {
                strike = strike.min(self.wall_distance(self.px, self.py, a.cos(), a.sin(), 18.0));
            }
            for n in 1..8 {
                let t = n as f32 * 0.4;
                if t >= strike { break; }
                self.spawn_timed(EK_RAY, self.px + a.cos() * t, self.py + a.sin() * t, 0.08, -4.0);
            }
        }
        let reach = (strike * 0.92).max(0.6);
        self.explode(self.px + dx * reach, self.py + dy * reach, 1.8, 48.0);
    }

    /// CHIMERA's specimen fan. Acid bolts burst and leave a short pool.
    pub(crate) fn fire_chimera(&mut self) {
        self.cooldown = 0.62;
        self.muzzle = 1.0;
        self.kick = 1.05;
        self.shake = (self.shake + 0.22).min(1.0);
        for n in 0..5 {
            let a = self.pa + (n as f32 - 2.0) * 0.07;
            // Clear of the player. Spawning on the muzzle made the first
            // step count as a self-hit and the burst killed the shooter.
            let x = self.px + a.cos() * 1.15;
            let y = self.py + a.sin() * 1.15;
            if let Some(i) = self.spawn(EK_PROJ, x, y) {
                let e = &mut self.ents[i];
                e.vx = a.cos() * 9.5;
                e.vy = a.sin() * 9.5;
                e.timer = 0.7;
                e.hp = 28; // 42 × 0.67, rounded to integer damage.
                e.effect_tick = 4.0;
                e.skin = 233;
                e.zoff = -6.0;
            }
        }
    }

    pub(crate) fn chimera_burst(&mut self, x: f32, y: f32) {
        self.sound(2, 3, x, y);
        // Allied blast: enemies only. The pool is the same.
        self.effect(EK_IMPACT, 3, x, y, 0.38, 10.0);
        self.spawn_smoke_cloud(x, y);
        self.events |= EV_EXPLODE;
        let mut hits = Vec::new();
        for (i, e) in self.ents.iter().enumerate() {
            if e.hp <= 0 || !solid_kind(e.kind) { continue; }
            let d = ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt();
            if d < 2.6 && self.los(x, y, e.x, e.y) {
                let fall = 1.0 - d / 2.6;
                hits.push((i, (80.0 * 0.67 * fall) as i32));
            }
        }
        for (i, dmg) in hits {
            self.player_hit(i, dmg.max(1), x, y,10);
        }
        self.scorch(x, y, 4.0);
    }

    pub(crate) fn scorch(&mut self, x: f32, y: f32, life: f32) {
        if self.blocked(x.floor() as i32, y.floor() as i32) { return; }
        for e in &mut self.ents {
            if e.kind != EK_FIREPATCH { continue; }
            if (e.x - x).powi(2) + (e.y - y).powi(2) < 0.36 {
                e.timer = e.timer.max(life);
                return;
            }
        }
        if let Some(i) = self.spawn(EK_FIREPATCH, x, y) {
            self.ents[i].timer = life;
            self.ents[i].radius = 0.7;
            self.ents[i].skin = 2;
            self.ents[i].hp = 100;
        }
    }

    /// Damage the player and hostiles standing in a live flame.
    pub(crate) fn burn_at(&mut self, x: f32, y: f32, dmg: i32, hurt_player: bool) {
        if (self.px-x).powi(2) + (self.py-y).powi(2) < 64.0 { self.sound(12, 0, x, y); }
        let pd = (self.px - x).powi(2) + (self.py - y).powi(2);
        if hurt_player && pd < 0.9 * 0.9 && self.los(x, y, self.px, self.py) {
            self.damage_player_from(dmg, x, y);
        }
        let mut hits = Vec::new();
        for (j, target) in self.ents.iter().enumerate() {
            if target.hp <= 0 || !solid_kind(target.kind) { continue; }
            let range = target.radius + 0.7;
            if (target.x - x).powi(2) + (target.y - y).powi(2) < range * range
                && self.los(x, y, target.x, target.y)
            {
                hits.push(j);
            }
        }
        for j in hits {
            if hurt_player {self.hurt_ent(j,dmg,x,y);}else{self.player_hit(j,dmg,x,y,10);}
        }
    }

    pub(crate) fn eject_casing(&mut self) {
        // Cosmetic shells never crowd out hostiles or projectiles.
        let shells: Vec<usize> = self.ents.iter().enumerate()
            .filter(|(_, e)| e.kind == EK_SPARK && e.effect_tick == 5.0).map(|(i, _)| i).collect();
        if shells.len() >= 24 {
            if let Some(i) = shells.into_iter().min_by(|a, b| self.ents[*a].timer.total_cmp(&self.ents[*b].timer)) {
                self.ents[i].kind = EK_NONE;
            }
        }
        let rx = -self.pa.sin();
        let ry = self.pa.cos();
        let (x, y) = (self.px + self.pa.cos() * 0.55 + rx * 0.18,
                      self.py + self.pa.sin() * 0.55 + ry * 0.18);
        let (x, y) = if self.circle_blocked(x, y, 0.06) { (self.px, self.py) } else { (x, y) };
        let jx = 1.2 + self.rnd();
        let jy = 1.2 + self.rnd();
        let pax = self.pa.cos() * 0.2;
        let pay = self.pa.sin() * 0.2;
        let slot = self.fx_n;
        self.queue_fx(EK_SPARK, x, y, rx * jx + pax, ry * jy + pay, 6.0, self.h as f32 * 0.08);
        if self.fx_n > slot { self.fx_q[slot].variant = 5; }
        self.spawn_timed(EK_SMOKE, x, y, 0.22, self.h as f32 * 0.08);
    }

    pub(crate) fn enemy_shoot(&mut self, i: usize) {
        if self.ents[i].kind == EK_BOSS {
            self.resolve_boss_attack(i);
            return;
        }
        let shooter = self.ents[i];
        let (x, y) = (shooter.x, shooter.y);
        let role = combat::profile(shooter.skin, shooter.kind);
        set_anim(&mut self.ents[i], ANIM_FIRE, 0.24);
        let sp = if enemies::combat_skin(shooter.skin) == SKIN_MARKSMAN { 8.0 } else { 5.4 };
        let zoff = if enemies::combat_skin(shooter.skin) == SKIN_HORNET { -35.0 } else { -8.0 };
        for n in 0..role.pellets {
            let a = shooter.aim + (n as f32 - (role.pellets - 1) as f32 * 0.5) * role.spread;
            if let Some(index) = self.spawn(EK_PROJ, x, y) {
                let e = &mut self.ents[index];
                e.vx = a.cos() * sp;
                e.vy = a.sin() * sp;
                e.timer = 2.8;
                e.hp = role.damage;
                e.effect_tick = if enemies::combat_skin(shooter.skin) == SKIN_SPITTER { 3.0 } else { 0.0 };
                e.skin = 100 + enemies::combat_skin(shooter.skin).min(ENEMY_PROJECTILE_COUNT as u8 - 1);
                e.zoff = zoff;
            }
        }
        self.effect(EK_SPARK, if role.pellets > 1 { 1 } else { 0 }, x, y, 0.12, zoff);
        if shooter.kind == EK_BOSS && self.boss_phase >= 2 {
            self.boss_vuln = self.boss_vuln.max(1.15);
        }
    }
}
