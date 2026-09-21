interface GPU {
  requestAdapter(opts?: { powerPreference?: string }): Promise<GPUAdapter | null>;
  getPreferredCanvasFormat(): GPUTextureFormat;
}

interface GPUAdapter {
  requestDevice(): Promise<GPUDevice>;
}

type GPUTextureFormat = string;

interface GPUDevice {
  createShaderModule(desc: { code: string }): GPUShaderModule;
  createRenderPipeline(desc: Record<string, unknown>): GPURenderPipeline;
  createBuffer(desc: { size: number; usage: number }): GPUBuffer;
  createTexture(desc: Record<string, unknown>): GPUTexture;
  createSampler(desc: Record<string, unknown>): GPUSampler;
  createBindGroup(desc: Record<string, unknown>): GPUBindGroup;
  createBindGroupLayout(desc: Record<string, unknown>): GPUBindGroupLayout;
  createPipelineLayout(desc: Record<string, unknown>): GPUPipelineLayout;
  createCommandEncoder(): GPUCommandEncoder;
  queue: GPUQueue;
  lost: Promise<unknown>;
  destroy(): void;
}

interface GPUSampler {}

interface GPUShaderModule {}
interface GPURenderPipeline {
  getBindGroupLayout(i: number): GPUBindGroupLayout;
}
interface GPUBindGroupLayout {}
interface GPUPipelineLayout {}
interface GPUBuffer { destroy(): void; }
interface GPUTexture {
  createView(desc?: Record<string, unknown>): GPUTextureView;
  destroy(): void;
}
interface GPUTextureView {}
interface GPUBindGroup {}
interface GPUCommandEncoder {
  beginRenderPass(desc: Record<string, unknown>): GPURenderPass;
  finish(): GPUCommandBuffer;
}
interface GPURenderPass {
  setPipeline(p: GPURenderPipeline): void;
  setBindGroup(i: number, g: GPUBindGroup): void;
  draw(n: number, instances?: number): void;
  end(): void;
}
interface GPUCommandBuffer {}
interface GPUQueue {
  writeTexture(
    dest: { texture: GPUTexture; origin?: { x?: number; y?: number; z?: number } },
    data: ArrayBufferView<ArrayBufferLike> | ArrayBuffer,
    layout: { bytesPerRow: number; rowsPerImage: number },
    size: { width: number; height: number; depthOrArrayLayers?: number },
  ): void;
  writeBuffer(buf: GPUBuffer, offset: number, data: ArrayBufferView<ArrayBufferLike> | ArrayBuffer): void;
  submit(cmds: GPUCommandBuffer[]): void;
}

interface GPUCanvasContext {
  unconfigure(): void;
  configure(desc: Record<string, unknown>): void;
  getCurrentTexture(): GPUTexture;
}

declare const GPUTextureUsage: {
  RENDER_ATTACHMENT: number;
  TEXTURE_BINDING: number;
  COPY_DST: number;
};

declare const GPUBufferUsage: {
  UNIFORM: number;
  STORAGE: number;
  COPY_DST: number;
};

declare const GPUShaderStage: {
  VERTEX: number;
  FRAGMENT: number;
  COMPUTE: number;
};

interface Navigator {
  gpu?: GPU;
}
