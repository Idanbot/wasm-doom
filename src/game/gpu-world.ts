/** GPU wall/floor/sprite fill. The sim still casts columns; this only textures them. */

export const VIEW_FLOATS = 16;
export const COL_FLOATS = 16;
export const SPR_FLOATS = 8;
export const MAP_W = 48;
export const MAP_H = 32;
export const ENEMY_ANIM_COUNT = 7;
export const ENEMY_SKIN_COUNT = 13;
export const ENEMY_TEX_BASE = 29;
export const T_ORDNANCE = ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * ENEMY_SKIN_COUNT;
export const T_GUN3 = T_ORDNANCE + 1;
export const T_GUN4 = T_ORDNANCE + 2;
export const T_GUN5 = T_ORDNANCE + 3;
export const T_GUN6 = T_ORDNANCE + 4;
export const T_GUN7 = T_ORDNANCE + 5;
export const T_GUN8 = T_GUN7 + 4;
export const T_GUN9 = T_GUN8 + 1;
export const T_GUN10 = T_GUN8 + 2;
export const T_GUN11 = T_GUN8 + 3;
export const TEX_N = T_GUN11 + 1;
export const MAX_COLS = 3840;
export const TEX = 256;
export type WorldFrame = {
  w: number;
  h: number;
  view: Float32Array;
  cols: Float32Array;
  sprites: Float32Array;
  spriteCount: number;
  floor: Uint8Array;
  light: Float32Array;
  smoke: Float32Array;
};


export type GpuWorld = {
  uploadAtlas(layers: Uint8Array): void;
  draw(encoder: GPUCommandEncoder, color: GPUTextureView, frame: WorldFrame): void;
  dispose(): void;
};

