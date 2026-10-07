import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { HUD_SIZE, HUD_OFFSETS as H } from '../../src/game/hud-abi.ts';
import { readEnemyCues } from '../../src/game/enemy-presentation.ts';

export const INPUT = Object.freeze({ advance: 1, retreat: 2, strafe_left: 4, strafe_right: 8,
  fire: 16, sprint: 32, interact: 64, turn_left: 1024, turn_right: 2048, reload: 4096 });
export const angle = (radians) => Math.atan2(Math.sin(radians), Math.cos(radians));
export const DT = 1 / 60;

export async function loadSimulation(path = 'public/blacksite.wasm') {
  const bytes = await readFile(path);
  const { instance } = await WebAssembly.instantiate(bytes);
  const w = instance.exports;
  for (const name of ['hs_init', 'hs_input', 'hs_tick', 'hs_hud_ptr', 'hs_hud_size',
    'hs_prepare_enemies', 'hs_enemy_cues', 'hs_map_ptr', 'hs_map_w', 'hs_map_h', 'hs_door_ptr']) {
    if (typeof w[name] !== 'function') throw new Error(`Missing ABI export: ${name}`);
  }
  if (w.hs_hud_size() !== HUD_SIZE) throw new Error('HUD ABI mismatch');
  w.hs_init(160, 100);
  w.hs_input(0, 0, 0);
  w.hs_tick(DT);
  return { w, wasmSha256: createHash('sha256').update(bytes).digest('hex') };
}

export function snapshot(w) {
  const v = new DataView(w.memory.buffer, w.hs_hud_ptr(), HUD_SIZE);
  const i = (o) => v.getInt32(o, true), f = (o) => v.getFloat32(o, true);
  const hud = { health: i(0), armor: i(4), ammo: i(8), weapon: i(12), kills: i(16), living: i(20),
    state: i(H.state), prompt: i(28), elapsedMs: i(44), hurt: f(56), hitmarker: f(68),
    yaw: f(72), x: f(80), y: f(84), reserve: i(H.reserve), reloading: f(H.reloading),
    events: v.getUint32(H.events, true), wave: i(H.wave), objective: i(H.objective),
    radioSeq: i(H.radioSeq), bossHealth: i(H.bossHealth), bossPhase: i(H.bossPhase), nodeX: f(160), nodeY: f(164),
    splash: f(H.splash), hurtDir: f(H.hurtDir) };
  const offsets = [32, 36, 100, 104, 132, 136, 140, 168, 172, 176, 192, 196, 200, 204, 208, 212, 216, 220];
  hud.ownedWeapons = [0, ...offsets.flatMap((o, n) => i(o) ? [n + 1] : [])];
  const extra = v.getUint32(H.extraWeapons, true);
  for (let n = 0; n < 14; n++) if (extra & (1 << n)) hud.ownedWeapons.push(19 + n);
  const count = w.hs_prepare_enemies();
  if (count < 0 || count > 192) throw new Error('Invalid enemy snapshot count');
  const enemies = readEnemyCues(w.memory.buffer, w.hs_enemy_cues(), count);
  const width = w.hs_map_w(), height = w.hs_map_h();
  if (width < 1 || height < 1 || width * height > 10000) throw new Error('Invalid map dimensions');
  const map = Array.from(new Uint8Array(w.memory.buffer, w.hs_map_ptr(), width * height));
  const doors = Array.from(new Float32Array(w.memory.buffer, w.hs_door_ptr(), width * height));
  return { hud, enemies, width, height, map, doors };
}

export function assertValid(s) {
  for (const [key, value] of Object.entries(s.hud)) {
    if (typeof value === 'number' && !Number.isFinite(value)) throw new Error(`Invalid HUD ${key}`);
  }
  if (s.hud.ammo < 0 || s.hud.reserve < 0 || s.hud.weapon < 0 || s.hud.weapon > 32 ||
      s.hud.living < 0 || s.hud.kills < 0 || ![0, 1, 2].includes(s.hud.state)) throw new Error('Invalid ammo/state');
  if (s.hud.x < 0 || s.hud.x >= s.width || s.hud.y < 0 || s.hud.y >= s.height) throw new Error('Player outside map');
  const cell = Math.floor(s.hud.y) * s.width + Math.floor(s.hud.x);
  if (![0, 10].includes(s.map[cell]) && !([8, 9].includes(s.map[cell]) && s.doors[cell] >= 0.98)) {
    throw new Error('Player embedded in solid geometry');
  }
  for (const e of s.enemies) for (const [key, value] of Object.entries(e)) {
    if (typeof value === 'number' && !Number.isFinite(value)) throw new Error(`Invalid enemy ${key}`);
  }
  if (s.doors.some((v) => !Number.isFinite(v) || v < 0 || v > 1)) throw new Error('Invalid door state');
}

