//! Software and GPU presentation.
//!
//! The simulation still casts every screen column; this module decides what the
//! frame looks like.  fills the column and sprite scratch buffers
//! that the WebGPU/WebGL path uploads, while  writes the software
//! framebuffer for the canvas2d fallback. Both read the same light and texture
//! grids, so they cannot disagree about lighting.

use super::*;

impl Engine {
    /// Texel footprint of one wall column. Near columns stay at LOD 0, so they
    /// match an unfiltered sample; distant columns use the floor mip chain.
    pub(crate) fn column_lod(perp: f32, line_h: i32, plane_len: f32, w: usize) -> (usize, u32) {
        let vertical = TEX as f32 / line_h.max(1) as f32;
        let horizontal = perp * 2.0 * plane_len / w.max(1) as f32 * TEX as f32;
        let lod = horizontal.max(vertical).max(1.0).log2().clamp(0.0, 8.0);
        (lod as usize, (lod.fract() * 256.0) as u32)
    }

    pub(crate) fn plane_sample(&self, floor: bool, fx: f32, fy: f32, level: usize) -> (usize, i32, i32, usize) {
        let tx = (fx * TEX as f32).floor() as i32;
        let ty = (fy * TEX as f32).floor() as i32;
        let id=if self.hell { T_BOSS_ARENA + map::level_index(self.wave) }
            else { T_SECTOR_SURFACE + map::level_index(self.wave) * 5 + if floor {3} else {4} };
        (id,tx,ty,level)
    }

    pub(crate) fn sigil_uv(&self, fx: f32, fy: f32) -> Option<(f32, f32, usize)> {
        let mx = fx.floor() as i32;
        let my = fy.floor() as i32;
        if mx < 0 || my < 0 || mx >= MAP_W as i32 || my >= MAP_H as i32 { return None; }
        if self.floor[my as usize * MAP_W + mx as usize] != 2 { return None; }
        let level = map::level_index(self.wave);
        let (cx, cy) = map::override_point(self.wave);
        let u = (fx - cx) / 1.55;
        let v = (fy - cy) / 1.55;
        if u.abs() > 1.15 || v.abs() > 1.15 { return None; }
        Some((u, v, level))
    }

    pub(crate) fn blend_sigil(&self, base: u32, fx: f32, fy: f32) -> u32 {
        let (x,y)=(fx.floor() as i32,fy.floor() as i32);
        if x>=0 && y>=0 && x<MAP_W as i32 && y<MAP_H as i32 {
            let style=self.floor[y as usize*MAP_W+x as usize];
            if (5..=7).contains(&style) {
                let (u,v)=(fx.fract(),fy.fract());
                let mark=match style {5=>((u-v*0.25-0.25).abs()<0.018)||((u-v*0.25-0.61).abs()<0.018),6=>(u-0.35).abs()<0.025||(u-0.65).abs()<0.025,_=>(v*9.0).fract()<0.14&&u>0.2&&u<0.8};
                if mark {return Self::blend(base,Self::pack(118,142,145,255),0.42);}
                return base;
            }
        }
        let Some((u, v, level)) = self.sigil_uv(fx, fy) else { return base; };
        let mark = sigil_rgba(level, u, v);
        if mark[3] == 0 { return base; }
        let a = mark[3];
        let inv = 255 - a;
        let ch = |shift: u32, src: u32| {
            (((base >> shift) & 255) * inv + src * a) / 255
        };
        ch(0, mark[0]) | (ch(8, mark[1]) << 8) | (ch(16, mark[2]) << 16) | (base & 0xFF00_0000)
    }

    pub(crate) fn plane_texel(&self, floor: bool, fx: f32, fy: f32, level: usize, mix: u32) -> u32 {
        let (id, u, v, lvl) = self.plane_sample(floor, fx, fy, level);
        let base = self.sample_mip(id, u, v, lvl, mix);
        if floor && self.boss_arena.armed {
            let x=fx.floor() as i32; let y=fy.floor() as i32;
            if x>=0 && y>=0 && x<MAP_W as i32 && y<MAP_H as i32 {
                let style=self.floor[y as usize*MAP_W+x as usize];
                if style==3 || style==4 {
                    let p=boss_arena::PROFILES[map::level_index(self.wave)];
                    let color=Self::pack((p.color[0]*255.0) as u32,(p.color[1]*255.0) as u32,(p.color[2]*255.0) as u32,255);
                    let line=((fx.fract()-0.5).abs()>0.39 || (fy.fract()-0.5).abs()>0.39) as u8 as f32;
                    let amount=if style==4 {0.32+line*0.38} else {line*0.5};
                    return Self::blend(base,color,amount);
                }
            }
            self.blend_sigil(base,fx,fy)
        } else if floor { self.blend_sigil(base,fx,fy) } else {base}
    }