const FILL_WGSL = `
struct View { v: array<vec4<f32>, 4> }
@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage, read> cols: array<vec4<f32>>;
@group(0) @binding(3) var atlas: texture_2d_array<f32>;
@group(0) @binding(4) var atlas_samp: sampler;
@group(0) @binding(5) var floor_tex: texture_2d<f32>;
@group(0) @binding(6) var light_tex: texture_2d<f32>;

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VSOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  let q = p[i];
  var o: VSOut;
  o.pos = vec4<f32>(q, 0.0, 1.0);
  o.uv = vec2<f32>(q.x * 0.5 + 0.5, 1.0 - (q.y * 0.5 + 0.5));
  return o;
}

fn sample_atlas(id: i32, u: f32, v: f32) -> vec4<f32> {
  return textureSampleLevel(atlas, atlas_samp, vec2<f32>(fract(u), fract(v)), clamp(id, 0, ${TEX_N - 1}), 0.0);
}

fn light_at(wx: f32, wy: f32) -> vec3<f32> {
  let x = clamp(wx - 0.5, 0.0, 46.0);
  let y = clamp(wy - 0.5, 0.0, 30.0);
  let ix = i32(floor(x));
  let iy = i32(floor(y));
  let f = vec2<f32>(fract(x), fract(y));
  let a = textureLoad(light_tex, vec2<i32>(ix, iy), 0).rgb;
  let b = textureLoad(light_tex, vec2<i32>(ix + 1, iy), 0).rgb;
  let c = textureLoad(light_tex, vec2<i32>(ix, iy + 1), 0).rgb;
  let d = textureLoad(light_tex, vec2<i32>(ix + 1, iy + 1), 0).rgb;
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

fn smoke_at(wx: f32, wy: f32) -> f32 {
  let x = i32(clamp(floor(wx), 0.0, 47.0));
  let y = i32(clamp(floor(wy), 0.0, 31.0));
  return textureLoad(light_tex, vec2<i32>(x, y), 0).a;
}

fn through_smoke(rgb: vec3<f32>, ax: f32, ay: f32, bx: f32, by: f32) -> vec3<f32> {
  var tau = 0.0;
  for (var i = 1; i <= 6; i++) {
    let t = f32(i) / 6.0;
    tau += smoke_at(mix(ax, bx, t), mix(ay, by, t));
  }
  tau *= length(vec2<f32>(bx - ax, by - ay)) / 6.0;
  let k = 1.0 - exp(-tau * 1.8);
  return mix(rgb, vec3<f32>(0.58, 0.60, 0.62), clamp(k, 0.0, 0.92));
}
}

fn sigil_center(level: i32) -> vec2<f32> {
  if (level == 1) { return vec2<f32>(43.5, 27.5); }
  if (level == 2) { return vec2<f32>(43.5, 15.5); }
  return vec2<f32>(40.5, 21.5);
}

fn sigil(level: i32, u: f32, v: f32) -> vec4<f32> {
  let d = length(vec2<f32>(u, v));
  if (d > 1.02) { return vec4<f32>(0.0); }
  let ring = smoothstep(0.045, 0.0, abs(d - 0.72));
  if (level == 1) {
    let chev = smoothstep(0.04, 0.0, abs(abs(u) * 0.7 + v * 0.45 - 0.15));
    let bar = smoothstep(0.035, 0.0, abs(v + 0.35)) * step(abs(u), 0.55);
    return vec4<f32>(1.0, 0.42, 0.12, max(ring, max(chev, bar)));
  }
  if (level == 2) {
    let hex = abs(fract(atan2(v, u) / 1.047) - 0.5);
    let edge = smoothstep(0.04, 0.0, abs(d - 0.62)) * step(hex, 0.2);
    let mote = smoothstep(0.07, 0.0, length(vec2<f32>(u - 0.28, v - 0.1)));
    return vec4<f32>(0.35, 0.95, 0.42, max(ring, max(edge, mote)));
  }
  let slit = smoothstep(0.04, 0.0, abs(u)) * step(abs(v), 0.36);
  let iris = smoothstep(0.15, 0.0, length(vec2<f32>(u * 1.4, v)));
  return vec4<f32>(0.95, 0.62, 0.22, max(ring, max(slit, iris * 0.85)));
}

@fragment
fn fs(inp: VSOut) -> @location(0) vec4<f32> {
  let w = max(view.v[2].z, 1.0);
  let h = max(view.v[2].w, 1.0);
  let x = clamp(i32(inp.uv.x * w), 0, i32(w) - 1);
  let py = i32(inp.uv.y * h);
  let c0 = cols[x * 4];
  let c1 = cols[x * 4 + 1];
  let c2 = cols[x * 4 + 2];
  let c3 = cols[x * 4 + 3];
  let perp = c0.x;
  let tex_x = c0.y;
  let light = vec3<f32>(c0.z, c0.w, c1.x);
  let draw0 = i32(c1.y);
  let draw1 = i32(c1.z);
  let line_h = max(c1.w, 1.0);
  let tid = i32(c2.y);
  let time = view.v[1].w;
  let muzzle = view.v[2].y;
  let hell = view.v[2].x;
  if (c3.y > 0.5 && py >= draw0 && py <= draw1) {
    var ty = (f32(py) - c2.x) * (256.0 / line_h);
    if (tid == 2) { ty += sin(time * 7.0) * 4.0; }
    else if (tid == 18) { ty += time * 26.0; }
    else if (tid == 26) { ty += time * 10.0; }
    else if (tid == 3) { ty += sin(time * 3.5) * 3.0; }
    var rgb = sample_atlas(tid, tex_x / 256.0, ty / 256.0).rgb;
    let side_mul = select(1.0, 0.65, c2.z > 0.5);
    let dist = (1.0 / (1.0 + perp * 0.12)) * side_mul + muzzle * 0.55 / (1.0 + perp * perp * 0.3);
    rgb = clamp(rgb * clamp(vec3<f32>(dist) + light, vec3<f32>(0.18), vec3<f32>(1.8)), vec3<f32>(0.0), vec3<f32>(1.0));
    let cam = 2.0 * (f32(x) + 0.5) / w - 1.0;
    let ray = view.v[0].zw + view.v[1].xy * cam;
    rgb = through_smoke(rgb, view.v[0].x, view.v[0].y, view.v[0].x + perp * ray.x, view.v[0].y + perp * ray.y);
    return vec4<f32>(rgb, clamp(perp / 28.0, 0.0, 1.0));
  }
  let p = f32(py) - view.v[1].z;
  if (abs(p) < 0.5) {
    return vec4<f32>(18.0 / 255.0, 11.0 / 255.0, 10.0 / 255.0, 1.0);
  }
  let on_floor = p > 0.0;
  let dist = (0.5 * h) / abs(p);
  let plane = view.v[1].xy;
  let dir = view.v[0].zw;
  let step = 2.0 * plane * dist / w;
  let fx = view.v[0].x + (dir.x - plane.x) * dist + step.x * (f32(x) + 0.5);
  let fy = view.v[0].y + (dir.y - plane.y) * dist + step.y * (f32(x) + 0.5);
  var id = 5;
  var u = fx;
  var v = fy;
  if (on_floor) {
    let mx = i32(floor(fx));
    let my = i32(floor(fy));
    var style = 0.0;
    if (mx >= 0 && my >= 0 && mx < 48 && my < 32) {
      style = textureLoad(floor_tex, vec2<i32>(mx, my), 0).r * 255.0;
    }
    if (style > 1.5) {
      let level = i32(view.v[3].z);
      let c = sigil_center(level);
      let mark = sigil(level, (fx - c.x) / 1.55, (fy - c.y) / 1.55);
      id = 6;
      u = fx;
      v = fy;
      var rgb = sample_atlas(id, u, v).rgb;
      rgb = mix(rgb, mark.rgb, mark.a * (1.0 - smoothstep(0.92, 1.02, length((vec2<f32>(fx, fy) - c) / 1.55))));
      let shade_k = 0.72 + 0.2 / (1.0 + dist * 0.2) + muzzle * 0.45 / (1.0 + dist * dist);
      let lit = light_at(fx, fy) * 1.35;
      rgb = clamp(rgb * clamp(vec3<f32>(shade_k) + lit, vec3<f32>(0.16), vec3<f32>(2.2)), vec3<f32>(0.0), vec3<f32>(1.0));
      rgb = through_smoke(rgb, view.v[0].x, view.v[0].y, fx, fy);
      return vec4<f32>(rgb, clamp(dist / 28.0, 0.0, 1.0));
    } else if (style > 0.5) {
      id = 6;
    }
  } else {
    id = select(7, 3, hell > 0.5);
  }
  var rgb = sample_atlas(id, u, v).rgb;
  let shade_k = select(0.78, 0.72 + 0.2 / (1.0 + dist * 0.2) + muzzle * 0.45 / (1.0 + dist * dist), on_floor);
  rgb = clamp(rgb * clamp(vec3<f32>(shade_k) + light_at(fx, fy), vec3<f32>(0.18), vec3<f32>(1.8)), vec3<f32>(0.0), vec3<f32>(1.0));
  rgb = through_smoke(rgb, view.v[0].x, view.v[0].y, fx, fy);
  return vec4<f32>(rgb, clamp(dist / 28.0, 0.0, 1.0));
}
`;

