import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { pathToFileURL } from 'node:url';

/** Verify the assembled deployment, not the source checkout's older engine. */
export async function checkPagesIntegrity(root = 'dist-pages') {
  const manifest = JSON.parse(await readFile(join(root, 'game/asset-manifest.json'), 'utf8'));
  const expected = manifest.assets?.['/blacksite.wasm'];
  const bytes = await readFile(join(root, 'blacksite.wasm'));
  const digest = createHash('sha256').update(bytes).digest('hex');
  if (!expected || expected.bytes !== bytes.length || expected.sha256 !== digest) {
    throw new Error('Pages WASM integrity mismatch: restore the asset manifest from the same tested build as blacksite.wasm');
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  await checkPagesIntegrity(process.argv[2]);
  console.log('Pages WASM matches its published asset manifest.');
}
