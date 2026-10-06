//! Spawning, wave progression, sector objectives and boss entrances.

// `crate::*` rather than `super::*`: these modules need the crate root
// imports (`consts::*`, `mod tactical`, the `Engine` type) as well as `sim`.
use crate::*;

impl Engine {
    pub(crate) fn boss_intro_duration(&self) -> f32 {
        boss_arena::PROFILES[map::level_index(self.wave)].intro
    }

    pub(crate) fn boss_intro_effect(&mut self, stage: u8) {
        let sector=map::level_index(self.wave);
        let (x,y)=map::boss_spots(self.wave)[0];
        let progress=stage as f32/4.0;
        // Each boss has its own ray count, rotation, radius and particle family.
        // Ray marches stop at solid geometry; entrances never shine through walls.
        let rays=3+sector%6;
        let rotation=sector as f32*0.43+progress*(1.1+sector as f32*0.08);
        let radius=1.0+(1.0-progress)*(2.0+sector as f32*0.07);
        for ray in 0..rays {
            let angle=rotation+ray as f32*core::f32::consts::TAU/rays as f32;
            for step in 1..=6 {
                let r=radius*step as f32/6.0;
                let theta=angle+if sector%5==2 {r*0.75+progress*2.0} else {0.0};
                let (dx,dy)=match sector%5 {
                    1=>((if ray%2==0 {-1.0}else{1.0})*(radius-progress), (step as f32-3.5)*0.65),
                    3=>((ray as f32-(rays-1) as f32*0.5)*0.6,(step as f32-3.5)*radius/3.0),
                    4=>(angle.cos()*radius,angle.sin()*radius),
                    _=>(theta.cos()*r,theta.sin()*r),
                };
                let distance=(dx*dx+dy*dy).sqrt().max(0.01);
                let reach=(self.wall_distance(x,y,dx/distance,dy/distance,distance)-0.16).max(0.0);
                if reach<0.1 {continue;}
                let (sx,sy)=(x+dx*reach/distance,y+dy*reach/distance);
                if self.blocked(sx.floor() as i32,sy.floor() as i32) || !self.los(x,y,sx,sy) {continue;}
                let family=[6,1,2,3,4,5,1,0,6,7,5,6,7,3,4,3,2,0,7,5,4,7,5,1,0][sector];
                self.effect(EK_SPARK,16+family,sx,sy,0.3+progress*0.35,
                    self.h as f32*(-0.28+step as f32*0.07));
            }
        }
        self.shake=(self.shake+0.06+progress*0.16).min(0.65);
        self.sound(24,(sector*5+stage as usize) as u8,x,y);
        if stage==0 {self.events|=EV_BOSS_HUSH;}
        if stage==4 {self.hell=true;self.events|=EV_DOOR;}
    }

    pub(crate) fn spawn(&mut self, kind: u8, x: f32, y: f32) -> Option<usize> {
        self.spawn_with_skin(kind, default_skin(kind), x, y)
    }

    pub(crate) fn spawn_with_skin(&mut self, kind: u8, skin: u8, x: f32, y: f32) -> Option<usize> {
        // Stats come from the roster table so new enemies need no code here.
        // Health rises gradually across the campaign and without a gameplay cap after 25.
        let (mut hp, radius, zoff) = match enemy_def(kind) {
            Some(d) => (d.hp, d.radius, d.zoff),
            None => (1, 0.2, 0.0),
        };
        let (x, y) = if is_hostile_kind(kind) {
            let position = self.nearest_open(x, y, radius);
            if self.circle_blocked(position.0, position.1, radius) { return None; }
            position
        } else { (x, y) };
        if is_hostile_kind(kind) { hp = ((hp as f32) * campaign::health_scale(self.wave)).round() as i32; }
        let zoff = skin_def(skin).map(|d| d.zoff).unwrap_or(zoff);
        for (i, e) in self.ents.iter_mut().enumerate() {
            if e.kind == 0 {
                *e = Ent {
                    aim: 0.0,
                    kind,
                    x,
                    y,
                    vx: 0.0,
                    vy: 0.0,
                    hp,
                    timer: if is_hostile_kind(kind) { 1.0 + (i % 5) as f32 * 0.12 } else { 0.4 },
                    frame: 0.0,
                    anim: ANIM_IDLE,
                    anim_time: 0.0,
                    anim_lock: 0.0,
                    skin,
                    projectile_visual: 0,
                    radius,
                    flash: 0.0,
                    stun: 0.0,
                    effect_tick: 0.0,
                    shield: 0,
                    face: 0.0,
                    zoff,
                    bar_t: 0.0,
                    armor_hp: enemies::armor_cap(kind, skin),
                    shield_hp: 0,
                };
                self.tactical.reset_entity(i);
                return Some(i);
            }
        }
        None
    }

