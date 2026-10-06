//! Procedural atlas generation, mipmapping and texture sampling.
//!
//!  builds every world surface once at boot into the engine-side
//! atlas; nothing here touches game state, so it can be reasoned about (and
//! regenerated) without reading the simulation.

use super::*;

impl Engine {
    pub(crate) fn gen_textures(&mut self) {
        for id in 0..TEX_N {
            // Authored extension layers have a flat fallback until preload completes.
            // Filling directly avoids hashing millions of pixels for unused noise.
            if id >= T_ENEMY_PROJECTILE {
                self.tex[id * TEX * TEX..(id + 1) * TEX * TEX].fill(Self::pack(22, 14, 12, 255));
                continue;
            }
            for y in 0..TEX {
                for x in 0..TEX {
                    let n = Self::nbit(id as u32 + 3, x as i32, y as i32);
                    let n2 = Self::nbit(id as u32 + 9, x as i32 / 2, y as i32 / 2);
                    let c = match id {
                        T_BRICK => {
                            let bx = (x.wrapping_add(if (y / 8) % 2 == 1 { 8 } else { 0 })) % 16;
                            let by = y % 8;
                            let mortar = bx == 0 || by == 0;
                            if mortar {
                                Self::pack(28, 18, 16, 255)
                            } else {
                                let r = 90.0 + n * 40.0 + n2 * 20.0;
                                Self::pack(r as u32, (r * 0.32) as u32, (r * 0.22) as u32, 255)
                            }
                        }
                        T_METAL => {
                            let rivet = (x % 32 == 4 || x % 32 == 27) && (y % 32 == 4 || y % 32 == 27);
                            let panel = (x % 32 < 2) || (y % 32 < 2);
                            if rivet {
                                Self::pack(160, 150, 140, 255)
                            } else if panel {
                                Self::pack(18, 18, 20, 255)
                            } else {
                                let g = 36.0 + n * 22.0;
                                Self::pack(g as u32, (g * 0.95) as u32, (g * 0.9) as u32, 255)
                            }
                        }
                        T_FLESH => {
                            let v = (n * 80.0 + n2 * 50.0) as u32;
                            let vein = ((x as i32 * 3 + y as i32 * 5) % 17) == 0;
                            if vein {
                                Self::pack(40, 8, 10, 255)
                            } else {
                                Self::pack(90 + v / 3, 18 + v / 8, 22 + v / 10, 255)
                            }
                        }
                        T_SKULL => {
                            let bone = n > 0.55;
                            if bone {
                                Self::pack(180, 165, 140, 255)
                            } else {
                                Self::pack(50, 28, 22, 255)
                            }
                        }
                        T_DOOR => {
                            let seam = x > 60 && x < 68;
                            let stripe = y % 24 < 5;
                            if seam {
                                Self::pack(12, 12, 12, 255)
                            } else if stripe {
                                Self::pack(90, 70, 18, 255)
                            } else {
                                Self::pack(40 + (n * 20.0) as u32, 38, 36, 255)
                            }
                        }
                        T_GRATE => {
                            let g = (x % 16 < 3) || (y % 16 < 3);
                            if g {
                                Self::pack(70, 68, 62, 255)
                            } else {
                                Self::pack(16, 12, 10, 255)
                            }
                        }
                        T_CONC => {
                            let g = 70.0 + n * 30.0;
                            Self::pack(g as u32, (g * 0.92) as u32, (g * 0.82) as u32, 255)
                        }
                        T_CEIL => {
                            let pipe = y % 32 < 6;
                            if pipe {
                                Self::pack(40, 38, 36, 255)
                            } else {
                                Self::pack(28, 24, 22, 255)
                            }
                        }
                        T_HUSK => Self::silhouette(x, y, 0xFF2040C0, n),
                        T_BRUTE => Self::silhouette(x, y, 0xFF103090, n),
                        T_WRAITH => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 28.0 {
                                Self::pack(220, 210, 190, 255)
                            } else if d < 48.0 {
                                Self::pack(220, 80, 20, 220)
                            } else if d < 58.0 {
                                Self::pack(180, 40, 10, 120)
                            } else {
                                0
                            }
                        }
                        T_MED => {
                            let boxy = x > 30 && x < 98 && y > 36 && y < 100;
                            let cross = (x > 56 && x < 72 && y > 44 && y < 92)
                                || (x > 40 && x < 88 && y > 58 && y < 74);
                            if cross {
                                Self::pack(200, 30, 30, 255)
                            } else if boxy {
                                Self::pack(50, 70, 40, 255)
                            } else {
                                0
                            }
                        }
                        T_AMMO => {
                            let boxy = x > 28 && x < 100 && y > 40 && y < 104;
                            if boxy {
                                Self::pack(150, 110, 40, 255)
                            } else {
                                0
                            }
                        }
                        T_ARMOR => {
                            let vest = x > 34 && x < 94 && y > 28 && y < 108;
                            if vest {
                                Self::pack(50, 55, 60, 255)
                            } else {
                                0
                            }
                        }
                        T_BARREL => {
                            let cx = x as i32 - 64;
                            let body = cx.abs() < 28 && y > 18 && y < 118;
                            let band = y > 54 && y < 70;
                            if band && body {
                                Self::pack(180, 150, 20, 255)
                            } else if body {
                                Self::pack(140, 40, 22, 255)
                            } else {
                                0
                            }
                        }
                        T_ORDNANCE => {
                            let dx = (x % 128) as f32 - 64.0;
                            let dy = (y % 128) as f32 - 64.0;
                            let alpha = (1.0 - (dx * dx + dy * dy).sqrt() / 44.0).clamp(0.0, 1.0);
                            let frame = (x / 128) + (y / 128) * 2;
                            let rgb = match frame { 1 => [255, 140, 30], 3 => [100, 230, 30], _ => [70, 180, 255] };
                            Self::pack(rgb[0], rgb[1], rgb[2], (alpha * 255.0) as u32)
                        }
                        T_BALL => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 18.0 {
                                Self::pack(255, 230, 160, 255)
                            } else if d < 32.0 {
                                Self::pack(255, 90, 20, 230)
                            } else if d < 42.0 {
                                Self::pack(180, 20, 10, 80)
                            } else {
                                0
                            }
                        }
                        T_SPLAT => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 20.0 + n * 16.0 {
                                Self::pack(120, 10, 10, 200)
                            } else {
                                0
                            }
                        }
                        T_TECH => {
                            let panel = (x % 32 < 3) || (y % 32 < 3);
                            let screen = x % 32 > 6 && x % 32 < 26 && y % 32 > 6 && y % 32 < 22;
                            let scan = (y / 2) % 3 == 0;
                            if panel {
                                Self::pack(12, 14, 16, 255)
                            } else if screen {
                                let glow = 40.0 + n * 80.0;
                                if scan {
                                    Self::pack(20, (glow * 0.3) as u32, (glow * 0.2) as u32, 255)
                                } else {
                                    Self::pack((glow * 0.9) as u32, 18, 22, 255)
                                }
                            } else {
                                Self::pack(28 + (n * 10.0) as u32, 24, 22, 255)
                            }
                        }
                        T_HAZARD => {
                            let stripe = ((x as i32 + y as i32) / 12) % 2 == 0;
                            if stripe {
                                Self::pack(170, 140, 28, 255)
                            } else {
                                Self::pack(18, 16, 12, 255)
                            }
                        }
                        T_LAMP => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 50;
                            let cage = cx.abs() < 22 && y > 18 && y < 100;
                            let bulb = (cx * cx + cy * cy) < 14 * 14;
                            if bulb {
                                Self::pack(255, 200, 90, 255)
                            } else if cage {
                                let wire = (x % 6 < 2) || (y % 8 < 2);
                                if wire {
                                    Self::pack(90, 70, 40, 255)
                                } else {
                                    Self::pack(40, 32, 22, 180)
                                }
                            } else {
                                0
                            }
                        }
                        T_CRATE => {
                            let boxy = x > 24 && x < 104 && y > 28 && y < 118;
                            let band = y > 68 && y < 78;
                            if band && boxy {
                                Self::pack(160, 90, 30, 255)
                            } else if boxy {
                                Self::pack(110, 48, 28, 255)
                            } else {
                                0
                            }
                        }
                        T_IMPACT => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            if d < 10.0 {
                                Self::pack(255, 230, 160, 255)
                            } else if d < 22.0 + n * 8.0 {
                                Self::pack(255, 120, 30, 200)
                            } else if d < 36.0 {
                                Self::pack(180, 40, 16, 90)
                            } else {
                                0
                            }
                        }
                        T_MUZZLEFX => {
                            let cx = x as i32 - 64;
                            let cy = y as i32 - 64;
                            let d = ((cx * cx + cy * cy) as f32).sqrt();
                            let star = (cx.abs() < 4 && cy.abs() < 40) || (cy.abs() < 4 && cx.abs() < 40);
                            if d < 12.0 || star {
                                Self::pack(255, 220, 140, 255)
                            } else if d < 28.0 {
                                Self::pack(255, 90, 20, 160)
                            } else {
                                0
                            }
                        }
                        T_FLAME => {
                            let cx = x as i32 - 64;
                            let taper = 18.0 - (127.0 - y as f32) * 0.12 + n * 6.0;
                            let body = cx.abs() < taper as i32 && y > 20;
                            if body {
                                let hot = y < 70;
                                if hot {
                                    Self::pack(255, 230, 140, 255)
                                } else {
                                    Self::pack(255, 90 + (n * 40.0) as u32, 20, 230)
                                }
                            } else {
                                0
                            }
                        }
                        T_CHAIN => {
                            let cx = x as i32 - 64;
                            let link = cx.abs() < 8 && (y % 14 < 9);
                            let hook = y > 96 && cx.abs() < 18 && y < 122;
                            if hook {
                                Self::pack(140, 40, 28, 255)
                            } else if link {
                                Self::pack(90, 80, 70, 255)
                            } else {
                                0
                            }
                        }
                        T_PIPES => {
                            let v = x % 22 < 7;
                            let hpipe = y % 28 < 6;
                            let valve = (x % 22 == 3) && (y % 28 == 3);
                            if valve {
                                Self::pack(160, 70, 30, 255)
                            } else if v {
                                Self::pack(70 + (n * 30.0) as u32, 48, 40, 255)
                            } else if hpipe {
                                Self::pack(50, 46, 42, 255)
                            } else {
                                Self::pack(22, 18, 16, 255)
                            }
                        }
                        T_GUN2 => {
                            let body = x > 36 && x < 92 && y > 22 && y < 118;
                            let barrel = x > 88 && x < 118 && y > 52 && y < 70;
                            if barrel {
                                Self::pack(220, 180, 70, 255)
                            } else if body {
                                Self::pack(180 + (n * 40.0) as u32, 150, 40, 255)
                            } else {
                                0
                            }
                        }
                        _ => Self::pack(22, 14, 12, 255),
                    };
                    self.tex[id * TEX * TEX + y * TEX + x] = c;
                }
            }
        }
    }

    pub(crate) fn silhouette(x: usize, y: usize, color: u32, n: f32) -> u32 {
        let cx = x as i32 - 64;
        let cy = y as i32 - 20;
        let head = {
            let dx = cx;
            let dy = cy - 8;
            (dx * dx + dy * dy) < 16 * 16
        };
        let body = cx.abs() < (18.0 + n * 4.0) as i32 && y > 40 && y < 100;
        let legs = (cx.abs() - 8).abs() < 7 && (96..122).contains(&y);
        if head || body || legs {
            color
        } else {
            0
        }
    }

    pub(crate) fn sample(&self, id: usize, u: i32, v: i32) -> u32 {
        let u = (u as usize) & (TEX - 1);
        let v = (v as usize) & (TEX - 1);
        self.tex[id * TEX * TEX + v * TEX + u]
    }

    pub(crate) fn rebuild_mipmaps(&mut self) {
        self.mipmaps.clear();
        for level in 1..=8 {
            let size = TEX >> level;
            let source = if level == 1 { &self.tex } else { &self.mipmaps[level - 2] };
            let mut pixels = vec![0u32; TEX_N * size * size];
            for id in 0..TEX_N {
                Self::mipmap_layer_into(source, &mut pixels, id, level);
            }
            self.mipmaps.push(pixels);
        }
    }

    pub(crate) fn mipmap_layer_into(source: &[u32], pixels: &mut [u32], id: usize, level: usize) {
        let size = TEX >> level;
        let prev_size = size * 2;
        for y in 0..size {
            for x in 0..size {
                let p = id * prev_size * prev_size + y * 2 * prev_size + x * 2;
                let samples = [source[p], source[p + 1], source[p + prev_size], source[p + prev_size + 1]];
                let mut color = 0;
                for shift in [0, 8, 16, 24] {
                    let channel: u32 = samples.iter().map(|c| (c >> shift) & 255).sum();
                    color |= (channel / 4) << shift;
                }
                pixels[id * size * size + y * size + x] = color;
            }
        }
    }

    /// Rebuild one layer's mipmap chain (used after a theme swap so a
    /// wave change doesn't pay for all 140 layers).
    pub(crate) fn rebuild_mipmap_layer(&mut self, id: usize) {
        for level in 1..=8 {
            if level > self.mipmaps.len() {
                return;
            }
            // Borrow the previous level's pixels without aliasing self.
            let prev: Vec<u32> = if level == 1 {
                self.tex.clone()
            } else {
                self.mipmaps[level - 2].clone()
            };
            Self::mipmap_layer_into(&prev, &mut self.mipmaps[level - 1], id, level);
        }
    }

    /// Theme index shared with THEME order in src/game/runtime.ts.
    pub(crate) fn theme_index(wave: i32) -> usize {
        [0, 4, 5, 3, 6, 7, 4, 3, 6, 0, 7].get(map::level_index(wave)).copied().unwrap_or(3 + map::level_index(wave) % 4)
    }

    /// Copy the wave's wall/door variants into the live T_TECH/T_DOOR
    /// atlas slots and refresh their mipmap chains.
    pub(crate) fn apply_theme(&mut self, wave: i32) {
        let theme = Self::theme_index(wave);
        let wall = &self.theme_tex[theme * TEX * TEX..(theme + 1) * TEX * TEX].to_vec();
        let door = &self.theme_tex[(8 + theme) * TEX * TEX..(9 + theme) * TEX * TEX].to_vec();
        self.tex[T_TECH * TEX * TEX..(T_TECH + 1) * TEX * TEX].copy_from_slice(wall);
        self.tex[T_DOOR * TEX * TEX..(T_DOOR + 1) * TEX * TEX].copy_from_slice(door);
        self.rebuild_mipmap_layer(T_TECH);
        self.rebuild_mipmap_layer(T_DOOR);
        self.light_dirty = true;
    }

    #[cfg(test)]
    pub(crate) fn sample_lod(&self, id: usize, u: i32, v: i32, footprint: f32) -> u32 {
        let lod = footprint.max(1.0).log2().clamp(0.0, 8.0);
        let level = lod as usize;
        self.sample_mip(id, u, v, level, (lod.fract() * 256.0) as u32)
    }

    pub(crate) fn mip_slot(id: usize, u: i32, v: i32, level: usize) -> (u8, usize) {
        if level == 0 {
            let u = (u as usize) & (TEX - 1);
            let v = (v as usize) & (TEX - 1);
            return (0, id * TEX * TEX + v * TEX + u);
        }
        let size = TEX >> level;
        let x = ((u & TEXM) as usize) >> level;
        let y = ((v & TEXM) as usize) >> level;
        (level as u8, id * size * size + y * size + x)
    }

    pub(crate) fn sample_mip_at(&self, id: usize, u: i32, v: i32, level: usize) -> u32 {
        let (which, idx) = Self::mip_slot(id, u, v, level);
        if which == 0 { self.tex[idx] } else { self.mipmaps[which as usize - 1][idx] }
    }

    pub(crate) fn blend_mips(a: u32, b: u32, mix: u32) -> u32 {
        let inv = 256 - mix;
        let rb = (((a & 0x00ff00ff) * inv + (b & 0x00ff00ff) * mix) >> 8) & 0x00ff00ff;
        let g = (((a & 0x0000ff00) * inv + (b & 0x0000ff00) * mix) >> 8) & 0x0000ff00;
        rb | g | (a & 0xff000000)
    }

    /// Four-wide mip blend. Wasm builds this with simd128; native tests use the
    /// same integer formula lane by lane so a refactor cannot drift the picture.
    #[inline(always)]
    pub(crate) fn blend_mips4(a: [u32; 4], b: [u32; 4], mix: u32) -> [u32; 4] {
        #[cfg(target_feature = "simd128")]
        {
            use core::arch::wasm32::*;
            let av = u32x4(a[0], a[1], a[2], a[3]);
            let bv = u32x4(b[0], b[1], b[2], b[3]);
            let mix_v = u32x4_splat(mix);
            let inv = u32x4_splat(256 - mix);
            let rb_mask = u32x4_splat(0x00ff_00ff);
            let g_mask = u32x4_splat(0x0000_ff00);
            let rb = v128_and(
                u32x4_shr(
                    u32x4_add(
                        u32x4_mul(v128_and(av, rb_mask), inv),
                        u32x4_mul(v128_and(bv, rb_mask), mix_v),
                    ),
                    8,
                ),
                rb_mask,
            );
            let g = v128_and(
                u32x4_shr(
                    u32x4_add(
                        u32x4_mul(v128_and(av, g_mask), inv),
                        u32x4_mul(v128_and(bv, g_mask), mix_v),
                    ),
                    8,
                ),
                g_mask,
            );
            let out = v128_or(v128_or(rb, g), v128_and(av, u32x4_splat(0xff00_0000)));
            return [
                u32x4_extract_lane::<0>(out),
                u32x4_extract_lane::<1>(out),
                u32x4_extract_lane::<2>(out),
                u32x4_extract_lane::<3>(out),
            ];
        }
        #[cfg(not(target_feature = "simd128"))]
        [
            Self::blend_mips(a[0], b[0], mix),
            Self::blend_mips(a[1], b[1], mix),
            Self::blend_mips(a[2], b[2], mix),
            Self::blend_mips(a[3], b[3], mix),
        ]
    }

    pub(crate) fn sample_mip(&self, id: usize, u: i32, v: i32, level: usize, mix: u32) -> u32 {
        let a = self.sample_mip_at(id, u, v, level);
        let b = self.sample_mip_at(id, u, v, (level + 1).min(8));
        Self::blend_mips(a, b, mix)
    }

    #[inline(always)]
    pub(crate) fn sample_mip4(&self, id: usize, uv: [(i32, i32); 4], level: usize, mix: u32) -> [u32; 4] {
        let next = (level + 1).min(8);
        // Close rows rarely share a texel, and the slot compare loses to four gathers.
        if level >= 3 {
            let slot = Self::mip_slot(id, uv[0].0, uv[0].1, level);
            let slot_b = Self::mip_slot(id, uv[0].0, uv[0].1, next);
            let shared = (1..4).all(|i| {
                Self::mip_slot(id, uv[i].0, uv[i].1, level) == slot
                    && Self::mip_slot(id, uv[i].0, uv[i].1, next) == slot_b
            });
            if shared {
                let a = self.sample_mip_at(id, uv[0].0, uv[0].1, level);
                let b = self.sample_mip_at(id, uv[0].0, uv[0].1, next);
                return Self::blend_mips4([a, a, a, a], [b, b, b, b], mix);
            }
        }
        let mut a = [0u32; 4];
        let mut b = [0u32; 4];
        for i in 0..4 {
            a[i] = self.sample_mip_at(id, uv[i].0, uv[i].1, level);
            b[i] = self.sample_mip_at(id, uv[i].0, uv[i].1, next);
        }
        Self::blend_mips4(a, b, mix)
    }

    pub(crate) fn shade(c: u32, f: f32) -> u32 {
        let f = f.clamp(0.0, 1.4);
        let r = ((c & 255) as f32 * f) as u32;
        let g = (((c >> 8) & 255) as f32 * f) as u32;
        let b = (((c >> 16) & 255) as f32 * f) as u32;
        let a = (c >> 24) & 255;
        (r.min(255)) | (g.min(255) << 8) | (b.min(255) << 16) | (a << 24)
    }

    pub(crate) fn fog(c: u32, dist: f32) -> u32 {
        let a = ((dist * (1.0 / 28.0)).clamp(0.0, 1.0) * 255.0) as u32;
        (c & 0x00FFFFFF) | (a << 24)
    }
}
