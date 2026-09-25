import { useEffect, useState } from "react";
import type { BarCue } from "@/game/enemy-presentation";

const LAYER = ["health", "armor", "shield"] as const;

export function EnemyBars({ getBars }: { getBars: () => BarCue[] }) {
  const [bars, setBars] = useState<BarCue[]>([]);

  useEffect(() => {
    let raf = 0;
    const tick = () => {
      setBars(getBars());
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [getBars]);

  if (bars.length === 0) return null;
  return (
    <div className="enemy-bars" aria-hidden="true">
      {bars.map((bar, i) => (
        <div
          key={i}
          className={`enemy-bar enemy-bar-${LAYER[bar.layer] ?? "health"}`}
          style={{
            left: `${bar.screenX * 100}%`,
            top: `${bar.screenY * 100}%`,
            opacity: bar.fade,
          }}
        >
          <i style={{ width: `${Math.max(0, Math.min(100, bar.frac * 100))}%` }} />
        </div>
      ))}
    </div>
  );
}
