import { ArrowUpRight, Volume2, VolumeX, Crosshair, ShieldAlert } from "lucide-react";
import { fmtTime, type Score } from "./data";
import { Settings, type SettingsProps } from "./Settings";

export function Menu(
  p: SettingsProps & {
    ready: boolean;
    err: string | null;
    board: Score[];
    muted: boolean;
    setMuted: (v: boolean) => void;
    onStart: () => void;
    onContinue?: () => void;
  },
) {
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
            Recover the arsenal. Break containment.
            <br />
            Eliminate the signal at its source.
          </p>
        </div>
      </div>
      {p.err && (
        <p role="alert" className="menu-error">
          {p.err}
        </p>
      )}
      <button type="button" className="deploy-button" disabled={!p.ready} onClick={p.onStart}>
        <Crosshair size={22} />
        <span>
          {p.ready ? "Enter blacksite" : "Preparing deployment…"}
          <small>{p.ready ? "Begin operation" : "Loading world, arsenal and enemy voices"}</small>
        </span>
        <ArrowUpRight size={24} />
      </button>
      {p.onContinue && (
        <button type="button" className="menu-secondary" onClick={p.onContinue}>
          <span>
            Resume last sector
            <small>Weapons and supplies carry</small>
          </span>
        </button>
      )}
      <Settings {...p} />
      <div className="menu-footer">
        <button type="button" onClick={() => p.setMuted(!p.muted)}>
          {p.muted ? <VolumeX size={16} /> : <Volume2 size={16} />}
          {p.muted ? "Sound off" : "Sound on"}
        </button>
        <span>WASD / MOUSE / 9 0 - BOSS GUNS / R TO RELOAD</span>
      </div>
      {p.board[0] && (
        <p className="best-run">
          PERSONAL BEST / WAVE {p.board[0].wave} · {p.board[0].kills} KILLS ·{" "}
          {fmtTime(p.board[0].time)}
        </p>
      )}
    </div>
  );
}
