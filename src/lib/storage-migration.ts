/** Preserve browser-local progress while moving the five original storage keys. */
export function migrateGameStorage(storage: Pick<Storage, "getItem" | "setItem" | "removeItem">) {
  // This retired prefix is used only to import existing player data.
  const legacyPrefix = "hellscan";
  for (const suffix of ["vol", "board", "gpu", "gfx", "checkpoint"]) {
    try {
      const oldKey = `${legacyPrefix}-${suffix}`;
      const value = storage.getItem(oldKey);
      if (value === null) continue;
      const newKey = `blacksite-${suffix}`;
      if (storage.getItem(newKey) === null) storage.setItem(newKey, value);
      storage.removeItem(oldKey);
    } catch {
      // Keep the original when storage is blocked or the new write fails.
    }
  }
}

export function readGameStorage(key: string): string | null {
  migrateGameStorage(localStorage);
  return localStorage.getItem(key);
}
