import { observe, waypoint, snapshot, translate, step, INPUT } from './simulation.mjs';

const cellOf = s => Math.floor(s.hud.y) * s.width + Math.floor(s.hud.x);
export function createMemory(s) {
  return { visited: new Map([[cellOf(s), 1]]), blocked: new Map(), recent: [], stuck: 0 };
}

export function actionDuration(action, o, baseFrames = 30) {
  if (o.health <= 25 || o.takingDamage) return baseFrames;
  if (action.utility === 'reload') return 180;
  if (action.utility !== 'nothing' || o.nearbyDoor) return 60;
  if (action.combat === 'engage_nearest') return 120;
  if (action.movement === 'move_toward_objective' && !o.visibleEnemies.length) return 360;
  return 60;
}

// No inference here. Refine the chosen goal with player-observable state every
// 0.1 seconds and end it at important events. Never choose reload/interact/equip
// on the model's behalf. Every resulting ABI input is recorded for exact replay.
export function executeAction(w, action, memory, onFrame = () => {}, { adaptive = true, baseFrames = 30 } = {}) {
  const initial = snapshot(w), o = observe(initial, undefined, memory);
  const maxFrames = adaptive ? actionDuration(action, o, baseFrames) : baseFrames;
  const segments = [], overrides = {};
  let current = initial, frames = 0, events = 0, interruptedBy = null, wasReloading = initial.hud.reloading > 0;
  let chunkStart = initial;
  while (frames < maxFrames && current.hud.state === 0 && !interruptedBy) {
    const now = observe(current, initial, memory);
    if (frames > 0 && !o.nearbyDoor && now.nearbyDoor && action.utility !== 'interact') {
      interruptedBy = 'door'; break;
    }
    const input = translate(action, now);
    const reasons = [];
    if (action.utility === 'interact' && now.nearbyDoor) {
      input.bits &= ~15; reasons.push('waiting_for_door');
    }
    if (action.utility === 'reload' && action.combat === 'engage_nearest') reasons.push('reload_priority');
    if (action.movement === 'move_toward_objective' && now.routeBearing !== null &&
        action.combat !== 'engage_nearest' && Math.abs(now.routeBearing) > 0.3) {
      input.bits &= ~INPUT.advance; reasons.push('turn_before_advance');
    }
    // Stop the chosen attack if its visible target disappears; don't pursue hidden positions.
    if (action.combat === 'engage_nearest' && action.utility !== 'reload' && !now.visibleEnemies.length) {
      if (frames > 0) { interruptedBy = 'target_lost'; break; }
    }
    const interval = step(w, input, Math.min(6, maxFrames - frames), frame => {
      const h = frame.hud;
      current = frame;
      memory.visited.set(cellOf(frame), (memory.visited.get(cellOf(frame)) ?? 0) + 1);
      onFrame(frame);
      if (reasons.length) for (const reason of reasons) overrides[reason] = (overrides[reason] ?? 0) + 1;
      if (h.health <= initial.hud.health - 8 || h.armor <= initial.hud.armor - 15) interruptedBy = 'damage';
      else if (h.state !== 0) interruptedBy = 'game_state';
      else if (h.kills > initial.hud.kills) interruptedBy = 'enemy_killed';
      else if (initial.hud.ammo > 0 && h.ammo === 0) interruptedBy = 'magazine_empty';
      else if (h.weapon !== initial.hud.weapon) interruptedBy = 'weapon_changed';
      else if (h.objective !== initial.hud.objective) interruptedBy = 'objective';
      else if (h.bossPhase !== initial.hud.bossPhase || Boolean(h.bossHealth) !== Boolean(initial.hud.bossHealth)) interruptedBy = 'boss';
      else if (wasReloading && h.reloading === 0) interruptedBy = 'reload_complete';
      else if (!o.visibleEnemies.length && frame.enemies.some(e => e.hp > 0 && e.sight && e.screenX >= 0 && e.screenX <= 1)) interruptedBy = 'enemy_appeared';
      else if (action.utility === 'interact' && (h.radioSeq !== initial.hud.radioSeq || frame.doors.some((v, i) => v >= 0.98 && initial.doors[i] < 0.98))) interruptedBy = 'interaction';
      wasReloading ||= h.reloading > 0;
      // Use elapsed simulation time, not model/CI latency, for stall detection.
      if (h.elapsedMs - chunkStart.hud.elapsedMs >= 500) {
        if ((input.bits & 15) && Math.hypot(h.x - chunkStart.hud.x, h.y - chunkStart.hud.y) < 0.02) interruptedBy = 'stuck';
        chunkStart = frame;
      }
      return !interruptedBy;
    });
    segments.push({ input, frames: interval.frames }); frames += interval.frames; events |= interval.events;
    current = interval.snapshot;
  }
  memory.stuck = interruptedBy === 'stuck' ? memory.stuck + 1 : 0;
  const target = waypoint(current, memory);
  if (memory.stuck && target && action.movement === 'move_toward_objective' && action.combat !== 'engage_nearest') {
    memory.blocked.set(`${cellOf(current)}:${target.cell}`, current.hud.elapsedMs + 10000);
  }
  for (const [edge, until] of memory.blocked) if (until <= current.hud.elapsedMs) memory.blocked.delete(edge);
  memory.recent.push({ movement: action.movement, combat: action.combat, utility: action.utility,
    result: interruptedBy ?? 'duration', distance: Math.round(Math.hypot(current.hud.x - initial.hud.x, current.hud.y - initial.hud.y) * 100) / 100 });
  memory.recent = memory.recent.slice(-3);
  return { snapshot: current, frames, events, segments, overrides, interruptedBy: interruptedBy ?? 'duration', maxFrames };
}
