//! Light propagation and volumetric smoke.
//!
//! The light grid is a coarse 48x32 colour buffer rebuilt once per frame: static
//! geometry plus a budgeted set of dynamic emitters, then two diffusion passes so
//! light wraps corners. Smoke rides the same grid in its alpha channel.
//!
//! Every cloud carries a `scale`: authored arena smoke is full size, while
//! weapon fire and blasts use [`SMOKE_FIRE_SCALE`] (half) so a single shot does
//! not fog the room.
//!
//! Dynamic emitters are ranked by distance to the player before the budget is
//! applied; walking the entity array in index order made which projectiles lit
//! the room an allocation accident.

use super::*;

/// Radius multiplier for smoke produced by weapon fire and blasts. Authored
/// arena smoke uses the full size; a single shot should not fog the room.
pub(crate) const SMOKE_FIRE_SCALE: f32 = 0.5;

/// Number of slow signature lamps added to each sector's static light.
const SECTOR_SIGNATURE_LIGHTS: usize = 5;
/// Peak brightness of a signature lamp.
const SECTOR_SIGNATURE_STRENGTH: f32 = 0.30;
/// Reach of a signature lamp, in cells.
const SECTOR_SIGNATURE_RADIUS: f32 = 5.2;

impl Engine {
    pub(crate) fn spawn_smoke_cloud(&mut self, x: f32, y: f32) {
        self.spawn_smoke_cloud_scaled(x, y, 1.0);
    }

    /// Smoke with a caller-chosen radius. Weapon fire and blast puffs use
    /// [`SMOKE_FIRE_SCALE`] so a single shot does not fog the room; authored
    /// arena smoke keeps its full size.
    pub(crate) fn spawn_smoke_cloud_scaled(&mut self, x: f32, y: f32, scale: f32) {
        let mut slot = 0usize;
        let mut oldest = -1.0f32;
        for (i, s) in self.smokes.iter().enumerate() {
            if s.age < 0.0 {
                slot = i;
                break;
            }
            if s.age > oldest {
                oldest = s.age;
                slot = i;
            }
        }
        // Grid-only smoke: the volumetric alpha effect in the smoke grid is
        // the whole effect now. No EK_SMOKE sprite companions are spawned.
        let drift = self.rnd() * core::f32::consts::TAU;
        self.smokes[slot] = Smoke {
            x,
            y,
            age: 0.0,
            scale,
            vx: drift.cos() * 0.22,
            vy: drift.sin() * 0.22,
        };
    }

    pub(crate) fn age_smoke(&mut self, dt: f32) {
        for i in 0..self.smokes.len() {
            if self.smokes[i].age < 0.0 { continue; }
            self.smokes[i].age += dt;
            if self.smokes[i].age >= 4.0 {
                self.smokes[i] = Smoke::DEAD;
                continue;
            }
            let s = self.smokes[i];
            let mut vx = s.vx + (s.y * 1.7 + s.age * 3.0).sin() * dt * 1.1;
            let mut vy = s.vy + (s.x * 1.3 - s.age * 2.4).cos() * dt * 1.1;
            let nx = s.x + vx * dt;
            let ny = s.y + vy * dt;
            let mut x = s.x;
            let mut y = s.y;
            if self.blocked(nx.floor() as i32, s.y.floor() as i32) { vx = -vx * 0.45; } else { x = nx; }
            if self.blocked(x.floor() as i32, ny.floor() as i32) { vy = -vy * 0.45; } else { y = ny; }
            self.smokes[i].x = x;
            self.smokes[i].y = y;
            self.smokes[i].vx = vx * 0.985;
            self.smokes[i].vy = vy * 0.985;
        }
    }

