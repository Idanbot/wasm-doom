export type HudState = {
  health: number;
  armor: number;
  ammo: number;
  weapon: number;
  kills: number;
  living: number;
  state: number;
  prompt: number;
  hasW2: boolean;
  hasW3: boolean;
  secrets: number;
  elapsedMs: number;
  shake: number;
  muzzle: number;
  hurt: number;
  bob: number;
  kick: number;
  hitmarker: number;
  spread: number;
  yaw: number;
  speed: number;
  x: number;
  y: number;
  reserve: number;
  reloading: number;
  weapFrame: number;
  hasW4: boolean;
  hasW5: boolean;
  events: number;
  evWeapon: number;
  wave: number;
  bossHealth: number;
  bossMaxHealth: number;
  bossPhase: number;
};

export type ResMode = {
  id: string;
  label: string;
  w: number;
  h: number;
};

/** Widths are multiples of 64 so RGBA rows are 256-byte aligned for WebGPU. */
export const RES_MODES: ResMode[] = [
  { id: "320", label: "320 x 200", w: 320, h: 200 },
  { id: "640", label: "640 x 400", w: 640, h: 400 },
  { id: "960", label: "960 x 600", w: 960, h: 600 },
  { id: "1280", label: "1280 x 800", w: 1280, h: 800 },
  { id: "1600", label: "1600 x 1000", w: 1600, h: 1000 },
];

export type GfxOpts = {
  crt: boolean;
  bloom: boolean;
  fog: boolean;
};

export const DEFAULT_GFX: GfxOpts = {
  crt: true,
  bloom: true,
  fog: true,
};

export const DEFAULT_HUD: HudState = {
  health: 100,
  armor: 0,
  ammo: 12,
  weapon: 0,
  kills: 0,
  living: 0,
  state: 0,
  prompt: 0,
  hasW2: false,
  hasW3: false,
  secrets: 0,
  elapsedMs: 0,
  shake: 0,
  muzzle: 0,
  hurt: 0,
  bob: 0,
  kick: 0,
  hitmarker: 0,
  spread: 0,
  yaw: 0,
  speed: 0,
  x: 4.5,
  y: 6.5,
  reserve: 36,
  reloading: 0,
  weapFrame: 0,
  hasW4: false,
  hasW5: false,
  events: 0,
  evWeapon: 0,
  wave: 1,
  bossHealth: 0,
  bossMaxHealth: 0,
  bossPhase: 0,
};
