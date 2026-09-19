import { useState } from "react";
import { Button } from "@/components/ui/button";
import type { GfxOpts, ResMode } from "@/game/types";
import { fmtTime, type Score, type Vol } from "./data";
import { GpuHelp } from "./GpuHelp";
import { Settings } from "./Settings";

export function Menu({
  ready,
  err,
  board,
  res,
  setRes,
  sens,
  setSens,
  muted,
  setMuted,
  vol,
  setVol,
  requireGpu,
  setRequireGpu,
  gfx,
  setGfx,
  onStart,
}: {
  ready: boolean;
  err: string | null;
  board: Score[];
  res: ResMode;
  setRes: (r: ResMode) => void;
  sens: number;
  setSens: (n: number) => void;
  muted: boolean;
  setMuted: (v: boolean) => void;
  vol: Vol;
  setVol: (v: Vol) => void;
  requireGpu: boolean;
  setRequireGpu: (v: boolean) => void;
  gfx: GfxOpts;
  setGfx: (g: GfxOpts) => void;
  onStart: () => void;
}) {
  const [gpuHelp, setGpuHelp] = useState(false);
  return (
    <div>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-display text-xs tracking-[0.32em] text-muted">
            NADIR-7 / WASM + WEBGPU
          </p>
          <h1 className="mt-2 font-display text-5xl tracking-[0.08em] sm:text-6xl">HELLSCAN</h1>
        </div>
        <button
          type="button"
          aria-label="How to enable WebGPU in Chrome"
          aria-expanded={gpuHelp}
          onClick={() => setGpuHelp((v) => !v)}
          className="mt-1 flex size-11 shrink-0 items-center justify-center rounded-full border border-border font-display text-lg leading-none text-steel hover:bg-elevated"
        >
          i
        </button>
      </div>
      {gpuHelp && <GpuHelp onClose={() => setGpuHelp(false)} />}
      <p className="mt-3 text-sm leading-relaxed text-muted">
        Site Nadir-7 went dark. Sweep the corridors. Walk the vault.
      </p>
      {err && <p className="mt-2 text-sm text-danger">{err}</p>}
      {!ready && !err && <p className="mt-2 font-mono text-xs text-steel">Loading sprites…</p>}
      {board[0] && (
        <p className="mt-2 font-mono text-xs text-steel">
          Best wave {board[0].wave} · {board[0].kills} kills · {fmtTime(board[0].time)}
        </p>
      )}
      <div className="mt-4 flex gap-3">
        <Button size="lg" onClick={onStart} className="flex-1" disabled={!ready}>
          Start Game
        </Button>
        <Button size="lg" variant="ghost" onClick={() => setMuted(!muted)}>
          {muted ? "Sound Off" : "Sound On"}
        </Button>
      </div>
      <Settings
        res={res}
        setRes={setRes}
        sens={sens}
        setSens={setSens}
        vol={vol}
        setVol={setVol}
        requireGpu={requireGpu}
        setRequireGpu={setRequireGpu}
        gfx={gfx}
        setGfx={setGfx}
      />
    </div>
  );
}
