/** Preserve row-major world coordinates, but show walkable cells as floor. */
export function automapCells(map: Uint8Array, doors: Float32Array): Uint8Array {
  return map.map((cell, i) =>
    cell === 10 || ((cell === 8 || cell === 9) && (doors[i] ?? 0) >= 0.98)
      ? 0 : cell,
  );
}
