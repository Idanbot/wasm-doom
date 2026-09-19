import { DEFAULT_GFX, type GfxOpts } from "./types";

export type BlitKind = "webgpu" | "webgl2" | "canvas2d";

export type BlitFx = {
  muzzle: number;
  hurt: number;
  time: number;
};

export type Blitter = {
  kind: BlitKind;
  draw: (pixels: Uint8Array, w: number, h: number, fx?: BlitFx) => void;
  resize: (w: number, h: number) => void;
  setGfx: (g: GfxOpts) => void;
  dispose: () => void;
};

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
  _pad: f32,
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
  let uv = inp.uv;
  var c = sample_sharp(uv);


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
  const uniData = new Float32Array(12);
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
  let recoveries = 0;
  let settingUp = false;


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
        usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
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
      const dev = await adapter.requestDevice();
      if (my !== gen) {
        dev.destroy();
        return;
      }
      device = dev;
      format = gpuApi.getPreferredCanvasFormat();
      ctx = canvas.getContext("webgpu") as GPUCanvasContext | null;
      if (!ctx) return;
      configure();
      const shader = dev.createShaderModule({ code: POST_WGSL });
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
        size: 48,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
      });
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

    device.queue.writeBuffer(uniBuf, 0, uniData);
  };

  return {
    kind: "webgpu",
    dispose() {
      gen += 1;
      ready = false;
      destroyTex();
      uniBuf?.destroy();
      ctx?.unconfigure();
      device?.destroy();
    },
    setGfx(g) {
      gfx = { ...g };
    },
    resize() {
      const { changed } = syncDisplay(canvas);
      if (changed) configure();
    },
    draw(pixels, w, h, fx) {
      if (!ready || !device || !ctx || !pipeline || !bloomPipe) return;
      const { dw, dh, changed } = syncDisplay(canvas);
      if (changed) configure();
      ensureTex(w, h);
      const i = ping;
      ping ^= 1;
      if (!tex[i] || !bind[i] || !bloomView) return;
      try {
        device.queue.writeTexture(
          { texture: tex[i] },
          pixels as unknown as BufferSource,
          { bytesPerRow: w * 4, rowsPerImage: h },
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
      } catch {
        ready = false;
        if (recoveries >= 2) return;
        recoveries += 1;
        void setup();
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
precision mediump float;
uniform sampler2D t;
uniform vec2 res;
uniform vec2 display;
uniform float muzzle;
uniform float hurt;
uniform float crt;
uniform float bloom;
uniform float fog;
in vec2 v;
out vec4 o;

vec3 applyFog(vec4 c) {
  if (fog < 0.5) return c.rgb;
  float t = 1.0 - clamp(c.a, 0.0, 1.0);
  float t2 = t * t;
  return mix(vec3(0.07, 0.043, 0.039), c.rgb, t2);
}

void main(){
  vec2 uv = v;
  vec4 raw = texture(t, uv);
  vec3 rgb = applyFog(raw);
  vec2 g = uv * 2.0 - 1.0;
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
  o = vec4(rgb, 1.0);
}`;

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
  const locRes = gl.getUniformLocation(prog, "res");
  const locDisplay = gl.getUniformLocation(prog, "display");
  const locMuzzle = gl.getUniformLocation(prog, "muzzle");
  const locHurt = gl.getUniformLocation(prog, "hurt");
  const locCrt = gl.getUniformLocation(prog, "crt");
  const locBloom = gl.getUniformLocation(prog, "bloom");
  const locFog = gl.getUniformLocation(prog, "fog");

  const vao = gl.createVertexArray();
  gl.bindVertexArray(vao);
  let texW = 0;
  let texH = 0;
  let gfx: GfxOpts = { ...DEFAULT_GFX };

  return {
    kind: "webgl2",
    dispose() {
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
    draw(pixels, w, h, fx) {
      const { dw, dh, changed } = syncDisplay(canvas);
      if (changed) gl.viewport(0, 0, dw, dh);
      gl.bindTexture(gl.TEXTURE_2D, tex);
      if (texW !== w || texH !== h) {
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, w, h, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        texW = w;
        texH = h;
      } else {
        gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
      }
      gl.uniform2f(locRes, w, h);
      gl.uniform2f(locDisplay, dw, dh);
      gl.uniform1f(locMuzzle, fx?.muzzle ?? 0);
      gl.uniform1f(locHurt, fx?.hurt ?? 0);
      gl.uniform1f(locCrt, gfx.crt ? 1 : 0);
      gl.uniform1f(locBloom, gfx.bloom ? 1 : 0);
      gl.uniform1f(locFog, gfx.fog ? 1 : 0);
      gl.drawArrays(gl.TRIANGLES, 0, 3);

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
    dispose() { frame = null; off.width = off.height = 1; },
    setGfx(g) {
      gfx = { ...g };
    },
    resize() {
      syncDisplay(canvas);
    },
    draw(pixels, w, h) {
      const { dw, dh } = syncDisplay(canvas);
      if (!octx) return;
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
    },
  };
}
