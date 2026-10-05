import { TEX_N, compileShader, createWebGlWorld, createWebGpuWorld, type GpuWorld, type WorldFrame } from "./gpu-world";
import { DEFAULT_GFX, type GfxOpts } from "./types";
import { sectorEffect, type Entrance, type Shockwave } from "./sector-effects";

export type BlitKind = "webgpu" | "webgl2" | "canvas2d";

export type BlitFx = {
  muzzle: number;
  hurt: number;
  time: number;
  boss: number;
  sector?: number;
  yaw?: number;
  shocks?: Shockwave[];
  entrance?: Entrance;
};

export type Blitter = {
  kind: BlitKind;
  isReady: () => boolean;
  revision: () => number;
  draw: (pixels: Uint8Array<ArrayBuffer>, w: number, h: number, fx?: BlitFx) => boolean;
  uploadAtlas?: (layers: Uint8Array<ArrayBuffer>) => void;
  uploadAtlasLayer?: (id: number, rgba: Uint8Array<ArrayBuffer>) => void;
  drawWorld?: (frame: WorldFrame, fx?: BlitFx) => boolean;
  resize: (w: number, h: number) => void;
  setGfx: (g: GfxOpts) => void;
  dispose: () => void;
};

function writeEffectUniforms(data: Float32Array, fx?: BlitFx) {
  data.fill(0, 12);
  if (fx?.sector !== undefined) {
    const profile = sectorEffect(fx.sector);
    data.set([...profile.color, profile.kind], 12);
    data.set([0, profile.drift, profile.index + 1, fx.yaw ?? 0], 16);
  }
  if(fx?.entrance) {
    const e=fx.entrance;data.set([e.x,e.y,e.phase,e.depth],40);
    data.set([...e.color,e.style+1],44);
  }
  for (const [i, s] of (fx?.shocks ?? []).slice(0, 4).entries()) {
    data.set([s.x, s.y, s.radius, s.strength], 20 + i * 4);
    data[36 + i] = s.depth;
  }
}

export async function createBlitter(
  canvas: HTMLCanvasElement,
  opts?: { requireGpu?: boolean },
): Promise<Blitter> {
  if (navigator.gpu) {
    const gpu = await createGpuBlit(canvas);
    if (gpu) return gpu;
  }
  if (opts?.requireGpu) {
    throw new Error("WebGPU is required. Enable it in your browser or turn off Enforce WebGPU.");
  }
  const gl = canvas.getContext("webgl2", {
    antialias: false,
    alpha: false,
    depth: false,
    stencil: false,
    powerPreference: "high-performance",
    preserveDrawingBuffer: false,
  });
  if (gl) return createGlBlit(canvas, gl);
  return createCanvas2dBlit(canvas);
}

function padRows(pixels: Uint8Array<ArrayBuffer>, w: number, h: number) {
  const row = w * 4;
  const stride = Math.ceil(row / 256) * 256;
  if (stride === row) return { data: pixels, stride };
  const data = new Uint8Array(stride * h);
  for (let y = 0; y < h; y++) data.set(pixels.subarray(y * row, y * row + row), y * stride);
  return { data, stride };
}

function syncDisplay(canvas: HTMLCanvasElement) {
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const dw = Math.max(1, Math.round((canvas.clientWidth || 320) * dpr));
  const dh = Math.max(1, Math.round((canvas.clientHeight || 200) * dpr));
  const changed = canvas.width !== dw || canvas.height !== dh;
  if (changed) {
    canvas.width = dw;
    canvas.height = dh;
  }
  return { dw, dh, changed };
}

