import type { HudState } from "@/game/types";
import { cn } from "@/lib/utils";
import { WEAPONS, fmtTime } from "./data";

export function HudBar({
  hud,
  fps,
  resolution,
  renderer,
}: {
  hud: HudState;
  fps: number;
  resolution: string;
  renderer: string;
}) {
  return (
    <div className="pointer-events-none absolute inset-0 p-4 sm:p-5">
      <div className="flex items-start justify-between">
        <div className="rounded-md border border-border bg-bg/70 px-3 py-2">
          <p className="font-display text-[10px] tracking-[0.22em] text-muted">HEALTH</p>
          <p className={cn("hud-num text-2xl", hud.health < 30 ? "text-danger" : "text-fg")}>
            {hud.health}
          </p>
          <p className="font-display text-[10px] tracking-[0.22em] text-muted">ARMOR {hud.armor}</p>
          <div className="vital-track" aria-hidden="true">
            <span
              className={cn("vital-fill", hud.health < 30 && "vital-critical")}
              style={{ width: `${Math.max(0, Math.min(100, hud.health))}%` }}
            />
          </div>
        </div>
        <div className="rounded-md border border-border bg-bg/70 px-3 py-2 text-right">
          <p className="font-display text-[10px] tracking-[0.22em] text-steel">
            {renderer === "webgpu" ? "WEBGPU" : renderer === "webgl2" ? "WEBGL2" : "2D"}
          </p>
          <p className="hud-num text-2xl text-fg">{Math.round(fps)}</p>
          <p className="font-mono text-[10px] text-muted">FPS · {resolution}</p>
        </div>
      </div>
      <div className="absolute inset-x-4 bottom-4 flex items-end justify-between sm:inset-x-5 sm:bottom-5">
        <div className="rounded-md border border-border bg-bg/70 px-3 py-2">
          <p className="font-display text-[10px] tracking-[0.22em] text-muted">
            {hud.reloading > 0.01 ? "RELOADING" : "AMMO"}
          </p>
          <p className={cn("hud-num text-2xl", hud.ammo <= 0 ? "text-danger" : "text-fg")}>
            {hud.ammo}
            <span className="ml-1 font-mono text-sm text-muted">/{hud.reserve}</span>
          </p>
          {hud.reloading > 0.01 ? (
            <span className="mt-1 block h-1 overflow-hidden rounded-sm bg-elevated">
              <span
                className="block h-full bg-steel"
                style={{ width: `${Math.round(hud.reloading * 100)}%` }}
              />
            </span>
          ) : (
            <p className="font-mono text-[10px] text-muted">
              {WEAPONS[hud.weapon]?.name ?? "Sidearm"}
            </p>
          )}
          <p className="weapon-role">{WEAPONS[hud.weapon]?.role}</p>
          {hud.weapon === 2 && (
            <div className="vital-track" aria-label="Weapon spread">
              <span
                className="vital-fill vital-critical"
                style={{ width: `${(hud.spread / 0.24) * 100}%` }}
              />
            </div>
          )}
        </div>
        <div className="hidden gap-1 sm:flex">
          {[0, 1, 2, 3, 4].map((i) => (
            <span
              key={i}
              className={cn(
                "rounded-sm border px-2 py-1 font-mono text-[11px]",
                hud.weapon === i ? "border-steel text-fg" : "border-border text-muted",
                i === 1 && !hud.hasW2 && "opacity-30",
                i === 2 && !hud.hasW3 && "opacity-30",
                i === 3 && !hud.hasW4 && "opacity-30",
                i === 4 && !hud.hasW5 && "opacity-30",
              )}
            >
              {i + 1}
            </span>
          ))}
        </div>
        <div className="rounded-md border border-border bg-bg/70 px-3 py-2 text-right">
          <p className="font-display text-[10px] tracking-[0.22em] text-muted">HOSTILES</p>
          <p className="hud-num text-2xl">{hud.living}</p>
          <p className="font-mono text-[10px] text-muted">
            {hud.kills} down · {fmtTime(hud.elapsedMs)} · W{hud.wave || 1}
          </p>
        </div>
      </div>
    </div>
  );
}
