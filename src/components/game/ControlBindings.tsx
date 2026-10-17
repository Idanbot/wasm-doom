import { useEffect, useState } from "react";
import {
  CONTROL_ACTIONS,
  actionLabel,
  assignBinding,
  bindingLabel,
  type ControlAction,
  type ControlBindings as Bindings,
} from "@/game/controls";
import { WEAPONS } from "./data";

export function ControlBindings({
  bindings,
  onChange,
}: {
  bindings: Bindings;
  onChange: (value: Bindings) => void;
}) {
  const [pending, setPending] = useState<{ action: string; index: number } | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    if (!pending) return;
    const capture = (event: KeyboardEvent | MouseEvent) => {
      event.preventDefault();
      event.stopImmediatePropagation();
      if (event instanceof KeyboardEvent && event.repeat) return;
      if (event instanceof KeyboardEvent && event.code === "Escape") {
        setPending(null);
        setError("");
        return;
      }
      if (event instanceof MouseEvent && event.button === 0) {
        // Consume the click produced by this capture, so a binding button
        // cannot immediately reopen capture after its mousedown finishes it.
        const swallow = (click: Event) => {
          click.preventDefault();
          click.stopImmediatePropagation();
        };
        window.addEventListener("click", swallow, { capture: true, once: true });
        window.addEventListener(
          "mouseup",
          () => setTimeout(() => window.removeEventListener("click", swallow, true), 0),
          { capture: true, once: true },
        );
      }
      const code = event instanceof KeyboardEvent ? event.code : `Mouse${event.button}`;
      const result = assignBinding(bindings, pending.action, pending.index, code);
      if (result.error) {
        setError(result.error);
        return;
      }
      if (result.bindings) onChange(result.bindings);
      setPending(null);
      setError("");
    };
    window.addEventListener("keydown", capture, true);
    window.addEventListener("mousedown", capture, true);
    return () => {
      window.removeEventListener("keydown", capture, true);
      window.removeEventListener("mousedown", capture, true);
    };
  }, [pending, bindings, onChange]);
  const begin = (action: string, index: number) => {
    setError("");
    setPending({ action, index });
  };
  const row = (action: ControlAction) => {
    const name =
      action.slot === undefined
        ? action.label
        : `${action.slot + 1}. ${WEAPONS[action.slot]?.name ?? action.label}`;
    const keys = bindings[action.id] ?? [];
    return (
      <div className="binding-row" key={action.id}>
        <strong>{name}</strong>
        <div className="binding-keys">
          {keys.map((code, index) => (
            <span className="binding-key" key={index}>
              <button
                type="button"
                aria-label={`Bind ${action.label} ${index + 1}`}
                aria-pressed={pending?.action === action.id && pending.index === index}
                onClick={() => begin(action.id, index)}
              >
                {pending?.action === action.id && pending.index === index
                  ? "Press a key…"
                  : bindingLabel(code)}
              </button>
              <button
                type="button"
                aria-label={`Clear ${action.label} ${index + 1}`}
                onClick={() => {
                  setPending(null);
                  onChange({ ...bindings, [action.id]: keys.filter((_, i) => i !== index) });
                }}
              >
                ×
              </button>
            </span>
          ))}
          {keys.length < 4 && (
            <button
              className="binding-add"
              type="button"
              aria-label={`Add ${action.label} binding`}
              aria-pressed={pending?.action === action.id && pending.index === keys.length}
              onClick={() => begin(action.id, keys.length)}
            >
              {pending?.action === action.id && pending.index === keys.length
                ? "Press a key…"
                : keys.length
                  ? "+ key"
                  : "Unbound · add key"}
            </button>
          )}
        </div>
      </div>
    );
  };
  return (
    <section
      className="control-bindings"
      aria-label="Key bindings"
      onContextMenu={(e) => e.preventDefault()}
    >
      <p className="section-note">
        Select a binding, then press a key or mouse button. Escape cancels capture and always pauses
        gameplay. Mouse wheel cycles weapons.
      </p>
      <p role={error ? "alert" : "status"} className="binding-status">
        {error ||
          (pending
            ? "Waiting for a key or mouse button…"
            : `Reload: ${actionLabel(bindings, "reload")} · Interact: ${actionLabel(bindings, "interact")}`)}
      </p>
      {CONTROL_ACTIONS.filter((a) => a.slot === undefined).map(row)}
      <details className="weapon-bindings">
        <summary>Direct weapon bindings · {WEAPONS.length} weapons</summary>
        {CONTROL_ACTIONS.filter((a) => a.slot !== undefined).map(row)}
      </details>
    </section>
  );
}