    pub(crate) fn arm_shield(&mut self, i: usize) {
        if self.outage_at(self.ents[i].x,self.ents[i].y,1) {return;}
        let face = (self.py - self.ents[i].y).atan2(self.px - self.ents[i].x);
        let e = &mut self.ents[i];
        e.shield = 1;
        e.hp = ((e.hp as i64 * 14 / 10).max(e.hp as i64 + 8)).min(i32::MAX as i64) as i32;
        e.shield_hp = enemies::SHIELD_CAP;
        e.face = face;
    }

    pub(crate) fn say(&mut self, line: i32) {
        self.radio_seq = self.radio_seq.wrapping_add(1);
        self.radio_line = line;
        self.events |= EV_RADIO;
    }

    pub(crate) fn announce_sector(&mut self) {
        for slot in 19..WEP_N {
            if self.owns_slot(slot) {
                self.mag[slot] = MAG_SZ[slot];
                self.ammo[slot] = RESERVE_CAP[slot];
            }
        }
        self.node_done = false;
        self.lockdown = false;
        self.boss_vuln = 0.0;
        self.say(field::RADIO_HANDLER);
    }

    pub(crate) fn use_field(&mut self) {
        if !self.node_done {
            let (x, y) = field::node_point(self.wave);
            if self.near_point(x, y) {
                self.node_done = true;
                self.say(field::RADIO_NODE);
                self.events |= EV_PICK_GOLD;
                return;
            }
        }
        let sector = map::level_index(self.wave);
        for (index, &(x, y)) in field::terminals(self.wave).iter().enumerate() {
            if !self.near_point(x, y) {
                continue;
            }
            let mut unread = false;
            for e in self.ents.iter_mut() {
                if e.kind == EK_TERMINAL && (e.x - x).abs() < 0.45 && (e.y - y).abs() < 0.45 {
                    if e.timer < 0.0 {
                        return;
                    }
                    e.timer = -1.0;
                    unread = true;
                }
            }
            if unread {
                self.say(field::radio_terminal(sector, index));
            }
            return;
        }
        if let Some(i) = self.facing_prop(EK_LAMP, 2.6) {
            let on = self.ents[i].timer >= 0.0;
            self.ents[i].timer = if on { -1.0 } else { 0.0 };
            self.ents[i].flash = 0.15;
            self.light_dirty = true;
            self.events |= EV_HIT;
            self.sound(3, 0, self.ents[i].x, self.ents[i].y);
            return;
        }
        if let Some(i) = self.facing_prop(EK_CRATE, 1.8) {
            let (x, y) = (self.ents[i].x, self.ents[i].y);
            self.hurt_ent(i, 40, x, y);
        }
    }

    pub(crate) fn ensure_drop(&mut self, kind: u8, x: f32, y: f32) {
        if self.blocked(x.floor() as i32, y.floor() as i32) {
            return;
        }
        let taken = self.ents.iter().any(|e| {
            e.kind != 0 && (e.x - x).abs() < 0.32 && (e.y - y).abs() < 0.32
        });
        if !taken {
            let _ = self.spawn(kind, x, y);
        }
    }

