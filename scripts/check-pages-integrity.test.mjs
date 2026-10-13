import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { checkPagesIntegrity } from './check-pages-integrity.mjs';

test('Pages assembly rejects checkout identities and accepts the tested engine manifest', async () => {
  const root = await mkdtemp(join(tmpdir(), 'blacksite-pages-integrity-'));
  const engine = Buffer.from('rebuilt CI engine');
  const identity = bytes => ({ bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
  const publish = entry => writeFile(join(root, 'game/asset-manifest.json'), JSON.stringify({ assets: { '/blacksite.wasm': entry } }));
  try {
    await mkdir(join(root, 'game'));
    await writeFile(join(root, 'blacksite.wasm'), engine);
    await publish(identity(Buffer.from('checkout engine')));
    await assert.rejects(checkPagesIntegrity(root), /Pages WASM integrity mismatch/);
    await publish(identity(engine));
    await checkPagesIntegrity(root);
    // Same-size corruption must also fail; size alone cannot establish identity.
    await writeFile(join(root, 'blacksite.wasm'), Buffer.alloc(engine.length));
    await assert.rejects(checkPagesIntegrity(root), /Pages WASM integrity mismatch/);
    await publish(undefined);
    await assert.rejects(checkPagesIntegrity(root), /Pages WASM integrity mismatch/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
