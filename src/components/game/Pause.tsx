import { Button } from "@/components/ui/button";
import type { GfxOpts, ResMode } from "@/game/types";
import type { Vol } from "./data";
import { Settings } from "./Settings";

export function Pause({
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
  onResume,
  onRestart,
  onMenu,
}: {
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
  onResume: () => void;
  onRestart: () => void;
  onMenu: () => void;
}) {
  return (
    <div>
      <p className="font-display text-xs tracking-[0.28em] text-muted">PAUSED</p>
      <h2 className="mt-2 font-display text-4xl tracking-[0.08em]">HOLD SCAN</h2>
      <p className="mt-2 text-sm text-muted">Mouse is free. Resume to lock look again.</p>
      <div className="mt-6 flex flex-col gap-3">
        <Button size="lg" onClick={onResume}>
          Resume
        </Button>
        <Button variant="ghost" onClick={onRestart}>
          Restart
        </Button>
        <Button variant="ghost" onClick={onMenu}>
          Menu
        </Button>
        <Button variant="ghost" onClick={() => setMuted(!muted)}>
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
