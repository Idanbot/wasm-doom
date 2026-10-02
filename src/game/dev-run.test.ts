import { test } from 'node:test';
import assert from 'node:assert/strict';
import { localQaRun } from './dev-run.ts';

test('local QA selects a one-based sector, defaulting invalid levels to the first', () => {
  assert.deepEqual(localQaRun('?qa=1', true, 'localhost'), { level: 1 });
  for (let level = 1; level <= 25; level++) assert.deepEqual(localQaRun(`?qa=1&lvl=${level}`, true, '127.0.0.1'), { level });
  for (const level of ['0', '26', '-1', '1.5', 'abc', 'Infinity', '1e2', '']) assert.deepEqual(localQaRun(`?qa=1&lvl=${level}`, true, 'localhost'), { level: 1 });
});

test('QA shortcuts require qa=1, local host and a development build', () => {
  assert.equal(localQaRun('?lvl=11', true, 'localhost'), null);
  assert.equal(localQaRun('?qa=0&lvl=11', true, 'localhost'), null);
  assert.equal(localQaRun('?qa=1&lvl=11', false, 'localhost'), null);
  assert.equal(localQaRun('?qa=1&lvl=11', true, 'idanbot.me'), null);
  assert.equal(localQaRun('?qa=1&lvl=11', true, 'localhost.example.com'), null);
});