    /// Sector palette colour, reused as that level's light signature. These are
    /// the same hues as `SECTOR_EFFECTS` in src/game/sector-effects.ts so the
    /// ambient wash, the fog tint and the drifting motes agree.
    pub(crate) fn sector_tint(wave: i32) -> [f32; 3] {
        const TINTS: [[f32; 3]; 25] = [
            [1.00, 0.42, 0.22], [0.34, 0.78, 0.98], [0.52, 0.82, 0.36], [0.24, 0.72, 0.94],
            [0.40, 0.94, 0.66], [0.62, 0.52, 0.92], [0.76, 0.92, 1.00], [0.86, 0.62, 0.96],
            [0.78, 0.62, 0.44], [0.70, 0.80, 0.68], [0.56, 0.48, 0.84], [0.92, 0.56, 0.40],
            [0.34, 0.80, 0.74], [0.94, 0.76, 0.54], [0.94, 0.58, 0.36], [0.94, 0.76, 0.42],
            [0.52, 0.76, 0.44], [0.56, 0.70, 1.00], [0.74, 0.86, 0.34], [0.62, 0.48, 0.78],
            [1.00, 0.72, 0.44], [0.56, 0.84, 0.92], [0.72, 0.66, 0.94], [0.48, 0.70, 0.96],
            [0.94, 0.72, 0.44],
        ];
        TINTS[crate::map::level_index(wave) % 25]
    }

    pub(crate) fn smoke_radius(age: f32) -> f32 {
        0.85 + (age / 2.0).clamp(0.0, 1.0) * 1.25
    }

    pub(crate) fn smoke_strength(age: f32) -> f32 {
        if age < 0.0 { return 0.0; }
        let bloom = 0.28 + 0.20 * (age / 0.4).clamp(0.0, 1.0);
        let fade = if age > 2.4 { ((4.0 - age) / 1.6).clamp(0.0, 1.0) } else { 1.0 };
        bloom * fade
    }

    pub(crate) fn rebuild_smoke_grid(&mut self) {
        // Scratch buffer: zeroed in place every frame, never reallocated.
        self.smoke_next.fill(0.0);
        let mut next = core::mem::take(&mut self.smoke_next);
        for y in 1..MAP_H - 1 {
            for x in 1..MAP_W - 1 {
                if self.blocked(x as i32, y as i32) { continue; }
                let i = y * MAP_W + x;
                let here = self.smoke_grid[i];
                if here < 0.015 { continue; }
                let gx = self.smoke_grid[i - 1] - self.smoke_grid[i + 1];
                let gy = self.smoke_grid[i - MAP_W] - self.smoke_grid[i + MAP_W];
                let sx = (x as f32 + 0.5 - gx * 0.55).clamp(0.0, (MAP_W - 1) as f32);
                let sy = (y as f32 + 0.5 - gy * 0.55).clamp(0.0, (MAP_H - 1) as f32);
                if self.blocked(sx.floor() as i32, sy.floor() as i32) { continue; }
                let dest = sy as usize * MAP_W + sx as usize;
                next[dest] = (next[dest] + here * 0.80).min(0.55);
            }
        }
        self.smoke_grid.copy_from_slice(&next);
        self.smoke_next = next;
        for s in self.smokes {
            if s.age < 0.0 { continue; }
            let radius = Self::smoke_radius(s.age) * s.scale;
            let strength = Self::smoke_strength(s.age) * s.scale;
            let x0 = (s.x - radius).floor().max(0.0) as usize;
            let y0 = (s.y - radius).floor().max(0.0) as usize;
            let x1 = ((s.x + radius).ceil() as usize).min(MAP_W - 1);
            let y1 = ((s.y + radius).ceil() as usize).min(MAP_H - 1);
            for cy in y0..=y1 {
                for cx in x0..=x1 {
                    if self.blocked(cx as i32, cy as i32) { continue; }
                    let d = ((cx as f32 + 0.5 - s.x).powi(2) + (cy as f32 + 0.5 - s.y).powi(2)).sqrt();
                    if d >= radius { continue; }
                    let f = 1.0 - d / radius;
                    let cover = f * f * (3.0 - 2.0 * f) * strength;
                    let i = cy * MAP_W + cx;
                    self.smoke_grid[i] = (self.smoke_grid[i] + cover).min(0.55);
                }
            }
        }
    }