const POST_WGSL = `
struct Uni {
  res: vec2<f32>,
  display: vec2<f32>,
  muzzle: f32,
  hurt: f32,
  time: f32,
  crt: f32,
  bloom: f32,
  fog: f32,
  boss: f32,
  padding: f32,
  atmosphere: vec4<f32>,
  motion: vec4<f32>,
  shocks: array<vec4<f32>, 4>,
  shockDepth: vec4<f32>,
  entrance: vec4<f32>,
  entranceColor: vec4<f32>,
};
@group(0) @binding(0) var fb: texture_2d<f32>;
@group(0) @binding(1) var bloomTex: texture_2d<f32>;
@group(0) @binding(2) var<uniform> u: Uni;

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VSOut {
  var p = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0)
  );
  let q = p[i];
  var o: VSOut;
  o.pos = vec4<f32>(q, 0.0, 1.0);
  o.uv = vec2<f32>(q.x * 0.5 + 0.5, 1.0 - (q.y * 0.5 + 0.5));
  return o;
}

fn load_px(tex: texture_2d<f32>, p: vec2<i32>) -> vec4<f32> {
  let dim = vec2<i32>(textureDimensions(tex));
  let t = clamp(p, vec2<i32>(0), dim - vec2<i32>(1));
  return textureLoad(tex, t, 0);
}

fn sample_sharp(uv: vec2<f32>) -> vec4<f32> {
  let p = uv * u.res - 0.5;
  let i = vec2<i32>(floor(p));
  let f = p - floor(p);
  let s = f * f * (3.0 - 2.0 * f);
  let a = load_px(fb, i);
  let b = load_px(fb, i + vec2<i32>(1, 0));
  let c = load_px(fb, i + vec2<i32>(0, 1));
  let d = load_px(fb, i + vec2<i32>(1, 1));
  return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

fn sample_near(uv: vec2<f32>) -> vec4<f32> {
  return load_px(fb, vec2<i32>(uv * u.res));
}

@fragment
fn fs_bloom(inp: VSOut) -> @location(0) vec4<f32> {
  let c = sample_near(inp.uv);
  let luma = dot(c.rgb, vec3<f32>(0.26, 0.45, 0.12));
  let b = max(0.0, luma - 0.42) * 1.8;
  return vec4<f32>(c.rgb * b, 1.0);
}

@fragment
fn fs(inp: VSOut) -> @location(0) vec4<f32> {
  var uv = inp.uv;
  let aspect = u.display.x / max(u.display.y, 1.0);
  let originalDepth = sample_near(uv).a;
  for (var i = 0; i < 4; i++) {
    let shock = u.shocks[i];
    let d = (uv - shock.xy) * vec2<f32>(aspect, 1.0);
    let r = length(d);
    let edge = (r - shock.z) / 0.018;
    let band = exp(-edge * edge);
    let visible = smoothstep(u.shockDepth[i] - 0.025, u.shockDepth[i], originalDepth);
    uv += d / max(r, 0.001) / vec2<f32>(aspect, 1.0) * band * shock.w * 0.014 * visible;
  }
  uv = clamp(uv, vec2<f32>(0.001), vec2<f32>(0.999));
  var c = sample_sharp(uv);
  let depth = c.a;
  // Screen-space light march. Alpha is view depth, so a short ray along the
  // view gathers light that the column caster already ray-tested into the frame.
  var ray = vec3<f32>(0.0);
  let dir = normalize(vec2<f32>(0.5 - uv.x, 0.18 - uv.y) + vec2<f32>(0.001));
  for (var i = 1; i <= 5; i++) {
    let p = uv + dir * (f32(i) * (0.0035 + depth * 0.006));
    let s = sample_near(p);
    let luma = dot(s.rgb, vec3<f32>(0.26, 0.45, 0.12));
    let open = smoothstep(0.12, 0.0, s.a - depth);
    ray += s.rgb * max(0.0, luma - 0.32) * open * 0.18;
  }
  c = vec4<f32>(c.rgb + ray, depth);
  // Fine floating motes and layered haze pick up actual scene light and muzzle illumination.
  let q = inp.uv * vec2<f32>(aspect, 1.0);
  let seed = u.motion.z;
  let drift = u.motion.y;
  let kind = u.atmosphere.w;
  let p = q * 38.0 + vec2<f32>(u.motion.w * 3.0, u.time * drift);
  let cell = floor(p);
  let rnd = fract(sin(dot(cell + seed, vec2<f32>(127.1, 311.7))) * 43758.5453);
  let point = vec2<f32>(rnd, fract(rnd * 17.23));
  let delta = fract(p) - point;
  let shape = select(dot(delta, delta), delta.x * delta.x * 2.0 + delta.y * delta.y * 0.28, kind == 6.0);
  var mote = exp(-shape * select(850.0, 220.0, kind == 2.0));
  if (kind == 1.0) { mote = exp(-(abs(delta.x) + abs(delta.y)) * 55.0); }
  if (kind == 7.0) { mote = exp(-min(abs(delta.x), abs(delta.y)) * 140.0) * exp(-dot(delta, delta) * 150.0); }
  let flicker = sin(u.time * (1.2 + drift) + rnd * 6.28);
  let twinkle = 0.45 + 0.55 * flicker * flicker;
  let haze = pow(max(0.0, sin(q.x * (8.0 + seed * 0.13) + sin(q.y * 13.0 + u.time * drift) + u.motion.w)), 6.0);
  let light = clamp(dot(c.rgb, vec3<f32>(0.3, 0.5, 0.2)) * 1.8 + u.muzzle * 1.7, 0.04, 1.5);
  let depthMask = smoothstep(0.015, 0.12, depth);
  let scan = select(1.0, pow(abs(sin(q.y * 55.0 + u.time * drift * 3.0)), 12.0), kind == 3.0 || kind == 7.0);
  let ion = select(1.0, 0.45 + 0.55 * abs(sin(q.x * 16.0 - u.time * drift * 2.0)), kind == 4.0 || kind == 5.0);
  c = vec4<f32>(c.rgb + u.atmosphere.rgb * (mote * twinkle * scan * ion * 0.28 + haze * 0.025) * light * depthMask, depth);
  if (u.entranceColor.w > 0.0) {
    let ep = (inp.uv-u.entrance.xy)*vec2<f32>(aspect,1.0);
    let radius=length(ep);let phase=u.entrance.z;
    let angle=atan2(ep.y,ep.x);
    let rays=3.0+u.entranceColor.w%6.0;
    let jag=sin(radius*95.0+u.time*15.0+u.entranceColor.w)*0.13;
    let beam=pow(abs(cos((angle+phase*(1.1+u.entranceColor.w*.08)+jag)*rays*.5)),90.0);
    let ring=exp(-abs(radius-(1.0-phase)*.32)*90.0);
    let visible=smoothstep(u.entrance.w-.025,u.entrance.w,depth);
    let fade=sin(phase*3.14159);
    let energy=(beam*.24+ring*.55)*exp(-radius*3.0)*fade*visible;
    c=vec4<f32>(c.rgb+u.entranceColor.rgb*energy,depth);
  }



  if (u.fog > 0.5) {
    let t = 1.0 - clamp(c.a, 0.0, 1.0);
    let t2 = t * t;
    let fogc = vec3<f32>(0.07, 0.043, 0.039);
    c = vec4<f32>(mix(fogc, c.rgb, t2), 1.0);
  } else {
    c.a = 1.0;
  }

  let g = uv * 2.0 - 1.0;
  c = vec4<f32>(c.rgb * (1.0 - 0.22 * dot(g, g)), 1.0);

  if (u.hurt > 0.04) {
    let edge = pow(max(abs(g.x), abs(g.y)), 2.2);
    let f = u.hurt * 0.38 * smoothstep(0.32, 1.0, edge);
    c = vec4<f32>(mix(c.rgb, vec3<f32>(0.55, 0.05, 0.05), f), 1.0);
  }

  if (u.muzzle > 0.05) {
    let muv = g - vec2<f32>(0.2, 0.44);
    let core = exp(-length(muv) * 4.0) * u.muzzle * 0.62;
    let fill = exp(-length(g) * 5.5) * u.muzzle * 0.14;
    c = vec4<f32>(c.rgb + vec3<f32>(1.0, 0.74, 0.32) * (core + fill), 1.0);
  }

  if (u.bloom > 0.5) {
    let bp = vec2<i32>(uv * vec2<f32>(textureDimensions(bloomTex)));
    var bl = load_px(bloomTex, bp).rgb * 0.25;
    bl += (load_px(bloomTex, bp + vec2<i32>(1, 0)).rgb
         + load_px(bloomTex, bp - vec2<i32>(1, 0)).rgb
         + load_px(bloomTex, bp + vec2<i32>(0, 1)).rgb
         + load_px(bloomTex, bp - vec2<i32>(0, 1)).rgb) * 0.125;
    bl += (load_px(bloomTex, bp + vec2<i32>(2, 2)).rgb
         + load_px(bloomTex, bp - vec2<i32>(2, 2)).rgb
         + load_px(bloomTex, bp + vec2<i32>(2, -2)).rgb
         + load_px(bloomTex, bp + vec2<i32>(-2, 2)).rgb) * 0.0625;
    c = vec4<f32>(c.rgb + bl * 0.42, 1.0);
  }

  if (u.crt > 0.5) {
    let scan = 1.0 - 0.14 * abs(sin(uv.y * u.res.y * 3.14159265));
    c = vec4<f32>(c.rgb * scan, 1.0);
    let tri = fract((inp.uv.x * u.display.x) / 3.0);
    let mask = vec3<f32>(
      select(0.78, 1.0, tri < 0.33),
      select(0.78, 1.0, tri >= 0.33 && tri < 0.66),
      select(0.78, 1.0, tri >= 0.66)
    );
    c = vec4<f32>(c.rgb * mask, 1.0);
  }

  return vec4<f32>(c.rgb, 1.0);
}
`;

