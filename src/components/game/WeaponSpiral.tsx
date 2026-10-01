import type { HudState } from "@/game/types";
import { WEAPON_THUMBNAILS } from "@/game/weapon-assets";
import { asset } from "@/lib/asset";
import { WEAPONS } from "./data";

export function WeaponSpiral({ hud, visible }: { hud: HudState; visible: boolean }) {
  const owned = [true, hud.hasW2, hud.hasW3, hud.hasW4, hud.hasW5, hud.hasW6, hud.hasW7, hud.hasW8, hud.hasW9, hud.hasW10, hud.hasW11, hud.hasW12, hud.hasW13, hud.hasW14, hud.hasW15, hud.hasW16, hud.hasW17, hud.hasW18, hud.hasW19];
  const slots = WEAPONS.map((weapon, index) => ({ weapon, index })).filter(({ index }) => owned[index]);
  const selected = slots.findIndex(({ index }) => index === hud.weapon);
  return (
    <div className={`weapon-spiral ${visible ? "visible" : ""}`} aria-hidden={!visible}>
      <span className="weapon-spiral-caption">ARSENAL · MOUSE WHEEL / NUMBER KEYS</span>
      <div className="weapon-spiral-orbit">
        {slots.map(({ weapon, index }, order) => {
          const distance = ((order - selected + slots.length + Math.floor(slots.length / 2)) % slots.length) - Math.floor(slots.length / 2);
          if (Math.abs(distance) > 2) return null;
          return (
            <div
              className={`weapon-spiral-slot ${distance === 0 ? "active" : ""}`}
              key={weapon.id}
              style={{
                transform: `translate(${distance * 92}px, ${Math.abs(distance) * 18 - 10}px) scale(${1 - Math.abs(distance) * 0.14}) rotate(${distance * -8}deg)`,
                zIndex: 5 - Math.abs(distance),
                opacity: distance === 0 ? 1 : 0.5,
              }}
            >
              <img src={asset(WEAPON_THUMBNAILS[index]!)} alt="" loading="eager" decoding="sync" />
              <span>{index + 1} · {weapon.name.split(" ")[0]}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
