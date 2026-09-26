import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { SlidersHorizontal, X, Volume2, Monitor, Mouse, Captions, RotateCcw } from "lucide-react";
import { DEFAULT_GFX, DEFAULT_RES, RES_MODES, type GfxOpts, type ResMode } from "@/game/types";
import { DEFAULT_ENEMY_OPTIONS, type EnemyOptions } from "@/game/enemy-presentation";
import { DEFAULT_VOL, type Vol } from "./data";

export type SettingsProps = {
  res: ResMode;
  setRes: (r: ResMode) => void;
  sens: number;
  setSens: (n: number) => void;
  vol: Vol;
  setVol: (v: Vol) => void;
  requireGpu: boolean;
  setRequireGpu: (v: boolean) => void;
  gfx: GfxOpts;
  setGfx: (g: GfxOpts) => void;
  enemyOptions: EnemyOptions;
  setEnemyOptions: (v: EnemyOptions) => void;
  onPreviewVoice: (skin: number) => void;
};
function Slider({
  label,
  description,
  value,
  min = 0,
  max = 1,
  step = 0.01,
  display,
  onChange,
}: {
  label: string;
  description?: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  display?: string;
  onChange: (v: number) => void;
}) {
  return (
    <label className="setting-row setting-slider">
      <span>
        <strong>{label}</strong>
        {description && <small>{description}</small>}
      </span>
      <output>{display ?? `${Math.round(value * 100)}%`}</output>
      <input
        aria-label={label}
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}
function Toggle({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="setting-row">
      <span>
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
      <input
        className="setting-toggle"
        type="checkbox"
        aria-label={label}
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
    </label>
  );
}
const tabs = [
  { id: "audio", name: "Audio", icon: Volume2 },
  { id: "display", name: "Display", icon: Monitor },
  { id: "controls", name: "Controls", icon: Mouse },
  { id: "access", name: "Accessibility", icon: Captions },
] as const;

export function Settings(p: SettingsProps) {
  const [tab, setTab] = useState<string>("audio");
  const reset = () => {
    p.setVol({ ...DEFAULT_VOL });
    p.setSens(1.4);
    p.setGfx({ ...DEFAULT_GFX });
    p.setRes(DEFAULT_RES);
    p.setRequireGpu(false);
    p.setEnemyOptions({ ...DEFAULT_ENEMY_OPTIONS });
  };
  return (
    <Dialog.Root>
      <Dialog.Trigger className="menu-secondary">
        <SlidersHorizontal size={17} />
        Settings
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="settings-scrim" />
        <Dialog.Content className="settings-dialog" aria-describedby="settings-description">
          <header className="settings-heading">
            <div>
              <p className="eyebrow">FIELD CONFIGURATION</p>
              <Dialog.Title>Settings</Dialog.Title>
              <Dialog.Description id="settings-description">
                Changes apply immediately and save on this device.
              </Dialog.Description>
            </div>
            <Dialog.Close className="icon-button" aria-label="Close settings">
              <X size={20} />
            </Dialog.Close>
          </header>
          <nav className="settings-tabs" aria-label="Settings categories">
            {tabs.map((t) => (
              <button
                type="button"
                key={t.id}
                aria-pressed={tab === t.id}
                onClick={() => setTab(t.id)}
              >
                <t.icon size={17} />
                {t.name}
              </button>
            ))}
          </nav>
          <div className="settings-body">
            {tab === "audio" && (
              <>
                <p className="section-note">
                  Headphones recommended. Enemy voices follow their position in the world.
                </p>
                {(
                  [
                    ["master", "Master volume", undefined],
                    ["music", "Music", "Field mix during play."],
                    ["menu", "Menu music", "Standby bed under menus and end cards."],
                    ["sfx", "Effects", undefined],
                  ] as const
                ).map(([key, label, description]) => (
                  <Slider
                    key={key}
                    label={label}
                    description={description}
                    value={p.vol[key]}
                    onChange={(v) => p.setVol({ ...p.vol, [key]: v })}
                  />
                ))}
                <Slider
                  label="Enemy voices"
                  description="Independent dialogue volume. Subtitles remain available at zero."
                  value={p.enemyOptions.voices}
                  onChange={(voices) => p.setEnemyOptions({ ...p.enemyOptions, voices })}
                />
                <Toggle
                  label="Spatial enemy audio"
                  description="Directional voices and enemy sounds, with distance and wall muffling."
                  checked={p.enemyOptions.spatial}
                  onChange={(spatial) => p.setEnemyOptions({ ...p.enemyOptions, spatial })}
                />
                <div className="voice-preview">
                  <span>Voice check</span>
                  {[
                    [0, "Soldier"],
                    [5, "Machine"],
                    [12, "Commander"],
                  ].map(([skin, label]) => (
                    <button type="button" key={skin} onClick={() => p.onPreviewVoice(Number(skin))}>
                      <Volume2 size={14} />
                      {label}
                    </button>
                  ))}
                </div>
              </>
            )}
            {tab === "display" && (
              <>
                <label className="setting-row">
                  <span>
                    <strong>Render resolution</strong>
                    <small>
                      Lower values improve performance; higher values sharpen the world.
                    </small>
                  </span>
                  <select
                    aria-label="Render resolution"
                    value={p.res.id}
                    onChange={(e) => p.setRes(RES_MODES.find((r) => r.id === e.target.value)!)}
                  >
                    {RES_MODES.map((r) => (
                      <option key={r.id} value={r.id}>
                        {r.label}
                      </option>
                    ))}
                  </select>
                </label>
                {(
                  [
                    ["crt", "CRT texture", "Scanlines and retro screen treatment."],
                    ["bloom", "Light bloom", "Glow around bright lights and weapon effects."],
                    ["fog", "Distance fog", "Atmosphere and depth across the facility."],
                  ] as const
                ).map(([key, label, description]) => (
                  <Toggle
                    key={key}
                    label={label}
                    description={description}
                    checked={p.gfx[key]}
                    onChange={(v) => p.setGfx({ ...p.gfx, [key]: v })}
                  />
                ))}
                <Toggle
                  label="Enforce WebGPU"
                  description="Use WebGPU when supported. Automatic fallback is the default."
                  checked={p.requireGpu}
                  onChange={p.setRequireGpu}
                />
                <Toggle
                  label="Performance overlay"
                  description="Show frame rate, renderer and resolution during play."
                  checked={p.enemyOptions.showStats}
                  onChange={(showStats) => p.setEnemyOptions({ ...p.enemyOptions, showStats })}
                />
              </>
            )}
            {tab === "controls" && (
              <>
                <Slider
                  label="Look sensitivity"
                  value={p.sens}
                  min={0.5}
                  max={3.5}
                  step={0.05}
                  display={`${p.sens.toFixed(2)}×`}
                  onChange={p.setSens}
                />
                <dl className="control-reference">
                  {[
                    ["Move", "W A S D"],
                    ["Look / fire", "Mouse / left click"],
                    ["Reload", "R"],
                    ["Interact", "E"],
                    ["Sprint", "Shift"],
                    ["Switch weapon", "1 – 7 / wheel"],
                    ["Pause", "Esc / P"],
                  ].map(([action, key]) => (
                    <div key={action}>
                      <dt>{action}</dt>
                      <dd>{key}</dd>
                    </div>
                  ))}
                </dl>
                <p className="section-note">
                  Touch: drag the left side to move, the right side to look. Use the on-screen
                  action buttons.
                </p>
              </>
            )}
            {tab === "access" && (
              <>
                <Toggle
                  label="Enemy subtitles"
                  description="Show spoken lines above the visible enemy. Hidden enemies never reveal their position."
                  checked={p.enemyOptions.subtitles}
                  onChange={(subtitles) => p.setEnemyOptions({ ...p.enemyOptions, subtitles })}
                />
                <Slider
                  label="Subtitle size"
                  value={p.enemyOptions.subtitleSize}
                  min={0.85}
                  max={1.4}
                  step={0.05}
                  display={`${Math.round(p.enemyOptions.subtitleSize * 100)}%`}
                  onChange={(subtitleSize) =>
                    p.setEnemyOptions({ ...p.enemyOptions, subtitleSize })
                  }
                />
                <div
                  className="subtitle-preview"
                  style={{ fontSize: `${p.enemyOptions.subtitleSize}rem` }}
                >
                  <span className="caption-speaker">DIRECTORATE RIFLEMAN</span>Contact! Lock this
                  sector.
                </div>
                <p className="section-note">
                  Weapon motion follows your system’s reduced-motion preference. Turn off CRT
                  texture and light bloom in Display for a calmer image.
                </p>
              </>
            )}
          </div>
          <footer className="settings-footer">
            <button type="button" onClick={reset}>
              <RotateCcw size={14} />
              Restore defaults
            </button>
            <Dialog.Close className="action-primary">Done</Dialog.Close>
          </footer>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
