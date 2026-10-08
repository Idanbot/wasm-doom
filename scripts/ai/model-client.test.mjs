import { test } from "node:test";
import assert from "node:assert/strict";
import { startPlayer } from "./model-client.mjs";

test("missing inference runtime rejects initialization and can close without hanging", async () => {
  const worker = startPlayer({ python: "/blacksite/nonexistent-python" });
  await assert.rejects(worker.ready(), /ENOENT/);
  await worker.close();
});

test("early inference process exit fails clearly rather than waiting for the inference deadline", async () => {
  const worker = startPlayer({ python: "/bin/false" });
  await assert.rejects(worker.ready(), /exited/);
  await worker.close();
});

test("timed-out model session rejects later decisions instead of consuming a stale response", async () => {
  const { mkdtemp, writeFile, chmod, rm } = await import("node:fs/promises");
  const { join } = await import("node:path");
  const { tmpdir } = await import("node:os");
  const dir = await mkdtemp(join(tmpdir(), "blacksite-worker-"));
  const executable = join(dir, "worker");
  await writeFile(
    executable,
    `#!/usr/bin/env python3
import sys,json,time
print(json.dumps({'type':'ready','device':'cpu'}),flush=True)
for line in sys.stdin:
    r=json.loads(line)
    if r.get('type')=='stop': break
    time.sleep(.1)
    print(json.dumps({'type':'decision','id':r['id'],'answers':{}}),flush=True)
`,
  );
  await chmod(executable, 0o755);
  const worker = startPlayer({ python: executable, decisionTimeoutMs: 10 });
  try {
    await worker.ready();
    await assert.rejects(worker.decide({}, {}), /timed out/);
    await assert.rejects(worker.decide({}, {}), /timed out/);
  } finally {
    await worker.close();
    await rm(dir, { recursive: true, force: true });
  }
});
