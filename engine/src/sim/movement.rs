//! Player collision, sliding against walls and line of sight.

// `crate::*` rather than `super::*`: these modules need the crate root
// imports (`consts::*`, `mod tactical`, the `Engine` type) as well as `sim`.
use crate::*;

impl Engine {
    pub(crate) fn try_move(&mut self, nx: f32, ny: f32) {
        let r = self.pr;
        if !self.circle_blocked(nx, ny, r) {
            self.px = nx;
            self.py = ny;
            return;
        }
        // Resolve both intended components independently so walls preserve tangential motion.
        if !self.circle_blocked(nx, self.py, r) {
            self.px = nx;
        }
        if !self.circle_blocked(self.px, ny, r) {
            self.py = ny;
        }
    }

    pub(crate) fn circle_blocked(&self, x: f32, y: f32, r: f32) -> bool {
        let x0 = (x - r).floor() as i32;
        let y0 = (y - r).floor() as i32;
        let x1 = (x + r).floor() as i32;
        let y1 = (y + r).floor() as i32;
        for j in y0..=y1 {
            for i in x0..=x1 {
                if !self.blocked(i, j) {
                    continue;
                }
                let cx = i as f32 + 0.5;
                let cy = j as f32 + 0.5;
                let dx = (x - cx).abs() - 0.5;
                let dy = (y - cy).abs() - 0.5;
                let ox = dx.max(0.0);
                let oy = dy.max(0.0);
                if ox * ox + oy * oy < r * r {
                    return true;
                }
            }
        }
        false
    }

    /// Nearest point whose radius does not intersect a wall. Spawn and
    /// unstick both use this so a listed coordinate inside geometry cannot
    /// leave a hostile embedded.
    pub(crate) fn nearest_open(&self, x: f32, y: f32, radius: f32) -> (f32, f32) {
        if !self.circle_blocked(x, y, radius) {
            return (x, y);
        }
        for ring in 1..10 {
            let step = ring as f32 * 0.45;
            for n in 0..8 {
                let a = n as f32 * core::f32::consts::TAU / 8.0;
                let nx = x + a.cos() * step;
                let ny = y + a.sin() * step;
                if nx < 1.2 || ny < 1.2 || nx > MAP_W as f32 - 1.2 || ny > MAP_H as f32 - 1.2 {
                    continue;
                }
                if !self.circle_blocked(nx, ny, radius) {
                    return (nx, ny);
                }
            }
        }
        // Rare fallback for a spawn deep inside solid geometry: search the
        // authored layout rather than returning an embedded hostile.
        let mut best = None;
        let mut distance = f32::INFINITY;
        for cy in 1..MAP_H-1 { for cx in 1..MAP_W-1 {
            let (nx, ny) = (cx as f32 + 0.5, cy as f32 + 0.5);
            let d = (nx-x).powi(2) + (ny-y).powi(2);
            if d < distance && !self.circle_blocked(nx, ny, radius) {
                best = Some((nx, ny)); distance = d;
            }
        }}
        best.unwrap_or((x, y))
    }

    pub(crate) fn spawn_clear(&mut self, kind: u8, x: f32, y: f32) -> Option<usize> {
        let radius = enemy_def(kind).map(|d| d.radius).unwrap_or(0.28);
        let (nx, ny) = self.nearest_open(x, y, radius);
        self.spawn(kind, nx, ny)
    }

    pub(crate) fn los(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        let steps = (dist * 8.0) as i32 + 1;
        let sx = dx / steps as f32;
        let sy = dy / steps as f32;
        let mut x = x0;
        let mut y = y0;
        for _ in 0..steps {
            x += sx;
            y += sy;
            if self.blocked(x.floor() as i32, y.floor() as i32) {
                return false;
            }
        }
        true
    }
}
