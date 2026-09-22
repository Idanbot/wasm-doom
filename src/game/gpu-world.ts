/** GPU wall/floor/sprite fill. The sim still casts columns; this only textures them. */

export const VIEW_FLOATS = 16;
export const COL_FLOATS = 16;
export const SPR_FLOATS = 8;
export const MAP_W = 48;
export const MAP_H = 32;
export const ENEMY_ANIM_COUNT = 7;
export const ENEMY_SKIN_COUNT = 13;
export const ENEMY_TEX_BASE = 29;
export const TEX_N = ENEMY_TEX_BASE + ENEMY_ANIM_COUNT * ENEMY_SKIN_COUNT;
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
      id = 28;
      u = (fx - 38.0) / 8.0;
      v = (fy - 19.0) / 9.0;
    } else if (style > 0.5) {
      id = 6;
    }
  } else {
    id = select(7, 3, hell > 0.5);
  }
  var rgb = sample_atlas(id, u, v).rgb;
  let shade_k = select(0.78, 0.72 + 0.2 / (1.0 + dist * 0.2) + muzzle * 0.45 / (1.0 + dist * dist), on_floor);
  rgb = clamp(rgb * clamp(vec3<f32>(shade_k) + light_at(fx, fy), vec3<f32>(0.18), vec3<f32>(1.8)), vec3<f32>(0.0), vec3<f32>(1.0));
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
  if (kind == 21) {
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
    rgb = tex.rgb;
    if (inp.info.z > 0.5) { rgb = vec3<f32>(1.0, 0.86, 0.86); }
    if (kind == 23) { rgb = mix(rgb, vec3<f32>(1.0, 0.14, 0.09), 0.42); }
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
  const colBuf = device.createBuffer({ size: 1920 * COL_FLOATS * 4, usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST });
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
          lightPad[d + 3] = f16(1);
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
  return { w, h: view[11]!, view, cols, sprites, spriteCount, floor, light };
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
uniform vec4 view0, view1, view2;
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
    if (style > 1.5) { id = 28; uv = vec2((fx - 38.0) / 8.0, (fy - 19.0) / 9.0); }
    else if (style > 0.5) id = 6;
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
    rgb = flash > 0.5 ? vec3(1.0, 0.86, 0.86) : tex.rgb;
    if (int(kind) == 23) rgb = mix(rgb, vec3(1.0, 0.14, 0.09), 0.42);
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
  const colsImg = new Float32Array(1920 * 4 * 4);
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
        lightRgba[i * 4 + 3] = 1;
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
