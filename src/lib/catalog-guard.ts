/**
 * Security guard for the Asset Catalog.
 * The catalog is strictly deactivated in production environments (such as idanbot.me/wasm-doom)
 * and is accessible ONLY during local development (localhost, 127.0.0.1, 0.0.0.0, or DEV mode).
 */
export function isCatalogEnabled(): boolean {
  if (typeof window === "undefined") {
    return process.env.NODE_ENV !== "production";
  }

  const hostname = window.location.hostname.toLowerCase();

  // Explicitly deny production domains
  if (hostname.includes("idanbot.me") || hostname.includes("github.io")) {
    return false;
  }

  // Allowed only on local dev addresses and dev sandboxes
  const isLocalHost =
    hostname === "localhost" ||
    hostname === "127.0.0.1" ||
    hostname === "0.0.0.0" ||
    hostname.endsWith(".local") ||
    hostname.includes("preview") ||
    hostname.includes("sandbox");

  // In non-local environments, verify Vite DEV flag
  const isDevMode = Boolean(import.meta.env?.DEV);

  return isLocalHost || isDevMode;
}
