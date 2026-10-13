/** One initial attempt and at most two retries, shared by boot asset types. */
export const ASSET_RETRIES = 2;
export type AssetProgress = (done: number, total: number) => void;
type LoadOptions = { signal?: AbortSignal; timeoutMs?: number; onAttempt?: AssetProgress; cache?: RequestCache };
type Attempt = { attempt: number; signal: AbortSignal };

export function createAssetLoader({
  concurrency = 12,
  timeoutMs = 30000,
  retryDelays = [200, 600],
}: { concurrency?: number; timeoutMs?: number; retryDelays?: number[] } = {}) {
  if (!Number.isInteger(concurrency) || concurrency < 1) throw new RangeError("Invalid asset concurrency");
  let active = 0;
  const queue: (() => void)[] = [];
  const abortError = () => new DOMException("Asset loading cancelled", "AbortError");

  async function slot<T>(task: () => Promise<T>, signal?: AbortSignal): Promise<T> {
    signal?.throwIfAborted();
    await new Promise<void>((resolve, reject) => {
      const start = () => {
        signal?.removeEventListener("abort", cancel);
        active++;
        resolve();
      };
      const cancel = () => {
        const index = queue.indexOf(start);
        if (index >= 0) queue.splice(index, 1);
        reject(signal?.reason ?? abortError());
      };
      if (active < concurrency) start();
      else {
        signal?.addEventListener("abort", cancel, { once: true });
        queue.push(start);
      }
    });
    try { signal?.throwIfAborted(); return await task(); }
    finally { active--; queue.shift()?.(); }
  }

  async function load<T>(read: (context: Attempt) => Promise<T>, options: LoadOptions = {}): Promise<T> {
    let error: unknown;
    for (let attempt = 0; attempt <= ASSET_RETRIES; attempt++) {
      options.signal?.throwIfAborted();
      try {
        const value = await slot(async () => {
          // Start the timeout after obtaining a slot; queueing is not a failed download.
          const controller = new AbortController();
          const cancel = () => controller.abort(options.signal?.reason ?? abortError());
          options.signal?.addEventListener("abort", cancel, { once: true });
          const timer = setTimeout(() => controller.abort(new DOMException("Asset timed out", "TimeoutError")), options.timeoutMs ?? timeoutMs);
          try {
            return await new Promise<T>((resolve, reject) => {
              controller.signal.addEventListener("abort", () => reject(controller.signal.reason), { once: true });
              Promise.resolve().then(() => {
                controller.signal.throwIfAborted();
                return read({ attempt, signal: controller.signal });
              }).then(resolve, reject);
            });
          } finally {
            clearTimeout(timer);
            options.signal?.removeEventListener("abort", cancel);
          }
        }, options.signal);
        options.onAttempt?.(ASSET_RETRIES + 1, ASSET_RETRIES + 1);
        return value;
      } catch (failure) {
        options.signal?.throwIfAborted();
        error = failure;
        options.onAttempt?.(attempt + 1, ASSET_RETRIES + 1);
        if (attempt < ASSET_RETRIES) await new Promise(resolve => setTimeout(resolve, retryDelays[attempt] ?? 0));
      }
    }
    throw error;
  }
  return { load };
}

export const assetLoader = createAssetLoader();

/** Decode/parse inside the retry boundary, so a corrupt HTTP 200 is retried too. */
export function fetchAsset<T>(url: string, decode: (response: Response, signal: AbortSignal) => Promise<T>, options: LoadOptions = {}) {
  return assetLoader.load(async ({ attempt, signal }) => {
    const response = await fetch(url, { signal, cache: attempt ? "reload" : options.cache ?? "default" });
    if (!response.ok) throw new Error(`Asset unavailable (${response.status}): ${url}`);
    return decode(response, signal);
  }, options);
}

/** Progress counts attempts on failed items without refetching successful ones. */
export async function preloadAssets<T>(
  urls: string[], read: (url: string, onAttempt: AssetProgress) => Promise<T>,
  limit: number, onProgress?: AssetProgress,
): Promise<(T | null)[]> {
  const result: (T | null)[] = new Array(urls.length).fill(null);
  const progress = new Float64Array(urls.length);
  let cursor = 0, done = 0;
  onProgress?.(0, urls.length);
  const advance = (index: number, fraction: number) => {
    const value = Math.max(progress[index]!, Math.min(1, fraction));
    if (value === progress[index]) return;
    done += value - progress[index]!;
    progress[index] = value;
    onProgress?.(Math.min(urls.length, done), urls.length);
  };
  await Promise.all(Array.from({ length: Math.min(limit, urls.length) }, async () => {
    while (cursor < urls.length) {
      const index = cursor++;
      try { result[index] = await read(urls[index]!, (n, total) => advance(index, total ? n / total : 1)); }
      catch { /* Report an exhausted item to the caller after the remaining items finish. */ }
      advance(index, 1);
    }
  }));
  return result;
}

const retrySession = Date.now();
let retryId = 0;
/** Fresh image URLs bypass cached 404s and corrupt image responses. */
export function retryImageUrl(url: string, attempt: number): string {
  if (!attempt) return url;
  const [path, hash] = url.split("#", 2);
  return `${path}${path!.includes("?") ? "&" : "?"}blacksite_retry=${retrySession}-${++retryId}${hash ? `#${hash}` : ""}`;
}
