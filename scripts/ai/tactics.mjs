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
