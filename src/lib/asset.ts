/** Prefix a root-absolute public path with Vite's `base` (GitHub project pages). */
export function asset(path: string): string {
  if (!path.startsWith("/")) return path;
  const base = import.meta.env?.BASE_URL ?? "/";
  if (!base || base === "/") return path;
  const prefix = base.endsWith("/") ? base.slice(0, -1) : base;
  if (path === prefix || path.startsWith(`${prefix}/`)) return path;
  return `${prefix}${path}`;
}
