import { CAMPAIGN_EXPANSION } from "./campaign25.ts";

/** Shared order for viewmodels and the mouse-wheel arsenal. */
export const WEAPON_SLUGS = [
  "mk23s", "br12", "kx9", "mr4", "vlk6", "ax12", "m91", "hx8", "vr9", "hc9", "cm9", "ar6", "or7", "gs4", "cr3", "sr0", "ts12", "ks8", "mn6", ...CAMPAIGN_EXPANSION.map(boss => boss.weapon),
] as const;

export const WEAPON_SHEETS = WEAPON_SLUGS.map((slug) => `/game/draft/v2/weap_${slug}_5x5.png`);
export const WEAPON_THUMBNAILS = WEAPON_SLUGS.map((slug) => `/game/ui/weapon-thumbs/${slug}.png`);

/** Every removable reload feed has its own tumbling artwork; BR-12 has a fixed tube. */
export const WEAPON_FEED_FILES = WEAPON_SLUGS.map((slug, slot) => {
  if (slot === 1) return null;
  const containers: Record<string, string> = {
    hx8: "fuel", cm9: "acid", my9: "specimen", lv8: "pressure",
  };
  return containers[slug]
    ? `/game/fx/${containers[slug]}-debris-hd.png`
    : `/game/fx/feeds/${slug}.png`;
});