// Map geometry and objective coordinates are already exposed by the minimap.
// Routing never uses enemy positions behind walls, entity arrays or debug hooks.
export function waypoint(s) {
  const start = Math.floor(s.hud.y) * s.width + Math.floor(s.hud.x);
  const goal = Math.floor(s.hud.nodeY) * s.width + Math.floor(s.hud.nodeX);
  const parent = new Int32Array(s.map.length).fill(-1), queue = [start];
  parent[start] = start;
  for (let n = 0; n < queue.length; n++) {
    const cell = queue[n];
    if (cell === goal) break;
    const x = cell % s.width, y = Math.floor(cell / s.width);
    for (const [nx, ny] of [[x + 1, y], [x, y - 1], [x - 1, y], [x, y + 1]]) {
      if (nx < 0 || ny < 0 || nx >= s.width || ny >= s.height) continue;
      const next = ny * s.width + nx;
      if (parent[next] !== -1 || ![0, 8, 10].includes(s.map[next])) continue;
      parent[next] = cell; queue.push(next);
    }
  }
  if (goal < 0 || goal >= parent.length || parent[goal] === -1 || start === goal) return null;
  let next = goal;
  while (parent[next] !== start) next = parent[next];
  return { x: next % s.width + 0.5, y: Math.floor(next / s.width) + 0.5,
    door: s.map[next] === 8 && s.doors[next] < 0.98 };
}

export function observe(s, previous) {
  const h = s.hud;
  const visible = s.enemies.filter((e) => e.hp > 0 && e.sight && e.screenX >= 0 && e.screenX <= 1);
  visible.sort((a, b) => a.distance - b.distance || a.id - b.id);
  const route = waypoint(s);
  const bearing = (x, y) => angle(Math.atan2(y - h.y, x - h.x) - h.yaw);
  const moved = previous ? Math.hypot(h.x - previous.hud.x, h.y - previous.hud.y) : null;
  const rounded = (v) => v === null ? null : Math.round(v * 100) / 100;
  const o = { sector: h.wave, health: h.health, armor: h.armor, magazine: h.ammo, reserve: h.reserve,
    weapon: h.weapon + 1, ownedWeapons: h.ownedWeapons.map((n) => n + 1), reloading: h.reloading > 0,
    position: { x: h.x, y: h.y, yaw: angle(h.yaw) }, takingDamage: h.hurt > 0,
    damageBearing: h.hurt > 0 ? h.hurtDir : null, hostilesLeft: h.living,
    visibleEnemies: visible.slice(0, 3).map((e) => ({ id: e.id, distance: e.distance,
      bearing: bearing(e.x, e.y), screenX: e.screenX })),
    objectiveCompleted: h.objective !== 0, objectiveBearing: bearing(h.nodeX, h.nodeY),
    objectiveDistance: Math.hypot(h.nodeX - h.x, h.nodeY - h.y),
    routeBearing: route ? bearing(route.x, route.y) : null,
    nearbyDoor: Boolean(route?.door), interactPrompt: h.prompt,
    splashRisk: h.splash > 0, movedDistance: moved, bossHealth: h.bossHealth, bossPhase: h.bossPhase };
  o.position = { x: rounded(h.x), y: rounded(h.y), yaw: rounded(angle(h.yaw)) };
  o.visibleEnemies = o.visibleEnemies.map((e) => ({ ...e, distance: rounded(e.distance), bearing: rounded(e.bearing), screenX: rounded(e.screenX) }));
  for (const k of ['damageBearing', 'objectiveBearing', 'objectiveDistance', 'routeBearing', 'movedDistance']) o[k] = rounded(o[k]);
  return o;
}

export function questions(o) {
  const make = (instructions, criteria) => ({ type: 'choice', instructions, criteria });
  const movement = { advance: 'Walk ahead to explore.', retreat: 'Back away from danger.',
    strafe_left: 'Move left to evade.', strafe_right: 'Move right to evade.', hold: 'Remain in place.' };
  if (o.routeBearing !== null) movement.move_toward_objective = 'Follow the minimap route toward the objective.';
  const combat = { hold: 'Do not fire or turn.', turn_left: 'Look left to scan.', turn_right: 'Look right to scan.' };
  if (o.visibleEnemies.length && o.magazine > 0 && !o.reloading && !o.splashRisk) combat.engage_nearest = 'Aim and fire at the nearest visible hostile.';
  const utility = { nothing: 'No utility action.' };
  if (o.magazine < 12 && o.weapon === 1 && o.reserve > 0 && !o.reloading) utility.reload = 'Reload the pistol with reserve ammunition.';
  if (o.nearbyDoor || [1, 3, 13, 14, 120, 121, 122, 123].includes(o.interactPrompt)) utility.interact = 'Use the nearby door or prompted object.';
  // Keep two legal choices for each model question; a single choice needs no ML.
  return { movement: make('Choose movement; prioritize reaching the objective and avoiding damage.', movement),
    combat: make('Choose combat; engage visible hostiles, otherwise scan or hold.', combat),
    ...(Object.keys(utility).length > 1 ? { utility: make('Choose utility; reload a depleted magazine or open a nearby door.', utility) } : {}) };
}