async function createGpuBlit(canvas: HTMLCanvasElement): Promise<Blitter | null> {
  const gpuApi = (navigator as Navigator).gpu;
  if (!gpuApi) return null;


  let gfx: GfxOpts = { ...DEFAULT_GFX };
  let device: GPUDevice | null = null;
  let ctx: GPUCanvasContext | null = null;
  let format: GPUTextureFormat = "bgra8unorm";
  let pipeline: GPURenderPipeline | null = null;
  let bloomPipe: GPURenderPipeline | null = null;
  let uniBuf: GPUBuffer | null = null;
  const uniData = new Float32Array(48);
  let tex: GPUTexture[] = [];
  let view: GPUTextureView[] = [];
  let bloomTex: GPUTexture | null = null;
  let bloomView: GPUTextureView | null = null;
  let bind: GPUBindGroup[] = [];
  let bloomBind: GPUBindGroup[] = [];
  let texW = 0;
  let texH = 0;
  let ping = 0;
  let ready = false;
  let gen = 0;
  let atlasGen = -1;
  let recoveries = 0;
  let settingUp = false;
  let world: GpuWorld | null = null;


  const configure = () => {
    if (!device || !ctx) return;
    ctx.configure({
      device,
      format,
      alphaMode: "opaque",
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });
  };

  const destroyTex = () => {
    for (const t of tex) t.destroy();
    tex = [];
    view = [];
    bind = [];
    bloomBind = [];
    bloomTex?.destroy();
    bloomTex = null;
    bloomView = null;
    texW = 0;
    texH = 0;
  };

  const ensureTex = (w: number, h: number) => {
    if (!device || !pipeline || !uniBuf) return;
    if (tex.length === 2 && texW === w && texH === h && bloomTex) return;
    destroyTex();
    for (let i = 0; i < 2; i++) {
      const t = device.createTexture({
        size: { width: w, height: h },
        format: "rgba8unorm",
        usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT,
      });
      tex.push(t);
      view.push(t.createView());
    }
    const bw = Math.max(1, w >> 1);
    const bh = Math.max(1, h >> 1);
    bloomTex = device.createTexture({
      size: { width: bw, height: bh },
      format: "rgba8unorm",
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    });
    bloomView = bloomTex.createView();
    for (let i = 0; i < 2; i++) {
      // The extraction pass must not sample the bloom attachment it writes.
      bloomBind.push(device.createBindGroup({
        layout: pipeline.getBindGroupLayout(0),
        entries: [
          { binding: 0, resource: view[i] },
          { binding: 1, resource: view[i] },
          { binding: 2, resource: { buffer: uniBuf } },
        ],
      }));
      bind.push(
        device.createBindGroup({
          layout: pipeline.getBindGroupLayout(0),
          entries: [
            { binding: 0, resource: view[i] },
            { binding: 1, resource: bloomView },
            { binding: 2, resource: { buffer: uniBuf } },
          ],
        }),
      );
    }
    texW = w;
    texH = h;
  };

  const setup = async () => {
    if (settingUp) return;
    settingUp = true;
    const my = ++gen;
    ready = false;
    destroyTex();
    uniBuf?.destroy();
    device?.destroy();
    try {
      const adapter = await gpuApi.requestAdapter({ powerPreference: "high-performance" });
      if (!adapter || my !== gen) return;
      if (adapter.limits.maxTextureArrayLayers < TEX_N) throw new Error("GPU atlas layer limit is too small");
      const dev = await adapter.requestDevice({ requiredLimits: { maxTextureArrayLayers: TEX_N } });
      if (my !== gen) {
        dev.destroy();
        return;
      }
      device = dev;
      dev.addEventListener("uncapturederror", () => {
        if (my === gen) ready = false;
      });
      format = gpuApi.getPreferredCanvasFormat();
      ctx = canvas.getContext("webgpu") as GPUCanvasContext | null;
      if (!ctx) return;
      configure();
      const shader = await compileShader(dev, POST_WGSL, "present");
      const layout: GPUBindGroupLayout = dev.createBindGroupLayout({
        entries: [
          { binding: 0, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: "float" } },
          { binding: 1, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: "float" } },
          { binding: 2, visibility: GPUShaderStage.FRAGMENT, buffer: { type: "uniform" } },
        ],
      });
      const pipeLayout = dev.createPipelineLayout({ bindGroupLayouts: [layout] });
      pipeline = dev.createRenderPipeline({
        layout: pipeLayout,
        vertex: { module: shader, entryPoint: "vs" },
        fragment: { module: shader, entryPoint: "fs", targets: [{ format }] },
        primitive: { topology: "triangle-list" },
      });
      bloomPipe = dev.createRenderPipeline({
        layout: pipeLayout,
        vertex: { module: shader, entryPoint: "vs" },
        fragment: {
          module: shader,
          entryPoint: "fs_bloom",
          targets: [{ format: "rgba8unorm" }],
        },
        primitive: { topology: "triangle-list" },
      });
      uniBuf = dev.createBuffer({
        size: 192,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
      });
      world?.dispose();
      try { world = await createWebGpuWorld(dev); } catch { world = null; }
      ready = true;
      void dev.lost.then(() => {
        if (my !== gen) return;
        ready = false;
        if (recoveries >= 2) return;
        recoveries += 1;
        void setup();
      });
    } catch {
      ready = false;
    } finally {
      settingUp = false;
    }
  };

  await setup();
  if (!ready) return null;

  const writeUni = (w: number, h: number, dw: number, dh: number, fx?: BlitFx) => {
    if (!device || !uniBuf) return;
    uniData[0] = w;
    uniData[1] = h;
    uniData[2] = dw;
    uniData[3] = dh;
    uniData[4] = fx?.muzzle ?? 0;
    uniData[5] = fx?.hurt ?? 0;
    uniData[6] = fx?.time ?? 0;
    uniData[7] = gfx.crt ? 1 : 0;
    uniData[8] = gfx.bloom ? 1 : 0;
    uniData[9] = gfx.fog ? 1 : 0;
    uniData[10] = fx?.boss ?? -1;
    writeEffectUniforms(uniData, fx);

    device.queue.writeBuffer(uniBuf, 0, uniData);
  };

  return {
    kind: "webgpu",
    isReady: () => ready,
    revision: () => gen,
    dispose() {
      gen += 1;
      ready = false;
      world?.dispose();
      world = null;
      destroyTex();
      uniBuf?.destroy();
      ctx?.unconfigure();
      device?.destroy();
    },
    setGfx(g) {
      gfx = { ...g };
    },
    resize(w: number, h: number) {
      const { changed } = syncDisplay(canvas);
      if (changed) configure();
      ensureTex(w, h);
    },
    uploadAtlas(layers) {
      world?.uploadAtlas(layers);
      atlasGen = gen;
    },
    uploadAtlasLayer(id, rgba) {
      world?.uploadAtlasLayer(id, rgba);
    },
    drawWorld(frame, fx) {
      if (!ready || !device || !ctx || !pipeline || !bloomPipe || !world || atlasGen !== gen) return false;
      const { dw, dh, changed } = syncDisplay(canvas);
      if (changed) configure();
      ensureTex(frame.w, frame.h);
      const i = ping;
      ping ^= 1;
      if (!tex[i] || !view[i] || !bind[i] || !bloomView) return false;
      try {
        const encoder = device.createCommandEncoder();
        world.draw(encoder, view[i]!, frame);
        writeUni(frame.w, frame.h, dw, dh, fx);
        if (gfx.bloom) {
          const bp = encoder.beginRenderPass({
            colorAttachments: [{ view: bloomView, loadOp: "clear", storeOp: "store", clearValue: { r: 0, g: 0, b: 0, a: 1 } }],
          });
          bp.setPipeline(bloomPipe);
          bp.setBindGroup(0, bloomBind[i]!);
          bp.draw(3);
          bp.end();
        }
        const viewOut = ctx.getCurrentTexture().createView();
        const pass = encoder.beginRenderPass({
          colorAttachments: [{ view: viewOut, loadOp: "clear", storeOp: "store", clearValue: { r: 0.04, g: 0.03, b: 0.03, a: 1 } }],
        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bind[i]!);
        pass.draw(3);
        pass.end();
        device.queue.submit([encoder.finish()]);
        return true;
      } catch {
        return false;
      }
    },
    draw(pixels, w, h, fx) {
      if (!ready || !device || !ctx || !pipeline || !bloomPipe) return false;
      const { dw, dh, changed } = syncDisplay(canvas);
      if (changed) configure();
      ensureTex(w, h);
      const i = ping;
      ping ^= 1;
      if (!tex[i] || !bind[i] || !bloomView) return false;
      try {
        const upload = padRows(pixels, w, h);
        device.queue.writeTexture(
          { texture: tex[i] },
          upload.data,
          { bytesPerRow: upload.stride, rowsPerImage: h },
          { width: w, height: h },
        );
        writeUni(w, h, dw, dh, fx);
        const encoder = device.createCommandEncoder();
        if (gfx.bloom) {
          const bp = encoder.beginRenderPass({
            colorAttachments: [
              {
                view: bloomView,
                loadOp: "clear",
                storeOp: "store",
                clearValue: { r: 0, g: 0, b: 0, a: 1 },
              },
            ],
          });
          bp.setPipeline(bloomPipe);
          bp.setBindGroup(0, bloomBind[i]);
          bp.draw(3);
          bp.end();
        }
        const viewOut = ctx.getCurrentTexture().createView();
        const pass = encoder.beginRenderPass({
          colorAttachments: [
            {
              view: viewOut,
              loadOp: "clear",
              storeOp: "store",
              clearValue: { r: 0.04, g: 0.03, b: 0.03, a: 1 },
            },
          ],
        });
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bind[i]);
        pass.draw(3);
        pass.end();
        device.queue.submit([encoder.finish()]);
        return true;
      } catch {
        ready = false;
        if (recoveries >= 2) return false;
        recoveries += 1;
        void setup();
        return false;
      }
    },
  };
}

