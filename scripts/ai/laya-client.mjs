import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';

export function startLaya({ python = process.env.BLACKSITE_AI_PYTHON ?? 'python3', stderr } = {}) {
  const child = spawn(python, ['-u', 'scripts/ai/laya-worker.py'], {
    stdio: ['pipe', 'pipe', 'pipe'], env: { ...process.env, USE_TF: '0', TOKENIZERS_PARALLELISM: 'false' },
  });
  child.stderr.on('data', (data) => stderr?.write(data));
  const lines = createInterface({ input: child.stdout });
  const queue = [];
  let waiting, failure, id = 0;
  const fail = (error) => { failure = error; waiting?.reject(error); waiting = undefined; };
  child.on('error', fail);
  child.stdin.on('error', fail);
  child.on('exit', (code, signal) => fail(new Error(`Laya worker exited (${code ?? signal})`)));
  lines.on('line', (line) => {
    try {
      const message = JSON.parse(line);
      if (message.type === 'error') { fail(new Error(message.message)); return; }
      if (waiting) { waiting.resolve(message); waiting = undefined; } else queue.push(message);
    } catch (error) { fail(new Error(`Invalid Laya protocol: ${error.message}`)); }
  });
  const receive = (milliseconds) => new Promise((resolve, reject) => {
    if (failure) { reject(failure); return; }
    if (queue.length) { resolve(queue.shift()); return; }
    const timer = setTimeout(() => { waiting = undefined; reject(new Error('Laya worker timed out')); }, milliseconds);
    waiting = { resolve: (v) => { clearTimeout(timer); resolve(v); }, reject: (e) => { clearTimeout(timer); reject(e); } };
  });
  return {
    async ready() {
      const value = await receive(240000);
      if (value.type !== 'ready' || value.device !== 'cpu') throw new Error('Laya did not initialize on CPU');
      return value;
    },
    async decide(state, questions) {
      const requestId = ++id;
      child.stdin.write(JSON.stringify({ id: requestId, state, questions }) + '\n');
      const value = await receive(60000);
      if (value.type !== 'decision' || value.id !== requestId) throw new Error('Mismatched Laya response');
      return value;
    },
    async close() {
      if (!child.pid || child.exitCode !== null || child.signalCode !== null) { lines.close(); return; }
      const exited = new Promise((resolve) => child.once('exit', resolve));
      child.stdin.end(JSON.stringify({ type: 'stop' }) + '\n');
      const timer = setTimeout(() => child.kill('SIGKILL'), 3000);
      await exited; clearTimeout(timer); lines.close();
    },
  };
}
