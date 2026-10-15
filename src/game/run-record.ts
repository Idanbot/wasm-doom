/** Local run records: simulation time, ammo-consuming fire and one hit per shot. */
export const DIFFICULTIES = [
  {
    id: "recruit",
    name: "Recruit",
    description: "65% incoming damage · 85% enemy speed · 125% attack recovery",
  },
  {
    id: "operative",
    name: "Operative",
    description: "Original balance · standard damage, speed and recovery",
  },
  {
    id: "veteran",
    name: "Veteran",
    description: "125% incoming damage · 110% enemy speed · 90% attack recovery",
  },
  {
    id: "blacksite",
    name: "Blacksite",
    description: "150% incoming damage · 120% enemy speed · 80% attack recovery",
  },
] as const;
export type Difficulty = (typeof DIFFICULTIES)[number]["id"];
export function difficulty(value: unknown): Difficulty {
  return DIFFICULTIES.find((p) => p.id === value)?.id ?? "operative";
}
export type SectorRecord = {
  sector: number;
  time: number;
  kills: number;
  completed: boolean;
  route: [number, number][];
  map: string;
};
export type RunRecord = {
  difficulty: Difficulty;
  shots: number;
  hits: number;
  ammoSpent: number;
  damageTaken: number;
  bosses: number;
  sectors: SectorRecord[];
  assisted: boolean;
};
export type RunTotals = {
  time: number;
  kills: number;
  shots: number;
  hits: number;
  ammoSpent: number;
  damageTaken: number;
  bosses: number;
};
export function accuracy(run: Pick<RunRecord, "shots" | "hits">): number {
  return run.shots > 0 ? Math.min(100, Math.max(0, (run.hits / run.shots) * 100)) : 0;
}
export function encodeRouteMap(map: Uint8Array): string {
  const runs: string[] = [];
  for (let i = 0; i < map.length;) {
    const value = map[i]!;
    let n = 1;
    while (i + n < map.length && map[i + n] === value) n++;
    runs.push(`${value.toString(36)}:${n.toString(36)}`);
    i += n;
  }
  return runs.join(",");
}
export function decodeRouteMap(value: string): number[] {
  const cells: number[] = [];
  for (const run of value.split(",")) {
    const [v, n] = run.split(":").map((x) => parseInt(x, 36));
    if (
      !Number.isInteger(v) ||
      !Number.isInteger(n) ||
      v! < 0 ||
      v! > 255 ||
      n! < 1 ||
      cells.length + n! > 1536
    )
      return [];
    cells.push(...Array<number>(n!).fill(v!));
  }
  return cells.length === 1536 ? cells : [];
}
/** Untrusted local storage is bounded and normalized before rendering or restoring. */
export function cleanRun(value: unknown): RunRecord | undefined {
  if (!value || typeof value !== "object") return undefined;
  const v = value as Partial<RunRecord>;
  const count = (n: unknown) =>
    typeof n === "number" && Number.isFinite(n)
      ? Math.min(1_000_000_000, Math.max(0, Math.floor(n)))
      : 0;
  const shots = Math.min(1_000_000, count(v.shots));
  return {
    difficulty: difficulty(v.difficulty),
    shots,
    hits: Math.min(shots, count(v.hits)),
    ammoSpent: count(v.ammoSpent),
    damageTaken: count(v.damageTaken),
    bosses: count(v.bosses),
    assisted: v.assisted === true,
    sectors: Array.isArray(v.sectors)
      ? v.sectors
          .slice(-128)
          .filter((s) => s && typeof s === "object")
          .map((s) => ({
            sector: Math.max(1, count(s.sector)),
            time: count(s.time),
            kills: count(s.kills),
            completed: s.completed === true,
            map:
              typeof s.map === "string" && s.map.length < 10_000 && decodeRouteMap(s.map).length
                ? s.map
                : "",
            route: Array.isArray(s.route)
              ? s.route
                  .slice(0, 128)
                  .filter(
                    (p) =>
                      Array.isArray(p) &&
                      p.length === 2 &&
                      p.every((n) => Number.isFinite(n) && n >= 0) &&
                      p[0] < 48 &&
                      p[1] < 32,
                  )
                  .map((p) => [p[0], p[1]] as [number, number])
              : [],
          }))
      : [],
  };
}

export class RunRecorder {
  preset: Difficulty = "operative";
  assisted = false;
  private sectors: SectorRecord[] = [];
  private active: SectorRecord | null = null;
  private base = { time: 0, kills: 0 };
  private lastSample = -Infinity;
  private interval = 1000;
  reset(preset: Difficulty, assisted = false) {
    this.preset = preset;
    this.assisted = assisted;
    this.sectors = [];
    this.active = null;
    this.base = { time: 0, kills: 0 };
    this.lastSample = -Infinity;
    this.interval = 1000;
  }
  restore(run: RunRecord, totals: RunTotals, sector: number) {
    this.reset(run.difficulty, run.assisted);
    this.sectors = structuredClone(run.sectors.filter((s) => s.sector !== sector));
    this.active = structuredClone(run.sectors.find((s) => s.sector === sector) ?? null);
    if (this.active) {
      // The engine's checkpoint totals are authoritative. A stale or edited
      // sector split must not create a negative cumulative baseline.
      this.active.time = Math.max(0, Math.min(totals.time, this.active.time));
      this.active.kills = Math.max(0, Math.min(totals.kills, this.active.kills));
    }
    this.base = {
      time: totals.time - (this.active?.time ?? 0),
      kills: totals.kills - (this.active?.kills ?? 0),
    };
  }
  sample(sector: number, totals: RunTotals, x: number, y: number, map: Uint8Array, state: number) {
    if (!this.active || this.active.sector !== sector) {
      if (this.active) {
        this.sectors.push(this.active);
        this.base.time += this.active.time;
        this.base.kills += this.active.kills;
      }
      this.sectors = this.sectors.slice(-127);
      this.active = {
        sector,
        time: 0,
        kills: 0,
        completed: false,
        route: [],
        map: encodeRouteMap(map),
      };
      this.lastSample = -Infinity;
      this.interval = 1000;
    }
    this.active.time = Math.max(0, totals.time - this.base.time);
    this.active.kills = Math.max(0, totals.kills - this.base.kills);
    this.active.completed ||= state === 2;
    if (
      Number.isFinite(x) &&
      Number.isFinite(y) &&
      x >= 0 &&
      x < 48 &&
      y >= 0 &&
      y < 32 &&
      (totals.time - this.lastSample >= this.interval || state !== 0)
    ) {
      const point: [number, number] = [Math.min(47.9, Math.round(x * 10) / 10), Math.min(31.9, Math.round(y * 10) / 10)];
      const last = this.active.route.at(-1);
      if (!last || last[0] !== point[0] || last[1] !== point[1]) {
        if (this.active.route.length >= 128) {
          this.active.route = this.active.route.filter((_, i) => i % 2 === 0);
          this.interval *= 2;
        }
        this.active.route.push(point);
      }
      this.lastSample = totals.time;
    }
  }
  record(totals: RunTotals): RunRecord {
    return {
      difficulty: this.preset,
      assisted: this.assisted,
      shots: totals.shots,
      hits: totals.hits,
      ammoSpent: totals.ammoSpent,
      damageTaken: totals.damageTaken,
      bosses: totals.bosses,
      sectors: structuredClone([...this.sectors, ...(this.active ? [this.active] : [])]),
    };
  }
}
