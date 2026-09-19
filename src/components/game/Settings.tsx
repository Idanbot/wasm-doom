import { useState } from "react";
import { Button } from "@/components/ui/button";
import type { GfxOpts, ResMode } from "@/game/types";
import { RES_MODES } from "@/game/types";
import type { Vol } from "./data";

export function Settings({
  res,
  setRes,
  sens,
  setSens,
  vol,
  setVol,
  requireGpu,
  setRequireGpu,
  gfx,
  setGfx,
}: {
  res: ResMode;
  setRes: (r: ResMode) => void;
  sens: number;
  setSens: (n: number) => void;
  vol: Vol;
  setVol: (v: Vol) => void;
  requireGpu: boolean;
  setRequireGpu: (v: boolean) => void;
  gfx: GfxOpts;
  setGfx: (g: GfxOpts) => void;
}) {
  const [open, setOpen] = useState(false);
  return (
    <div className="mt-4">
      <Button
        type="button"
        variant="ghost"
        size="lg"
        className="w-full"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        {open ? "Hide options" : "Options"}
      </Button>
      {open && (
        <div className="mt-3 grid grid-cols-2 gap-x-3 gap-y-2">
          <label className="col-span-2 block text-[10px] tracking-[0.18em] text-muted uppercase">
            Resolution
            <select
              className="mt-1 h-10 w-full rounded-sm border border-border bg-elevated px-2 font-mono text-sm text-fg"
              value={res.id}
              onChange={(e) => {
                const next = RES_MODES.find((r) => r.id === e.target.value);
                if (next) setRes(next);
              }}
            >
              {RES_MODES.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.label}
                </option>
              ))}
            </select>
          </label>
          <label className="col-span-2 block text-[10px] tracking-[0.18em] text-muted uppercase">
            Look {sens.toFixed(2)}
            <input
              type="range"
              min={0.5}
              max={3.5}
              step={0.05}
              value={sens}
              onChange={(e) => setSens(Number(e.target.value))}
              className="mt-1 w-full accent-danger"
            />
          </label>
          {(
            [
              ["master", "Master"],
              ["music", "Music"],
              ["sfx", "FX"],
            ] as const
          ).map(([key, label]) => (
            <label key={key} className="block text-[10px] tracking-[0.18em] text-muted uppercase">
              {label} {Math.round(vol[key] * 100)}
              <input
                type="range"
                min={0}
                max={1}
                step={0.01}
                value={vol[key]}
                onChange={(e) => setVol({ ...vol, [key]: Number(e.target.value) })}
                className="mt-1 w-full accent-danger"
              />
            </label>
          ))}
          {(
            [
              ["crt", "CRT"],
              ["bloom", "Bloom"],
              ["fog", "Fog"],
            ] as const
          ).map(([key, label]) => (
            <label
              key={key}
              className="flex h-10 cursor-pointer items-center gap-2 text-[10px] tracking-[0.18em] text-muted uppercase"
            >
              <input
                type="checkbox"
                checked={gfx[key]}
                onChange={(e) => setGfx({ ...gfx, [key]: e.target.checked })}
                className="size-4 accent-danger"
              />
              {label}
            </label>
          ))}
          <label className="flex h-10 cursor-pointer items-center gap-2 text-[10px] tracking-[0.18em] text-muted uppercase">
            <input
              type="checkbox"
              checked={requireGpu}
              onChange={(e) => {
                const on = e.target.checked;
                setRequireGpu(on);
                try {
                  localStorage.setItem("hellscan-gpu", on ? "1" : "0");
                } catch {
                  /* ignore */
                }
              }}
              className="size-4 accent-danger"
            />
            WebGPU
          </label>
        </div>
      )}
    </div>
  );
}