const GL_VS = `#version 300 es
const vec2 P[3] = vec2[3](vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
out vec2 v;
void main(){
  vec2 p = P[gl_VertexID];
  gl_Position = vec4(p,0.0,1.0);
  v = vec2(p.x * 0.5 + 0.5, 1.0 - (p.y * 0.5 + 0.5));
}`;

const GL_FS = `#version 300 es
precision highp float;
uniform sampler2D t;
uniform bool worldTexture;
uniform vec2 res;
uniform vec2 display;
uniform float muzzle;
uniform float hurt;
uniform float crt;
uniform float bloom;
uniform float fog;
uniform float boss;
uniform float time;
uniform vec4 atmosphere;
uniform vec4 motion;
uniform vec4 shocks[4];
uniform vec4 shockDepth;
uniform vec4 entrance;
uniform vec4 entranceColor;
in vec2 v;
out vec4 o;

vec3 applyFog(vec4 c) {
  if (fog < 0.5) return c.rgb;
  float t = 1.0 - clamp(c.a, 0.0, 1.0);
  float t2 = t * t;
  return mix(vec3(0.07, 0.043, 0.039), c.rgb, t2);
}

void main(){
  vec2 uv = worldTexture ? vec2(v.x, 1.0 - v.y) : v;
  float aspect = display.x / max(display.y, 1.0);
  float originalDepth = texture(t, uv).a;
  vec2 offset = vec2(0.0);
  for (int i=0; i<4; i++) {
    vec2 d = (v-shocks[i].xy)*vec2(aspect,1.0);
    float r = length(d);
    float edge = (r-shocks[i].z)/0.018;
    float band = exp(-edge*edge);
    float visible = smoothstep(shockDepth[i]-0.025,shockDepth[i],originalDepth);
    offset += d/max(r,0.001)/vec2(aspect,1.0)*band*shocks[i].w*0.014*visible;
  }
  uv = clamp(uv + offset * vec2(1.0, worldTexture ? -1.0 : 1.0), vec2(0.001), vec2(0.999));
  vec4 raw = texture(t, uv);
  vec3 rgb = applyFog(raw);
  vec2 g = v * 2.0 - 1.0;
  rgb *= 1.0 - 0.22 * dot(g, g);

  if (hurt > 0.04) {
    float edge = pow(max(abs(g.x), abs(g.y)), 2.2);
    float f = hurt * 0.38 * smoothstep(0.32, 1.0, edge);
    rgb = mix(rgb, vec3(0.55, 0.05, 0.05), f);
  }
  if (muzzle > 0.05) {
    vec2 muv = g - vec2(0.2, 0.44);
    float core = exp(-length(muv) * 4.0) * muzzle * 0.62;
    float fill = exp(-length(g) * 5.5) * muzzle * 0.14;
    rgb += vec3(1.0, 0.74, 0.32) * (core + fill);
  }
  if (bloom > 0.5) {
    vec3 acc = vec3(0.0);
    vec2 px = 1.0 / res;
    acc += texture(t, uv + px * vec2(1.0, 0.0)).rgb;
    acc += texture(t, uv + px * vec2(-1.0, 0.0)).rgb;
    acc += texture(t, uv + px * vec2(0.0, 1.0)).rgb;
    acc += texture(t, uv + px * vec2(0.0, -1.0)).rgb;
    float luma = dot(raw.rgb, vec3(0.26, 0.45, 0.12));
    rgb += acc * 0.08 * max(0.0, luma - 0.35);
  }
  if (crt > 0.5) {
    rgb *= 1.0 - 0.14 * abs(sin(uv.y * res.y * 3.14159265));
  }
  vec2 dir = normalize(vec2(0.5 - uv.x, 0.18 - uv.y) + vec2(0.001));
  vec3 ray = vec3(0.0);
  for (int i = 1; i <= 4; i++) {
    vec2 p = uv + dir * (float(i) * (0.0035 + raw.a * 0.006));
    vec3 s = texture(t, p).rgb;
    float luma = dot(s, vec3(0.26, 0.45, 0.12));
    ray += s * max(0.0, luma - 0.32) * 0.16;
  }
  rgb += ray;
  vec2 q = v * vec2(aspect, 1.0);
  vec2 p = q*38.0 + vec2(motion.w*3.0,time*motion.y);
  vec2 cell = floor(p);
  float rnd = fract(sin(dot(cell+motion.z,vec2(127.1,311.7)))*43758.5453);
  vec2 delta = fract(p)-vec2(rnd,fract(rnd*17.23));
  float shape = atmosphere.w==6.0?delta.x*delta.x*2.0+delta.y*delta.y*.28:dot(delta,delta);
  float mote = exp(-shape*(atmosphere.w==2.0?220.0:850.0));
  if (atmosphere.w==1.0) mote=exp(-(abs(delta.x)+abs(delta.y))*55.0);
  if (atmosphere.w==7.0) mote=exp(-min(abs(delta.x),abs(delta.y))*140.0)*exp(-dot(delta,delta)*150.0);
  float flicker = sin(time*(1.2+motion.y)+rnd*6.28);
  float haze = pow(max(0.0,sin(q.x*(8.0+motion.z*.13)+sin(q.y*13.0+time*motion.y)+motion.w)),6.0);
  float light = clamp(dot(raw.rgb,vec3(.3,.5,.2))*1.8+muzzle*1.7,.04,1.5);
  float scan = atmosphere.w==3.0||atmosphere.w==7.0?pow(abs(sin(q.y*55.0+time*motion.y*3.0)),12.0):1.0;
  float ion = atmosphere.w==4.0||atmosphere.w==5.0?.45+.55*abs(sin(q.x*16.0-time*motion.y*2.0)):1.0;
  rgb += atmosphere.rgb*(mote*(.45+.55*flicker*flicker)*scan*ion*.28+haze*.025)*light*smoothstep(.015,.12,raw.a);
  if(entranceColor.w>0.0) {
    vec2 ep=(v-entrance.xy)*vec2(aspect,1.0);
    float radius=length(ep),phase=entrance.z;
    float angle=atan(ep.y,ep.x),rays=3.0+mod(entranceColor.w,6.0);
    float jag=sin(radius*95.0+time*15.0+entranceColor.w)*.13;
    float beam=pow(abs(cos((angle+phase*(1.1+entranceColor.w*.08)+jag)*rays*.5)),90.0);
    float ring=exp(-abs(radius-(1.0-phase)*.32)*90.0);
    float visible=smoothstep(entrance.w-.025,entrance.w,raw.a);
    rgb+=entranceColor.rgb*(beam*.24+ring*.55)*exp(-radius*3.0)*sin(phase*3.14159)*visible;
  }
  o = vec4(rgb, 1.0);
}
`;