    pub(crate) fn render_planes(&mut self, dx: f32, dy: f32, plane_x: f32, plane_y: f32,
        horizon: f32, walls: &[[i32; 2]]) {
        let w = self.w;
        let h = self.h;
        let plane_len = (plane_x * plane_x + plane_y * plane_y).sqrt();
        for y in 0..h {
            let p = y as f32 - horizon;
            if p.abs() < 0.5 {
                for x in 0..w {
                    if (y as i32) < walls[x][0] || (y as i32) > walls[x][1] {
                        self.fb[y * w + x] = Self::fog(Self::pack(18, 11, 10, 255), 28.0);
                    }
                }
                continue;
            }
            let floor = p > 0.0;
            let distance = (0.5 * h as f32) / p.abs();
            let step_x = 2.0 * plane_x * distance / w as f32;
            let step_y = 2.0 * plane_y * distance / w as f32;
            let start_x = self.px + (dx - plane_x) * distance + step_x * 0.5;
            let start_y = self.py + (dy - plane_y) * distance + step_y * 0.5;
            let footprint = (distance * 2.0 * plane_len / w as f32)
                .max(distance / p.abs()) * TEX as f32;
            let lod = footprint.max(1.0).log2().clamp(0.0, 8.0);
            let level = lod as usize;
            let mix = (lod.fract() * 256.0) as u32;
            let shade = if floor { 0.72 + 0.2 / (1.0 + distance * 0.2)
                + self.muzzle * 0.45 / (1.0 + distance * distance) } else { 0.78 };
            let alpha = Self::fog(0, distance);
            let mut gain = [0i32; 3];
            let mut gain_step = [0i32; 3];
            let mut x = 0;
            while x < w {
                let n = (w - x).min(4);
                let mut colors = [0u32; 4];
                let mut gains = [[0i32; 3]; 4];
                let mut write = [false; 4];
                let mut uv = [(0i32, 0i32); 4];
                let mut tid = 0usize;
                let mut sample_level = level;
                let mut uniform = true;
                let mut seen = false;
                for i in 0..n {
                    let px = x + i;
                    if px % 8 == 0 {
                        let light = self.light_at(start_x + step_x * px as f32, start_y + step_y * px as f32);
                        let next = self.light_at(start_x + step_x * (px + 8) as f32, start_y + step_y * (px + 8) as f32);
                        for c in 0..3 {
                            gain[c] = ((shade + light[c]).clamp(0.18, 1.8) * 65536.0) as i32;
                            let target = ((shade + next[c]).clamp(0.18, 1.8) * 65536.0) as i32;
                            gain_step[c] = (target - gain[c]) / 8;
                        }
                    } else {
                        for c in 0..3 { gain[c] += gain_step[c]; }
                    }
                    gains[i] = gain;
                    if (y as i32) >= walls[px][0] && (y as i32) <= walls[px][1] {
                        continue;
                    }
                    write[i] = true;
                    let fx = start_x + step_x * px as f32;
                    let fy = start_y + step_y * px as f32;
                    let (id, u, v, lvl) = self.plane_sample(floor, fx, fy, level);
                    uv[i] = (u, v);
                    if !seen {
                        tid = id;
                        sample_level = lvl;
                        seen = true;
                    } else if id != tid || lvl != sample_level {
                        uniform = false;
                    }
                }
                if seen {
                    if uniform {
                        colors = self.sample_mip4(tid, uv, sample_level, mix);
                    } else {
                        for i in 0..n {
                            if !write[i] { continue; }
                            let fx = start_x + step_x * (x + i) as f32;
                            let fy = start_y + step_y * (x + i) as f32;
                            colors[i] = self.plane_texel(floor, fx, fy, level, mix);
                        }
                    }
                    let shaded = Self::shade_texels4(colors, gains, alpha);
                    let row = y * w;
                    for i in 0..n {
                        if write[i] { self.fb[row + x + i] = shaded[i]; }
                    }
                }
                x += n;
            }
        }
    }