    pub(crate) fn pay_secret(&mut self, cx: i32, cy: i32) {
        let Some((x, y)) = self.secret_room(cx, cy) else { return };
        if self.tactical.secrets_paid.iter().any(|&(a,b)| (x-a).hypot(y-b)<1.5) {return;}
        self.tactical.secrets_paid.push((x,y));
        self.ensure_drop(EK_MED, x, y);
        self.ensure_drop(EK_ARMOR, x + 0.35, y);
        self.ensure_drop(EK_AMMO, x - 0.35, y);
        self.say(210+(map::level_index(self.wave)%3) as i32);
    }

    pub(crate) fn boss_pos(&self) -> Option<(f32, f32)> {
        self.ents.iter().find(|e| e.kind == EK_BOSS && e.hp > 0).map(|e| (e.x, e.y))
    }

    pub(crate) fn seal_lockdown(&mut self) {
        self.lockdown = true;
        // Slam the leaves shut, but leave them as doors. A wall would trap
        // anyone who stepped out before the seal.
        for &(x, y) in field::lockdown_doors(self.wave) {
            if self.cell(x, y) != 8 || x < 0 || y < 0 {
                continue;
            }
            let idx = y as usize * MAP_W + x as usize;
            if idx < self.door.len() {
                if self.door[idx] > 0.05 {self.sound(1, 1, x as f32 + 0.5, y as f32 + 0.5);}
                self.door[idx] = 0.0;
            }
        }
        // Eject anyone whose center is caught inside a slamming leaf: a
        // shut door cell is solid, so that wedges the player with no legal
        // move. Mere overlap from outside still allows backing out.
        for &(x, y) in field::lockdown_doors(self.wave) {
            if self.cell(x, y) != 8 {
                continue;
            }
            if self.px.floor() as i32 == x && self.py.floor() as i32 == y {
                let (nx, ny) = self.nearest_open(self.px, self.py, self.pr);
                self.px = nx;
                self.py = ny;
                break;
            }
        }
        self.light_dirty = true;
        match map::level_index(self.wave) {
            1 => {
                if let Some((bx, by)) = self.boss_pos() {
                    self.ignite(bx + 1.2, by);
                    self.ignite(bx - 1.1, by + 0.8);
                    self.ignite(bx, by - 1.3);
                }
            }
            2 => {
                if let Some((bx, by)) = self.boss_pos() {
                    self.ignite(bx + 1.4, by);
                    self.ignite(bx - 1.2, by + 0.9);
                    self.ignite(bx, by - 1.5);
                }
            }
            _ => {}
        }
        self.say(field::RADIO_LOCKDOWN);
    }

    pub(crate) fn expose_boss(&mut self) {
        self.boss_vuln = 3.4;
        self.say(field::RADIO_EXPOSED);
        for e in self.ents.iter_mut() {
            if e.kind == EK_BOSS && e.hp > 0 {
                e.stun = e.stun.max(0.85);
                e.flash = 0.45;
            }
        }
    }

    pub(crate) fn drop_boss_case(&mut self, x: f32, y: f32) {
        let kind = field::boss_case(self.wave);
        // A one-tick event lets the client play the death voice at the kill,
        // without replaying the persistent radio line on the next sector.
        self.events |= EV_BOSS_VOICE;
        if self.spawn(kind, x, y).is_some() {
            self.say(field::RADIO_BOSS_KILL);
            self.shake = (self.shake + 0.4).min(1.0);
        } else {
            self.grant_slot(field::boss_slot(kind));
            self.state = 2;
        }
    }

    pub(crate) fn place_ents(&mut self) {
        map::place_level(self);
        self.spawn_hostiles(1);
    }