    pub(crate) fn add_light(&self, grid: &mut [[f32; 3]], x: f32, y: f32, radius: f32, rgb: [f32; 3]) {
        let x0 = (x - radius).floor().max(0.0) as usize;
        let y0 = (y - radius).floor().max(0.0) as usize;
        let x1 = ((x + radius).ceil() as usize).min(MAP_W - 1);
        let y1 = ((y + radius).ceil() as usize).min(MAP_H - 1);
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let wx = cx as f32 + 0.5;
                let wy = cy as f32 + 0.5;
                let d2 = (wx - x).powi(2) + (wy - y).powi(2);
                if d2 >= radius * radius || self.blocked(cx as i32, cy as i32)
                    || !self.los(x, y, wx, wy) { continue; }
                let falloff = (1.0 - d2 / (radius * radius)).powi(2) / (1.0 + d2 * 0.08);
                for channel in 0..3 { grid[cy * MAP_W + cx][channel] += rgb[channel] * falloff; }
            }
        }
    }

    pub(crate) fn update_lighting(&mut self) {
        if self.light_dirty {
            let mut grid = vec![[0.0; 3]; MAP_CELLS];
            for y in 1..MAP_H - 1 {
                for x in 1..MAP_W - 1 {
                    let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter()
                        .filter(|&&(dx, dy)| self.blocked(x as i32 + dx, y as i32 + dy)).count();
                    grid[y * MAP_W + x] = [-0.045 * neighbors as f32; 3];
                }
            }
            for e in &self.ents {
                if e.kind == EK_LAMP && e.timer >= 0.0 {
                    self.add_light(&mut grid, e.x, e.y, 8.2, [0.78, 0.40, 0.14]);
                } else if e.kind == EK_FLAME {
                    self.add_light(&mut grid, e.x, e.y, 5.4, [0.62, 0.18, 0.04]);
                } else if matches!(e.kind, EK_TERMINAL | EK_NODE | EK_OVERRIDE_CONSOLE) {
                    self.add_light(&mut grid, e.x, e.y, 6.4, [0.16, 0.42, 0.62]);
                } else if e.kind == EK_PROP_WLIGHT_C {
                    self.add_light(&mut grid, e.x, e.y, 6.0, [0.20, 0.70, 0.80]);
                } else if e.kind == EK_PROP_WLIGHT_W {
                    self.add_light(&mut grid, e.x, e.y, 6.0, [0.70, 0.70, 0.62]);
                } else if e.kind == EK_PROP_BEACON {
                    self.add_light(&mut grid, e.x, e.y, 5.0, [0.80, 0.36, 0.10]);
                } else if matches!(e.kind, EK_CRATE | EK_BARREL) {
                    let i = (e.y.floor() as usize).min(MAP_H - 1) * MAP_W
                        + (e.x.floor() as usize).min(MAP_W - 1);
                    for c in &mut grid[i] { *c -= 0.10; }
                }
            }
            for y in 1..MAP_H - 1 {
                for x in 1..MAP_W - 1 {
                    if self.cell(x as i32, y as i32) != 6 || (x + y) % 3 != 0 { continue; }
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        if !self.blocked(x as i32 + dx, y as i32 + dy) {
                            self.add_light(&mut grid, x as f32 + 0.5 + dx as f32 * 0.6,
                                y as f32 + 0.5 + dy as f32 * 0.6, 6.4, [0.10, 0.42, 0.62]);
                        }
                    }
                }
            }
            // Sector light signature: every level gets a faint ambient wash in
            // its own palette colour plus a handful of slow-pulsing signature
            // lamps, so a room reads as "Foundry" or "Cryo" before the HUD says
            // so. Static because it only depends on the built map.
            let tint = Self::sector_tint(self.wave);
            for cell in grid.iter_mut() {
                for c in 0..3 { cell[c] += tint[c] * 0.030; }
            }
            let mut placed = 0;
            'outer: for y in 2..MAP_H - 2 {
                for x in 2..MAP_W - 2 {
                    if placed >= SECTOR_SIGNATURE_LIGHTS { break 'outer; }
                    if self.blocked(x as i32, y as i32) || self.cell(x as i32, y as i32) == 6 { continue; }
                    // Spread the lamps out instead of clustering them.
                    if !(x * 7 + y * 13 + crate::map::level_index(self.wave)).is_multiple_of(11) { continue; }
                    if !self.los(self.px, self.py, x as f32 + 0.5, y as f32 + 0.5) { continue; }
                    let pulse = 0.55 + 0.45 * (self.time * 0.6 + (x + y) as f32 * 0.35).sin();
                    let amp = SECTOR_SIGNATURE_STRENGTH * pulse;
                    self.add_light(&mut grid, x as f32 + 0.5, y as f32 + 0.5,
                        SECTOR_SIGNATURE_RADIUS, [tint[0] * amp, tint[1] * amp, tint[2] * amp]);
                    placed += 1;
                }
            }
            self.static_light = grid;
            self.light_dirty = false;
        }
        let mut grid = std::mem::take(&mut self.light_grid);
        grid.copy_from_slice(&self.static_light);
        // Emitters are ranked by distance to the player before the budget is
        // applied. Walking the entity array in index order made which
        // projectiles lit the room an allocation accident.
        let mut emitters: Vec<(f32, usize)> = self.ents.iter().enumerate()
            .filter(|(_, e)| matches!(e.kind, EK_PROJ | EK_RAY | EK_BOLT | EK_FIREPATCH | EK_IMPACT | EK_BOSS))
            .map(|(i, e)| ((e.x - self.px).powi(2) + (e.y - self.py).powi(2), i))
            .filter(|&(d2, _)| d2 <= DYNAMIC_LIGHT_RANGE * DYNAMIC_LIGHT_RANGE)
            .collect();
        emitters.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        emitters.truncate(DYNAMIC_LIGHT_BUDGET);
        let count = emitters.len();
        for &(_, i) in &emitters {
            let e = &self.ents[i];
            let rgb = if e.kind == EK_BOSS && (23..=36).contains(&e.skin) {
                campaign::LIGHTS[(e.skin - 23) as usize]
            } else if e.kind == EK_BOSS {
                match map::level_index(self.wave) {
                    1 => [0.55, 0.22, 0.05],
                    2 => [0.12, 0.48, 0.16],
                    3 => [0.05, 0.44, 0.62],
                    4 => [0.62, 0.28, 0.04],
                    _ => [0.55, 0.28, 0.08],
                }
            } else if e.kind == EK_PROJ && (100..=136).contains(&e.skin) {
                let source = e.skin - 100;
                if source >= 23 { campaign::LIGHTS[(source - 23) as usize] }
                else { match source { 10 | 14 => [0.14,0.62,0.08], 18 => [0.12,0.48,0.72], 22 => [0.40,0.10,0.65], 12 | 17 => [0.72,0.32,0.04], 20 => [0.72,0.22,0.04], _ => [0.08,0.55,0.72] } }
            } else if e.kind == EK_PROJ && (230..=234).contains(&e.skin) {
                [[0.72,0.22,0.04],[0.08,0.55,0.72],[0.12,0.48,0.72],[0.14,0.62,0.08],[0.40,0.10,0.65]][(e.skin - 230) as usize]
            } else if e.kind == EK_PROJ && (e.effect_tick == 3.0 || e.effect_tick == 4.0) {
                [0.12, 0.55, 0.08]
            } else if matches!(e.kind, EK_RAY | EK_PROJ) {
                [0.08, 0.42, 0.72]
            } else {
                [0.42, 0.12, 0.03]
            };
            let radius = if e.kind == EK_BOSS { 9.5 } else { 5.2 };
            self.add_light(&mut grid, e.x, e.y, radius, rgb);
        }
        debug_assert!(count <= DYNAMIC_LIGHT_BUDGET);
        if self.boss_intro>0.0 && !self.boss_spawned {
            let (x,y)=map::boss_spots(self.wave)[0];
            let profile=boss_arena::PROFILES[map::level_index(self.wave)];
            let progress=1.0-self.boss_intro/profile.intro;
            let intensity=0.2+progress*0.9+(progress*40.0).sin().abs()*0.18;
            self.add_light(&mut grid,x,y,7.0+progress*4.0,profile.color.map(|v|v*intensity));
        }
        if self.muzzle > 0.05 {
            let power = self.muzzle * if self.weapon == 0 { 0.28 } else { 0.72 };
            self.add_light(&mut grid, self.px, self.py, 5.2, [power, power * 0.55, power * 0.18]);
        }
        if self.boss_arena.armed && self.boss_arena.stage != 0 {
            let color=boss_arena::PROFILES[map::level_index(self.wave)].color;
            let intensity=if self.boss_arena.stage==4 {0.44}else{0.10};
            for (i,cell) in grid.iter_mut().enumerate() {
                if self.boss_arena.mask[i]!=0 {for channel in 0..3 {cell[channel]+=color[channel]*intensity;}}
            }
        }
        self.light_grid = grid;
        self.bounce_light();
    }

    /// Two passes at decreasing weight. A single pass stopped light dead at
    /// wall corners; the second lets it wrap one cell further, which is what
    /// makes a lit doorway read as connected to the room beyond.
    /// Two diffusion passes at decreasing weight, ping-ponging between the two
    /// scratch buffers so no pass can cascade within itself. One pass left
    /// light dead at wall corners; the second lets it wrap a cell further,
    /// which is what makes a lit doorway read as connected to the room beyond.
    /// Diffuses light from `src` into `dst` by a fraction of each open cell's
    /// mean neighbour brightness. `src` and `dst` must be distinct buffers so a
    /// pass cannot cascade within itself.
    pub(crate) fn bounce_pass(&self, src: &[[f32; 3]], dst: &mut [[f32; 3]], weight: f32) {
        for y in 1..MAP_H - 1 {
            for x in 1..MAP_W - 1 {
                if self.blocked(x as i32, y as i32) { continue; }
                let i = y * MAP_W + x;
                let mut acc = [0.0f32; 3];
                let mut n = 0.0f32;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if self.blocked(nx, ny) { continue; }
                    let j = ny as usize * MAP_W + nx as usize;
                    for c in 0..3 { acc[c] += src[j][c].max(0.0); }
                    n += 1.0;
                }
                if n <= 0.0 { continue; }
                for c in 0..3 { dst[i][c] += acc[c] / n * weight; }
            }
        }
    }

    /// Two passes at decreasing weight. A single pass left light dead at wall
    /// corners; the second lets it wrap a cell further, which is what makes a
    /// lit doorway read as connected to the room beyond.
    pub(crate) fn bounce_light(&mut self) {
        let mut grid = core::mem::take(&mut self.light_grid);
        let mut scratch = core::mem::take(&mut self.light_src);
        scratch.copy_from_slice(&grid);
        self.bounce_pass(&scratch, &mut grid, 0.22);
        self.bounce_pass(&grid, &mut scratch, 0.10);
        for (cell, extra) in grid.iter_mut().zip(scratch.iter()) {
            for c in 0..3 { cell[c] += extra[c]; }
        }
        self.light_grid = grid;
        self.light_src = scratch;
    }

    pub(crate) fn light_at(&self, x: f32, y: f32) -> [f32; 3] {
        let x = (x - 0.5).clamp(0.0, (MAP_W - 2) as f32);
        let y = (y - 0.5).clamp(0.0, (MAP_H - 2) as f32);
        let ix = x as usize;
        let iy = y as usize;
        let fx = x.fract();
        let fy = y.fract();
        let i = iy * MAP_W + ix;
        let mut out = [0.0; 3];
        for c in 0..3 {
            let a = self.light_grid[i][c] * (1.0 - fx) + self.light_grid[i + 1][c] * fx;
            let b = self.light_grid[i + MAP_W][c] * (1.0 - fx) + self.light_grid[i + MAP_W + 1][c] * fx;
            out[c] = a * (1.0 - fy) + b * fy;
        }
        out
    }

    pub(crate) fn shade_rgb(color: u32, ambient: f32, light: [f32; 3]) -> u32 {
        let mut out = color & 0xff000000;
        for channel in 0..3 {
            let value = ((color >> (channel * 8)) & 255) as f32;
            let lit = (value * (ambient + light[channel]).clamp(0.18, 1.8)).min(255.0) as u32;
            out |= lit << (channel * 8);
        }
        out
    }

    /// Four-wide copy of [`shade_rgb`]. Scale is constant across a wall column.
    #[inline(always)]
    pub(crate) fn shade_rgb4(colors: [u32; 4], ambient: f32, light: [f32; 3]) -> [u32; 4] {
        #[cfg(target_feature = "simd128")]
        {
            use core::arch::wasm32::*;
            let scale = [
                (ambient + light[0]).clamp(0.18, 1.8),
                (ambient + light[1]).clamp(0.18, 1.8),
                (ambient + light[2]).clamp(0.18, 1.8),
            ];
            let color = u32x4(colors[0], colors[1], colors[2], colors[3]);
            let mask = u32x4_splat(255);
            let cap = f32x4_splat(255.0);
            let mut out = v128_and(color, u32x4_splat(0xff00_0000));
            for channel in 0..3 {
                let chan = v128_and(u32x4_shr(color, channel * 8), mask);
                let lit = f32x4_min(f32x4_mul(f32x4_convert_i32x4(chan), f32x4_splat(scale[channel as usize])), cap);
                let bits = i32x4_trunc_sat_f32x4(lit);
                out = v128_or(out, u32x4_shl(bits, channel * 8));
            }
            return [
                u32x4_extract_lane::<0>(out),
                u32x4_extract_lane::<1>(out),
                u32x4_extract_lane::<2>(out),
                u32x4_extract_lane::<3>(out),
            ];
        }
        #[cfg(not(target_feature = "simd128"))]
        [
            Self::shade_rgb(colors[0], ambient, light),
            Self::shade_rgb(colors[1], ambient, light),
            Self::shade_rgb(colors[2], ambient, light),
            Self::shade_rgb(colors[3], ambient, light),
        ]
    }

    #[cfg_attr(target_feature = "simd128", allow(dead_code))]
    pub(crate) fn shade_texel(color: u32, gain: [i32; 3], alpha: u32) -> u32 {
        let r = (((color & 255) * gain[0] as u32) >> 16).min(255);
        let g = ((((color >> 8) & 255) * gain[1] as u32) >> 16).min(255);
        let b = ((((color >> 16) & 255) * gain[2] as u32) >> 16).min(255);
        r | (g << 8) | (b << 16) | alpha
    }

    /// Floor/ceiling shade of four texels. Pixel-identical to [`shade_texel`];
    /// the wasm build widens the multiplies with simd128.
    #[inline(always)]
    pub(crate) fn shade_texels4(colors: [u32; 4], gain: [[i32; 3]; 4], alpha: u32) -> [u32; 4] {
        #[cfg(target_feature = "simd128")]
        {
            use core::arch::wasm32::*;
            let color = u32x4(colors[0], colors[1], colors[2], colors[3]);
            let mask = u32x4_splat(255);
            let shade_ch = |shift: u32, g0: i32, g1: i32, g2: i32, g3: i32| {
                let chan = v128_and(u32x4_shr(color, shift), mask);
                let g = i32x4(g0, g1, g2, g3);
                u32x4_min(u32x4_shr(u32x4_mul(chan, g), 16), mask)
            };
            let r = shade_ch(0, gain[0][0], gain[1][0], gain[2][0], gain[3][0]);
            let g = u32x4_shl(shade_ch(8, gain[0][1], gain[1][1], gain[2][1], gain[3][1]), 8);
            let b = u32x4_shl(shade_ch(16, gain[0][2], gain[1][2], gain[2][2], gain[3][2]), 16);
            let out = v128_or(v128_or(v128_or(r, g), b), u32x4_splat(alpha));
            return [
                u32x4_extract_lane::<0>(out),
                u32x4_extract_lane::<1>(out),
                u32x4_extract_lane::<2>(out),
                u32x4_extract_lane::<3>(out),
            ];
        }
        #[cfg(not(target_feature = "simd128"))]
        [
            Self::shade_texel(colors[0], gain[0], alpha),
            Self::shade_texel(colors[1], gain[1], alpha),
            Self::shade_texel(colors[2], gain[2], alpha),
            Self::shade_texel(colors[3], gain[3], alpha),
        ]
    }
}
