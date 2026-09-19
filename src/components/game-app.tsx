import { useCallback, useEffect, useRef, useState } from "react";
import { HellscanRuntime } from "@/game/runtime";
import { DEFAULT_HUD, type GfxOpts, type HudState, type ResMode } from "@/game/types";
import { Crosshair } from "./game/Crosshair";
import {
  gpuEnabled,
  loadBoard,
  loadGfx,
  loadRes,
  loadVol,
  saveBoard,
  sheetPos,
  WEAPONS,
  type Score,
  type Screen,
} from "./game/data";
import { EndCard } from "./game/EndCard";
import { HudBar } from "./game/HudBar";
import { Menu } from "./game/Menu";
import { Pause } from "./game/Pause";
import { TouchLayer } from "./game/TouchLayer";
import { WeaponView } from "./game/WeaponView";

export function GameApp() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rtRef = useRef<HellscanRuntime | null>(null);
  const lookPtr = useRef<number | null>(null);
  const movePtr = useRef<number | null>(null);
  const weaponRef = useRef<HTMLDivElement>(null);
  const lastHudAt = useRef(0);
  const [screen, setScreen] = useState<Screen>("menu");
  const [hud, setHud] = useState<HudState>(DEFAULT_HUD);
  const [fps, setFps] = useState(0);
  const [renderResolution, setRenderResolution] = useState("");
  const [res, setRes] = useState<ResMode>(loadRes);
  const [sens, setSens] = useState(1.4);
  const [muted, setMuted] = useState(false);
  const [vol, setVol] = useState(loadVol);
  const [requireGpu, setRequireGpu] = useState(gpuEnabled);
  const qaRef = useRef(false);
  const [gfx, setGfx] = useState<GfxOpts>(loadGfx);
  const [board, setBoard] = useState<Score[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [ready, setReady] = useState(false);
  const [isCoarse, setIsCoarse] = useState(false);
  const [renderer, setRenderer] = useState("webgl2");

  // Boot once. Renderer switches happen via switchRenderer (no sim restart,
  // no canvas remount) — see handleRequireGpu below.
  useEffect(() => {
    setBoard(loadBoard());
    setIsCoarse(window.matchMedia("(pointer: coarse)").matches);
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rt = new HellscanRuntime(canvas, {
      onHud: (h, f, resolution) => {
        const weapEl = weaponRef.current;
        if (weapEl) {
          const reloadDip = h.reloading > 0.01 ? 32 + Math.sin(h.reloading * Math.PI) * 18 : 0;
          const bobY = h.bob * 10 + h.kick * 18 + reloadDip;
          const bobX = h.kick * -8 + (h.reloading > 0.01 ? 20 : 0);
          weapEl.style.transform = `translate(-50%, ${bobY}px) translateX(${bobX}px)`;
          weapEl.style.filter = "";
          weapEl.style.setProperty("--flash-s", "1");
          weapEl.style.setProperty("--flash-x", "62%");
          weapEl.style.setProperty("--flash-y", "46%");
          weapEl.style.setProperty(
            "--muzzle",
            h.muzzle > 0.04 && h.reloading < 0.01 ? String(Math.min(1, h.muzzle)) : "0",
          );

          const wpn = WEAPONS[h.weapon] ?? WEAPONS[0]!;
          const fr = h.weapFrame | 0;
          const mini = h.weapon === 2;
          if (fr >= 5) {
            weapEl.style.backgroundImage = `url(${wpn.reload})`;
            weapEl.style.backgroundSize = "200% 200%";
            weapEl.style.backgroundPosition = sheetPos(Math.min(3, fr - 5));
          } else if (fr >= 1 && !mini) {
            weapEl.style.backgroundImage = `url(${wpn.fire})`;
            weapEl.style.backgroundSize = "200% 200%";
            weapEl.style.backgroundPosition = sheetPos(Math.min(3, fr - 1));
          } else {
            weapEl.style.backgroundImage = `url(${wpn.idle})`;
            weapEl.style.backgroundSize = "contain";
            weapEl.style.backgroundPosition = "center bottom";
          }
          weapEl.style.backgroundRepeat = "no-repeat";
        }
        const now = performance.now();
        if (now - lastHudAt.current > 100) {
          lastHudAt.current = now;
          setHud({ ...h });
          setFps(f);
          setRenderResolution(resolution);
        }
      },
      onState: (st) => {
        if (st === 1) {
          rt.setPlaying(false);
          setScreen("dead");
        }
        if (st === 2) {
          rt.setPlaying(false);
          setScreen("win");
        }
      },
    });
    rtRef.current = rt;
    rt.setGfx(loadGfx());
    let dead = false;
    setErr(null);
    setReady(false);
    const qa = new URLSearchParams(window.location.search).get("qa") === "1";
    const initialRes = loadRes();
    setRes(initialRes);
    const initialGpu = qa ? false : gpuEnabled();
    setRequireGpu(initialGpu);
    rt.boot(initialRes, { requireGpu: initialGpu })
      .then(() => {
        if (dead) return;
        setReady(true);
        setRenderer(rt.renderer);
        if (qa) {
          qaRef.current = true;
          rt.setPlaying(true);
          setScreen("play");
        }
      })
      .catch((e: unknown) => {
        if (dead) return;
        setReady(false);
        setErr(e instanceof Error ? e.message : "Failed to load WASM core");
      });
    return () => {
      dead = true;
      rt.stop();
    };
  }, []);

  const handleRequireGpu = useCallback(async (on: boolean) => {
    setRequireGpu(on);
    try {
      localStorage.setItem("hellscan-gpu", on ? "1" : "0");
    } catch {
      /* ignore */
    }
    const rt = rtRef.current;
    if (!rt) return;
    try {
      const kind = await rt.switchRenderer(on);
      setRenderer(kind);
      setErr(null);
    } catch (e) {
      setErr(e instanceof Error ? e.message : "Failed to switch renderer");
      // Fall back to the automatic path so the game stays playable.
      try {
        const kind = await rt.switchRenderer(false);
        setRenderer(kind);
        setRequireGpu(false);
        localStorage.setItem("hellscan-gpu", "0");
      } catch {
        /* keep last working renderer */
      }
    }
  }, []);

  const start = useCallback(() => {
    const rt = rtRef.current;
    if (!rt) return;
    rt.setPlaying(true);
    rt.setSens(sens);
    rt.setMuted(muted);
    rt.setVolumes(vol.master, vol.music, vol.sfx);
    setScreen("play");
    rt.requestLock();
  }, [sens, muted, vol]);

  const pause = useCallback(() => {
    rtRef.current?.setPlaying(false);
    if (document.pointerLockElement) document.exitPointerLock();
    setScreen("pause");
  }, []);

  const resume = useCallback(() => {
    rtRef.current?.setPlaying(true);
    setScreen("play");
    rtRef.current?.requestLock();
  }, []);

  const restart = useCallback(() => {
    rtRef.current?.restart();
    rtRef.current?.setPlaying(true);
    setScreen("play");
    rtRef.current?.requestLock();
  }, []);

  const nextWave = useCallback(() => {
    rtRef.current?.nextWave();
    rtRef.current?.setPlaying(true);
    setScreen("play");
    rtRef.current?.requestLock();
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.code === "Escape" || e.code === "KeyP") {
        if (screen === "play") {
          e.preventDefault();
          pause();
        } else if (screen === "pause" && e.code === "KeyP") {
          resume();
        }
      }
      if (e.code === "Enter" && screen === "menu") start();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [screen, pause, start, resume]);

  useEffect(() => {
    if (screen !== "play" && document.pointerLockElement) {
      document.exitPointerLock();
    }
  }, [screen]);

  useEffect(() => {
    const onLock = () => {
      if (qaRef.current) return;
      // Touch devices never take pointer lock — never auto-pause them here.
      if (window.matchMedia("(pointer: coarse)").matches) return;
      if (screen === "play" && !document.pointerLockElement) pause();
    };
    document.addEventListener("pointerlockchange", onLock);
    return () => document.removeEventListener("pointerlockchange", onLock);
  }, [screen, pause]);

  useEffect(() => {
    rtRef.current?.setResolution(res);
    try {
      localStorage.setItem("hellscan-res", res.id);
    } catch {
      /* ignore */
    }
  }, [res]);

  useEffect(() => {
    rtRef.current?.setGfx(gfx);
    try {
      localStorage.setItem("hellscan-gfx", JSON.stringify(gfx));
    } catch {
      /* ignore */
    }
  }, [gfx]);

  useEffect(() => {
    rtRef.current?.setSens(sens);
  }, [sens]);

  useEffect(() => {
    rtRef.current?.setMuted(muted);
  }, [muted]);

  useEffect(() => {
    rtRef.current?.setVolumes(vol.master, vol.music, vol.sfx);
    try {
      localStorage.setItem("hellscan-vol", JSON.stringify(vol));
    } catch {
      /* ignore */
    }
  }, [vol]);

  const overlay = screen !== "play";

  return (
    <main className="relative h-dvh w-full overflow-hidden bg-bg text-fg">
      <canvas
        ref={canvasRef}
        className="game-canvas"
        onClick={() => {
          if (screen === "play") rtRef.current?.requestLock();
        }}
      />

      {screen === "play" && (
        <>
          <WeaponView hud={hud} weaponRef={weaponRef} />
          <Crosshair flash={hud.hitmarker} spread={hud.weapon === 2 ? hud.spread : 0} />
          <HudBar hud={hud} fps={fps} resolution={renderResolution} renderer={renderer} />
          {hud.prompt === 1 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-steel">
              USE E
            </p>
          )}
          {hud.prompt === 2 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-danger">
              RELOAD R
            </p>
          )}
          {hud.prompt === 4 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-steel">
              WEAPON
            </p>
          )}
          {hud.prompt === 5 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-steel">
              SUPPLY
            </p>
          )}
          {hud.prompt === 6 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-danger">
              STEP ON THE SEAL
            </p>
          )}
          {isCoarse && (
            <TouchLayer
              onMove={(x, y) => rtRef.current?.setTouchMove(x, y)}
              onLook={(x, y) => rtRef.current?.setTouchLook(x, y)}
              onFire={(v) => rtRef.current?.setFireHeld(v)}
              onUse={() => rtRef.current?.pulseUse()}
              onReload={() => rtRef.current?.pulseReload()}
              onWeapon={() => rtRef.current?.nextWeapon()}
              lookPtr={lookPtr}
              movePtr={movePtr}
            />
          )}
          <button
            type="button"
            className="absolute right-4 top-20 z-10 h-11 rounded-md border border-border bg-bg/70 px-3 font-display text-xs tracking-[0.18em] text-muted"
            onClick={pause}
          >
            Pause
          </button>
        </>
      )}

      {overlay && (
        <div className="absolute inset-0 z-20 flex items-center justify-center bg-bg/75">
          <div
            className="absolute inset-0 bg-cover bg-center opacity-55"
            style={{ backgroundImage: "url(/game/menu.jpg)" }}
          />
          <div className="absolute inset-0 bg-gradient-to-t from-bg via-bg/75 to-bg/45" />
          <section className="doom-panel relative z-10 mx-4 w-full max-w-md p-5">
            {screen === "menu" && (
              <Menu
                ready={ready}
                err={err}
                board={board}
                res={res}
                setRes={setRes}
                sens={sens}
                setSens={setSens}
                muted={muted}
                setMuted={setMuted}
                vol={vol}
                setVol={setVol}
                requireGpu={requireGpu}
                setRequireGpu={handleRequireGpu}
                gfx={gfx}
                setGfx={setGfx}
                onStart={start}
              />
            )}

            {screen === "pause" && (
              <Pause
                res={res}
                setRes={setRes}
                sens={sens}
                setSens={setSens}
                muted={muted}
                setMuted={setMuted}
                vol={vol}
                setVol={setVol}
                requireGpu={requireGpu}
                setRequireGpu={handleRequireGpu}
                gfx={gfx}
                setGfx={setGfx}
                onResume={resume}
                onRestart={restart}
                onMenu={() => {
                  rtRef.current?.restart();
                  rtRef.current?.setPlaying(false);
                  setScreen("menu");
                }}
              />
            )}
            {screen === "dead" && (
              <EndCard
                title="FLATLINED"
                body={`Wave ${hud.wave || 1} is over. The scanline ends here.`}
                hud={hud}
                board={board}
                showBoard
                nextLabel="Start again"
                onAgain={restart}
                onMenu={() => {
                  rtRef.current?.restart();
                  rtRef.current?.setPlaying(false);
                  setScreen("menu");
                }}
                onSave={(name) =>
                  setBoard(
                    saveBoard({ name, wave: hud.wave || 1, kills: hud.kills, time: hud.elapsedMs }),
                  )
                }
              />
            )}
            {screen === "win" && (
              <EndCard
                title="SITE CLEARED"
                body="The vault's master is dead. The pit goes quiet."
                hud={hud}
                board={board}
                showBoard={false}
                nextLabel="Next wave"
                onAgain={nextWave}
                onMenu={() => {
                  rtRef.current?.restart();
                  rtRef.current?.setPlaying(false);
                  setScreen("menu");
                }}
              />
            )}
          </section>
        </div>
      )}
    </main>
  );
}
