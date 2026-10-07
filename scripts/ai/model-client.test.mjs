import { test } from 'node:test';
import assert from 'node:assert/strict';
import { startPlayer } from './model-client.mjs';

test('missing inference runtime rejects initialization and can close without hanging', async () => {
  const worker = startPlayer({ python: '/blacksite/nonexistent-python' });
  await assert.rejects(worker.ready(), /ENOENT/);
  await worker.close();
});

test('early inference process exit fails clearly rather than waiting for the inference deadline', async () => {
  const worker = startPlayer({ python: '/bin/false' });
  await assert.rejects(worker.ready(), /exited/);
  await worker.close();
});
