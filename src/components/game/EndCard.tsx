import { useState } from "react";
import { Activity, ChevronRight, Crosshair, RotateCcw, Timer, Trophy } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { HudState } from "@/game/types";
import { fmtTime, type Score } from "./data";

export function EndCard({
  title,
  body,
  hud,
  board,
  showBoard,
  nextLabel,
  onAgain,
  onMenu,
  onSave,
  variant,
}: {
  title: string;
  body: string;
  hud: HudState;
  board: Score[];
  showBoard: boolean;
  nextLabel: string;
  onAgain: () => void;
  onMenu: () => void;
  onSave?: (name: string) => void;
  variant: "dead" | "win";
}) {
  const [name, setName] = useState("");
  const [saved, setSaved] = useState(false);
  const submit = () => {
    if (saved || !onSave) return;
    onSave((name.trim() || "MARINE").slice(0, 12).toUpperCase());
    setSaved(true);
  };
  return (
    <div className={`end-screen end-screen-${variant}`}>
      <header className="end-header">
        <span className="eyebrow">
          {variant === "win" ? "OPERATION COMPLETE" : "BIOSIGNAL LOST"}
        </span>
        <span className="end-status">
          <i /> {variant === "win" ? "SITE SECURED" : "UNIT OFFLINE"}
        </span>
      </header>
      <div className="end-title-row">
        {variant === "win" ? <Trophy aria-hidden /> : <Activity aria-hidden />}
        <div>
          <h2>{title}</h2>
          <p>{body}</p>
        </div>
      </div>
      <div className="end-stats" aria-label="Run statistics">
        <div>
          <span>LEVEL</span>
          <strong>{String(hud.wave || 1).padStart(2, "0")}</strong>
        </div>
        <div>
          <Timer aria-hidden />
          <span>TIME</span>
          <strong>{fmtTime(hud.elapsedMs)}</strong>
        </div>
        <div>
          <Crosshair aria-hidden />
          <span>ELIMINATED</span>
          <strong>{hud.kills}</strong>
        </div>
      </div>
      {showBoard && (
        <>
          <form
            className="end-log-form"
            onSubmit={(e) => {
              e.preventDefault();
              submit();
            }}
          >
            <input
              value={name}
              onChange={(e) => setName(e.target.value.slice(0, 12))}
              maxLength={12}
              placeholder="YOUR NAME"
              disabled={saved}
              className="end-name-input"
            />
            <Button type="submit" size="lg" variant="ghost" disabled={saved}>
              {saved ? "Logged" : "Log run"}
            </Button>
          </form>
          <div className="end-board">
            <p>FIELD RECORDS</p>
            <ol>
              {board.length === 0 && <li>No runs recorded</li>}
              {board.map((row, i) => (
                <li
                  key={`${row.name}-${row.wave}-${row.kills}-${row.time}-${i}`}
                  className="end-board-row"
                >
                  <span className="text-fg">
                    {i + 1}. {row.name} · WAVE {row.wave}
                  </span>
                  <span>
                    {row.kills} kills · {fmtTime(row.time)}
                  </span>
                </li>
              ))}
            </ol>
          </div>
        </>
      )}
      <div className="end-actions">
        <Button
          size="lg"
          className={variant === "win" ? "flex-1 border-primary bg-primary text-bg" : "flex-1"}
          onClick={onAgain}
        >
          {variant === "win" ? <ChevronRight aria-hidden /> : <RotateCcw aria-hidden />}
          {nextLabel}
        </Button>
        <Button size="lg" variant="ghost" onClick={onMenu}>
          Menu
        </Button>
      </div>
    </div>
  );
}