const SPRITE_WGSL = `
struct View { v: array<vec4<f32>, 4> }
@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage, read> cols: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> sprites: array<vec4<f32>>;
@group(0) @binding(3) var atlas: texture_2d_array<f32>;
@group(0) @binding(4) var atlas_samp: sampler;
@group(0) @binding(6) var light_tex: texture_2d<f32>;

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) depth: f32,
  @location(2) @interpolate(flat) info: vec4<f32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VSOut {
  let s0 = sprites[ii * 2];
  let s1 = sprites[ii * 2 + 1];
  let dir = view.v[0].zw;
  let plane = view.v[1].xy;
  let sx = s0.x - view.v[0].x;
  let sy = s0.y - view.v[0].y;
  let inv_det = 1.0 / (plane.x * dir.y - dir.x * plane.y);
  let tx = inv_det * (dir.y * sx - dir.x * sy);
  let ty_raw = inv_det * (-plane.y * sx + plane.x * sy);
  let ty = max(ty_raw, 0.05);
  let w = max(view.v[2].z, 1.0);
  let h = max(view.v[2].w, 1.0);
  let sh = abs(h / ty * max(s0.w, 0.05));
  let voff = s0.z / ty;
  let screen_x = w * 0.5 * (1.0 + tx / ty);
  let x0 = -sh * 0.5 + screen_x;
  let x1 = sh * 0.5 + screen_x;
  let y0 = -sh * 0.5 + view.v[1].z + voff;
  let y1 = sh * 0.5 + view.v[1].z + voff;
  var q = array<vec2<f32>, 6>(
    vec2<f32>(x0, y0), vec2<f32>(x1, y0), vec2<f32>(x0, y1),
    vec2<f32>(x0, y1), vec2<f32>(x1, y0), vec2<f32>(x1, y1),
  );
  var uvs = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
  );
  let p = q[vi];
  var o: VSOut;
  let clip = vec2<f32>(p.x / w * 2.0 - 1.0, 1.0 - p.y / h * 2.0);
  o.pos = vec4<f32>(select(clip, vec2<f32>(0.0), ty_raw < 0.12), 0.0, 1.0);
  o.uv = uvs[vi];
  o.depth = ty;
  o.info = s1;
  return o;
}

@fragment
fn fs(inp: VSOut) -> @location(0) vec4<f32> {
  let w = max(view.v[2].z, 1.0);
  let x = i32(clamp(inp.pos.x, 0.0, w - 1.0));
  if (inp.depth >= cols[x * 4 + 3].z) { discard; }
  let kind = i32(inp.info.w);
  let frame = inp.info.y;
  var uv = inp.uv;
  if (frame >= 0.0) {
    let fr = i32(frame);
    uv = vec2<f32>(f32(fr & 1), f32(fr >> 1)) * 0.5 + inp.uv * 0.5;
  }
  var rgb: vec3<f32>;
  var a: f32;
  if (kind == 17 && frame < 0.0) {
    let d = inp.uv * 2.0 - 1.0;
    let rad = dot(d, d);
    if (rad >= 1.0) { discard; }
    let n = sin(d.x * 11.0 + view.v[1].w) * sin(d.y * 8.0 - view.v[1].w * 0.7);
    let soft = pow(1.0 - rad, 1.6) * (0.7 + 0.2 * n);
    rgb = mix(vec3<f32>(0.28, 0.29, 0.30), vec3<f32>(0.74, 0.76, 0.78), soft);
    a = 1.0;
  } else if (kind == 21) {
    let d = inp.uv * 2.0 - 1.0;
    let rad = dot(d, d);
    if (rad >= 1.0) { discard; }
    let core = pow(1.0 - rad, 1.7);
    rgb = vec3<f32>(40.0 + core * 215.0, 170.0 + core * 85.0, 255.0) / 255.0;
    a = 1.0;
  } else {
    let tex = textureSampleLevel(atlas, atlas_samp, uv, clamp(i32(inp.info.x), 0, ${TEX_N - 1}), 0.0);
    a = tex.a;
    if (a < 16.0 / 255.0) { discard; }
    let parity = vec2<i32>(inp.pos.xy) & vec2<i32>(1);
    let coverage = (f32((parity.x ^ parity.y) * 2 + parity.y) + 0.5) / 4.0;
    if (a < coverage) { discard; }
    rgb = tex.rgb;
    if (kind == 23) { rgb = mix(rgb, vec3<f32>(1.0, 0.14, 0.09), 0.42); }
    if (inp.info.z > 1.5) { rgb = mix(rgb, vec3<f32>(1.0, 0.72, 0.24), 0.32); }
    else if (inp.info.z > 0.5) { rgb = vec3<f32>(1.0, 0.86, 0.86); }
    if (kind == 12 && inp.info.z < 0.0) { rgb *= 0.22; }
  }
  return vec4<f32>(rgb, clamp(inp.depth / 28.0, 0.0, 1.0));
}
`;