function createGlBlit(canvas: HTMLCanvasElement, gl: WebGL2RenderingContext): Blitter {
  const vs = gl.createShader(gl.VERTEX_SHADER);
  const fs = gl.createShader(gl.FRAGMENT_SHADER);
  if (!vs || !fs) return createCanvas2dBlit(canvas);
  gl.shaderSource(vs, GL_VS);
  gl.shaderSource(fs, GL_FS);
  gl.compileShader(vs);
  gl.compileShader(fs);
  const prog = gl.createProgram();
  if (!prog) return createCanvas2dBlit(canvas);
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    return createCanvas2dBlit(canvas);
  }
  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 4);
  gl.useProgram(prog);
  const locWorldTexture = gl.getUniformLocation(prog, "worldTexture");
  const locRes = gl.getUniformLocation(prog, "res");
  const locDisplay = gl.getUniformLocation(prog, "display");
  const locMuzzle = gl.getUniformLocation(prog, "muzzle");
  const locHurt = gl.getUniformLocation(prog, "hurt");
  const locCrt = gl.getUniformLocation(prog, "crt");
  const locBloom = gl.getUniformLocation(prog, "bloom");
  const locFog = gl.getUniformLocation(prog, "fog");
  const locBoss = gl.getUniformLocation(prog, "boss");
  const locTime = gl.getUniformLocation(prog, "time");
  const locAtmosphere = gl.getUniformLocation(prog, "atmosphere");
  const locMotion = gl.getUniformLocation(prog, "motion");
  const locShocks = gl.getUniformLocation(prog, "shocks[0]");
  const locEntrance=gl.getUniformLocation(prog,"entrance");
  const locEntranceColor=gl.getUniformLocation(prog,"entranceColor");
  const locShockDepth = gl.getUniformLocation(prog, "shockDepth");
  const effectData = new Float32Array(48);
  const uploadEffects = (fx?: BlitFx) => {
    writeEffectUniforms(effectData, fx);
    gl.uniform4fv(locAtmosphere, effectData.subarray(12, 16));
    gl.uniform4fv(locMotion, effectData.subarray(16, 20));
    gl.uniform4fv(locShocks, effectData.subarray(20, 36));
    gl.uniform4fv(locEntrance,effectData.subarray(40,44));
    gl.uniform4fv(locEntranceColor,effectData.subarray(44,48));
    gl.uniform4fv(locShockDepth, effectData.subarray(36, 40));
  };

  const vao = gl.createVertexArray();
  gl.bindVertexArray(vao);
  let texW = 0;
  let texH = 0;
  let gfx: GfxOpts = { ...DEFAULT_GFX };
  const world = createWebGlWorld(gl);

  return {
    kind: "webgl2",
    isReady: () => !gl.isContextLost(),
    revision: () => 0,
    dispose() {
      world?.dispose();
      gl.deleteTexture(tex);
      gl.deleteVertexArray(vao);
      gl.deleteProgram(prog);
      gl.deleteShader(vs);
      gl.deleteShader(fs);
    },
    setGfx(g) {
      gfx = { ...g };
    },
    resize() {
      const { dw, dh, changed } = syncDisplay(canvas);
      if (changed) gl.viewport(0, 0, dw, dh);
    },
    uploadAtlas(layers) {
      world?.uploadAtlas(layers);
    },
    uploadAtlasLayer(id, rgba) {
      world?.uploadAtlasLayer(id, rgba);
    },
    drawWorld(frame, fx) {
      if (!world || gl.isContextLost()) return false;
      const { dw, dh, changed } = syncDisplay(canvas);
      if (changed) gl.viewport(0, 0, dw, dh);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, tex);
      if (texW !== frame.w || texH !== frame.h) {
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, frame.w, frame.h, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
        texW = frame.w;
        texH = frame.h;
      }
      if (!world.draw(tex, frame)) return false;
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.useProgram(prog);
      // FBO textures use bottom-left rows; CPU uploads use top-left rows.
      gl.uniform1i(locWorldTexture, 1);
      gl.uniform2f(locRes, frame.w, frame.h);
      gl.uniform2f(locDisplay, dw, dh);
      gl.uniform1f(locMuzzle, fx?.muzzle ?? 0);
      gl.uniform1f(locHurt, fx?.hurt ?? 0);
      gl.uniform1f(locCrt, gfx.crt ? 1 : 0);
      gl.uniform1f(locBloom, gfx.bloom ? 1 : 0);
      gl.uniform1f(locFog, gfx.fog ? 1 : 0);
      gl.uniform1f(locBoss, fx?.boss ?? -1);
      gl.uniform1f(locTime, fx?.time ?? 0);
      uploadEffects(fx);
      gl.bindVertexArray(vao);
      gl.viewport(0, 0, dw, dh);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      return true;
    },
    draw(pixels, w, h, fx) {
      if (gl.isContextLost()) return false;
      const { dw, dh, changed } = syncDisplay(canvas);
      if (changed) gl.viewport(0, 0, dw, dh);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, tex);
      if (texW !== w || texH !== h) {
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        texW = w;
        texH = h;
      } else {
        gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      }
      gl.useProgram(prog);
      gl.bindVertexArray(vao);
      gl.uniform1i(locWorldTexture, 0);
      gl.uniform2f(locRes, w, h);
      gl.uniform2f(locDisplay, dw, dh);
      gl.uniform1f(locMuzzle, fx?.muzzle ?? 0);
      gl.uniform1f(locHurt, fx?.hurt ?? 0);
      gl.uniform1f(locCrt, gfx.crt ? 1 : 0);
      gl.uniform1f(locBloom, gfx.bloom ? 1 : 0);
      gl.uniform1f(locFog, gfx.fog ? 1 : 0);
      gl.uniform1f(locBoss, fx?.boss ?? -1);
      gl.uniform1f(locTime, fx?.time ?? 0);
      uploadEffects(fx);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      return !gl.isContextLost();

    },
  };
}