    pub(crate) fn prepare_gpu(&mut self) {
        self.update_lighting();
        self.rebuild_smoke_grid();
        let w = self.w.min(MAX_W);
        let h = self.h;
        let dir_x = self.pa.cos();
        let dir_y = self.pa.sin();
        let aspect = w as f32 / h.max(1) as f32;
        let plane_len = 0.72 * (aspect / 1.6);
        let plane_x = -dir_y * plane_len;
        let plane_y = dir_x * plane_len;
        let horizon = h as f32 * 0.5 + self.pitch * h as f32 * 0.9;
        let tnow = self.time;
        let scratch = gpu_scratch();
        scratch.view = GpuView {
            px: self.px,
            py: self.py,
            dir_x,
            dir_y,
            plane_x,
            plane_y,
            horizon,
            time: tnow,
            hell: if self.hell { 1.0 } else { 0.0 },
            muzzle: self.muzzle,
            w: w as f32,
            h: h as f32,
            plane_len,
            sprite_n: 0.0,
            _p1: map::level_index(self.wave) as f32,
            _p2: if self.ents.iter().any(|e| e.kind == EK_BOSS && e.hp > 0) { 1.0 } else { 0.0 },
        };
        for x in 0..w {
            let cam = 2.0 * (x as f32 + 0.5) / w as f32 - 1.0;
            let mut rdx = dir_x + plane_x * cam;
            let mut rdy = dir_y + plane_y * cam;
            if rdx.abs() < 1e-6 { rdx = 1e-6; }
            if rdy.abs() < 1e-6 { rdy = 1e-6; }
            let mut map_x = self.px.floor() as i32;
            let mut map_y = self.py.floor() as i32;
            let ddx = (1.0 / rdx).abs();
            let ddy = (1.0 / rdy).abs();
            let step_x = if rdx < 0.0 { -1 } else { 1 };
            let step_y = if rdy < 0.0 { -1 } else { 1 };
            let mut sdx = if rdx < 0.0 { (self.px - map_x as f32) * ddx } else { (map_x as f32 + 1.0 - self.px) * ddx };
            let mut sdy = if rdy < 0.0 { (self.py - map_y as f32) * ddy } else { (map_y as f32 + 1.0 - self.py) * ddy };
            let mut side = 0;
            let mut hit = 0u8;
            for _ in 0..(MAP_W + MAP_H) {
                if sdx < sdy {
                    sdx += ddx;
                    map_x += step_x;
                    side = 0;
                } else {
                    sdy += ddy;
                    map_y += step_y;
                    side = 1;
                }
                let c = self.cell(map_x, map_y);
                if c != 0 && c != 10 {
                    let open = if (c == 8 || c == 9) && map_x >= 0 && map_y >= 0 && (map_x as usize) < MAP_W && (map_y as usize) < MAP_H {
                        self.door[map_y as usize * MAP_W + map_x as usize]
                    } else { 0.0 };
                    if open >= 0.98 { continue; }
                    hit = c;
                    break;
                }
            }
            let perp = if side == 0 {
                (map_x as f32 - self.px + (1 - step_x) as f32 / 2.0) / rdx
            } else {
                (map_y as f32 - self.py + (1 - step_y) as f32 / 2.0) / rdy
            }.abs().max(0.05);
            let z = if hit == 0 { 40.0 } else { perp };
            let line_h = (h as f32 / perp) as i32;
            let ds_full = -line_h / 2 + horizon as i32;
            let mut draw0 = ds_full;
            let mut draw1 = line_h / 2 + horizon as i32;
            if draw0 < 0 { draw0 = 0; }
            if draw1 >= h as i32 { draw1 = h as i32 - 1; }
            if hit == 0 { draw0 = h as i32; draw1 = -1; }
            let mut wall_x = if side == 0 { self.py + perp * rdy } else { self.px + perp * rdx };
            wall_x -= wall_x.floor();
            let mut tex_x = (wall_x * TEX as f32) as i32;
            if side == 0 && rdx > 0.0 { tex_x = TEX as i32 - tex_x - 1; }
            if side == 1 && rdy < 0.0 { tex_x = TEX as i32 - tex_x - 1; }
            let open = if (hit == 8 || hit == 9) && map_x >= 0 && map_y >= 0 && (map_x as usize) < MAP_W && (map_y as usize) < MAP_H {
                self.door[map_y as usize * MAP_W + map_x as usize]
            } else { 0.0 };
            tex_x = (tex_x + (open * TEX as f32) as i32) & TEXM;
            let mut tid = if hit == 0 { 0 } else { self.wall_tex(hit, map_x, map_y) };
            let hash = (map_x.wrapping_mul(19) + map_y.wrapping_mul(7)) as u32;
            if tid == T_METAL && hash.is_multiple_of(7) { tid = T_HAZARD; }
            if tid == T_BRICK && hash.is_multiple_of(5) { tid = T_SKULL; }
            let light = self.light_at(self.px + (perp - 0.03) * rdx, self.py + (perp - 0.03) * rdy);
            let dec = if map_x >= 0 && map_y >= 0 && (map_x as usize) < MAP_W && (map_y as usize) < MAP_H {
                self.decal[map_y as usize * MAP_W + map_x as usize]
            } else { 0 };
            scratch.cols[x] = GpuCol {
                perp,
                tex_x: tex_x as f32,
                light_r: light[0],
                light_g: light[1],
                light_b: light[2],
                draw0: draw0 as f32,
                draw1: draw1 as f32,
                line_h: line_h.max(1) as f32,
                ds_full: ds_full as f32,
                tex: tid as f32,
                side: side as f32,
                dec: dec as f32,
                hash: hash as f32,
                hit: hit as f32,
                z,
                _pad: 0.0,
            };
        }
        let mut n = 0usize;
        for e in &self.ents {
            if e.kind == 0 || n >= ENT_N { continue; }
            let (tex, scale, sheet4, frame_i) = sprite_style(e);
            let frame = if sheet4 { frame_i as f32 } else { -1.0 };
            scratch.sprites[n] = GpuSprite {
                x: e.x,
                y: e.y,
                zoff: e.zoff,
                scale,
                tex: tex as f32,
                frame,
                flash: if e.kind == EK_LAMP && e.timer < 0.0 { -1.0 } else if e.flash > 0.0 { 1.0 } else if is_hostile_kind(e.kind) && e.effect_tick > 0.0 { 2.0 } else { 0.0 },
                kind: e.kind as f32,
            };
            n += 1;
        }
        // Sprites write RGB and packed depth into the shared world target.
        // Paint far to near, matching the CPU path; transparent holes keep
        // the already drawn scenery instead of overwriting closer entities.
        scratch.sprites[..n].sort_by(|a, b| {
            let depth = |s: &GpuSprite| (s.x - self.px) * dir_x + (s.y - self.py) * dir_y;
            depth(b).total_cmp(&depth(a))
        });
        scratch.sprite_n = n;
        scratch.view.sprite_n = n as f32;
    }