function f16(n: number): number {
  const f = new Float32Array(1);
  const u = new Uint32Array(f.buffer);
  f[0] = n;
  const x = u[0]!;
  const sign = (x >>> 16) & 0x8000;
  const exp = (x >>> 23) & 0xff;
  const mant = x & 0x7fffff;
  if (exp === 255) return sign | 0x7c00 | (mant ? 0x200 : 0);
  if (exp < 113) return sign;
  if (exp > 142) return sign | 0x7c00;
  return sign | ((exp - 112) << 10) | (mant >> 13);
}


export function createWebGpuWorld(device: GPUDevice): GpuWorld {
  const fillMod = device.createShaderModule({ code: FILL_WGSL });
  const sprMod = device.createShaderModule({ code: SPRITE_WGSL });
  const bgl = device.createBindGroupLayout({
    entries: [
      { binding: 0, visibility: GPUShaderStage.FRAGMENT | GPUShaderStage.VERTEX, buffer: { type: "uniform" } },
      { binding: 1, visibility: GPUShaderStage.FRAGMENT, buffer: { type: "read-only-storage" } },
      { binding: 2, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: "read-only-storage" } },
      { binding: 3, visibility: GPUShaderStage.FRAGMENT, texture: { viewDimension: "2d-array", sampleType: "float" } },
      { binding: 4, visibility: GPUShaderStage.FRAGMENT, sampler: {} },
      { binding: 5, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: "float" } },
      { binding: 6, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: "unfilterable-float" } },
    ],
  });
  const fillPipe = device.createRenderPipeline({
    layout: device.createPipelineLayout({ bindGroupLayouts: [bgl] }),
    vertex: { module: fillMod, entryPoint: "vs" },
    fragment: { module: fillMod, entryPoint: "fs", targets: [{ format: "rgba8unorm" }] },
    primitive: { topology: "triangle-list" },
  });
  const sprPipe = device.createRenderPipeline({
    layout: device.createPipelineLayout({ bindGroupLayouts: [bgl] }),
    vertex: { module: sprMod, entryPoint: "vs" },
    fragment: { module: sprMod, entryPoint: "fs", targets: [{ format: "rgba8unorm" }] },
    primitive: { topology: "triangle-list" },
  });
  const viewBuf = device.createBuffer({ size: 64, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
  const colBuf = device.createBuffer({ size: MAX_COLS * COL_FLOATS * 4, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
  const sprBuf = device.createBuffer({ size: 192 * SPR_FLOATS * 4, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
  const atlas = device.createTexture({
    size: { width: TEX, height: TEX, depthOrArrayLayers: TEX_N },
    format: "rgba8unorm",
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
  });
  const atlasView = atlas.createView({ dimension: "2d-array" });
  const samp = device.createSampler({ magFilter: "nearest", minFilter: "nearest", addressModeU: "repeat", addressModeV: "repeat" });
  const floorTex = device.createTexture({
    size: { width: MAP_W, height: MAP_H },
    format: "r8unorm",
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
  });
  const lightTex = device.createTexture({
    size: { width: MAP_W, height: MAP_H },
    format: "rgba16float",
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
  });
  const bind = device.createBindGroup({
    layout: bgl,
    entries: [
      { binding: 0, resource: { buffer: viewBuf } },
      { binding: 1, resource: { buffer: colBuf } },
      { binding: 2, resource: { buffer: sprBuf } },
      { binding: 3, resource: atlasView },
      { binding: 4, resource: samp },
      { binding: 5, resource: floorTex.createView() },
      { binding: 6, resource: lightTex.createView() },
    ],
  });
  let atlasReady = false;
  const floorPad = new Uint8Array(256 * MAP_H);
  const lightPad = new Uint16Array((512 / 2) * MAP_H);

  return {
    uploadAtlas(layers) {
      for (let id = 0; id < TEX_N; id++) {
        device.queue.writeTexture(
          { texture: atlas, origin: { z: id } },
          layers.subarray(id * TEX * TEX * 4, (id + 1) * TEX * TEX * 4),
          { bytesPerRow: TEX * 4, rowsPerImage: TEX },
          { width: TEX, height: TEX, depthOrArrayLayers: 1 },
        );
      }
      atlasReady = true;
    },
    draw(encoder, color, frame) {
      if (!atlasReady) return;
      device.queue.writeBuffer(viewBuf, 0, frame.view);
      device.queue.writeBuffer(colBuf, 0, frame.cols.subarray(0, frame.w * COL_FLOATS));
      if (frame.spriteCount > 0) {
        device.queue.writeBuffer(sprBuf, 0, frame.sprites.subarray(0, frame.spriteCount * SPR_FLOATS));
      }
      floorPad.fill(0);
      for (let y = 0; y < MAP_H; y++) floorPad.set(frame.floor.subarray(y * MAP_W, y * MAP_W + MAP_W), y * 256);
      device.queue.writeTexture(
        { texture: floorTex },
        floorPad,
        { bytesPerRow: 256, rowsPerImage: MAP_H },
        { width: MAP_W, height: MAP_H },
      );
      lightPad.fill(0);
      for (let y = 0; y < MAP_H; y++) {
        for (let x = 0; x < MAP_W; x++) {
          const s = (y * MAP_W + x) * 3;
          const d = y * 256 + x * 4;
          lightPad[d] = f16(frame.light[s] ?? 0);
          lightPad[d + 1] = f16(frame.light[s + 1] ?? 0);
          lightPad[d + 2] = f16(frame.light[s + 2] ?? 0);
          lightPad[d + 3] = f16(frame.smoke[y * MAP_W + x] ?? 0);
        }
      }
      device.queue.writeTexture(
        { texture: lightTex },
        lightPad,
        { bytesPerRow: 512, rowsPerImage: MAP_H },
        { width: MAP_W, height: MAP_H },
      );
      const pass = encoder.beginRenderPass({
        colorAttachments: [{ view: color, loadOp: "clear", storeOp: "store", clearValue: { r: 0.07, g: 0.04, b: 0.04, a: 1 } }],
      });
      pass.setPipeline(fillPipe);
      pass.setBindGroup(0, bind);
      pass.draw(3);
      if (frame.spriteCount > 0) {
        pass.setPipeline(sprPipe);
        pass.draw(6, frame.spriteCount);
      }
      pass.end();
    },
    dispose() {
      atlas.destroy();
      floorTex.destroy();
      lightTex.destroy();
      viewBuf.destroy();
      colBuf.destroy();
      sprBuf.destroy();
    },
  };
}

export function readWorldFrame(
  memory: ArrayBuffer,
  viewPtr: number,
  colPtr: number,
  sprPtr: number,
  spriteCount: number,
  floorPtr: number,
  lightPtr: number,
  smokePtr: number,
  w: number,
): WorldFrame {
  const view = new Float32Array(VIEW_FLOATS);
  view.set(new Float32Array(memory, viewPtr, VIEW_FLOATS));
  const cols = new Float32Array(w * COL_FLOATS);
  cols.set(new Float32Array(memory, colPtr, w * COL_FLOATS));
  const sprites = new Float32Array(Math.max(spriteCount, 1) * SPR_FLOATS);
  if (spriteCount > 0) sprites.set(new Float32Array(memory, sprPtr, spriteCount * SPR_FLOATS));
  const floor = new Uint8Array(MAP_W * MAP_H);
  floor.set(new Uint8Array(memory, floorPtr, MAP_W * MAP_H));
  const light = new Float32Array(MAP_W * MAP_H * 3);
  light.set(new Float32Array(memory, lightPtr, MAP_W * MAP_H * 3));
  const smoke = new Float32Array(MAP_W * MAP_H);
  if (smokePtr) smoke.set(new Float32Array(memory, smokePtr, MAP_W * MAP_H));
  return { w, h: view[11]!, view, cols, sprites, spriteCount, floor, light, smoke };
}

const GL_FILL_VS = `#version 300 es
const vec2 P[3] = vec2[3](vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
out vec2 vUv;
void main() {
  vec2 q = P[gl_VertexID];
  gl_Position = vec4(q, 0.0, 1.0);
  vUv = vec2(q.x * 0.5 + 0.5, 1.0 - (q.y * 0.5 + 0.5));
}
`;

const GL_FILL_FS = `#version 300 es
precision highp float;
precision highp sampler2DArray;
in vec2 vUv;
out vec4 outColor;
uniform sampler2DArray atlas;
uniform sampler2D floorTex;
uniform sampler2D lightTex;
uniform sampler2D cols;
uniform vec4 view0, view1, view2, view3;
vec4 sigilMark(int level, vec2 p) {
  float d = length(p);
  if (d > 1.02) return vec4(0.0);
  float ring = smoothstep(0.045, 0.0, abs(d - 0.72));
  if (level == 1) {
    float chev = smoothstep(0.04, 0.0, abs(abs(p.x) * 0.7 + p.y * 0.45 - 0.15));
    float bar = smoothstep(0.035, 0.0, abs(p.y + 0.35)) * step(abs(p.x), 0.55);
    return vec4(1.0, 0.42, 0.12, max(ring, max(chev, bar)));
  }
  if (level == 2) {
    float hex = abs(fract(atan(p.y, p.x) / 1.047) - 0.5);
    float edge = smoothstep(0.04, 0.0, abs(d - 0.62)) * step(hex, 0.2);
    float mote = smoothstep(0.07, 0.0, length(p - vec2(0.28, 0.1)));
    return vec4(0.35, 0.95, 0.42, max(ring, max(edge, mote)));
  }
  float slit = smoothstep(0.04, 0.0, abs(p.x)) * step(abs(p.y), 0.36);
  float iris = smoothstep(0.15, 0.0, length(vec2(p.x * 1.4, p.y)));
  return vec4(0.95, 0.62, 0.22, max(ring, max(slit, iris * 0.85)));
}
vec4 sampleAtlas(int id, vec2 uv) {
  return texture(atlas, vec3(fract(uv), float(clamp(id, 0, ${TEX_N - 1}))));
}
vec3 lightAt(vec2 w) {
  vec2 p = clamp(w - 0.5, vec2(0.0), vec2(46.0, 30.0));
  ivec2 i = ivec2(floor(p));
  vec2 f = fract(p);
  vec3 a = texelFetch(lightTex, i, 0).rgb;
  vec3 b = texelFetch(lightTex, i + ivec2(1, 0), 0).rgb;
  vec3 c = texelFetch(lightTex, i + ivec2(0, 1), 0).rgb;
  vec3 d = texelFetch(lightTex, i + ivec2(1, 1), 0).rgb;
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
void main() {
  float w = max(view2.z, 1.0);
  float h = max(view2.w, 1.0);
  int x = clamp(int(vUv.x * w), 0, int(w) - 1);
  int py = int(vUv.y * h);
  vec4 c0 = texelFetch(cols, ivec2(x, 0), 0);
  vec4 c1 = texelFetch(cols, ivec2(x, 1), 0);
  vec4 c2 = texelFetch(cols, ivec2(x, 2), 0);
  vec4 c3 = texelFetch(cols, ivec2(x, 3), 0);
  if (c3.y > 0.5 && py >= int(c1.y) && py <= int(c1.z)) {
    float ty = (float(py) - c2.x) * (256.0 / max(c1.w, 1.0));
    int tid = int(c2.y);
    if (tid == 2) ty += sin(view1.w * 7.0) * 4.0;
    else if (tid == 18) ty += view1.w * 26.0;
    else if (tid == 26) ty += view1.w * 10.0;
    else if (tid == 3) ty += sin(view1.w * 3.5) * 3.0;
    vec3 rgb = sampleAtlas(tid, vec2(c0.y / 256.0, ty / 256.0)).rgb;
    float sideMul = c2.z > 0.5 ? 0.65 : 1.0;
    float dist = (1.0 / (1.0 + c0.x * 0.12)) * sideMul + view2.y * 0.55 / (1.0 + c0.x * c0.x * 0.3);
    vec3 light = vec3(c0.z, c0.w, c1.x);
    rgb = clamp(rgb * clamp(vec3(dist) + light, vec3(0.18), vec3(1.8)), vec3(0.0), vec3(1.0));
    outColor = vec4(rgb, clamp(c0.x / 28.0, 0.0, 1.0));
    return;
  }
  float p = float(py) - view1.z;
  if (abs(p) < 0.5) { outColor = vec4(18.0/255.0, 11.0/255.0, 10.0/255.0, 1.0); return; }
  bool onFloor = p > 0.0;
  float dist = (0.5 * h) / abs(p);
  vec2 plane = view1.xy;
  vec2 dir = view0.zw;
  vec2 stepv = 2.0 * plane * dist / w;
  float fx = view0.x + (dir.x - plane.x) * dist + stepv.x * (float(x) + 0.5);
  float fy = view0.y + (dir.y - plane.y) * dist + stepv.y * (float(x) + 0.5);
  int id = 5;
  vec2 uv = vec2(fx, fy);
  if (onFloor) {
    ivec2 m = ivec2(floor(fx), floor(fy));
    float style = 0.0;
    if (m.x >= 0 && m.y >= 0 && m.x < 48 && m.y < 32) style = texelFetch(floorTex, m, 0).r * 255.0;
    if (style > 1.5) {
      int level = int(view3.z);
      vec2 c = level == 1 ? vec2(43.5, 27.5) : level == 2 ? vec2(43.5, 15.5) : vec2(40.5, 21.5);
      vec2 p = (vec2(fx, fy) - c) / 1.55;
      vec4 mark = sigilMark(level, p);
      vec3 sealed = sampleAtlas(6, vec2(fx, fy)).rgb;
      sealed = mix(sealed, mark.rgb, mark.a * (1.0 - smoothstep(0.92, 1.02, length(p))));
      float shadeK = 0.72 + 0.2 / (1.0 + dist * 0.2) + view2.y * 0.45 / (1.0 + dist * dist);
      sealed = clamp(sealed * clamp(vec3(shadeK) + lightAt(vec2(fx, fy)) * 1.35, vec3(0.16), vec3(2.2)), vec3(0.0), vec3(1.0));
      outColor = vec4(sealed, clamp(dist / 28.0, 0.0, 1.0));
      return;
    } else if (style > 0.5) id = 6;
  } else {
    id = view2.x > 0.5 ? 3 : 7;
  }
  vec3 rgb = sampleAtlas(id, uv).rgb;
  float shadeK = onFloor ? 0.72 + 0.2 / (1.0 + dist * 0.2) + view2.y * 0.45 / (1.0 + dist * dist) : 0.78;
  rgb = clamp(rgb * clamp(vec3(shadeK) + lightAt(vec2(fx, fy)), vec3(0.18), vec3(1.8)), vec3(0.0), vec3(1.0));
  outColor = vec4(rgb, clamp(dist / 28.0, 0.0, 1.0));
}
`;

function glProgram(gl: WebGL2RenderingContext, vsSrc: string, fsSrc: string): WebGLProgram | null {
  const vs = gl.createShader(gl.VERTEX_SHADER);
  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  const prog = gl.createProgram();
  if (!vs || !fs || !prog) return null;
  gl.shaderSource(vs, vsSrc);
  gl.shaderSource(fs, fsSrc);
  gl.compileShader(vs);
  gl.compileShader(fs);
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) return null;
  return prog;
}

export type GlWorld = {
  uploadAtlas(layers: Uint8Array): void;
  draw(target: WebGLTexture, frame: WorldFrame): boolean;
  dispose(): void;
};

const GL_SPR_VS = `#version 300 es
layout(location=0) in vec2 pos;
layout(location=1) in vec2 uv;
out vec2 vUv;
void main() {
  gl_Position = vec4(pos, 0.0, 1.0);
  vUv = uv;
}
`;

const GL_SPR_FS = `#version 300 es
precision highp float;
precision highp sampler2DArray;
in vec2 vUv;
out vec4 outColor;
uniform sampler2DArray atlas;
uniform sampler2D cols;
uniform float depth;
uniform float texId;
uniform float frame;
uniform float flash;
uniform float kind;
uniform float viewW;
void main() {
  int x = clamp(int(gl_FragCoord.x), 0, int(viewW) - 1);
  if (depth >= texelFetch(cols, ivec2(x, 3), 0).z) discard;
  vec2 uv = vUv;
  if (frame >= 0.0) {
    int fr = int(frame);
    uv = vec2(float(fr & 1), float(fr >> 1)) * 0.5 + vUv * 0.5;
  }
  vec3 rgb;
  if (int(kind) == 21) {
    vec2 d = vUv * 2.0 - 1.0;
    float rad = dot(d, d);
    if (rad >= 1.0) discard;
    float core = pow(1.0 - rad, 1.7);
    rgb = vec3(40.0 + core * 215.0, 170.0 + core * 85.0, 255.0) / 255.0;
  } else {
    vec4 tex = texture(atlas, vec3(uv, clamp(texId, 0.0, ${TEX_N - 1}.0)));
    if (tex.a < 16.0 / 255.0) discard;
    ivec2 parity = ivec2(gl_FragCoord.xy) & ivec2(1);
    float coverage = (float((parity.x ^ parity.y) * 2 + parity.y) + 0.5) / 4.0;
    if (tex.a < coverage) discard;
    rgb = tex.rgb;
    if (int(kind) == 23) rgb = mix(rgb, vec3(1.0, 0.14, 0.09), 0.42);
    rgb = flash > 1.5 ? mix(rgb, vec3(1.0, 0.72, 0.24), 0.32)
        : flash > 0.5 ? vec3(1.0, 0.86, 0.86) : rgb;
    if (int(kind) == 12 && flash < 0.0) rgb *= 0.22;
  }
  outColor = vec4(rgb, clamp(depth / 28.0, 0.0, 1.0));
}
`;

export function createWebGlWorld(gl: WebGL2RenderingContext): GlWorld | null {
  const prog = glProgram(gl, GL_FILL_VS, GL_FILL_FS);
  const sprProg = glProgram(gl, GL_SPR_VS, GL_SPR_FS);
  if (!prog || !sprProg) return null;
  const colsTex = gl.createTexture();
  const floorTex = gl.createTexture();
  const lightTex = gl.createTexture();
  const atlas = gl.createTexture();
  const fbo = gl.createFramebuffer();
  const sprBuf = gl.createBuffer();
  const sprVao = gl.createVertexArray();
  const fillVao = gl.createVertexArray();
  if (!colsTex || !floorTex || !lightTex || !atlas || !fbo || !sprBuf || !sprVao || !fillVao) return null;
  const colsImg = new Float32Array(MAX_COLS * 4 * 4);
  let atlasReady = false;
  gl.bindTexture(gl.TEXTURE_2D_ARRAY, atlas);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, gl.REPEAT);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, gl.REPEAT);
  gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, TEX, TEX, TEX_N, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
  return {
    uploadAtlas(layers) {
      gl.bindTexture(gl.TEXTURE_2D_ARRAY, atlas);
      for (let id = 0; id < TEX_N; id++) {
        gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, id, TEX, TEX, 1, gl.RGBA, gl.UNSIGNED_BYTE, layers.subarray(id * TEX * TEX * 4, (id + 1) * TEX * TEX * 4));
      }
      atlasReady = true;
    },
    draw(target, frame) {
      if (!atlasReady) return false;
      for (let x = 0; x < frame.w; x++) {
        for (let row = 0; row < 4; row++) {
          const s = (x * 4 + row) * 4;
          const d = (row * frame.w + x) * 4;
          colsImg[d] = frame.cols[s]!;
          colsImg[d + 1] = frame.cols[s + 1]!;
          colsImg[d + 2] = frame.cols[s + 2]!;
          colsImg[d + 3] = frame.cols[s + 3]!;
        }
      }
      gl.bindTexture(gl.TEXTURE_2D, colsTex);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, frame.w, 4, 0, gl.RGBA, gl.FLOAT, colsImg.subarray(0, frame.w * 16));
      gl.bindTexture(gl.TEXTURE_2D, floorTex);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, MAP_W, MAP_H, 0, gl.RED, gl.UNSIGNED_BYTE, frame.floor);
      const lightRgba = new Float32Array(MAP_W * MAP_H * 4);
      for (let i = 0; i < MAP_W * MAP_H; i++) {
        lightRgba[i * 4] = frame.light[i * 3] ?? 0;
        lightRgba[i * 4 + 1] = frame.light[i * 3 + 1] ?? 0;
        lightRgba[i * 4 + 2] = frame.light[i * 3 + 2] ?? 0;
        lightRgba[i * 4 + 3] = frame.smoke[i] ?? 0;
      }
      gl.bindTexture(gl.TEXTURE_2D, lightTex);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, MAP_W, MAP_H, 0, gl.RGBA, gl.FLOAT, lightRgba);
      gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
      gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, target, 0);
      if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        return false;
      }
      gl.viewport(0, 0, frame.w, frame.h);
      gl.bindVertexArray(fillVao);
      gl.useProgram(prog);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D_ARRAY, atlas);
      gl.uniform1i(gl.getUniformLocation(prog, "atlas"), 0);
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, floorTex);
      gl.uniform1i(gl.getUniformLocation(prog, "floorTex"), 1);
      gl.activeTexture(gl.TEXTURE2);
      gl.bindTexture(gl.TEXTURE_2D, lightTex);
      gl.uniform1i(gl.getUniformLocation(prog, "lightTex"), 2);
      gl.activeTexture(gl.TEXTURE3);
      gl.bindTexture(gl.TEXTURE_2D, colsTex);
      gl.uniform1i(gl.getUniformLocation(prog, "cols"), 3);
      gl.uniform4fv(gl.getUniformLocation(prog, "view0"), frame.view.subarray(0, 4));
      gl.uniform4fv(gl.getUniformLocation(prog, "view1"), frame.view.subarray(4, 8));
      gl.uniform4fv(gl.getUniformLocation(prog, "view2"), frame.view.subarray(8, 12));
      gl.uniform4fv(gl.getUniformLocation(prog, "view3"), frame.view.subarray(12, 16));
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      const v = frame.view;
      const invDet = 1 / (v[4]! * v[3]! - v[2]! * v[5]!);
      gl.bindVertexArray(sprVao);
      gl.bindBuffer(gl.ARRAY_BUFFER, sprBuf);
      gl.useProgram(sprProg);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D_ARRAY, atlas);
      gl.uniform1i(gl.getUniformLocation(sprProg, "atlas"), 0);
      gl.activeTexture(gl.TEXTURE3);
      gl.bindTexture(gl.TEXTURE_2D, colsTex);
      gl.uniform1i(gl.getUniformLocation(sprProg, "cols"), 3);
      gl.uniform1f(gl.getUniformLocation(sprProg, "viewW"), frame.w);
      gl.bindBuffer(gl.ARRAY_BUFFER, sprBuf);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 16, 0);
      gl.enableVertexAttribArray(1);
      gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 16, 8);
      const quad = new Float32Array(24);
      for (let i = 0; i < frame.spriteCount; i++) {
        const o = i * 8;
        const sx = frame.sprites[o]! - v[0]!;
        const sy = frame.sprites[o + 1]! - v[1]!;
        const tx = invDet * (v[3]! * sx - v[2]! * sy);
        const ty = invDet * (-v[5]! * sx + v[4]! * sy);
        if (ty < 0.12) continue;
        const sh = Math.abs(v[11]! / ty * Math.max(frame.sprites[o + 3]!, 0.05));
        const voff = frame.sprites[o + 2]! / ty;
        const screenX = v[10]! * 0.5 * (1 + tx / ty);
        const x0 = (-sh * 0.5 + screenX) / v[10]! * 2 - 1;
        const x1 = (sh * 0.5 + screenX) / v[10]! * 2 - 1;
        const y0 = 1 - (-sh * 0.5 + v[6]! + voff) / v[11]! * 2;
        const y1 = 1 - (sh * 0.5 + v[6]! + voff) / v[11]! * 2;
        const verts = [x0, y0, 0, 0, x1, y0, 1, 0, x0, y1, 0, 1, x0, y1, 0, 1, x1, y0, 1, 0, x1, y1, 1, 1];
        quad.set(verts);
        gl.bufferData(gl.ARRAY_BUFFER, quad, gl.DYNAMIC_DRAW);
        gl.uniform1f(gl.getUniformLocation(sprProg, "depth"), ty);
        gl.uniform1f(gl.getUniformLocation(sprProg, "texId"), frame.sprites[o + 4]!);
        gl.uniform1f(gl.getUniformLocation(sprProg, "frame"), frame.sprites[o + 5]!);
        gl.uniform1f(gl.getUniformLocation(sprProg, "flash"), frame.sprites[o + 6]!);
        gl.uniform1f(gl.getUniformLocation(sprProg, "kind"), frame.sprites[o + 7]!);
        gl.drawArrays(gl.TRIANGLES, 0, 6);
      }
      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      return true;
    },
    dispose() {
      gl.deleteProgram(prog);
      gl.deleteProgram(sprProg);
      gl.deleteBuffer(sprBuf);
      gl.deleteVertexArray(sprVao);
      gl.deleteVertexArray(fillVao);
      gl.deleteTexture(colsTex);
      gl.deleteTexture(floorTex);
      gl.deleteTexture(lightTex);
      gl.deleteTexture(atlas);
      gl.deleteFramebuffer(fbo);
    },
  };
}