    /// Map guns the player already owns become a supply drop instead.
    pub(crate) fn replace_owned_weapon_drops(&mut self) {
        let owned = self.owned;
        let slot = |kind: u8| match kind {
            EK_GUN2 => Some(0),
            EK_GUN3 => Some(1),
            EK_GUN4 => Some(2),
            EK_GUN5 => Some(3),
            EK_GUN6 => Some(4),
            EK_GUN7 => Some(5),
            EK_GUN8 => Some(6),
            EK_GUN9 => Some(7),
            EK_GUN10 => Some(8),
            EK_GUN11 => Some(9),
            EK_GUN12 => Some(10),
            EK_GUN13 => Some(11),
            EK_GUN14 => Some(12),
            EK_GUN15 => Some(13),
            EK_GUN16 => Some(14),
            EK_GUN17 => Some(15),
            EK_GUN18 => Some(16),
            EK_GUN19 => Some(17),
            _ => None,
        };
        let mut n = 0i32;
        for e in self.ents.iter_mut() {
            let Some(i) = slot(e.kind) else { continue; };
            if !owned[i] { continue; }
            let kind = if n % 2 == 0 { EK_AMMO } else { EK_MED };
            n += 1;
            e.kind = kind;
            e.skin = SKIN_NONE;
            if let Some(d) = enemy_def(kind) {
                e.hp = d.hp;
                e.radius = d.radius;
            }
        }
    }

    /// Missing starting weapons remain recoverable in later sectors.
    pub(crate) fn recover_basic_weapons(&mut self) {
        if self.wave<=1 {return;}
        let kinds=[EK_GUN2,EK_GUN3,EK_GUN4,EK_GUN5,EK_GUN6,EK_GUN7];
        let owned=self.owned;
        for e in &mut self.ents {if kinds.contains(&e.kind) {e.kind=EK_NONE;}}
        let mut cells=Vec::new();
        for y in 1..MAP_H-1 {for x in 1..MAP_W-1 {
            let (x,y)=(x as f32+0.5,y as f32+0.5);
            if !map::in_spawn_room(self.wave,x,y) && !self.circle_blocked(x,y,0.35)
                && self.ents.iter().all(|e| e.kind==EK_NONE || (e.x-x).powi(2)+(e.y-y).powi(2)>2.25) {
                cells.push((x,y));
            }
        }}
        for (i,&kind) in kinds.iter().enumerate() {
            if owned[i] || cells.is_empty() {continue;}
            let choice=(self.rnd()*cells.len() as f32) as usize % cells.len();
            let (x,y)=cells.swap_remove(choice);self.spawn(kind,x,y);
            cells.retain(|&(sx,sy)|(sx-x).powi(2)+(sy-y).powi(2)>16.0);
        }
    }

    pub(crate) fn spawn_hostiles(&mut self, mult: i32) {
        let roster = map::hostiles(self.wave).len();
        self.pending_hostiles = if self.wave > 25 { campaign::hostile_total(self.wave, roster) }
            else { roster * mult.max(1) as usize };
        self.reinforcement_cursor = 0;
        self.reinforcement_t = 1.25;
        while self.pending_hostiles > 0 && map::living_hostiles(self) < campaign::ACTIVE_HOSTILES {
            if !self.spawn_reinforcement(false) { break; }
        }
    }

    pub(crate) fn spawn_reinforcement(&mut self, telegraph: bool) -> bool {
        if self.pending_hostiles == 0 || map::living_hostiles(self) >= campaign::ACTIVE_HOSTILES { return false; }
        let roster = map::hostiles(self.wave);
        let mut point = roster[self.reinforcement_cursor % roster.len()];
        if telegraph {
            let mut safe = None;
            for n in 0..roster.len() {
                let candidate = roster[(self.reinforcement_cursor + n) % roster.len()];
                if (candidate.2 - self.px).powi(2) + (candidate.3 - self.py).powi(2) > 64.0 {
                    safe = Some(candidate); break;
                }
            }
            let Some(candidate) = safe else { return false; };
            point = candidate;
        }
        let (mut kind, mut packed, x, y) = point;
        if telegraph && self.outage_at(x,y,2) {
            self.pending_hostiles-=1;self.reinforcement_cursor=self.reinforcement_cursor.saturating_add(1);return true;
        }
        if (self.reinforcement_cursor % roster.len()).is_multiple_of(2) {
            (kind, packed) = enemies::sector_spawn(self.wave, packed);
        }
        let copy = self.reinforcement_cursor / roster.len();
        let jx = if copy == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
        let jy = if copy == 0 { 0.0 } else { (self.rnd() - 0.5) * 2.2 };
        let radius = enemy_def(kind).map(|d| d.radius).unwrap_or(0.28);
        let (sx, sy) = self.safe_arrival_point(x + jx, y + jy, radius);
        if telegraph && (sx - self.px).powi(2) + (sy - self.py).powi(2) < 49.0 { return false; }
        let Some(i) = self.spawn_with_skin(kind, field::visual_skin(packed), sx, sy) else { return false; };
        if field::is_shielded_spawn(packed) { self.arm_shield(i); }
        if telegraph {
            self.ents[i].stun = 0.9;
            self.ents[i].timer = 1.2;
            self.campaign_impact(sx, sy, 2);
        }
        self.pending_hostiles -= 1;
        self.reinforcement_cursor = self.reinforcement_cursor.saturating_add(1);
        true
    }

