import { Play, RotateCcw, LogOut, Volume2 } from "lucide-react";
import { Settings, type SettingsProps } from "./Settings";

export function Pause(
  p: SettingsProps & {
    muted: boolean;
    setMuted: (v: boolean) => void;
    onResume: () => void;
    onRetryWave: () => void;
    onMenu: () => void;
  },
) {
  return (
    <div className="pause-menu">
      <p className="eyebrow">OPERATOR CONNECTION / STANDBY</p>
      <h2>Operation paused</h2>
      <p className="mission-copy">Take a breath. The facility can wait.</p>
      <button type="button" className="deploy-button" onClick={p.onResume}>
        <Play size={20} />
        <span>
          Resume operation<small>Return to the field</small>
        </span>
      </button>
      <Settings {...p} />
      <div className="pause-actions">
        <button type="button" onClick={p.onRetryWave}>
          <RotateCcw size={16} />
          Retry wave
        </button>
        <button type="button" onClick={p.onMenu}>
          <LogOut size={16} />
          Main menu
        </button>
        <button type="button" onClick={() => p.setMuted(!p.muted)}>
          <Volume2 size={16} />
          {p.muted ? "Sound off" : "Sound on"}
        </button>
      </div>
    </div>
  );
}