    pub(crate) fn render(&mut self) {
        self.update_lighting();
        self.rebuild_smoke_grid();
        let w = self.w;
        let h = self.h;
        let dir_x = self.pa.cos();
        let dir_y = self.pa.sin();
        let aspect = w as f32 / h as f32;
        let plane_len = 0.72 * (aspect / 1.6);
        let plane_x = -dir_y * plane_len;
        let plane_y = dir_x * plane_len;
        let horizon = h as f32 * 0.5 + self.pitch * h as f32 * 0.9;
        let shx = 0i32;
        let shy = 0i32;
        let tnow = self.time;

        let mut wall_spans = [[0i32; 2]; MAX_W];

        for x in 0..w {
            let cam = 2.0 * (x as f32 + 0.5) / w as f32 - 1.0;
            let mut rdx = dir_x + plane_x * cam;
            let mut rdy = dir_y + plane_y * cam;
            if rdx.abs() < 1e-6 {
                rdx = 1e-6;
            }
            if rdy.abs() < 1e-6 {
                rdy = 1e-6;
            }
            let mut map_x = self.px.floor() as i32;
            let mut map_y = self.py.floor() as i32;
            let ddx = (1.0 / rdx).abs();
            let ddy = (1.0 / rdy).abs();
            let step_x = if rdx < 0.0 { -1 } else { 1 };
            let step_y = if rdy < 0.0 { -1 } else { 1 };
            let mut sdx = if rdx < 0.0 {
                (self.px - map_x as f32) * ddx
            } else {
                (map_x as f32 + 1.0 - self.px) * ddx
            };
            let mut sdy = if rdy < 0.0 {
                (self.py - map_y as f32) * ddy
            } else {
                (map_y as f32 + 1.0 - self.py) * ddy
            };
            let mut side = 0;
            let mut hit = 0u8;
            for _ in 0..(MAP_W + MAP_H) {
                if sdx < sdy {
                    sdx += ddx;
                    map_x += step_x;
                    side = 0;
                } else {
                    sdy += ddy;
                    map_y += step_y;
                    side = 1;
                }
                let c = self.cell(map_x, map_y);
                if c != 0 && c != 10 {
                    let open = if c == 8 || c == 9 {
                        self.door[map_y as usize * MAP_W + map_x as usize]
                    } else {
                        0.0
                    };
                    if open >= 0.98 {
                        continue;
                    }
                    hit = c;
                    break;
                }
            }
            let perp = if side == 0 {
                (map_x as f32 - self.px + (1 - step_x) as f32 / 2.0) / rdx
            } else {
                (map_y as f32 - self.py + (1 - step_y) as f32 / 2.0) / rdy
            }
            .abs()
            .max(0.05);
            self.zbuf[x] = if hit == 0 { 40.0 } else { perp };
            let line_h = (h as f32 / perp) as i32;
            let mut draw0 = -line_h / 2 + horizon as i32;
            let mut draw1 = line_h / 2 + horizon as i32;
            let ds_full = draw0;
            if draw0 < 0 {
                draw0 = 0;
            }
            if draw1 >= h as i32 {
                draw1 = h as i32 - 1;
            }
            wall_spans[x] = if hit == 0 { [h as i32, -1] } else { [draw0, draw1] };

            let mut wall_x = if side == 0 {
                self.py + perp * rdy
            } else {
                self.px + perp * rdx
            };
            wall_x -= wall_x.floor();
            let mut tex_x = (wall_x * TEX as f32) as i32;
            if side == 0 && rdx > 0.0 {
                tex_x = TEX as i32 - tex_x - 1;
            }
            if side == 1 && rdy < 0.0 {
                tex_x = TEX as i32 - tex_x - 1;
            }
            let open = if hit == 8 || hit == 9 {
                self.door[map_y as usize * MAP_W + map_x as usize]
            } else {
                0.0
            };
            tex_x = (tex_x + (open * TEX as f32) as i32) & TEXM;
            let mut tid = self.wall_tex(hit, map_x, map_y);
            let hash = (map_x.wrapping_mul(19) + map_y.wrapping_mul(7)) as u32;
            if tid == T_METAL && hash.is_multiple_of(7) {
                tid = T_HAZARD;
            }
            if tid == T_BRICK && hash.is_multiple_of(5) {
                tid = T_SKULL;
            }
            let step = TEX as f32 / line_h.max(1) as f32;
            let (level, mix) = Self::column_lod(perp, line_h, plane_len, w);
            let scroll = if tid == T_FLESH {
                ((tnow * 7.0).sin() * 4.0) as i32
            } else if tid == T_TECH {
                (tnow * 26.0) as i32
            } else if tid == T_PIPES {
                (tnow * 10.0) as i32
            } else if tid == T_SKULL {
                ((tnow * 3.5).sin() * 3.0) as i32
            } else if tid == T_BRICK {
                let drip = Self::nbit(3, tex_x, 0);
                if drip > 0.84 { (tnow * (10.0 + drip * 18.0)) as i32 } else { 0 }
            } else {
                0
            };
            let mut tex_pos = (draw0 - ds_full) as f32 * step;
            let side_mul = if side == 1 { 0.65 } else { 1.0 };
            let mut dist_mul = (1.0 / (1.0 + perp * 0.12)) * side_mul;
            if tid == T_FLESH {
                dist_mul *= 0.82 + 0.22 * (tnow * 3.4 + map_x as f32).sin().abs();
            }
            if tid == T_TECH {
                dist_mul *= 0.9 + 0.35 * ((tnow * 11.0 + map_y as f32).sin().abs());
            }
            if tid == T_HAZARD || tid == T_METAL {
                let flick = (tnow * 17.0 + map_x as f32 * 2.1).sin().abs();
                if (hash & 1) == 0 {
                    dist_mul *= 0.85 + 0.4 * flick;
                }
            }
            let light = self.light_at(self.px + (perp - 0.03) * rdx, self.py + (perp - 0.03) * rdy);
            dist_mul += self.muzzle * 0.55 / (1.0 + perp * perp * 0.3);
            let dec = if map_x >= 0 && map_y >= 0 && map_x < MAP_W as i32 && map_y < MAP_H as i32 {
                self.decal[map_y as usize * MAP_W + map_x as usize]
            } else {
                0
            };

            if hit != 0 && tex_x >= 0 && tex_x < TEX as i32 {
                let mut y = draw0;
                let end = draw1 + 1;
                while y < end {
                    let n = (end - y).min(4) as usize;
                    let mut tex_y = [0i32; 4];
                    let mut uv = [(tex_x, 0); 4];
                    for i in 0..n {
                        let ty = (tex_pos as i32).wrapping_add(scroll) & TEXM;
                        tex_pos += step;
                        tex_y[i] = ty;
                        uv[i] = (tex_x, ty);
                    }
                    if n < 4 {
                        let pad = uv[0];
                        uv[n..4].fill(pad);
                    }
                    let mut colors = self.sample_mip4(tid, uv, level, mix);
                    for i in 0..n {
                        let ty = tex_y[i];
                        if tid == T_TECH && (ty & 7) == 0 {
                            colors[i] = Self::shade(colors[i], 1.35);
                        }
                        if (tid == T_METAL || tid == T_HAZARD) && (ty & 31) < 3 && (hash & 1) == 0 {
                            colors[i] = Self::blend(colors[i], Self::pack(255, 140, 40, 255), 0.35);
                        }
                        if dec > 0 {
                            let splat = self.sample_mip(
                                T_SPLAT, tex_x, ty.wrapping_add(dec as i32 * 17), level, mix,
                            );
                            if ((splat >> 24) & 255) > 24 {
                                colors[i] = Self::blend(colors[i], splat, 0.28 + dec as f32 * 0.18);
                            }
                        }
                    }
                    let shaded = Self::shade_rgb4(colors, dist_mul, light);
                    for i in 0..n {
                        let ty = tex_y[i];
                        let mut col = shaded[i];
                        if tid == T_TECH && (ty & 31) < 2 {
                            col = Self::blend(col, Self::pack(64, 190, 224, 255), 0.62);
                        }
                        col = Self::fog(col, perp);
                        let yy = y + i as i32 + shy;
                        let xx = x as i32 + shx;
                        if yy >= 0 && yy < h as i32 && xx >= 0 && xx < w as i32 {
                            self.fb[yy as usize * w + xx as usize] = col;
                        }
                    }
                    y += n as i32;
                }
            }

        }
        self.render_planes(dir_x, dir_y, plane_x, plane_y, horizon, &wall_spans[..w]);

        // sprites
        let mut order = [(0.0f32, 0usize); ENT_N];
        let mut count = 0;
        for (i, e) in self.ents.iter().enumerate() {
            if e.kind == 0 {
                continue;
            }
            let depth = (e.x - self.px) * dir_x + (e.y - self.py) * dir_y;
            if depth <= 0.12 { continue; }
            order[count] = (depth, i);
            count += 1;
        }
        // Painter's order, far to near. Index tiebreak so equal depths resolve
        // identically every frame instead of flickering.
        order[..count].sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
        for &(_d, i) in &order[..count] {
            let e = self.ents[i];
            let sx = e.x - self.px;
            let sy = e.y - self.py;
            let tx = inv_det * (dir_y * sx - dir_x * sy);
            let ty = inv_det * (-plane_y * sx + plane_x * sy);
            if ty <= 0.12 {
                continue;
            }
            // Presentation comes from the roster table: texture slot,
            // world scale and sheet layout per kind (see enemies.rs).
            let (tid, scale, sheet4, fr) = sprite_style(&e);
            let sprite_h = (h as f32 / ty * scale).abs();
            let voff = e.zoff / ty;
            let ds_y = (-sprite_h * 0.5 + horizon + voff) as i32;
            let de_y = (sprite_h * 0.5 + horizon + voff) as i32;
            let sprite_w = sprite_h;
            let screen_x = (w as f32 / 2.0) * (1.0 + tx / ty);
            let ds_x = (-sprite_w / 2.0 + screen_x) as i32;
            let de_x = (sprite_w / 2.0 + screen_x) as i32;
            let flash = e.flash > 0.0;
            let half = TEX as i32 / 2;
            let ou = if sheet4 { (fr & 1) * half } else { 0 };
            let ov = if sheet4 { ((fr >> 1) & 1) * half } else { 0 };
            let cell = if sheet4 { half } else { TEX as i32 };
            let glow = if e.kind == EK_LAMP && e.timer < 0.0 {
                0.28
            } else if e.kind == EK_LAMP {
                1.1 + 0.45 * (tnow * 13.0 + e.x).sin().abs()
            } else if matches!(e.kind, EK_FLAME | EK_BOLT | EK_FIREPATCH) {
                1.3 + 0.45 * (tnow * 18.0 + e.x).sin().abs()
            } else if e.kind == EK_SPARK && e.effect_tick == 5.0 {
                1.0
            } else if e.kind == EK_IMPACT || e.kind == EK_SPARK || e.kind == EK_RAY {
                1.45
            } else {
                1.05
            };
            let sprite_light = self.light_at(e.x, e.y);
            let _distp = _d.sqrt();
            let item = is_pickup(e.kind);
            let gold = is_weapon_item(e.kind);
            let pulse = 0.62 + 0.38 * (tnow * 3.8 + e.x).sin().abs();
            if item {
                let hs_w = sprite_w * 1.12;
                let hs_h = sprite_h * 1.12;
                let hx0 = (-hs_w / 2.0 + screen_x) as i32;
                let hx1 = (hs_w / 2.0 + screen_x) as i32;
                let hy0 = (-hs_h * 0.5 + horizon + voff) as i32;
                let hy1 = (hs_h * 0.5 + horizon + voff) as i32;
                let (hr, hg, hb) = if gold {
                    (255.0, 196.0, 72.0)
                } else {
                    (210.0, 220.0, 232.0)
                };
                for stripe in hx0.max(0)..hx1.min(w as i32) {
                    if ty >= self.zbuf[stripe as usize] * 1.02 {
                        continue;
                    }
                    let nx = (stripe as f32 - screen_x) / (hs_w * 0.5);
                    for y in hy0.max(0)..hy1.min(h as i32) {
                        let ny = (y as f32 - (horizon + voff)) / (hs_h * 0.5);
                        let rad = (nx * nx + ny * ny).sqrt();
                        if !(0.9..=1.0).contains(&rad) {
                            continue;
                        }
                        let band = 1.0 - ((rad - 0.95).abs() / 0.05);
                        let f = band.max(0.0) * (0.18 + 0.14 * pulse);
                        let yy = y + shy;
                        let xx = stripe + shx;
                        if yy < 0 || yy >= h as i32 || xx < 0 || xx >= w as i32 {
                            continue;
                        }
                        let p = self.fb[yy as usize * w + xx as usize];
                        let r = ((p & 255) as f32 + hr * f).min(255.0) as u32;
                        let g = (((p >> 8) & 255) as f32 + hg * f).min(255.0) as u32;
                        let b = (((p >> 16) & 255) as f32 + hb * f).min(255.0) as u32;
                        self.fb[yy as usize * w + xx as usize] =
                            r | (g << 8) | (b << 16) | (p & 0xFF000000);
                    }
                }
            }
            for stripe in ds_x.max(0)..de_x.min(w as i32) {
                if ty >= self.zbuf[stripe as usize] {
                    continue;
                }
                let mut tex_x = ((stripe as f32 - (-sprite_w / 2.0 + screen_x)) * cell as f32 / sprite_w) as i32;
                if tex_x < 0 || tex_x >= cell {
                    continue;
                }
                tex_x += ou;
                for y in ds_y.max(0)..de_y.min(h as i32) {
                    let d = y as f32 - ds_y as f32;
                    let mut tex_y = (d * cell as f32 / (de_y - ds_y).max(1) as f32) as i32;
                    if tex_y < 0 || tex_y >= cell {
                        continue;
                    }
                    tex_y += ov;
                    let mut col = self.sample(tid, tex_x, tex_y);
                    if e.kind == EK_RAY {
                        let nx = (stripe as f32 - screen_x) / (sprite_w * 0.5).max(0.001);
                        let ny = (y as f32 - (horizon + voff)) / (sprite_h * 0.5).max(0.001);
                        let rad = nx * nx + ny * ny;
                        if rad >= 1.0 {
                            continue;
                        }
                        let core = (1.0 - rad).powf(2.2);
                        col = if e.zoff >= 11.0 {
                            Self::pack(255, (148.0 + core * 90.0) as u32, (34.0 + core * 150.0) as u32, (core * 235.0) as u32)
                        } else {
                            Self::pack((115.0 + core * 140.0) as u32, (210.0 + core * 45.0) as u32, 255, (core * 230.0) as u32)
                        };
                    } else {
                        let a = (col >> 24) & 255;
                        if a < 16 {
                            continue;
                        }
                        let lum = (col & 255) + ((col >> 8) & 255) + ((col >> 16) & 255);
                        if item && lum < 48 {
                            continue;
                        }
                        if e.kind == EK_BOSS {
                            col = Self::blend(col, Self::pack(255, 36, 24, 255), 0.42);
                        }
                        if flash {
                            col = Self::pack(255, 220, 220, a);
                        } else if is_hostile_kind(e.kind) && e.effect_tick > 0.0 {
                            col = Self::blend(col, Self::pack(255, 184, 60, a), 0.32);
                        }
                    }
                    col = Self::fog(Self::shade_rgb(col, glow, sprite_light), ty);
                    let yy = y + shy;
                    let xx = stripe + shx;
                    if yy >= 0 && yy < h as i32 && xx >= 0 && xx < w as i32 {
                        let index = yy as usize * w + xx as usize;
                        let alpha = ((col >> 24) & 255) as f32 / 255.0;
                        self.fb[index] = Self::blend(self.fb[index], col, alpha);
                    }
                }
            }
        }

    }
}

