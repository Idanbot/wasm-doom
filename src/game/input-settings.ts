export const SENSITIVITY_MIN = 0.1;
export const SENSITIVITY_MAX = 3.0;
export const DEFAULT_SENSITIVITY = 0.5;

export function normalizeSensitivity(value: number | string | null): number {
  if (value === null || value === "") return DEFAULT_SENSITIVITY;
  const n = Number(value);
  return Number.isFinite(n)
    ? Math.max(SENSITIVITY_MIN, Math.min(SENSITIVITY_MAX, n))
    : DEFAULT_SENSITIVITY;
}

/** The old default was saved automatically, even without touching settings. */
export function readSensitivity(storage: Pick<Storage, "getItem">): number {
  const value = storage.getItem("blacksite-sensitivity");
  if (!storage.getItem("blacksite-sensitivity-version") && Number(value) === 1.4)
    return DEFAULT_SENSITIVITY;
  return normalizeSensitivity(value);
}