export function validateDecision(answers, schema) {
  const action = { movement: 'hold', combat: 'hold', utility: 'nothing' };
  const errors = [];
  for (const [name, question] of Object.entries(schema)) {
    const value = answers?.[name];
    if (value?.type !== 'choice' || typeof value.choice !== 'string' || !Object.hasOwn(question.criteria, value.choice)) {
      errors.push(`Invalid ${name} choice`);
    } else action[name] = value.choice;
  }
  // Reject the whole request on malformed output; safe input still advances time.
  return { action: errors.length ? { movement: 'hold', combat: 'hold', utility: 'nothing' } : action, errors };
}

export function translate(action, o) {
  const movement = ['advance', 'retreat', 'strafe_left', 'strafe_right'];
  let bits = movement.includes(action.movement) ? INPUT[action.movement] : 0;
  let turn = action.movement === 'move_toward_objective' ? o.routeBearing : 0;
  if (action.movement === 'move_toward_objective' && turn !== null) bits |= INPUT.advance;
  if (action.combat === 'engage_nearest' && o.visibleEnemies.length && o.magazine > 0 && !o.reloading && !o.splashRisk) {
    turn = o.visibleEnemies[0].bearing; bits |= INPUT.fire;
  } else if (action.movement === 'hold' && (action.combat === 'turn_left' || action.combat === 'turn_right')) {
    bits |= INPUT[action.combat];
  }
  if (action.utility === 'reload' && o.reserve > 0 && !o.reloading) bits |= INPUT.reload;
  if (action.utility === 'interact') bits |= INPUT.interact;
  const mx = Number.isFinite(turn) ? Math.max(-0.7, Math.min(0.7, turn)) / 0.0062 : 0;
  return { bits, mx, my: 0 };
}

export function fingerprint(s) {
  return createHash('sha256').update(JSON.stringify(s)).digest('hex');
}

export function step(w, input, frames, onFrame = () => {}) {
  if (!Number.isInteger(input.bits) || input.bits < 0 || input.bits > 8191 ||
      !Number.isFinite(input.mx) || !Number.isFinite(input.my) || !Number.isInteger(frames) || frames < 1 || frames > 180) {
    throw new Error('Unsafe simulation input');
  }
  const initial = snapshot(w);
  let s = initial, events = 0, executed = 0;
  for (let n = 0; n < frames && s.hud.state === 0; n++) {
    // Mouse deltas apply once; key state is held over the interval.
    w.hs_input(input.bits, n === 0 ? input.mx : 0, n === 0 ? input.my : 0);
    w.hs_tick(DT); s = snapshot(w); assertValid(s); events |= s.hud.events; executed++;
    if (onFrame(s) === false) break;
  }
  return { snapshot: s, events, frames: executed };
}

export function preflight(w) {
  let s = snapshot(w); assertValid(s);
  const start = s.hud;
  const moved = step(w, { bits: INPUT.advance, mx: 0, my: 0 }, 30).snapshot;
  if (Math.hypot(moved.hud.x - start.x, moved.hud.y - start.y) < 0.1) throw new Error('Movement input has no effect');
  const turned = step(w, { bits: 0, mx: 20, my: 0 }, 1).snapshot;
  if (Math.abs(angle(turned.hud.yaw - moved.hud.yaw)) < 0.05) throw new Error('Look input has no effect');
  const fired = step(w, { bits: INPUT.fire, mx: 0, my: 0 }, 30);
  if (!(fired.events & 1) || fired.snapshot.hud.ammo >= turned.hud.ammo) throw new Error('Firing did not consume ammo');
  const reloaded = step(w, { bits: INPUT.reload, mx: 0, my: 0 }, 90);
  if (!(reloaded.events & 4) || reloaded.snapshot.hud.ammo <= fired.snapshot.hud.ammo ||
      reloaded.snapshot.hud.reserve >= fired.snapshot.hud.reserve) throw new Error('Reload did not transfer reserve ammo');
  s = step(w, { bits: INPUT.interact, mx: 0, my: 0 }, 1).snapshot;
  if (s.hud.elapsedMs <= start.elapsedMs) throw new Error('Simulation clock did not advance');
  return { movement: true, turn: true, firingConsumesAmmo: true, reloadTransfersAmmo: true,
    interactInputAccepted: true, stateValid: true, simulationAdvances: true };
}
