import { useEffect, useState } from "react";
import type { HudState } from "@/game/types";

/** One first-play reminder, after three seconds moving with an empty magazine. */
export function ReloadHint({ hud }: { hud: HudState }) {
  const [seen, setSeen] = useState(() => {
    try {
      return localStorage.getItem("blacksite-reload-hint-seen") === "1";
    } catch {
      return false;
    }
  });
  const [visible, setVisible] = useState(false);
  const eligible =
    !seen && hud.ammo === 0 && hud.reserve > 0 && hud.reloading <= 0 && hud.speed > 0.05;
  useEffect(() => {
    if (!eligible) return;
    const timer = setTimeout(() => {
      setSeen(true);
      setVisible(true);
      try {
        localStorage.setItem("blacksite-reload-hint-seen", "1");
      } catch {
        /* session only */
      }
    }, 3000);
    return () => clearTimeout(timer);
  }, [eligible]);
  useEffect(() => {
    if (!visible) return;
    const timer = setTimeout(() => setVisible(false), 3000);
    return () => clearTimeout(timer);
  }, [visible]);
  return visible ? (
    <div className="reload-hint" role="status">
      Press <kbd>R</kbd> to reload
    </div>
  ) : null;
}
