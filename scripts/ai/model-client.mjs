import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

export function startPlayer({
  decisionTimeoutMs = 60000,
  python = process.env.BLACKSITE_AI_PYTHON ?? "python3",
  stderr,
} = {}) {
  if (!Number.isInteger(decisionTimeoutMs) || decisionTimeoutMs < 10 || decisionTimeoutMs > 180000)
    throw new Error("Invalid model timeout");
  const child = spawn(
    python,
    [
      "-u",
      (process.env.BLACKSITE_AI_MODEL ?? "decider") === "decider"
        ? "scripts/ai/decider-worker.py"
        : "scripts/ai/laya-worker.py",
    ],
    {
      stdio: ["pipe", "pipe", "pipe"],
      env: { ...process.env, USE_TF: "0", TOKENIZERS_PARALLELISM: "false" },
    },
  );
  child.stderr.on("data", (data) => stderr?.write(data));
  const lines = createInterface({ input: child.stdout });
  const queue = [];
  let waiting,
    failure,
    id = 0;
  const fail = (error) => {
    failure = error;
    waiting?.reject(error);
    waiting = undefined;
  };
  child.on("error", fail);
  child.stdin.on("error", fail);
  child.on("exit", (code, signal) => fail(new Error(`QA model worker exited (${code ?? signal})`)));
  lines.on("line", (line) => {
    if (failure) return;
    try {
      const message = JSON.parse(line);
      if (message.type === "error") {
        fail(new Error(message.message));
        return;
      }
      if (waiting) {
        waiting.resolve(message);
        waiting = undefined;
      } else queue.push(message);
    } catch (error) {
      fail(new Error(`Invalid model protocol: ${error.message}`));
    }
  });
  const receive = (milliseconds) =>
    new Promise((resolve, reject) => {
      if (failure) {
        reject(failure);
        return;
      }
      if (queue.length) {
        resolve(queue.shift());
        return;
      }
      const timer = setTimeout(() => fail(new Error("QA model worker timed out")), milliseconds);
      waiting = {
        resolve: (v) => {
          clearTimeout(timer);
          resolve(v);
        },
        reject: (e) => {
          clearTimeout(timer);
          reject(e);
        },
      };
    });
  return {
    async ready() {
      const value = await receive(240000);
      if (value.type !== "ready" || value.device !== "cpu")
        throw new Error("Model did not initialize on CPU");
      return value;
    },
    async decide(state, questions) {
      if (failure) throw failure;
      const requestId = ++id;
      child.stdin.write(JSON.stringify({ id: requestId, state, questions }) + "\n");
      const value = await receive(decisionTimeoutMs);
      if (value.type !== "decision" || value.id !== requestId)
        throw new Error("Mismatched model response");
      return value;
    },
    async close() {
      if (!child.pid || child.exitCode !== null || child.signalCode !== null) {
        lines.close();
        return;
      }
      const exited = new Promise((resolve) => child.once("exit", resolve));
      child.stdin.end(JSON.stringify({ type: "stop" }) + "\n");
      const timer = setTimeout(() => child.kill("SIGKILL"), 3000);
      await exited;
      clearTimeout(timer);
      lines.close();
    },
  };
}
