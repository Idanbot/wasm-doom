import type { HudState } from "@/game/types";
import { WEAPONS } from "./data";

export function WeaponView({
  hud,
  weaponRef,
}: {
  hud: HudState;
  weaponRef: { current: HTMLDivElement | null };
}) {
  const w = WEAPONS[hud.weapon] ?? WEAPONS[0]!;
  return (
    <div
      ref={weaponRef}
      className="weapon-view pointer-events-none absolute bottom-[-2%] left-1/2 origin-bottom select-none"
      style={{
        backgroundImage: `url(${w.idle})`,
        backgroundSize: "contain",
        backgroundPosition: "center bottom",
        backgroundRepeat: "no-repeat",
        transform: "translate(-50%, 0)",
      }}
    />
  );
}
