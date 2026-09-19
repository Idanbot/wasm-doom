import { useState } from "react";
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
}) {
  const [name, setName] = useState("");
  const [saved, setSaved] = useState(false);
  const submit = () => {
    if (saved || !onSave) return;
    onSave((name.trim() || "MARINE").slice(0, 12).toUpperCase());
    setSaved(true);
  };
  return (
    <div>
      <h2 className="font-display text-4xl tracking-[0.08em]">{title}</h2>
      <p className="mt-3 text-sm text-muted">{body}</p>
      <p className="mt-4 font-mono text-sm text-steel">
        Wave {hud.wave || 1} · {fmtTime(hud.elapsedMs)} · {hud.kills} kills
      </p>
      {showBoard && (
        <>
          <form
            className="mt-4 flex gap-2"
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
              className="min-h-11 flex-1 rounded-sm border border-border bg-bg px-3 font-mono text-sm uppercase tracking-[0.16em] text-fg outline-none focus:border-steel"
            />
            <Button type="submit" size="lg" variant="ghost" disabled={saved}>
              {saved ? "Logged" : "Log run"}
            </Button>
          </form>
          <div className="mt-5">
            <p className="font-display text-[10px] tracking-[0.22em] text-muted">LEADERBOARD</p>
            <ol className="mt-2 space-y-1 font-mono text-xs text-muted">
              {board.length === 0 && <li>No runs recorded</li>}
              {board.map((row, i) => (
                <li
                  key={`${row.name}-${row.wave}-${row.kills}-${row.time}-${i}`}
                  className="flex justify-between gap-3"
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
      <div className="mt-6 flex flex-col gap-3 sm:flex-row">
        <Button size="lg" className="flex-1" onClick={onAgain}>
          {nextLabel}
        </Button>
        <Button size="lg" variant="ghost" onClick={onMenu}>
          Menu
        </Button>
      </div>
    </div>
  );
}
