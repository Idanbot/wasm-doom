import { useEffect, useRef } from "react";
import type { HudState } from "@/game/types";
import type { EnemyCue } from "@/game/enemy-presentation";

const MAP_W = 48;
const MAP_H = 32;

export function Automap({
  hud,
  readMap,
  getEnemies,
}: {
  hud: HudState;
  readMap: () => Uint8Array | null;
  getEnemies: () => EnemyCue[];
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const seen = useRef(new Uint8Array(MAP_W * MAP_H));
  const wave = useRef(hud.wave);

  useEffect(() => {
    if (wave.current !== hud.wave) {
      wave.current = hud.wave;
      seen.current.fill(0);
    }
    const canvas = canvasRef.current;
    const map = readMap();
    if (!canvas || !map) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const px = Math.floor(hud.x);
    const py = Math.floor(hud.y);
    for (let y = py - 5; y <= py + 5; y++) {
      for (let x = px - 5; x <= px + 5; x++) {
        if (x < 0 || y < 0 || x >= MAP_W || y >= MAP_H) continue;
        seen.current[y * MAP_W + x] = 1;
      }
    }
    const scale = 5;
    canvas.width = MAP_W * scale;
    canvas.height = MAP_H * scale;
    const token = getComputedStyle(canvas);
    const open = token.getPropertyValue("--color-bg").trim();
    const wall = token.getPropertyValue("--color-elevated").trim();
    const door = token.getPropertyValue("--color-steel").trim();
    const secret = token.getPropertyValue("--color-danger").trim();
    const node = token.getPropertyValue("--color-primary").trim();
    const player = token.getPropertyValue("--color-fg").trim();
    const hostile = token.getPropertyValue("--color-danger").trim() || "#e23b2f";
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    for (let i = 0; i < map.length; i++) {
      if (!seen.current[i]) continue;
      const cell = map[i] ?? 0;
      const x = (i % MAP_W) * scale;
      const y = Math.floor(i / MAP_W) * scale;
      ctx.fillStyle = cell === 0 ? open : cell === 8 ? door : cell === 9 ? secret : wall;
      ctx.fillRect(x, y, scale, scale);
    }
    const nx = Math.floor(hud.nodeX);
    const ny = Math.floor(hud.nodeY);
    if (nx >= 0 && ny >= 0 && seen.current[ny * MAP_W + nx]) {
      ctx.fillStyle = hud.objective ? door : node;
      ctx.fillRect(nx * scale, ny * scale, scale, scale);
    }
    for (const enemy of getEnemies()) {
      if (enemy.hp <= 0) continue;
      const ex = Math.floor(enemy.x);
      const ey = Math.floor(enemy.y);
      if (ex < 0 || ey < 0 || ex >= MAP_W || ey >= MAP_H) continue;
      const near = Math.abs(ex - px) <= 7 && Math.abs(ey - py) <= 7;
      if (!near && !seen.current[ey * MAP_W + ex]) continue;
      ctx.fillStyle = hostile;
      ctx.beginPath();
      ctx.arc(enemy.x * scale, enemy.y * scale, 2.4, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.save();
    ctx.translate(hud.x * scale, hud.y * scale);
    ctx.rotate(hud.yaw);
    ctx.fillStyle = player;
    ctx.beginPath();
    ctx.moveTo(5.2, 0);
    ctx.lineTo(-3.4, 2.4);
    ctx.lineTo(-3.4, -2.4);
    ctx.closePath();
    ctx.fill();
    ctx.restore();
  }, [hud.x, hud.y, hud.yaw, hud.wave, hud.nodeX, hud.nodeY, hud.objective, hud.living, hud.elapsedMs, readMap, getEnemies]);

  return <canvas ref={canvasRef} className="automap" aria-label="Sector map" />;
}
