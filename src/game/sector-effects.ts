import bossProfiles from "./boss-arena-data.json" with { type: "json" };
import { SECTOR_SLUGS } from "./sector-assets.ts";

/** A quiet visual signature and a licensed non-vocal sound accent for every sector. */
export const SECTOR_EFFECTS = [
  ["Welding dust", "d8b080", 0, .32, 7],
  ["Foundry ice crystals", "90dafa", 1, .24, 8],
  ["Bioforge spores", "b4d777", 2, .20, 9],
  ["Data scan motes", "79dff4", 3, .42, 6],
  ["Reactor ion haze", "8cf1bd", 4, .30, 8],
  ["Archive phase wisps", "b7a5ef", 5, .16, 10],
  ["Reserve snow drift", "c1ecff", 1, .15, 9],
  ["Signal interference", "e3b2fa", 3, .27, 7],
  ["Siege grit", "d5ae86", 0, .55, 6],
  ["Bunker vent mist", "c6d4bc", 6, .18, 9],
  ["Vault obsidian glints", "a38ae1", 7, .22, 10],
  ["Transit ash", "e9a27d", 0, .19, 8],
  ["Pumpstation spray", "82d6d0", 6, .48, 7],
  ["Optics diffraction", "f0d5a0", 7, .12, 9],
  ["Forge magnetic filings", "f2a575", 4, .45, 6],
  ["Hatchery tracking lights", "f4d779", 3, .61, 7],
  ["Fungal pollen", "a2d582", 2, .36, 10],
  ["Uplink charged dust", "a3c3ff", 4, .23, 8],
  ["Reclaimer corrosive vapor", "d3de74", 6, .26, 9],
  ["Shadow lab phase shimmer", "af8bcc", 5, .28, 11],
  ["Fusion plasma wisps", "ffc283", 4, .67, 7],
  ["Silo suspended droplets", "a0dbed", 6, .33, 8],
  ["Chrono drifting echoes", "cab4f4", 5, .08, 10],
  ["Black ice diamond frost", "87bcf5", 1, .39, 9],
  ["Apex command lattice", "f5c67c", 7, .31, 6],
] as const;

export function sectorEffect(wave: number) {
  const index = ((Math.max(1, Math.floor(wave)) - 1) % 25);
  const [name, hex, kind, drift, interval] = SECTOR_EFFECTS[index]!;
  return { index, slug: SECTOR_SLUGS[index]!, name, kind, drift, interval,
    color: [0, 2, 4].map(i => parseInt(hex.slice(i, i + 2), 16) / 255) };
}

export type Entrance = { x: number; y: number; depth: number; phase: number; style: number; color: number[] };

export type Shockwave = { x: number; y: number; radius: number; depth: number; strength: number };

/** Simulation time freezes with pause; four recent explosions bound GPU work. */
export class WorldEffects {
  time = 0;
  private wave = 0;
  private entrance: {x:number;y:number;age:number;duration:number;style:number} | null = null;
  private nextAccent = 3;
  private shocks: { x: number; y: number; age: number }[] = [];
  reset() { this.time = 0; this.wave = 0; this.nextAccent = 3; this.shocks = []; this.entrance = null; }
  tick(dt: number, wave: number): boolean {
    if (wave !== this.wave) { this.reset(); this.wave = wave; }
    this.time += dt;
    if (this.entrance) {this.entrance.age += dt;if(this.entrance.age>=this.entrance.duration)this.entrance=null;}
    for (const s of this.shocks) s.age += dt;
    this.shocks = this.shocks.filter(s => s.age < .75);
    if (this.time < this.nextAccent) return false;
    this.nextAccent = this.time + sectorEffect(wave).interval;
    return true;
  }
  bossEntrance(x:number,y:number,variant:number) {
    const style=Math.floor(variant/5),stage=variant%5;
    // Cues are emitted for five stages. Only the first starts the clock.
    if(stage===0)this.entrance={x,y,age:0,duration:bossProfiles[style]?.introSeconds??6,style};
  }
  projectEntrance(player:{x:number;y:number;yaw:number},aspect:number):Entrance|undefined {
    const e=this.entrance;if(!e)return;
    const dx=e.x-player.x,dy=e.y-player.y,c=Math.cos(player.yaw),s=Math.sin(player.yaw);
    const depth=dx*c+dy*s;if(depth<.18)return;
    const x=.5+(-s*dx+c*dy)/depth/(.72*aspect/1.6)*.5;
    if(x<-.5||x>1.5)return;
    return {x,y:.5,depth:Math.min(depth/28,1),phase:e.age/e.duration,
      style:e.style,color:[0,2,4].map(i=>parseInt((bossProfiles[e.style]?.color??"ffffff").slice(i,i+2),16)/255)};
  }
  explosion(x: number, y: number) {
    this.shocks.push({ x, y, age: 0 });
    if (this.shocks.length > 4) this.shocks.shift();
  }
  project(player: { x: number; y: number; yaw: number }, aspect: number): Shockwave[] {
    const c = Math.cos(player.yaw), s = Math.sin(player.yaw), plane = .72 * aspect / 1.6;
    return this.shocks.flatMap(p => {
      const dx = p.x - player.x, dy = p.y - player.y, depth = dx * c + dy * s;
      if (depth < .18) return [];
      const x = .5 + (-s * dx + c * dy) / depth / plane * .5;
      if (x < -.5 || x > 1.5) return [];
      return [{ x, y: .5, radius: (.12 + p.age * 3.6) / depth,
        depth: Math.min(depth / 28, 1), strength: (1 - p.age / .75) ** 2 }];
    });
  }
}
