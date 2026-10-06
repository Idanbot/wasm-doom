//! Map geometry: cell access, wall/room helpers, flow-field rebuilds
//! and door state. Purely spatial - no game rules live here.

// `crate::*` rather than `super::*`: these modules need the crate root
// imports (`consts::*`, `mod tactical`, the `Engine` type) as well as `sim`.
use crate::*;

impl Engine {
    pub(crate) fn cell(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return 1;
        }
        self.map[y as usize * MAP_W + x as usize]
    }

    pub(crate) fn set_cell(&mut self, x: i32, y: i32, v: u8) {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return;
        }
        self.map[y as usize * MAP_W + x as usize] = v;
    }

    pub(crate) fn blocked(&self, x: i32, y: i32) -> bool {
        let c = self.cell(x, y);
        if c == 0 || c == 10 {
            return false;
        }
        if c == 8 || c == 9 {
            let o = self.door[y as usize * MAP_W + x as usize];
            return o < 0.98;
        }
        true
    }

    pub(crate) fn near_override(&self) -> bool {
        let (x, y) = map::override_point(self.wave);
        let distance = (self.px - x).powi(2) + (self.py - y).powi(2);
        distance < 1.7 * 1.7 && self.los(self.px, self.py, x, y)
    }

    pub(crate) fn room(&mut self, x: i32, y: i32, w: i32, h: i32, wall: u8, fl: u8) {
        for j in y..y + h {
            for i in x..x + w {
                let edge = i == x || j == y || i == x + w - 1 || j == y + h - 1;
                self.set_cell(i, j, if edge { wall } else { 0 });
                if i >= 0 && j >= 0 && (i as usize) < MAP_W && (j as usize) < MAP_H {
                    self.floor[j as usize * MAP_W + i as usize] = fl;
                }
            }
        }
    }

    pub(crate) fn hall_h(&mut self, x0: i32, x1: i32, y: i32, door_x: i32) {
        let (a, b) = if x0 < x1 { (x0, x1) } else { (x1, x0) };
        for x in a..=b {
            self.set_cell(x, y, 0);
            self.set_cell(x, y + 1, 0);
        }
        self.set_cell(door_x, y, 8);
        self.set_cell(door_x, y + 1, 8);
    }

    pub(crate) fn hall_v(&mut self, x: i32, y0: i32, y1: i32, door_y: i32) {
        let (a, b) = if y0 < y1 { (y0, y1) } else { (y1, y0) };
        for y in a..=b {
            self.set_cell(x, y, 0);
            self.set_cell(x + 1, y, 0);
        }
        self.set_cell(x, door_y, 8);
        self.set_cell(x + 1, door_y, 8);
    }

    pub(crate) fn pillar(&mut self, x: i32, y: i32, t: u8) {
        self.set_cell(x, y, t);
        self.set_cell(x + 1, y, t);
        self.set_cell(x, y + 1, t);
        self.set_cell(x + 1, y + 1, t);
    }

    pub(crate) fn mix_edge(&mut self, x: i32, y: i32, w: i32, h: i32, kinds: &[u8]) {
        if kinds.is_empty() {
            return;
        }
        let mut n = 0usize;
        for j in y..y + h {
            for i in x..x + w {
                let edge = i == x || j == y || i == x + w - 1 || j == y + h - 1;
                if !edge {
                    continue;
                }
                let cur = self.cell(i, j);
                if cur == 0 || cur == 8 || cur == 9 || cur == 10 {
                    continue;
                }
                self.set_cell(i, j, kinds[n % kinds.len()]);
                n += 1;
            }
        }
    }

    pub(crate) fn build_map(&mut self) {
        map::build_level(self);
        self.install_secret_cache();
    }

    pub(crate) fn pack(r: u32, g: u32, b: u32, a: u32) -> u32 {
        (r.min(255)) | (g.min(255) << 8) | (b.min(255) << 16) | (a.min(255) << 24)
    }

    pub(crate) fn nbit(n: u32, x: i32, y: i32) -> f32 {
        let mut h = n.wrapping_add((x as u32).wrapping_mul(374761393));
        h = h.wrapping_add((y as u32).wrapping_mul(668265263));
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        (h >> 8) as f32 / 16777216.0
    }

    pub(crate) fn near_point(&self, x: f32, y: f32) -> bool {
        let d = (self.px - x).powi(2) + (self.py - y).powi(2);
        d < 1.7 * 1.7 && self.los(self.px, self.py, x, y)
    }

    /// Nearest prop of `kind` in the facing cone, within `reach`.
    pub(crate) fn facing_prop(&self, kind: u8, reach: f32) -> Option<usize> {
        let dx = self.pa.cos();
        let dy = self.pa.sin();
        let mut best: Option<(usize, f32)> = None;
        for (i, e) in self.ents.iter().enumerate() {
            if e.kind != kind || e.hp <= 0 || !self.los(self.px,self.py,e.x,e.y) {
                continue;
            }
            let ex = e.x - self.px;
            let ey = e.y - self.py;
            let t = ex * dx + ey * dy;
            if t < 0.25 || t > reach {
                continue;
            }
            if (ex * dy - ey * dx).abs() > 0.75 {
                continue;
            }
            if best.is_none_or(|(_, d)| t < d) {
                best = Some((i, t));
            }
        }
        best.map(|(i, _)| i)
    }

    pub(crate) fn walkable(&self, x: i32, y: i32) -> bool {
        !self.blocked(x, y)
    }

    pub(crate) fn rebuild_flow(&mut self) {
        for i in 0..MAP_CELLS {
            self.flow_dir[i] = 0;
            self.flow_dist[i] = 0xFFFF;
        }
        let sx = clamp_i(self.px.floor() as i32, 0, MAP_W as i32 - 1) as usize;
        let sy = clamp_i(self.py.floor() as i32, 0, MAP_H as i32 - 1) as usize;
        let start = sy * MAP_W + sx;
        self.flow_dist[start] = 0;
        self.flow_q[0] = start as u16;
        let mut head = 0usize;
        let mut tail = 1usize;
        let dirs: [(i32, i32, u8); 4] = [(1, 0, 2), (-1, 0, 1), (0, 1, 4), (0, -1, 3)];
        while head < tail {
            let i = self.flow_q[head] as usize;
            head += 1;
            let x = (i % MAP_W) as i32;
            let y = (i / MAP_W) as i32;
            let d0 = self.flow_dist[i];
            for (dx, dy, back) in dirs {
                let nx = x + dx;
                let ny = y + dy;
                if !self.walkable(nx, ny) {
                    continue;
                }
                let ni = ny as usize * MAP_W + nx as usize;
                if self.flow_dist[ni] != 0xFFFF {
                    continue;
                }
                self.flow_dist[ni] = d0 + 1;
                self.flow_dir[ni] = back;
                if tail < MAP_CELLS {
                    self.flow_q[tail] = ni as u16;
                    tail += 1;
                }
            }
        }
    }

    pub(crate) fn stamp_decal(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return;
        }
        let i = y as usize * MAP_W + x as usize;
        self.decal[i] = self.decal[i].saturating_add(1).min(3);
    }

    pub(crate) fn blend(a: u32, b: u32, t: f32) -> u32 {
        let t = t.clamp(0.0, 1.0);
        let ar = (a & 255) as f32;
        let ag = ((a >> 8) & 255) as f32;
        let ab = ((a >> 16) & 255) as f32;
        let br = (b & 255) as f32;
        let bg = ((b >> 8) & 255) as f32;
        let bb = ((b >> 16) & 255) as f32;
        let r = ar * (1.0 - t) + br * t;
        let g = ag * (1.0 - t) + bg * t;
        let bl = ab * (1.0 - t) + bb * t;
        let alpha = (a >> 24) & 255;
        (r as u32) | ((g as u32) << 8) | ((bl as u32) << 16) | (alpha << 24)
    }

    /// Relocate authored threats outside the arrival room and doorway buffer.
    pub(crate) fn safe_arrival_point(&self,x:f32,y:f32,radius:f32)->(f32,f32) {
        let mut position = self.nearest_open(x, y, radius);
        if map::in_spawn_room(self.wave, position.0, position.1) {
            let mut best=None;
            for cy in 1..MAP_H-1 { for cx in 1..MAP_W-1 {
                let (sx,sy)=(cx as f32+0.5,cy as f32+0.5);
                if map::in_spawn_room(self.wave,sx,sy) || self.circle_blocked(sx,sy,radius) {continue;}
                let distance=(sx-x).powi(2)+(sy-y).powi(2);
                if best.is_none_or(|(d,_,_)|distance<d) {best=Some((distance,sx,sy));}
            }}
            if let Some((_,sx,sy))=best {position=(sx,sy);}
            // Fully synthetic test arenas may have no outside floor.
        }
        position
    }

    pub(crate) fn wall_tex(&self, c: u8, _x: i32, _y: i32) -> usize {
        let base = T_SECTOR_SURFACE + map::level_index(self.wave) * 5;
        if c==9 {return base+1;}
        if c==8 {return base+2;}
        if self.hell { return T_BOSS_ARENA + map::level_index(self.wave); }
        base + if matches!(c, 4 | 6) { 1 } else { 0 }
    }

    /// True when this leaf is a sealed boss door and the player is on the
    /// arena side. The outside face still opens so they can walk back in.
    pub(crate) fn lockdown_refuses(&self, cx: i32, cy: i32) -> bool {
        if !self.lockdown || !field::lockdown_doors(self.wave).contains(&(cx, cy)) {
            return false;
        }
        !self.outside_lock(cx, cy)
    }

    pub(crate) fn outside_lock(&self, dx: i32, dy: i32) -> bool {
        let (bx, by) = self.boss_pos().unwrap_or_else(|| {
            let spots = map::boss_spots(self.wave);
            spots[0]
        });
        let ix = bx - (dx as f32 + 0.5);
        let iy = by - (dy as f32 + 0.5);
        let px = self.px - (dx as f32 + 0.5);
        let py = self.py - (dy as f32 + 0.5);
        // Opposite side of the boss, with slack so the threshold still opens.
        px * ix + py * iy < 0.4
    }

    pub(crate) fn open_door_at(&mut self, cx: i32, cy: i32, force: bool) -> bool {
        if self.lockdown_refuses(cx, cy) {
            return false;
        }
        let c = self.cell(cx, cy);
        if c != 8 && !(c == 9 && force) {
            return false;
        }
        if cx < 0 || cy < 0 {
            return false;
        }
        let idx = cy as usize * MAP_W + cx as usize;
        if idx >= self.door.len() || self.door[idx] >= 0.05 {
            return false;
        }
        self.door[idx] = 0.06;
        self.sound(1, 0, cx as f32 + 0.5, cy as f32 + 0.5);
        self.events |= EV_DOOR;
        if c == 9 {
            self.secrets += 1;
            self.pay_secret(cx, cy);
        }
        true
    }

    pub(crate) fn open_door_pair(&mut self, cx: i32, cy: i32, force: bool) {
        if !self.open_door_at(cx, cy, force) {
            return;
        }
        const N: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in N {
            self.open_door_at(cx + dx, cy + dy, force);
        }
    }

    pub(crate) fn open_nearby_doors(&mut self, force: bool) {
        let fx = self.px + self.pa.cos() * 0.9;
        let fy = self.py + self.pa.sin() * 0.9;
        let cells = [
            (self.px.floor() as i32, self.py.floor() as i32),
            (fx.floor() as i32, fy.floor() as i32),
            (fx.floor() as i32 + 1, fy.floor() as i32),
            (fx.floor() as i32, fy.floor() as i32 + 1),
        ];
        for (cx, cy) in cells {
            self.open_door_pair(cx, cy, force);
        }
        let x0 = (self.px - 1.3).floor() as i32;
        let y0 = (self.py - 1.3).floor() as i32;
        let x1 = (self.px + 1.3).floor() as i32;
        let y1 = (self.py + 1.3).floor() as i32;
        for j in y0..=y1 {
            for i in x0..=x1 {
                if self.cell(i, j) == 8 || (force && self.cell(i, j) == 9) {
                    self.open_door_pair(i, j, force);
                }
            }
        }
    }
}
