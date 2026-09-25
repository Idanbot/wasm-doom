import { asset } from "@/lib/asset";
import { WEAPONS } from "./data";

export function WeaponView({
  weaponRef,
}: {
  weaponRef: { current: HTMLDivElement | null };
}) {
  // The weapon sheet is owned imperatively by the runtime frame loop
  // (idle / fire / reload cells switch every frame). React must not set
  // backgroundImage here: re-rendering on HUD state would reset the sheet
  // to idle mid-burst and kill fire/reload animation with flicker.
  return (
    <div
      ref={weaponRef}
      className="weapon-view pointer-events-none absolute bottom-[-2%] left-1/2 origin-bottom select-none"
      style={{
        backgroundImage: `url(${asset(WEAPONS[0]!.idle)})`,
        backgroundSize: "contain",
        backgroundPosition: "center bottom",
        backgroundRepeat: "no-repeat",
        transform: "translate(-50%, 0)",
      }}
    />
  );
}
