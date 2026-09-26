import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { asset } from "@/lib/asset";
import { HellscanRuntime } from "@/game/runtime";
import { DEFAULT_HUD, type GfxOpts, type HudState, type ResMode } from "@/game/types";
import { Crosshair } from "./game/Crosshair";
import {
  clearCheckpoint,
  gpuEnabled,
  loadBoard,
  loadCheckpoint,
  loadGfx,
  loadRes,
  loadVol,
  radioCopy,
  saveBoard,
  saveCheckpoint,
  sectorForWave,
  gridPos,
  sheetPos,
  WEAPONS,
  type RunSave,
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
import { Automap } from "./game/Automap";
import { EnemyBars } from "./game/EnemyBars";
import { RadioCard } from "./game/RadioCard";
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
  const [load, setLoad] = useState({ ratio: 0, label: "Engine" });
  const [isCoarse, setIsCoarse] = useState(false);
  const [renderer, setRenderer] = useState("webgl2");
  const [checkpoint, setCheckpoint] = useState<RunSave | null>(loadCheckpoint);
  const [radio, setRadio] = useState<{ speaker: string; text: string } | null>(null);
  const persistRef = useRef<(save: RunSave) => void>(() => {});
  persistRef.current = (save) => {
    saveCheckpoint(save);
    setCheckpoint(save);
  };

  // Boot once. Renderer switches happen via switchRenderer (no sim restart,
  // no canvas remount) — see handleRequireGpu below.
  useEffect(() => {
    setBoard(loadBoard());
    setIsCoarse(window.matchMedia("(pointer: coarse)").matches);
    const canvas = canvasRef.current;
    if (!canvas) return;
    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

    const rt = new HellscanRuntime(canvas, {
      onLoad: (progress) => setLoad(progress),
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
          const weight = [0.65, 1.25, 0.5, 1.1, 1.4, 0.9, 1.45, 1.1, 1.5, 0.8, 1.0][h.weapon] ?? 1;
          const motion = reducedMotion.matches ? 0.2 : 1;
          const bobY = (h.bob * 7 + h.kick * 18 * weight + reloadDip + swapDip) * motion;
          const bobX = (h.kick * -6 * weight + (reloading ? 20 : 0)) * motion;
          const roll = h.kick * -1.8 * weight * motion;
          weapEl.style.transform = `translate(-50%, ${bobY}px) translateX(${bobX}px) rotate(${roll}deg)`;
          weapEl.style.filter = h.muzzle > 0.05 ? `brightness(${1 + h.muzzle * 0.22})` : "";
          const wpn = WEAPONS[h.weapon] ?? WEAPONS[0]!;
          const fr = h.weapFrame | 0;
          if (fr >= 5) {
            weapEl.style.backgroundImage = `url(${asset(wpn.reload)})`;
            const pistolReload = h.weapon === 0;
            weapEl.style.backgroundSize = pistolReload ? "200% 200%" : "400% 200%";
            weapEl.style.backgroundPosition = pistolReload
              ? sheetPos(Math.min(3, fr - 5))
              : gridPos(Math.min(7, fr - 5), 4, 2);
          } else if (fr >= 1) {
            weapEl.style.backgroundImage = `url(${asset(wpn.fire)})`;
            weapEl.style.backgroundSize = "200% 200%";
            weapEl.style.backgroundPosition = sheetPos(Math.min(3, fr - 1));
          } else {
            weapEl.style.backgroundImage = `url(${asset(wpn.idle)})`;
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
          h.hasW8,
          h.hasW9,
          h.hasW10,
          h.hasW11,
          h.objective,
          h.radioSeq,
          h.bossPhase,
          h.reloading > 0.001,
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
          if (!qaRef.current) {
            const save = rt.exportSave();
            if (save) {
              // Winning keeps every unlocked weapon, restores full health
              // and full ammo, and preserves armor clamped to [0, 100].
              const sizes = [12, 8, 36, 5, 4, 10, 90, 6, 4, 14, 5];
              const full = [120, 48, 216, 20, 16, 80, 450, 36, 24, 84, 30];
              const mag = [...save.mag, ...Array(11).fill(0)].slice(0, 11);
              const ammo = [...save.ammo, ...Array(11).fill(0)].slice(0, 11);
              mag[0] = sizes[0]!;
              ammo[0] = full[0]!;
              for (let i = 1; i < 11; i++) {
                if (save.flags & (1 << (i - 1))) {
                  mag[i] = sizes[i]!;
                  ammo[i] = full[i]!;
                }
              }
              persistRef.current({
                ...save,
                wave: Math.min(999, save.wave + 1),
                health: 100,
                armor: Math.min(100, Math.max(0, save.armor)),
                mag,
                ammo,
              });
            }
          }
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

  useLayoutEffect(() => {
    const root = document.documentElement;
    root.style.setProperty("--ui-plaque", `url("${asset("/game/ui-plaque.svg")}")`);
    root.style.setProperty("--hud-panel", `url("${asset("/game/ui/hud-panel.webp")}")`);
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
    clearCheckpoint();
    setCheckpoint(null);
    rt.restart();
    rt.kick();
    rt.setPlaying(true);
    rt.setSens(sens);
    rt.setMuted(muted);
    rt.setVolumes(vol.master, vol.music, vol.sfx, vol.menu);
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

  const continueRun = useCallback(() => {
    const save = loadCheckpoint();
    const rt = rtRef.current;
    if (!save || !rt) return;
    rt.loadSave(save);
    rt.kick();
    rt.setPlaying(true);
    rt.setSens(sens);
    rt.setMuted(muted);
    rt.setVolumes(vol.master, vol.music, vol.sfx, vol.menu);
    setScreen("play");
    rt.requestLock();
  }, [sens, muted, vol]);

  const readMap = useCallback(() => rtRef.current?.readMap() ?? null, []);
  const getEnemies = useCallback(() => rtRef.current?.getEnemies() ?? [], []);
  const getBars = useCallback(() => rtRef.current?.getBars() ?? [], []);

  useEffect(() => {
    if (screen !== "play" || qaRef.current) return;
    const save = rtRef.current?.exportSave();
    if (save) persistRef.current(save);
  }, [hud.wave, screen]);

  useEffect(() => {
    if (screen !== "play" || !hud.radioSeq) return;
    const copy = radioCopy(hud.radioLine, hud.wave);
    if (!copy) return;
    setRadio(copy);
    const id = window.setTimeout(() => setRadio(null), 5600);
    return () => window.clearTimeout(id);
  }, [hud.radioSeq, hud.radioLine, hud.wave, screen]);
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
    // Standby bed (bgm-menu.ogg, cut from the 31-minute bgm.ogg) under
    // menus and end cards; the field mix takes over on play via setMusic.
    rtRef.current?.setMenuBed(screen !== "play");
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
      localStorage.setItem("blacksite-res", res.id);
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
    rtRef.current?.setVolumes(vol.master, vol.music, vol.sfx, vol.menu);
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
          <Automap hud={hud} readMap={readMap} getEnemies={getEnemies} />
          <EnemyBars getBars={getBars} />
          {radio && <RadioCard speaker={radio.speaker} text={radio.text} />}
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
          {(hud.prompt === 13 || hud.prompt === 15) && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-danger">
              {hud.prompt === 13 ? "USE E — SECTOR NODE" : "FIND THE SECTOR NODE"}
            </p>
          )}
          {hud.prompt === 14 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-steel">
              USE E — READ TERMINAL
            </p>
          )}
          {hud.prompt === 3 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-danger">
              USE E — INITIATE OVERRIDE
            </p>
          )}
          {hud.prompt === 16 && (
            <p className="pointer-events-none absolute bottom-28 left-1/2 -translate-x-1/2 font-display text-sm tracking-[0.2em] text-primary">
              CLAIM THE{" "}
              {
                [WEAPONS[8], WEAPONS[9], WEAPONS[10]][
                  (Math.max(1, hud.wave) - 1) % 3
                ]!.name
              }{" "}
              — SECTOR OPEN
            </p>
          )}
          {hud.prompt >= 7 && hud.prompt <= 12 && (
            <p className="pointer-events-none absolute left-1/2 top-1/4 -translate-x-1/2 border border-danger bg-bg/90 px-6 py-4 text-center font-display text-sm tracking-[0.2em] text-danger">
              {[
                "VAULT LOCKDOWN · COMMAND SIGNAL DETECTED",
                "VEYRAN INBOUND · BRACE FOR CONTACT",
                "FOUNDRY POWER SURGE · DRONES ONLINE",
                "HECATE–9 ACTIVATING · AVOID THE ARC",
                "CONTAINMENT BREACH · BIOLOCK FAILED",
                "CHIMERA–9 RELEASED · KEEP MOVING",
              ][hud.prompt - 7]}
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
            style={{ backgroundImage: `url(${asset("/game/ui/menu-reactor.webp")})` }}
          />
          <div className="operation-shade" />
          <section className={`operation-content ${screen !== "menu" ? "operation-card" : ""}`}>
            {screen === "menu" && (
              <Menu
                enemyOptions={enemyOptions}
                setEnemyOptions={setEnemyOptions}
                onPreviewVoice={(skin) => rtRef.current?.previewEnemy(skin)}
                onStart={start}
                onContinue={checkpoint ? continueRun : undefined}
                ready={ready}
                load={load}
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
                onResume={checkpoint ? continueRun : undefined}
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
