/**
 * Security guard for the Asset Catalog.
 * The catalog is strictly deactivated in production environments
 * and is accessible only from the local Vite development server.
 */
export function isCatalogEnabled(): boolean {
  if (typeof window === "undefined") {
    return false;
  }

  const hostname = window.location.hostname.toLowerCase();

  const isLocalHost =
    hostname === "localhost" ||
    hostname === "127.0.0.1" ||
    hostname === "0.0.0.0";
  return Boolean(import.meta.env.DEV) && isLocalHost;
}
