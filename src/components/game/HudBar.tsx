import { HeartPulse, Shield, Crosshair, Radio } from "lucide-react";
import type { HudState } from "@/game/types";
import { cn } from "@/lib/utils";
import { WEAPONS, fmtTime, sectorForWave } from "./data";

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
  const lowAmmo = hud.ammo > 0 && hud.ammo <= weapon.lowAmmoAt && hud.reloading <= 0.001;
  return (
    <div className="field-hud">
      <div className="hud-mission">
        <Radio size={15} />
        <div>
          <span>{sector.code} / {sector.name}</span>
          <strong>{hud.prompt === 3 ? "INITIATE OVERRIDE" : hud.prompt === 6 ? "REACH THE OVERRIDE" : "ELIMINATE THE SIGNAL"}</strong>
        </div>
      </div>
      <div className="hud-threat">
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
            <em>PHASE {hud.bossPhase + 1} / 3</em>
          </div>
          <div className="hud-boss-track">
            <i
              style={{
                width: `${Math.max(0, Math.min(100, (hud.bossHealth / hud.bossMaxHealth) * 100))}%`,
              }}
            />
          </div>
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
        </section>
        <div className="hud-loadout">
          <span>ARSENAL</span>
          <div>
            {WEAPONS.map((w, i) => (
              <span
                key={i}
                className={cn(
                  "weapon-slot",
                  hud.weapon === i && "selected",
                  i > 0 &&
                    ![true, hud.hasW2, hud.hasW3, hud.hasW4, hud.hasW5, hud.hasW6, hud.hasW7][i] &&
                    "locked",
                )}
              >
                <b>{i + 1}</b>
                <small>{w.name}</small>
              </span>
            ))}
          </div>
          <p>
            {hud.kills} ELIMINATED · {hud.secrets} SECRETS
          </p>
        </div>
        <section
          className={cn("hud-plate hud-ammo", lowAmmo && "hud-low-ammo")}
          aria-label="Weapon ammunition"
        >
          <div className="hud-ammo-heading">
            <span>{weapon.name}</span>
            <span>{hud.reloading > 0.001 ? "RELOADING" : lowAmmo ? "LOW AMMO" : "AMMO"}</span>
          </div>
          <strong className={hud.ammo === 0 ? "text-danger" : ""}>
            {String(hud.ammo).padStart(2, "0")}
            <small> / {hud.reserve}</small>
          </strong>
          <div className="vital-track">
            <span
              className="vital-fill"
              style={{
                width: `${hud.reloading > 0.001 ? hud.reloading * 100 : (hud.ammo / weapon.magSize) * 100}%`,
              }}
            />
          </div>
          <p>{hud.reloading > 0.001 ? "CHANGING MAGAZINE" : weapon.role}</p>
        </section>
      </div>
    </div>
  );
}
