import { cn } from "@/lib/utils";

export function Crosshair({
  flash,
  spread,
  danger,
}: {
  flash: number;
  spread: number;
  danger?: boolean;
}) {
  const on = flash > 0.2;
  const killed = flash > 1.0;
  return (
    <div className="pointer-events-none absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
      <div
        className={cn(
          "crosshair relative size-5",
          danger ? "text-primary" : on ? "text-danger" : "text-fg",
        )}
        style={{ width: 20 + spread * 100, height: 20 + spread * 100 }}
      >
        <span className="absolute left-1/2 top-0 h-1.5 w-px -translate-x-1/2 bg-current" />
        <span className="absolute bottom-0 left-1/2 h-1.5 w-px -translate-x-1/2 bg-current" />
        <span className="absolute left-0 top-1/2 h-px w-1.5 -translate-y-1/2 bg-current" />
        <span className="absolute right-0 top-1/2 h-px w-1.5 -translate-y-1/2 bg-current" />
        {on && <span className={cn("hit-confirm", killed && "hit-confirm-kill")}>×</span>}
        {killed && <span className="kill-confirm">KILL</span>}
        {danger && !on && <span className="kill-confirm">BLAST</span>}
      </div>
    </div>
  );
}