/// repr(C) frame shared with TypeScript through `hs_gpu_view`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuView {
    pub(crate) px: f32, pub(crate) py: f32, pub(crate) dir_x: f32, pub(crate) dir_y: f32,
    pub(crate) plane_x: f32, pub(crate) plane_y: f32, pub(crate) horizon: f32, pub(crate) time: f32,
    pub(crate) hell: f32, pub(crate) muzzle: f32, pub(crate) w: f32, pub(crate) h: f32,
    pub(crate) plane_len: f32, pub(crate) sprite_n: f32, pub(crate) _p1: f32, pub(crate) _p2: f32,
}

/// repr(C) frame shared with TypeScript through `hs_gpu_cols`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuCol {
    pub(crate) perp: f32, pub(crate) tex_x: f32, pub(crate) light_r: f32, pub(crate) light_g: f32,
    pub(crate) light_b: f32, pub(crate) draw0: f32, pub(crate) draw1: f32, pub(crate) line_h: f32,
    pub(crate) ds_full: f32, pub(crate) tex: f32, pub(crate) side: f32, pub(crate) dec: f32,
    pub(crate) hash: f32, pub(crate) hit: f32, pub(crate) z: f32, pub(crate) _pad: f32,
}

/// repr(C) frame shared with TypeScript through `hs_gpu_sprites`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuSprite {
    pub(crate) x: f32, pub(crate) y: f32, pub(crate) zoff: f32, pub(crate) scale: f32,
    pub(crate) tex: f32, pub(crate) frame: f32, pub(crate) flash: f32, pub(crate) kind: f32,
}

