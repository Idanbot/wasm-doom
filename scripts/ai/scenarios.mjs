import { SAVE_SIZE } from '../../src/game/save-abi.ts';
import { loadSimulation, snapshot, assertValid } from './simulation.mjs';

export const SECTORS = Object.freeze([1, 2, 3]);
export const EXTENDED_SECTORS = Object.freeze([1, 2, 3, 5, 10, 15, 20, 25]);

// Seeded sampling without replacement keeps randomized boss fixtures reproducible.
export function extendedScenarios(seed = 1729) {
  if (!Number.isInteger(seed) || seed < 0 || seed > 0xffffffff) throw new Error('Invalid scenario seed');
  let state = seed >>> 0;
  const available = Array.from({ length: 25 }, (_, n) => n + 1), bosses = [];
  for (let n = 0; n < 3; n++) {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    const sector = available.splice(state % available.length, 1)[0];
    bosses.push({ sector, kind: 'boss', id: `boss-${sector}` });
  }
  return [...EXTENDED_SECTORS.map(sector => ({ sector, kind: 'sector', id: `sector-${sector}` })), ...bosses];
}

// Independent ordinary sector starts, using the same checkpoint ABI as Continue.
// Only the sector field changes; inventory/health are the fresh stock game's.
// This is fixture setup, never an action available to the model.
export async function loadScenario(sector = 1, kind = 'sector') {
  if (!Number.isInteger(sector) || sector < 1 || sector > 25 || !['sector', 'boss'].includes(kind)) throw new Error('Unsupported scenario sector/type');
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
  if (kind === 'boss') {
    // Fixture setup only: enter the arena, clear ordinary enemies and spawn the
    // real full-health boss. Disable QA before any observed gameplay/input.
    const w = engine.w;
    w.hs_qa(0, 1);
    w.hs_qa_objective();
    w.hs_qa_boss(0);
    w.hs_qa(0, 0);
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
