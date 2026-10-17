/** Physical key codes keep bindings stable across keyboard layouts. */
export const INPUT_BITS = {
  W: 1,
  S: 2,
  A: 4,
  D: 8,
  FIRE: 16,
  SPRINT: 32,
  USE: 64,
  W1: 128,
  W2: 256,
  W3: 512,
  TURNL: 1024,
  TURNR: 2048,
  RELOAD: 4096,
  W4: 8192,
  W5: 16384,
  W6: 32768,
  W7: 65536,
  W8: 131072,
  W9: 262144,
  W10: 524288,
  W11: 1048576,
  W12: 2097152,
  W13: 4194304,
  W14: 8388608,
  W15: 16777216,
  W16: 33554432,
  W17: 67108864,
  W18: 134217728,
  W19: 268435456,
};
export type ControlAction = {
  id: string;
  label: string;
  defaults: string[];
  bits?: number;
  slot?: number;
};
const weaponCodes = [
  "Digit1",
  "Digit2",
  "Digit3",
  "Digit4",
  "Digit5",
  "Digit6",
  "Digit7",
  "Digit8",
  "Digit9",
  "Digit0",
  "Minus",
  "Equal",
  "BracketLeft",
  "BracketRight",
  "Backslash",
  "Semicolon",
  "Quote",
  "Backquote",
  "Comma",
];
export const CONTROL_ACTIONS: ControlAction[] = [
  { id: "forward", label: "Move forward", defaults: ["KeyW", "ArrowUp"], bits: INPUT_BITS.W },
  { id: "backward", label: "Move backward", defaults: ["KeyS", "ArrowDown"], bits: INPUT_BITS.S },
  { id: "left", label: "Strafe left", defaults: ["KeyA"], bits: INPUT_BITS.A },
  { id: "right", label: "Strafe right", defaults: ["KeyD"], bits: INPUT_BITS.D },
  { id: "turnLeft", label: "Turn left", defaults: ["ArrowLeft", "KeyQ"], bits: INPUT_BITS.TURNL },
  { id: "turnRight", label: "Turn right", defaults: ["ArrowRight"], bits: INPUT_BITS.TURNR },
  {
    id: "fire",
    label: "Fire",
    defaults: ["Mouse0", "Space", "ControlLeft", "ControlRight"],
    bits: INPUT_BITS.FIRE,
  },
  { id: "reload", label: "Reload / vent", defaults: ["KeyR"], bits: INPUT_BITS.RELOAD },
  { id: "interact", label: "Interact", defaults: ["KeyE"], bits: INPUT_BITS.USE },
  { id: "sprint", label: "Sprint", defaults: ["ShiftLeft", "ShiftRight"], bits: INPUT_BITS.SPRINT },
  { id: "previousWeapon", label: "Previous weapon", defaults: [] },
  { id: "nextWeapon", label: "Next weapon", defaults: [] },
  { id: "pause", label: "Pause / resume", defaults: ["KeyP"] },
  ...Array.from({ length: 33 }, (_, slot) => ({
    id: `weapon${slot + 1}`,
    label: `Weapon ${slot + 1}`,
    slot,
    defaults: weaponCodes[slot] ? [weaponCodes[slot]!] : [],
    bits: slot < 19 ? INPUT_BITS[`W${slot + 1}` as keyof typeof INPUT_BITS] : undefined,
  })),
];
export type ControlBindings = Record<string, string[]>;
export const DEFAULT_BINDINGS: ControlBindings = Object.fromEntries(
  CONTROL_ACTIONS.map((a) => [a.id, [...a.defaults]]),
);

export function validBinding(code: unknown): code is string {
  return (
    typeof code === "string" &&
    /^(Key[A-Z]|Digit[0-9]|Numpad(?:[0-9]|Add|Subtract|Multiply|Divide|Decimal|Enter)|Arrow(?:Up|Down|Left|Right)|(?:Shift|Control|Alt)(?:Left|Right)|Space|Enter|Tab|Backspace|Delete|Home|End|PageUp|PageDown|Insert|Minus|Equal|BracketLeft|BracketRight|Backslash|Semicolon|Quote|Backquote|Comma|Period|Slash|CapsLock|Mouse[0-4])$/.test(
      code,
    )
  );
}
export function normalizeBindings(value: unknown): ControlBindings {
  const saved = value && typeof value === "object" ? (value as Record<string, unknown>) : {};
  const used = new Set<string>();
  return Object.fromEntries(
    CONTROL_ACTIONS.map((action) => {
      const raw = saved[action.id];
      const codes = (Array.isArray(raw) ? raw : action.defaults)
        .filter(validBinding)
        .slice(0, 4)
        .filter((code) => {
          if (used.has(code)) return false;
          used.add(code);
          return true;
        });
      return [action.id, codes];
    }),
  );
}
export function loadBindings(): ControlBindings {
  try {
    const saved = JSON.parse(localStorage.getItem("blacksite-controls") ?? "null");
    return normalizeBindings(saved?.bindings);
  } catch {
    return normalizeBindings(null);
  }
}
export function controlForCode(bindings: ControlBindings, code: string) {
  return CONTROL_ACTIONS.find((a) => bindings[a.id]?.includes(code));
}
export function controlBits(bindings: ControlBindings, codes: Iterable<string>): number {
  let bits = 0;
  for (const code of codes) bits |= controlForCode(bindings, code)?.bits ?? 0;
  return bits;
}
export function bindingLabel(code: string): string {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  return (
    (
      {
        Mouse0: "Mouse left",
        Mouse1: "Mouse middle",
        Mouse2: "Mouse right",
        Mouse3: "Mouse back",
        Mouse4: "Mouse forward",
        ShiftLeft: "Left Shift",
        ShiftRight: "Right Shift",
        ControlLeft: "Left Ctrl",
        ControlRight: "Right Ctrl",
        AltLeft: "Left Alt",
        AltRight: "Right Alt",
        ArrowUp: "↑",
        ArrowDown: "↓",
        ArrowLeft: "←",
        ArrowRight: "→",
        Minus: "−",
        Equal: "=",
        BracketLeft: "[",
        BracketRight: "]",
        Backslash: "\\",
        Semicolon: ";",
        Quote: "'",
        Backquote: "`",
        Comma: ",",
        Period: ".",
        Slash: "/",
      } as Record<string, string>
    )[code] ?? code
  );
}
export function actionLabel(bindings: ControlBindings, action: string): string {
  const code = bindings[action]?.[0];
  return code ? bindingLabel(code) : "Unbound";
}
export function assignBinding(
  bindings: ControlBindings,
  action: string,
  index: number,
  code: string,
) {
  if (
    !CONTROL_ACTIONS.some((a) => a.id === action) ||
    index < 0 ||
    index >= (bindings[action]?.length ?? 0) + 1 ||
    index >= 4
  )
    return { error: "Choose a valid binding slot." };
  if (!validBinding(code))
    return { error: "That key is reserved. Choose another key or mouse button." };
  const conflict = controlForCode(bindings, code);
  if (conflict && (conflict.id !== action || bindings[action]?.indexOf(code) !== index))
    return {
      error: `${bindingLabel(code)} is already assigned to ${conflict.label}. Clear it there first.`,
    };
  const keys = [...(bindings[action] ?? [])];
  keys[index] = code;
  return { bindings: { ...bindings, [action]: keys } };
}
