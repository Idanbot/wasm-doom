import { useState } from "react";
import {
  accuracy,
  decodeRouteMap,
  DIFFICULTIES,
  type Difficulty,
  type SectorRecord,
} from "@/game/run-record";
import { fmtTime, sectorForWave, type Score } from "./data";

function SectorRoute({ sector }: { sector: SectorRecord }) {
  const map = decodeRouteMap(sector.map);
  const walls = map
    .map((cell, i) =>
      cell !== 0 && cell !== 10 && cell !== 8 && cell !== 9
        ? `M${i % 48} ${Math.floor(i / 48)}h1v1h-1z`
        : "",
    )
    .join("");
  const start = sector.route[0],
    end = sector.route.at(-1);
  return (
    <svg
      viewBox="0 0 48 32"
      role="img"
      aria-label={`Traced player route through sector ${sector.sector}`}
      className="run-route"
    >
      <rect width="48" height="32" fill="#0b1720" />
      <path d={walls} fill="#304450" />
      <polyline
        points={sector.route.map((p) => p.join(",")).join(" ")}
        fill="none"
        stroke="#fbbf24"
        strokeWidth="0.35"
        strokeLinejoin="round"
      />
      {start && <circle cx={start[0]} cy={start[1]} r="0.6" fill="#34d399" />}
      {end && <circle cx={end[0]} cy={end[1]} r="0.6" fill="#f87171" />}
    </svg>
  );
}

export function RunBoard({
  board,
  initialDifficulty,
}: {
  board: Score[];
  initialDifficulty: Difficulty;
}) {
  const [selected, setSelected] = useState(initialDifficulty);
  const rows = board.filter(
    (row) => (row.difficulty ?? row.run?.difficulty ?? "operative") === selected,
  );
  return (
    <div className="run-board">
      <div className="difficulty-tabs" aria-label="Leaderboard difficulty">
        {DIFFICULTIES.map((p) => (
          <button
            type="button"
            key={p.id}
            aria-pressed={selected === p.id}
            onClick={() => setSelected(p.id)}
          >
            {p.name}
          </button>
        ))}
      </div>
      <p className="run-board-caption">
        LOCAL FIELD RECORDS · furthest sector, kills, then fastest time
      </p>
      {rows.length === 0 && <p>No {selected} runs recorded.</p>}
      {rows.map((row, i) => (
        <RunCard key={`${row.recordedAt}-${row.name}-${i}-${selected}`} row={row} i={i} />
      ))}
    </div>
  );
}

function RunCard({ row, i }: { row: Score; i: number }) {
  const [open, setOpen] = useState(false);
  return (
    <details className="run-card" onToggle={(e) => setOpen(e.currentTarget.open)}>
      <summary>
        <span>
          {i + 1}. {row.name} · SECTOR {row.wave}
        </span>
        <span>
          {row.kills} kills · {fmtTime(row.time)}
        </span>
      </summary>
      {open && (
        <div className="run-card-body">
          {row.run ? (
            <>
              <dl className="run-totals">
                <div>
                  <dt>Ammo spent</dt>
                  <dd>{row.run.ammoSpent}</dd>
                </div>
                <div>
                  <dt>Damage taken</dt>
                  <dd>{row.run.damageTaken}</dd>
                </div>
                <div>
                  <dt>Accuracy</dt>
                  <dd>{accuracy(row.run).toFixed(1)}%</dd>
                </div>
                <div>
                  <dt>Total kills</dt>
                  <dd>{row.kills}</dd>
                </div>
                <div>
                  <dt>Bosses defeated</dt>
                  <dd>{row.run.bosses}</dd>
                </div>
                <div>
                  <dt>Total time</dt>
                  <dd>{fmtTime(row.time)}</dd>
                </div>
              </dl>
              {row.run.assisted && <p>Assisted / local QA run</p>}
              <p className="run-route-key">
                Route: green start · amber path · red endpoint. Sampled approximately once per
                second.
              </p>
              {row.run.sectors.map((s) => (
                <details className="run-sector" key={s.sector}>
                  <summary>
                    Sector {s.sector} · {sectorForWave(s.sector).name} · {fmtTime(s.time)} ·{" "}
                    {s.kills} kills · {s.completed ? "Cleared" : "Unfinished"}
                  </summary>
                  {s.route.length > 0 ? <SectorRoute sector={s} /> : <p>Route unavailable.</p>}
                </details>
              ))}
            </>
          ) : (
            <p>Legacy record: detailed combat statistics and routes were not recorded.</p>
          )}
        </div>
      )}
    </details>
  );
}
