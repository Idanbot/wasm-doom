import { assetDigest } from "./asset-digest.ts";
export type AssetManifest = { version: 1; build: string; assets: Record<string, { bytes: number; sha256: string }> };
const CACHE = "blacksite-validated-assets-v1";
export const CACHE_BUDGET = 384 * 1024 * 1024;
let manifest: AssetManifest | undefined;

export function setAssetManifest(value: AssetManifest) {
  if (value.version !== 1 || !/^[a-f0-9]{64}$/.test(value.build) || !value.assets || Array.isArray(value.assets) || !Object.keys(value.assets).length) throw new Error("Invalid asset manifest");
  for (const [url, entry] of Object.entries(value.assets)) {
    if (!url.startsWith("/game/") && url !== "/blacksite.wasm") throw new Error("Invalid asset path");
    if (!Number.isSafeInteger(entry.bytes) || entry.bytes <= 0 || !/^[a-f0-9]{64}$/.test(entry.sha256)) throw new Error("Invalid asset identity");
  }
  manifest = value;
}

function entryFor(url: string) {
  const pathname = url.split(/[?#]/, 1)[0]!;
  const start = pathname.indexOf("/game/");
  return manifest?.assets[start >= 0 ? pathname.slice(start) : pathname.endsWith("/blacksite.wasm") ? "/blacksite.wasm" : pathname];
}
export function assetGroupBytes(include: (url: string) => boolean) {
  return Object.entries(manifest?.assets ?? {}).reduce((sum,[url,entry])=>sum+(include(url)?entry.bytes:0),0);
}
export function assetBytes(url: string) { return entryFor(url)?.bytes ?? 1; }
export function versionedAssetUrl(url: string) {
  const [path,hash]=url.split("#",2);
  const entry = entryFor(path!);
  return entry ? `${path}${path!.includes("?") ? "&" : "?"}blacksite_version=${entry.sha256}${hash ? `#${hash}` : ""}` : url;
}


/** Hash before decode; write only after decode/parse succeeded. Storage is optional. */
export async function validatedAsset<T>(url: string, attempt: number, signal: AbortSignal,
  decode: (response: Response, signal: AbortSignal) => Promise<T>, requestCache?: RequestCache): Promise<T> {
  const entry = entryFor(url), key = versionedAssetUrl(url);
  const cache = entry && typeof caches !== "undefined" ? await openCache() : undefined;
  let response = attempt === 0 ? await cache?.match(key).catch(() => undefined) : undefined;
  const cached = !!response;
  response ??= await fetch(retryAssetUrl(key,attempt), { signal, cache: attempt ? "reload" : requestCache ?? "default" });
  try {
    if (!response.ok) throw new Error(`Asset unavailable (${response.status}): ${url}`);
    let stored: Response | undefined;
    if (entry) {
      const bytes = await response.arrayBuffer();
      if (bytes.byteLength !== entry.bytes || await assetDigest(bytes) !== entry.sha256) throw new Error(`Asset integrity mismatch: ${url}`);
      const headers = new Headers(response.headers);
      headers.set("content-length", String(bytes.byteLength));
      response = new Response(bytes, { headers });
      stored = response.clone();
    }
    signal.throwIfAborted();
    const value = await decode(response, signal);
    signal.throwIfAborted();
    if (cache && stored && !cached) {
      // Serialize writes/evictions so simultaneous downloads cannot exceed the budget.
      const copy = stored;
      writes = writes.then(async () => {
        await cache.put(key, copy);
        const absolute = new URL(key, location.href).href;
        cacheSizes.delete(absolute); cacheSizes.set(absolute, entry!.bytes);
        let total = Array.from(cacheSizes.values()).reduce((sum,n)=>sum+n,0);
        for (const [old,size] of cacheSizes) {
          if (total <= CACHE_BUDGET) break;
          await cache.delete(old); cacheSizes.delete(old); total -= size;
        }
      }).catch(() => { /* Quota/private mode must never block a successful boot. */ });
      await writes;
    }
    return value;
  } catch (error) {
    // Abandoning a runtime says nothing about the validity of cached bytes.
    // Keep them for the next boot; actual integrity/decode failures still evict.
    if (!signal.aborted) await cache?.delete(key).catch(() => false);
    throw error;
  }
}
let writes: Promise<void> = Promise.resolve();

const cacheSizes = new Map<string, number>();
let cachePromise: Promise<Cache | undefined> | undefined;
function openCache() {
  cachePromise ??= caches.open(CACHE).then(async cache => {
    for (const request of await cache.keys()) {
      const response = await cache.match(request);
      cacheSizes.set(request.url, Number(response?.headers.get("content-length")) || 0);
    }
    return cache;
  }).catch(() => undefined);
  return cachePromise;
}

let retryRequest = 0;

/** Refresh failed fetches while retaining content identity, query and fragment. */
export function retryAssetUrl(url: string,attempt: number) {
  if(!attempt)return url;
  const [path,hash]=url.split("#",2);
  return `${path}${path!.includes("?") ? "&" : "?"}blacksite_retry=${Date.now()}-${++retryRequest}${hash ? `#${hash}` : ""}`;
}
