import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// Keep fault injection and renderer recovery off the interface runner. Each
// suite owns a browser at a time and retains the existing regression coverage.
export const BROWSER_SUITES = {
  interface: ['difficulty-runs', 'settings-update', 'hud-insets', 'map-controls', 'arsenal', 'catalog-return'],
  loading: ['difficulty-loading', 'loading-ready', 'loading-tiers', 'loading-fault-matrix'],
  'loading-recovery': ['loading-recovery'],
  renderer: ['runtime-input', 'resolution-recovery', 'weapon-viewmodel', 'asset-reload'],
  combat: ['boss-weapons', 'reload-effects', 'weapon-feed-art', 'world-effects', 'world-materials', 'gameplay-polish', 'tactical', 'boss-arena', 'boss-counterplay', 'projectile-presentation', 'sfx'],
  campaign: ['reward-pause', 'qa-run', 'campaign25', 'layouts-10-25', 'enemy-sector'],
};
export function browserTestArgs(group) {
  if (!Object.hasOwn(BROWSER_SUITES, group)) throw new Error(`Unknown browser suite: ${group}`);
  return ['--experimental-strip-types', '--test', '--test-concurrency=1', '--test-timeout=600000',
    ...BROWSER_SUITES[group].map(name => `scripts/${name}.browser.mjs`)];
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const group = process.argv[2];
  console.log(`BLACKSITE browser suite: ${group}`);
  const result = spawnSync(process.execPath, browserTestArgs(group), {
    stdio: 'inherit', timeout: 16 * 60 * 1000, killSignal: 'SIGKILL',
  });
  if (result.error) console.error(`Browser suite ${group}: ${result.error.message}`);
  process.exitCode = result.status ?? 1;
}
