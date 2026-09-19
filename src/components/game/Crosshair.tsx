import { cn } from "@/lib/utils";

export function Crosshair({ flash, spread }: { flash: number; spread: number }) {
  const on = flash > 0.2;
  return (
    <div className="pointer-events-none absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
      <div
        className={cn("crosshair relative size-5", on ? "text-danger" : "text-fg")}
        style={{ width: 20 + spread * 100, height: 20 + spread * 100 }}
      >
        <span className="absolute left-1/2 top-0 h-1.5 w-px -translate-x-1/2 bg-current" />
        <span className="absolute bottom-0 left-1/2 h-1.5 w-px -translate-x-1/2 bg-current" />
        <span className="absolute left-0 top-1/2 h-px w-1.5 -translate-y-1/2 bg-current" />
        <span className="absolute right-0 top-1/2 h-px w-1.5 -translate-y-1/2 bg-current" />
        {on && (
          <span className="absolute left-1/2 top-1/2 h-3 w-3 -translate-x-1/2 -translate-y-1/2 border border-danger" />
        )}
      </div>
    </div>
  );
}