    pub(crate) fn next_wave(&mut self) {
        // Twenty-five authored sectors repeat in endless mode. Health and total
        // enemies grow continuously; queued arrivals preserve effect slots.
        self.wave = self.wave.saturating_add(1);
        // Capped at 4x (56 hostiles + ambushes): the uncapped 1<<8 shift
        // filled all 192 entity slots with hostiles and starved FX.
        let shift = (self.wave - 1).clamp(0, 2);
        let mult = 1i32 << shift;
        // Winning keeps every unlocked weapon, restores full health and full
        // ammo, and preserves armor clamped to [0, 100].
        self.health = 100;
        self.armor = self.armor.clamp(0, 100);
        self.iframes = 1.4;
        let start = map::player_start(self.wave);
        self.px = start.0;
        self.py = start.1;
        self.pa = start.2;
        self.pitch = 0.0;
        self.state = 0;
        self.cooldown = 0.0;
        self.reload_t = 0.0;
        self.hurt = 0.0;
        self.muzzle = 0.0;
        self.kick = 0.0;
        self.shake = 0.0;
        // Slot 0 is always carried; owned slots refill to their authored cap.
        for slot in 0..WEP_N {
            if slot != 0 && !self.has_w(slot - 1) { continue; }
            self.mag[slot] = MAG_SZ[slot];
            self.ammo[slot] = RESERVE_CAP[slot];
        }
        self.node_done = false;
        self.lockdown = false;
        self.boss_vuln = 0.0;
        self.apply_theme(self.wave);
        self.clear_boss_arena();
        self.tactical=tactical::Tactical::new();
        self.build_map();
        self.door.fill(0.0);
        self.hell = false;
        self.light_dirty = true;
        self.boss_spawned = false;
        self.boss_intro = 0.0;
        self.boss_phase = 0;
        map::place_level(self);
        self.spawn_hostiles(mult);
    }

