import { useRef } from "react";

export function TouchLayer({
  onMove,
  onLook,
  onFire,
  onUse,
  onReload,
  onWeapon,
  lookPtr,
  movePtr,
}: {
  onMove: (x: number, y: number) => void;
  onLook: (dx: number, dy: number) => void;
  onFire: (v: boolean) => void;
  onUse: () => void;
  onReload: () => void;
  onWeapon: () => void;
  lookPtr: { current: number | null };
  movePtr: { current: number | null };
}) {
  const moveOrigin = useRef({ x: 0, y: 0 });
  const lookOrigin = useRef({ x: 0, y: 0 });
  return (
    <div className="absolute inset-0">
      <div
        className="absolute bottom-0 left-0 h-[46%] w-[46%]"
        onPointerDown={(e) => {
          movePtr.current = e.pointerId;
          moveOrigin.current = { x: e.clientX, y: e.clientY };
          (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (movePtr.current !== e.pointerId) return;
          const dx = (e.clientX - moveOrigin.current.x) / 70;
          const dy = (e.clientY - moveOrigin.current.y) / 70;
          onMove(Math.max(-1, Math.min(1, dx)), Math.max(-1, Math.min(1, dy)));
        }}
        onPointerUp={(e) => {
          if (movePtr.current === e.pointerId) {
            movePtr.current = null;
            onMove(0, 0);
          }
        }}
        onPointerCancel={() => {
          movePtr.current = null;
          onMove(0, 0);
        }}
      />
      <div
        className="absolute bottom-0 right-0 h-[70%] w-[54%]"
        onPointerDown={(e) => {
          lookPtr.current = e.pointerId;
          lookOrigin.current = { x: e.clientX, y: e.clientY };
          (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (lookPtr.current !== e.pointerId) return;
          onLook(e.clientX - lookOrigin.current.x, e.clientY - lookOrigin.current.y);
          lookOrigin.current = { x: e.clientX, y: e.clientY };
        }}
        onPointerUp={(e) => {
          if (lookPtr.current === e.pointerId) lookPtr.current = null;
        }}
        onPointerCancel={() => {
          lookPtr.current = null;
        }}
      />
      <button
        type="button"
        className="absolute right-4 bottom-28 size-16 rounded-full border border-border bg-elevated/80 font-display text-xs tracking-[0.18em] text-fg"
        onPointerDown={(e) => {
          e.currentTarget.setPointerCapture(e.pointerId);
          onFire(true);
        }}
        onLostPointerCapture={() => onFire(false)}
        onPointerUp={() => onFire(false)}
        onPointerCancel={() => onFire(false)}
      >
        FIRE
      </button>
      <button
        type="button"
        aria-label="Next weapon"
        className="absolute right-4 bottom-48 size-12 rounded-full border border-border bg-elevated/80 font-display text-[10px] tracking-[0.16em] text-muted"
        onClick={onWeapon}
      >
        WPN
      </button>
      <button
        type="button"
        className="absolute right-24 bottom-20 size-12 rounded-full border border-border bg-elevated/80 font-display text-[10px] tracking-[0.16em] text-muted"
        onClick={onUse}
      >
        USE
      </button>
      <button
        type="button"
        className="absolute right-24 bottom-36 size-12 rounded-full border border-border bg-elevated/80 font-display text-[10px] tracking-[0.16em] text-muted"
        onClick={onReload}
      >
        RLD
      </button>
    </div>
  );
}
