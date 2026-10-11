// Only visible targets are eligible. Keep a nearby lock to avoid oscillating aim,
// but immediately respond to a close threat or a substantially nearer enemy.
export function combatTarget(visible, previousId) {
  const nearest = visible[0];
  if (!nearest || nearest.distance < 3) return nearest;
  const previous = visible.find((enemy) => enemy.id === previousId);
  return previous && previous.distance <= nearest.distance * 1.2 ? previous : nearest;
}

// Reserve counts have different meanings for a pistol and a launcher. Compare
// each owned weapon against its own magazine size instead of pooling all rounds.
export function needsAmmo(inventory) {
  return inventory.some((gun) => gun.magazine + gun.reserve < gun.capacity * 2);
}

// Mouse movement is applied once before the six-frame input interval. Wait for
// the residual aim error to shrink before spending ammunition (radians).
export function combatAim(turn) {
  const delta = Math.max(-0.45, Math.min(0.45, turn));
  return { delta, aligned: Math.abs(turn - delta) <= 0.08 };
}

export function availableWeapons(inventory) {
  return inventory
    .filter((gun) => gun.magazine + gun.reserve > 0)
    .sort(
      (a, b) =>
        Number(b.magazine > 0) - Number(a.magazine > 0) ||
        Number(b.id !== 1) - Number(a.id !== 1) ||
        b.magazine + b.reserve - (a.magazine + a.reserve) ||
        a.id - b.id,
    );
}