    pub(crate) fn maybe_spawn_boss(&mut self) {
        if self.boss_spawned || self.state != 0 {
            return;
        }
        self.boss_spawned = true;
        self.hell = true;
        self.shake = 1.0;
        self.events |= EV_EXPLODE | EV_BOSS_DROP;
        // Endless scaling: 480 x1.5 per wave, capped at 6000 so deep
        // runs stay killable (480 / 720 / 1080 / 1620 / ... / 6000).
        let hp = self.boss_max_health();
        let fx = self.pa.cos();
        let fy = self.pa.sin();
        let boss_spots = map::boss_spots(self.wave);
        let spots = [
            boss_spots[0],
            boss_spots[1],
            (self.px + fx * 4.6, self.py + fy * 4.6),
            (self.px + fx * 3.2 - fy * 2.4, self.py + fy * 3.2 + fx * 2.4),
        ];
        for (x, y) in spots {
            if self.blocked(x.floor() as i32, y.floor() as i32) {
                continue;
            }
            if let Some(i) = self.spawn_with_skin(EK_BOSS, map::boss_skin(self.wave), x, y) {
                self.ents[i].hp = hp;
                self.sound(2, 0, x, y);
                if matches!(map::level_index(self.wave), 5 | 6) {
                    // The Archivist opens behind a breakable directional
                    // memory shield; flanking or sustained fire strips it.
                    self.ents[i].shield = 1;
                    self.ents[i].shield_hp = if map::level_index(self.wave) == 6 { 100 } else { 80 };
                    self.ents[i].face = (self.py - y).atan2(self.px - x);
                }
                break;
            }
        }
        let sector = map::level_index(self.wave);
        let entry = map::boss_spots(self.wave)[0];
        let burst = [14, 22, 10, 18, 20, 24, 18, 16, 28, 12, 24].get(sector).copied().unwrap_or(18);
        if sector == 1 {
            self.spawn_smoke_cloud(entry.0, entry.1);
            self.spawn_smoke_cloud(entry.0 + 1.8, entry.1 - 1.2);
        }
        for _ in 0..burst {
            let a = self.rnd() * core::f32::consts::TAU;
            let r = 0.5 + self.rnd() * 1.6;
            self.spawn_timed(
                EK_SPARK,
                entry.0 + a.cos() * r,
                entry.1 + a.sin() * r,
                0.7,
                if sector == 1 { -32.0 } else { -16.0 },
            );
        }
        let escorts = match map::level_index(self.wave) {
            1 => [(EK_WRAITH, SKIN_HORNET, -2.4, -1.6), (EK_MARTYR, SKIN_MARTYR, 2.4, -1.6), (EK_HUSK, SKIN_GUNNER, -2.8, 1.8), (EK_BRUTE, SKIN_HAZMAT, 2.8, 1.8)],
            2 => [(EK_HUSK, SKIN_HOUND, -2.4, -1.6), (EK_WRAITH, SKIN_SPITTER, 2.4, -1.6), (EK_HUSK, SKIN_SUBJECT, -2.8, 1.8), (EK_BRUTE, SKIN_VATBRUTE, 2.8, 1.8)],
            3 => [(EK_WRAITH, SKIN_MARKSMAN, -2.4, -1.6), (EK_WRAITH, SKIN_HORNET, 2.4, -1.6), (EK_HUSK, SKIN_RIFLEMAN, -2.8, 1.8), (EK_BRUTE, SKIN_GUNNER, 2.8, 1.8)],
            4 => [(EK_BRUTE, SKIN_HAZMAT, -2.4, -1.6), (EK_WRAITH, SKIN_HORNET, 2.4, -1.6), (EK_HUSK, SKIN_GUNNER, -2.8, 1.8), (EK_BRUTE, SKIN_LOADER, 2.8, 1.8)],
            5 => [(EK_WRAITH, SKIN_MARKSMAN, -2.4, -1.6), (EK_HUSK, SKIN_RIFLEMAN, 2.4, -1.6), (EK_WRAITH, SKIN_HORNET, -2.8, 1.8), (EK_BRUTE, SKIN_GUNNER, 2.8, 1.8)],
            _ => [(EK_WRAITH, SKIN_HORNET, -2.4, -1.6), (EK_WRAITH, SKIN_MARKSMAN, 2.4, -1.6), (EK_HUSK, SKIN_RIFLEMAN, -2.8, 1.8), (EK_BRUTE, SKIN_LOADER, 2.8, 1.8)],
        };
        for (n,&(kind, skin, ox, oy)) in escorts[..(2 + self.wave.min(2) as usize)].iter().enumerate() {
            let (kind,skin)=if n%2==0 {enemies::sector_spawn(self.wave,skin)} else {(kind,skin)};
            let skin=field::visual_skin(skin);
            let x = entry.0 + ox;
            let y = entry.1 + oy;
            if !self.blocked(x.floor() as i32, y.floor() as i32) {
                let _ = self.spawn_with_skin(kind, skin, x, y);
            }
        }
    }

    pub(crate) fn boss_max_health(&self) -> i32 {
        campaign::boss_health(self.wave)
    }
}
