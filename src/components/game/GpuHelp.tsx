export function GpuHelp({ onClose }: { onClose: () => void }) {
  return (
    <div className="mt-4 rounded-md border border-border bg-elevated p-4 text-sm text-fg">
      <div className="flex items-start justify-between gap-3">
        <p className="font-display text-[10px] tracking-[0.22em] text-muted">WEBGPU ON CHROME</p>
        <button
          type="button"
          onClick={onClose}
          className="font-mono text-xs text-muted hover:text-fg"
        >
          Close
        </button>
      </div>
      <ol className="mt-3 list-decimal space-y-2 pl-5 text-muted">
        <li>Use a current Chrome or Chrome-based browser (not Firefox).</li>
        <li>
          Open <span className="font-mono text-steel">chrome://settings/system</span> and turn on{" "}
          <span className="text-fg">Use graphics acceleration when available</span>.
        </li>
        <li>
          On Linux, open{" "}
          <span className="font-mono text-steel">chrome://flags/#enable-unsafe-webgpu</span> and
          set it to Enabled, then relaunch.
        </li>
        <li>
          Check <span className="font-mono text-steel">chrome://gpu</span> — WebGPU Status should
          say Hardware accelerated.
        </li>
        <li>Relaunch Chrome, then come back and start the game.</li>
      </ol>
      <p className="mt-3 text-xs text-muted">
        Still blocked? Uncheck Enforce WebGPU below to play on WebGL2.
      </p>
    </div>
  );
}
