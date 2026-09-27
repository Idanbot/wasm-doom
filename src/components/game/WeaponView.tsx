export function WeaponView({
  weaponRef,
  missing,
  name,
}: {
  weaponRef: { current: HTMLDivElement | null };
  missing?: boolean;
  name?: string;
}) {
  // The frame loop draws one 512x384 sheet cell into this canvas. Drawing the
  // full 5x5 sheet as a 500% CSS background delayed first paint in production.
  if (missing) {
    // Art failed to decode: show a labeled plate instead of nothing.
    return (
      <div
        ref={weaponRef}
        className="weapon-view weapon-missing pointer-events-none absolute bottom-[-2%] left-1/2 origin-bottom select-none"
        style={{ transform: "translate(-50%, 0)" }}
      >
        <span>{name ?? "WEAPON"}</span>
      </div>
    );
  }
  return (
    <div
      ref={weaponRef}
      className="weapon-view pointer-events-none absolute bottom-[-2%] left-1/2 origin-bottom select-none"
      style={{ transform: "translate(-50%, 0)" }}
    >
      <canvas width={512} height={384} aria-hidden="true" />
    </div>
  );
}
