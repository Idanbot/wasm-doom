import { SAVE_SIZE } from '../../src/game/save-abi.ts';
import { loadSimulation, snapshot, assertValid } from './simulation.mjs';

export const SECTORS = Object.freeze([1, 2, 3]);

// Independent ordinary sector starts, using the same checkpoint ABI as Continue.
// Only the sector field changes; inventory/health are the fresh stock game's.
// This is fixture setup, never an action available to the model.
export async function loadScenario(sector = 1) {
  if (!SECTORS.includes(sector)) throw new Error('Unsupported scenario sector');
  const engine = await loadSimulation();
  if (sector !== 1) {
    const w = engine.w;
    if (w.hs_save_size() !== SAVE_SIZE) throw new Error('Checkpoint ABI mismatch');
    const saved = new Uint8Array(w.memory.buffer, w.hs_save_ptr(), SAVE_SIZE).slice();
    new DataView(saved.buffer).setInt32(0, sector, true);
    new Uint8Array(w.memory.buffer, w.hs_load_ptr(), SAVE_SIZE).set(saved);
    w.hs_load_run();
    w.hs_input(0, 0, 0); w.hs_tick(1 / 60);
  }
  const s = snapshot(engine.w); assertValid(s);
  if (s.hud.wave !== sector || s.hud.state !== 0) throw new Error('Scenario initialization failed');
  return engine;
}

// Baseline uses exactly the same observable state and input translator as ML.
export function scriptedDecision(o, schema) {
  const choose = (name, value) => ({ type: 'choice', choice: Object.hasOwn(schema[name].criteria, value)
    ? value : Object.keys(schema[name].criteria)[0] });
  const answers = {
    movement: choose('movement', o.visibleEnemies.length ? 'hold' : 'move_toward_objective'),
    combat: choose('combat', o.visibleEnemies.length ? 'engage_nearest' : o.routeBearing === null ? 'turn_right' : 'hold'),
  };
  if (schema.utility) answers.utility = choose('utility', o.magazine === 0 && o.reserve > 0 ? 'reload'
    : o.nearbyDoor || o.interactPrompt ? 'interact' : 'nothing');
  return answers;
}
