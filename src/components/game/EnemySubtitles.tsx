import type { EnemySubtitle } from "@/game/enemy-presentation";

export function EnemySubtitles({ lines, size }: { lines: EnemySubtitle[]; size: number }) {
  return (
    <div className="enemy-subtitles" aria-live="polite" aria-atomic="true">
      {lines.map((line) => (
        <div
          key={line.id}
          className="enemy-caption"
          style={{ left: `${line.x}%`, top: `${line.y}%`, fontSize: `${size}rem` }}
        >
          <span className="caption-speaker">{line.name}</span>
          <span>{line.text}</span>
        </div>
      ))}
    </div>
  );
}
