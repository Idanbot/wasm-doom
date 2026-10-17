import { actionLabel } from "@/game/controls";
import { DIFFICULTIES, type Difficulty } from "@/game/run-record";
import { RunBoard } from "./RunBoard";
import { ArrowUpRight, Volume2, VolumeX, Crosshair, ShieldAlert } from "lucide-react";
import { fmtTime, WEAPONS, type Score } from "./data";
import { LazySettings, type SettingsProps } from "./LazySettings";
import { asset } from "@/lib/asset";

export function Menu(
  p: SettingsProps & {
    ready: boolean;
    err: string | null;
    load?: { ratio: number; label: string };
    loadingSpriteSrc?: string;
    board: Score[];
    difficulty: Difficulty;
    setDifficulty: (value: Difficulty) => void;
    muted: boolean;
    setMuted: (v: boolean) => void;
    onStart: () => void;
    onRetry: () => void;
    onOpenCatalog?: () => void;
  },
) {
  const best = p.board.find(row => (row.difficulty ?? "operative") === p.difficulty);
  return (
    <div className="deployment-menu">
      <div className="menu-status">
        <span className="status-dot" />
        NADIR–7<span>CONNECTION LOST</span>
      </div>
      <p className="eyebrow menu-kicker">TACTICAL CONTAINMENT / 07</p>
      <h1 className="blacksite-title">
        BLACK<span>SITE</span>
      </h1>
      <p className="menu-subtitle">P R O J E C T &nbsp; I F R I T</p>
      <p className="mission-copy">
        The facility went dark.
        <br />
        Whatever answers back isn't human.
      </p>
      <div className="mission-order">
        <ShieldAlert size={19} />
        <div>
          <span>YOUR ORDERS</span>
          <p>
            Defeat all 25 bosses. Claim their weapons.
            <br />
            Turn their arsenal against the next boss.
          </p>
        </div>
      </div>
      {p.err && (
        <p role="alert" className="menu-error">
          {p.err}
        </p>
      )}
      {!p.ready && !p.err && (
        <div className="deployment-loading" role="status" aria-live="polite">
          <span className="loading-operator" aria-hidden="true">
            <img
              src={p.loadingSpriteSrc ?? asset("/game/ui/loading-operator.webp")}
              alt=""
              width={1536}
              height={96}
            />
          </span>
          <span>
            Preparing your deployment<small>{p.load?.label ?? "Engine"}</small>
          </span>
        </div>
      )}
      {!p.ready && !p.err && (
        <div
          className="load-meter"
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.round((p.load?.ratio ?? 0) * 100)}
          aria-label={p.load?.label ?? "Loading"}
        >
          <span style={{ width: `${Math.round((p.load?.ratio ?? 0) * 100)}%` }} />
        </div>
      )}
      {p.err && (
        <button type="button" className="menu-secondary" onClick={p.onRetry}>
          Retry loading
        </button>
      )}
      <fieldset className="difficulty-picker" disabled={!p.ready}>
        <legend>Deployment difficulty</legend>
        <div className="difficulty-tabs">
          {DIFFICULTIES.map((preset) => (
            <button
              type="button"
              key={preset.id}
              disabled={!p.ready}
              aria-pressed={p.difficulty === preset.id}
              onClick={() => p.setDifficulty(preset.id)}
            >
              {preset.name}
            </button>
          ))}
        </div>
        <p>{DIFFICULTIES.find((preset) => preset.id === p.difficulty)!.description}</p>
        <small>
          {p.ready
            ? "Fixed for this run. Weapon damage, enemy health, ammo and attack warnings remain unchanged."
            : "Difficulty selection unlocks when loading finishes."}
        </small>
      </fieldset>
      <button type="button" className="deploy-button" disabled={!p.ready} onClick={p.onStart}>
        <Crosshair size={22} />
        <span>
          {p.ready ? "Enter blacksite" : "Preparing deployment…"}
          <small>
            {p.ready
              ? "Begin operation"
              : `${p.load?.label ?? "Loading"} · ${Math.round((p.load?.ratio ?? 0) * 100)}%`}
          </small>
        </span>
        <ArrowUpRight size={24} />
      </button>
      {p.onOpenCatalog && (
        <button
          type="button"
          className="menu-secondary"
          onClick={p.onOpenCatalog}
          style={{ borderColor: "rgba(245, 158, 11, 0.4)", color: "#fef08a" }}
        >
          <span>
            Asset Catalog & Weapon Preview
            <small>Inspect all {WEAPONS.length} current guns, frame loops & hashes</small>
          </span>
        </button>
      )}
      <LazySettings {...p} />
      <div className="menu-footer">
        <button type="button" onClick={() => p.setMuted(!p.muted)}>
          {p.muted ? <VolumeX size={16} /> : <Volume2 size={16} />}
          {p.muted ? "Sound off" : "Sound on"}
        </button>
        <span>{actionLabel(p.bindings,"forward")} {actionLabel(p.bindings,"left")} {actionLabel(p.bindings,"backward")} {actionLabel(p.bindings,"right")} / {actionLabel(p.bindings,"fire")} / {actionLabel(p.bindings,"reload")} TO RELOAD</span>
      </div>
      <details className="menu-records">
        <summary>Local leaderboards</summary>
        <RunBoard key={p.difficulty} board={p.board} initialDifficulty={p.difficulty} />
      </details>
      {best && (
        <p className="best-run">
          PERSONAL BEST / WAVE{" "}
          {best!.wave} ·{" "}
          {best!.kills} KILLS ·{" "}
          {fmtTime(best!.time)}
        </p>
      )}
    </div>
  );
}