pub(crate) struct GpuScratch {
    pub(crate) enemy_cues: [voices::EnemyCue; ENT_N],
    pub(crate) bar_cues: [voices::BarCue; 48],
    pub(crate) cols: Vec<GpuCol>,
    pub(crate) sprites: Vec<GpuSprite>,
    pub(crate) sprite_n: usize,
    pub(crate) view: GpuView,
}

// Single-threaded WASM: every entry point runs on the same JS thread, so
// exclusive access holds (same contract as eng() above).
#[allow(static_mut_refs)]
pub(crate) fn gpu_scratch() -> &'static mut GpuScratch {
    static mut G: Option<GpuScratch> = None;
    unsafe {
        if G.is_none() {
            G = Some(GpuScratch {
                enemy_cues: [voices::EnemyCue::default(); ENT_N],
                bar_cues: [voices::BarCue::default(); 48],
                cols: vec![GpuCol {
                    perp: 0.0, tex_x: 0.0, light_r: 0.0, light_g: 0.0,
                    light_b: 0.0, draw0: 0.0, draw1: -1.0, line_h: 1.0,
                    ds_full: 0.0, tex: 0.0, side: 0.0, dec: 0.0,
                    hash: 0.0, hit: 0.0, z: 40.0, _pad: 0.0,
                }; MAX_W],
                sprites: vec![GpuSprite {
                    x: 0.0, y: 0.0, zoff: 0.0, scale: 0.0,
                    tex: 0.0, frame: -1.0, flash: 0.0, kind: 0.0,
                }; ENT_N],
                sprite_n: 0,
                view: GpuView {
                    px: 0.0, py: 0.0, dir_x: 1.0, dir_y: 0.0,
                    plane_x: 0.0, plane_y: 0.0, horizon: 0.0, time: 0.0,
                    hell: 0.0, muzzle: 0.0, w: 0.0, h: 0.0,
                    plane_len: 0.0, sprite_n: 0.0, _p1: 0.0, _p2: 0.0,
                },
            });
        }
        G.as_mut().unwrap_unchecked()
    }
}
