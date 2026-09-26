import { asset } from "@/lib/asset";
import { WEAPONS } from "./data";

export function WeaponView({
  weaponRef,
  missing,
  name,
}: {
  weaponRef: { current: HTMLDivElement | null };
  missing?: boolean;
  name?: string;
}) {
  // The weapon sheet is owned imperatively by the runtime frame loop
  // (idle / fire / reload cells switch every frame). React must not set
  // backgroundImage here: re-rendering on HUD state would reset the sheet
  // to idle mid-burst and kill fire/reload animation with flicker.
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
      style={{
        backgroundImage: `url(${asset(WEAPONS[0]!.sheet)})`,
        backgroundSize: "500% 500%",
        backgroundPosition: "0% 0%",
        backgroundRepeat: "no-repeat",
        transform: "translate(-50%, 0)",
      }}
    />
  );
}
