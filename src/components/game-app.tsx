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
  sectorForWave,
  gridPos,
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
import { EnemySubtitles } from "./game/EnemySubtitles";
import { loadEnemyOptions, type EnemySubtitle } from "@/game/enemy-presentation";

export function GameApp() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rtRef = useRef<HellscanRuntime | null>(null);
  const lookPtr = useRef<number | null>(null);
  const movePtr = useRef<number | null>(null);
  const weaponRef = useRef<HTMLDivElement>(null);
  const lastHudAt = useRef(0);
  const lastHudKey = useRef("");
  const lastSubtitleAt = useRef(0);
  const lastWeapon = useRef(-1);
  const swapAt = useRef(-1e9);
  const [enemyOptions, setEnemyOptions] = useState(loadEnemyOptions);
  const [subtitles, setSubtitles] = useState<EnemySubtitle[]>([]);
  const [screen, setScreen] = useState<Screen>("menu");
  const [hud, setHud] = useState<HudState>(DEFAULT_HUD);
  const [fps, setFps] = useState(0);
  const [renderResolution, setRenderResolution] = useState("");
  const [res, setRes] = useState<ResMode>(loadRes);
  const [sens, setSens] = useState(() => {
    try {
      const n = Number(localStorage.getItem("blacksite-sensitivity"));
      return n >= 0.5 && n <= 3.5 ? n : 1.4;
    } catch {
      return 1.4;
    }
  });
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
    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

    const rt = new HellscanRuntime(canvas, {
      onSubtitles: (lines) => {
        const now = performance.now();
        if (!lines.length || now - lastSubtitleAt.current > 50) {
          lastSubtitleAt.current = now;
          setSubtitles((previous) => (!lines.length && !previous.length ? previous : lines));
        }
      },
      onHud: (h, f, resolution) => {
        const weapEl = weaponRef.current;
        if (weapEl) {
          if (h.weapon !== lastWeapon.current) {
            lastWeapon.current = h.weapon;
            swapAt.current = performance.now();
          }
          // Swap raise/lower dip so weapon changes read as an animation,
          // not an instant cut. 200ms, eased with a sine hump.
          const swapAge = (performance.now() - swapAt.current) / 200;
          const swapDip = swapAge < 1 ? Math.sin(swapAge * Math.PI) * 30 : 0;
          const reloading = h.reloading > 0.001;
          const reloadDip = reloading ? 32 + Math.sin(Math.min(1, h.reloading) * Math.PI) * 18 : 0;
          const weight = [0.65, 1.25, 0.5, 1.1, 1.4, 0.9, 1.45][h.weapon] ?? 1;
          const motion = reducedMotion.matches ? 0.2 : 1;
          const bobY = (h.bob * 7 + h.kick * 18 * weight + reloadDip + swapDip) * motion;
          const bobX = (h.kick * -6 * weight + (reloading ? 20 : 0)) * motion;
          const roll = h.kick * -1.8 * weight * motion;
          weapEl.style.transform = `translate(-50%, ${bobY}px) translateX(${bobX}px) rotate(${roll}deg)`;
          weapEl.style.filter = h.muzzle > 0.05 ? `brightness(${1 + h.muzzle * 0.22})` : "";
          const wpn = WEAPONS[h.weapon] ?? WEAPONS[0]!;
          const fr = h.weapFrame | 0;
          if (fr >= 5) {
            weapEl.style.backgroundImage = `url(${wpn.reload})`;
            const pistolReload = h.weapon === 0;
            weapEl.style.backgroundSize = pistolReload ? "200% 200%" : "400% 200%";
            weapEl.style.backgroundPosition = pistolReload
              ? sheetPos(Math.min(3, fr - 5))
              : gridPos(Math.min(7, fr - 5), 4, 2);
          } else if (fr >= 1) {
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
        // Discrete combat state (ammo, health, weapon, kills, prompts)
        // syncs immediately so the HUD never lags the gun; the running
        // timer/fps still throttle to 10Hz to avoid re-rendering 60/s.
        const key = [
          h.health,
          h.armor,
          h.ammo,
          h.reserve,
          h.weapon,
          h.kills,
          h.living,
          h.state,
          h.prompt,
          h.secrets,
          h.hasW2,
          h.hasW3,
          h.hasW4,
          h.hasW5,
          h.hasW6,
          h.hasW7,
          h.reloading > 0.001,
          h.reloading === 0,
        ].join("|");
        if (now - lastHudAt.current > 100 || key !== lastHudKey.current) {
          lastHudAt.current = now;
          lastHudKey.current = key;
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
      onError: (msg) => {
        rt.setPlaying(false);
        setErr(`Engine fault: ${msg}. Reload the page and deploy again.`);
        setScreen("menu");
      },
    });
    rtRef.current = rt;
    rt.setEnemyOptions(loadEnemyOptions());
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
    rt.kick();
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
      if (
        document.querySelector('[role="dialog"]') ||
        /INPUT|SELECT|TEXTAREA/.test((e.target as HTMLElement)?.tagName)
      )
        return;
      if (e.code === "Escape" || e.code === "KeyP") {
        if (screen === "play") {
          e.preventDefault();
          pause();
        } else if (screen === "pause" && e.code === "KeyP") {
          resume();
        }
      }
      if (
        e.code === "Enter" &&
        screen === "menu" &&
        ready &&
        (e.target === document.body || e.target === document.documentElement)
      )
        start();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [screen, pause, start, resume, ready]);

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
    try {
      localStorage.setItem("blacksite-sensitivity", String(sens));
    } catch {}
  }, [sens]);

  useEffect(() => {
    rtRef.current?.setEnemyOptions(enemyOptions);
    try {
      localStorage.setItem("blacksite-enemy-options", JSON.stringify(enemyOptions));
    } catch {}
  }, [enemyOptions]);

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
        onWheel={(e) => {
          if (screen === "play") rtRef.current?.cycleWeapon(e.deltaY >= 0 ? 1 : -1);
        }}
      />

      {screen === "play" && (
        <>
          <WeaponView weaponRef={weaponRef} />
          <Crosshair
            flash={hud.hitmarker}
            spread={(hud.weapon === 2 ? hud.spread : hud.weapon === 1 ? 0.1 : 0) + hud.kick * 0.04}
          />
          <HudBar
            hud={hud}
            fps={fps}
            resolution={renderResolution}
            renderer={renderer}
            showStats={enemyOptions.showStats}
          />
          {enemyOptions.subtitles && (
            <EnemySubtitles lines={subtitles} size={enemyOptions.subtitleSize} />
          )}
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
              REACH THE OVERRIDE STATION
            </p>
          )}
          {hud.prompt === 3 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-danger">
              USE E — INITIATE OVERRIDE
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
          <button type="button" className="hud-pause" onClick={pause}>
            Pause
          </button>
        </>
      )}

      {overlay && (
        <div className={`operation-overlay ${screen === "menu" ? "operation-home" : ""}`}>
          <div
            className="operation-backdrop"
            style={{ backgroundImage: "url(/game/ui/menu-reactor.webp)" }}
          />
          <div className="operation-shade" />
          <section className={`operation-content ${screen !== "menu" ? "operation-card" : ""}`}>
            {screen === "menu" && (
              <Menu
                enemyOptions={enemyOptions}
                setEnemyOptions={setEnemyOptions}
                onPreviewVoice={(skin) => rtRef.current?.previewEnemy(skin)}
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
                enemyOptions={enemyOptions}
                setEnemyOptions={setEnemyOptions}
                onPreviewVoice={(skin) => rtRef.current?.previewEnemy(skin)}
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
                variant="dead"
                title="FLATLINED"
                body={`${sectorForWave(hud.wave).name} remains hostile. The scanline ends here.`}
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
                variant="win"
                title={`${sectorForWave(hud.wave).name} CLEARED`}
                body={`${sectorForWave(hud.wave).bossName} is down. A new sector route is unlocked.`}
                hud={hud}
                board={board}
                showBoard={false}
                nextLabel={`Enter ${sectorForWave(hud.wave + 1).name}`}
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