function createCanvas2dBlit(canvas: HTMLCanvasElement): Blitter {
  const ctx = canvas.getContext("2d", { alpha: false });
  if (!ctx) {
    throw new Error("Unable to create a graphics context. Reload to try again.");
  }
  const off = document.createElement("canvas");
  const octx = off.getContext("2d", { willReadFrequently: true });
  ctx.imageSmoothingEnabled = false;
  let gfx: GfxOpts = { ...DEFAULT_GFX };
  let frame: ImageData | null = null;
  return {
    kind: "canvas2d",
    isReady: () => !!octx,
    revision: () => 0,
    dispose() { frame = null; off.width = off.height = 1; },
    setGfx(g) {
      gfx = { ...g };
    },
    resize() {
      syncDisplay(canvas);
    },
    draw(pixels, w, h, fx) {
      const { dw, dh } = syncDisplay(canvas);
      if (!octx) return false;
      if (off.width !== w || off.height !== h) {
        off.width = w;
        off.height = h;
      }
      if (!frame || frame.width !== w || frame.height !== h) frame = new ImageData(w, h);
      const data = frame.data;
      // Engine alpha stores depth, not opacity. Decode it before compositing.
      for (let i = 0; i < pixels.length; i += 4) {
        const visibility = gfx.fog ? (1 - pixels[i + 3] / 255) ** 2 : 1;
        data[i] = 18 + (pixels[i] - 18) * visibility;
        data[i + 1] = 11 + (pixels[i + 1] - 11) * visibility;
        data[i + 2] = 10 + (pixels[i + 2] - 10) * visibility;
        data[i + 3] = 255;
      }
      octx.putImageData(frame, 0, 0);
      ctx.fillStyle = "#0a0908";
      ctx.fillRect(0, 0, dw, dh);
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(off, 0, 0, dw, dh);
      // CPU fallback refracts the same frozen source, never the UI or previously warped tiles.
      for (const shock of fx?.shocks ?? []) {
        const tile = Math.max(12, Math.ceil(dh / 45));
        for (let y = 0; y < dh; y += tile) for (let x = 0; x < dw; x += tile) {
          const dx = (x + tile * .5 - shock.x * dw) / dh;
          const dy = (y + tile * .5 - shock.y * dh) / dh;
          const r = Math.hypot(dx, dy);
          const band = Math.exp(-(((r - shock.radius) / .018) ** 2));
          const px = Math.min(w - 1, Math.floor(x / dw * w));
          const py = Math.min(h - 1, Math.floor(y / dh * h));
          if (band < .03 || pixels[(py * w + px) * 4 + 3]! / 255 < shock.depth - .025) continue;
          const amount = band * shock.strength * .014;
          const sx = Math.max(0, Math.min(w - 1, x / dw * w + dx / Math.max(r, .001) * amount * w));
          const sy = Math.max(0, Math.min(h - 1, y / dh * h + dy / Math.max(r, .001) * amount * h));
          ctx.drawImage(off, sx, sy, Math.min(tile / dw * w, w - sx), Math.min(tile / dh * h, h - sy), x, y, tile, tile);
        }
      }
      if (fx?.sector !== undefined) {
        const profile = sectorEffect(fx.sector);
        ctx.save(); ctx.globalCompositeOperation = "screen";
        for (let i = 0; i < 110; i++) {
          const seed = Math.sin(i * 127.1 + profile.index * 311.7) * 43758.5453;
          const rand = seed - Math.floor(seed);
          const x = ((rand + (fx.yaw ?? 0) * .04) % 1 + 1) % 1 * dw;
          const y = ((rand * 17.23 + fx.time * profile.drift * .026) % 1) * dh;
          const px = Math.min(w - 1, Math.floor(x / dw * w)), py = Math.min(h - 1, Math.floor(y / dh * h));
          const k = (py * w + px) * 4;
          if (pixels[k + 3]! < 5) continue;
          const light = Math.min(.3, (pixels[k]! + pixels[k + 1]! + pixels[k + 2]!) / 2550 + fx.muzzle * .2);
          ctx.fillStyle = `rgba(${profile.color.map(c => Math.round(c * 255)).join(",")},${light})`;
          ctx.beginPath(); ctx.arc(x, y, Math.max(.6, dh / 850), 0, Math.PI * 2); ctx.fill();
        }
        ctx.restore();
      }
      return true;
    },
  };
}
