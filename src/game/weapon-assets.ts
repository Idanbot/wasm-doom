/** Shared order for viewmodels and the mouse-wheel arsenal. */
export const WEAPON_SLUGS = [
  "mk23s", "br12", "kx9", "mr4", "vlk6", "ax12", "m91", "hx8", "vr9", "hc9", "cm9", "ar6", "or7", "gs4", "cr3", "sr0", "ts12", "ks8", "mn6",
] as const;

export const WEAPON_SHEETS = WEAPON_SLUGS.map((slug) => `/game/draft/v2/weap_${slug}_5x5.png`);
export const WEAPON_THUMBNAILS = WEAPON_SLUGS.map((slug) => `/game/ui/weapon-thumbs/${slug}.png`);
