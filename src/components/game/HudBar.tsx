import { HeartPulse, Shield, Crosshair, Radio } from "lucide-react";
import { bossAttackForWave } from "@/game/boss-attacks";
import type { HudState } from "@/game/types";
import { cn } from "@/lib/utils";
import { WEAPONS, fmtTime, missionLine, sectorForWave } from "./data";

export function HudBar({
  hud,
  fps,
  resolution,
  renderer,
  showStats = false,
}: {
  hud: HudState;
  fps: number;
  resolution: string;
  renderer: string;
  showStats?: boolean;
}) {
  const weapon = WEAPONS[hud.weapon] ?? WEAPONS[0]!;
  const sector = sectorForWave(hud.wave);
  const attack = bossAttackForWave(hud.wave);
  const lowAmmo = hud.ammo > 0 && hud.ammo <= weapon.lowAmmoAt && hud.reloading <= 0.001;
  return (
    <div className="field-hud">
      <div className="hud-mission">
        <Radio size={15} />
        <div>
          <span>
            {sector.code} / {sector.name}
          </span>
          <strong>{missionLine(hud)}</strong>
        </div>
      </div>
      <div
        className="hud-threat"
        aria-label="Remaining hostiles"
        title="Living enemies and incoming reinforcements"
      >
        <Crosshair size={15} />
        <b>{hud.living}</b>
        <span>HOSTILES</span>
        <i /> <span>{fmtTime(hud.elapsedMs)}</span>
        <i /> <span className="hud-fps">{Math.round(fps)} FPS</span>
      </div>
      {showStats && (
        <div className="hud-performance">
          {Math.round(fps)} FPS · {renderer.toUpperCase()} · {resolution}
        </div>
      )}
      {hud.bossHealth > 0 && hud.bossMaxHealth > 0 && (
        <section className="hud-boss" aria-label={`${sector.bossName} health`}>
          <div>
            <span>{sector.bossTitle}</span>
            <strong>{sector.bossName}</strong>
            <em>
              {hud.bossAttackState === 1
                ? "CHARGING"
                : hud.vuln > 0.05
                  ? "EXPOSED"
                  : `PHASE ${hud.bossPhase + 1} / 3`}
            </em>
          </div>
          <div className="hud-boss-track">
            <i
              style={{
                width: `${Math.max(0, Math.min(100, (hud.bossHealth / hud.bossMaxHealth) * 100))}%`,
              }}
            />
          </div>
          {(hud.bossAttackState === 1 || hud.vuln > 0.05) && (
            <p className={cn("hud-boss-cue", hud.bossAttackState === 1 && "hud-boss-warning")}>
              <b>{hud.bossAttackState === 1 ? attack.name : "CORE EXPOSED"}</b>
              <span>
                {hud.bossAttackState === 1
                  ? attack.dodge
                  : hud.bossAttackState === 2
                    ? `${attack.weak.toUpperCase()} · +25% recovery damage`
                    : "Fire while vulnerable · double damage"}
              </span>
              <time>
                {Math.max(0, hud.bossAttackState === 1 ? hud.bossAttackT : hud.vuln).toFixed(1)}s
              </time>
            </p>
          )}
          <small>
            {hud.bossHealth} / {hud.bossMaxHealth}
          </small>
        </section>
      )}
      <div className="hud-bottom">
        <section
          className={cn("hud-plate hud-vitals", hud.health < 30 && "hud-critical")}
          aria-label="Vitals"
        >
          <div className="hud-content">
            <div className="hud-health">
              <HeartPulse size={18} />
              <div>
                <span className="hud-label">HEALTH</span>
                <strong>
                  {hud.health}
                  <small> / 100</small>
                </strong>
              </div>
            </div>
            <div className="vital-track">
              <span
                className={cn("vital-fill", hud.health < 30 && "vital-critical")}
                style={{ width: `${Math.max(0, Math.min(100, hud.health))}%` }}
              />
            </div>
            <div className="hud-armour">
              <Shield size={13} />
              <span>ARMOR</span>
              <b>{hud.armor}</b>
              <span className="armour-track">
                <i style={{ width: `${Math.min(100, hud.armor)}%` }} />
              </span>
            </div>
          </div>
        </section>
        <section
          className={cn("hud-plate hud-ammo", lowAmmo && "hud-low-ammo")}
          aria-label="Weapon ammunition"
        >
          <div className="hud-content">
            <div className="hud-ammo-heading">
              <span>{weapon.name}</span>
              <span>{hud.reloading > 0.001 ? "RELOADING" : lowAmmo ? "LOW AMMO" : "AMMO"}</span>
            </div>
            <strong className={cn("hud-amount", hud.ammo === 0 && "text-danger")}>
              {String(hud.ammo).padStart(2, "0")}
              <small> / {hud.reserve}</small>
            </strong>
            <div className="vital-track">
              <span
                className="vital-fill"
                style={{
                  width: `${hud.weapon !== 1 && hud.reloading > 0.001 ? hud.reloading * 100 : (hud.ammo / weapon.magSize) * 100}%`,
                }}
              />
            </div>
            <p>
              {hud.reloading > 0.001
                ? hud.weapon === 1
                  ? "LOADING SHELLS"
                  : "CHANGING MAGAZINE"
                : weapon.role}
            </p>
          </div>
        </section>
      </div>
    </div>
  );
}
