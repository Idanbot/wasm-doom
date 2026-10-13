import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { latestUpdateDate } from './build-info.ts';

test('latest update follows the source commit date and supports source archives', async () => {
  const root = await mkdtemp(join(tmpdir(), 'blacksite-update-'));
  try {
    assert.match(latestUpdateDate(root), /^\d{4}-\d{2}-\d{2}$/);
    const git = args => execFileSync('git', args, { cwd: root, stdio: 'ignore' });
    git(['init']);
    await writeFile(join(root, 'game.txt'), 'BLACKSITE');
    git(['add', 'game.txt']);
    execFileSync('git', ['-c', 'user.name=BLACKSITE test', '-c', 'user.email=test@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '-m', 'fixture'], {
      cwd: root, stdio: 'ignore', env: { ...process.env, GIT_AUTHOR_DATE: '2025-01-02T12:00:00Z', GIT_COMMITTER_DATE: '2025-01-03T12:00:00Z' },
    });
    assert.equal(latestUpdateDate(root), '2025-01-03');
  } finally { await rm(root, { recursive: true, force: true }); }
});
