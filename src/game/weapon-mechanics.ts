/** Player-facing identities for the ten reworked boss rewards. */
export const BOSS_WEAPON_ROLES: Readonly<Record<number, string>> = {
  11: "Mark, then rupture · repeat within 2.5s",
  12: "Lead pulse · delayed precision strike",
  13: "Gravity scatter · banked pressure",
  14: "Freeze, then shatter · two volleys",
  15: "Chain arc · jumps to 3 nearby hostiles",
  17: "Tracking burst · three timed rounds",
  20: "Harpoon pull · banked pressure",
  22: "Magnetic lane · strips armor, pierces 3",
  28: "Solar pulses · banked pressure",
  30: "Echo marks · two delayed area strikes",
};

export function hasPressureBank(slot: number) {
  return slot === 13 || slot === 20 || slot === 28;
}

export function pressureHint(pressure: number, venting: number, ammo: number) {
  if (venting > 0) return ammo > 0 && pressure >= 0.2 ? "FIRE TO DISCHARGE · WAIT TO COOL" : "VENTING SAFELY";
  if (pressure >= 0.999) return "PRESS R TO VENT · PRESSURE LIMIT";
  return "R TO VENT · FIRE DURING VENT FOR BLAST";
}
