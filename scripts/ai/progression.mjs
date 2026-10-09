import { navigationTree } from "./navigation.mjs";
import { INPUT, angle, observe, snapshot, step, fingerprint } from "./simulation.mjs";

export const APPROACHES = Object.freeze(["compact", "persistent", "commander"]);
const cellOf = (s) => Math.floor(s.hud.y) * s.width + Math.floor(s.hud.x);
export function progressionMemory() {
  return {
    seen: new Set(),
    visited: new Map(),
    items: new Map(),
    blocked: new Map(),
    failed: new Map(),
    target: null,
    equipUntil: 0,
  };
}
export function remember(s, m) {
  const h = s.hud;
  for (let y = Math.floor(h.y) - 5; y <= Math.floor(h.y) + 5; y++)
    for (let x = Math.floor(h.x) - 5; x <= Math.floor(h.x) + 5; x++) {
      if (x >= 0 && y >= 0 && x < s.width && y < s.height) m.seen.add(y * s.width + x);
    }
  for (const cache of [m.blocked, m.failed])
    for (const [key, until] of cache) if (until <= h.elapsedMs) cache.delete(key);
  const cell = cellOf(s);
  m.visited.set(cell, (m.visited.get(cell) ?? 0) + 1);
  for (const item of s.items ?? []) m.items.set(item.id, { ...item });
  // Forget a disappeared pickup only after its remembered position is visibly checked.
  for (const [id, item] of m.items) {
    const distance = Math.hypot(item.x - h.x, item.y - h.y),
      bearing = angle(Math.atan2(item.y - h.y, item.x - h.x) - h.yaw);
    if (
      distance < 0.67 ||
      (distance < 6 && Math.abs(bearing) < 0.4 && lineClear(s, h.x, h.y, item.x, item.y))
    ) {
      if (!s.items.some((i) => i.id === id)) m.items.delete(id);
    }
  }
}
export function lineClear(s, x, y, tx, ty) {
  const d = Math.hypot(tx - x, ty - y),
    n = Math.max(1, Math.ceil(d * 10));
  for (let i = 1; i <= n; i++) {
    const px = x + ((tx - x) * i) / n,
      py = y + ((ty - y) * i) / n;
    const cell = Math.floor(py) * s.width + Math.floor(px),
      k = s.map[cell];
    if (![0, 10].includes(k) && !([8, 9].includes(k) && s.doors[cell] >= 0.98)) return false;
  }
  return true;
}
export function plan(s, m) {
  const h = s.hud,
    distance = (p) => Math.hypot(p.x - h.x, p.y - h.y);
  const items = [...m.items.values()].filter(
    (p) => (m.failed.get(`item:${p.id}`) ?? 0) <= h.elapsedMs,
  );
  const byDistance = (a) => a.sort((a, b) => distance(a) - distance(b) || a.id - b.id);
  const routeTo = navigationTree(s, m),
    candidates = [];
  const reward = byDistance(items.filter((i) => i.category === 7))[0];
  const supplies = byDistance(
    items.filter(
      (i) =>
        (i.category === 1 && !h.ownedWeapons.includes(i.slot)) ||
        (i.category === 3 && h.health < 75) ||
        (i.category === 4 && h.armor < 60) ||
        (i.category === 2 && h.inventory.reduce((sum, g) => sum + g.magazine + g.reserve, 0) < 65),
    ),
  );
  if (reward) candidates.push({ ...reward, mode: "reward" });
  candidates.push(...supplies.map((item) => ({ ...item, mode: "supply" })));
  if (!h.objective) candidates.push({ x: h.nodeX, y: h.nodeY, mode: "node" });
  // These exact contacts are also drawn by the shipped minimap. Never route to unrestricted hidden entities.
  const contacts = byDistance(
    s.enemies.filter(
      (e) =>
        e.hp > 0 &&
        ((Math.abs(e.x - h.x) <= 7 && Math.abs(e.y - h.y) <= 7) ||
          m.seen.has(Math.floor(e.y) * s.width + Math.floor(e.x))),
    ),
  );
  if (h.objective && h.living > 0 && contacts.length)
    candidates.push(...contacts.map((contact) => ({ ...contact, mode: "contact" })));
  if (h.objective && h.living === 0) {
    const console = items.find((i) => i.category === 6);
    if (console) candidates.push({ ...console, mode: "console" });
  }
  for (const target of candidates) {
    const route = routeTo(target);
    if (route) return { target, route };
  }
  // Patrol unexplored frontiers, then least visited reachable space. Stable ties.
  const cells = s.map.flatMap((k, c) => ([0, 8, 10].includes(k) && c !== cellOf(s) ? [c] : []));
  cells.sort(
    (a, b) =>
      Number(m.seen.has(a)) - Number(m.seen.has(b)) ||
      (m.visited.get(a) ?? 0) - (m.visited.get(b) ?? 0) ||
      Math.hypot((a % s.width) - h.x, Math.floor(a / s.width) - h.y) -
        Math.hypot((b % s.width) - h.x, Math.floor(b / s.width) - h.y) ||
      a - b,
  );
  if (
    m.target?.mode === "explore" &&
    distance(m.target) > 0.55 &&
    !(
      m.failed.get(`cell:${Math.floor(m.target.y) * s.width + Math.floor(m.target.x)}`) >
      h.elapsedMs
    )
  ) {
    const route = routeTo(m.target);
    if (route) return { target: m.target, route };
  }
  for (const c of cells) {
    if ((m.failed.get(`cell:${c}`) ?? 0) > h.elapsedMs) continue;
    const target = { x: (c % s.width) + 0.5, y: Math.floor(c / s.width) + 0.5, mode: "explore" };
    const route = routeTo(target);
    if (route) {
      m.target = target;
      return { target, route };
    }
  }
  return { target: null, route: null };
}
export function view(s, m) {
  const o = observe(s),
    p = plan(s, m);
  const visible = s.enemies
    .filter((e) => e.hp > 0 && e.sight && e.screenX >= 0 && e.screenX <= 1)
    .sort((a, b) => a.distance - b.distance || a.id - b.id);
  return {
    ...o,
    visible,
    weaponRoles: Object.fromEntries(s.hud.inventory.map((g) => [g.id, g.role.split(" · ")[0]])),
    plan: p,
    nearbyDoor: !!p.route && s.map[p.route.cell] === 8 && s.doors[p.route.cell] < 0.98,
  };
}
export function compactRequest(v, approach) {
  const state = {
    hp: v.health,
    armor: v.armor,
    ammo: [v.magazine, v.reserve],
    capacity: v.magazineCapacity,
    reload: v.reloading,
    weapon: v.weapon,
    inventory: v.inventory,
    hostiles: v.hostilesLeft,
    objective: v.objectiveCompleted ? "clear then console then reward" : "activate node",
    goal: v.plan.target?.mode ?? "scan",
    distance: v.plan.target
      ? Math.round(
          Math.hypot(v.plan.target.x - v.position.x, v.plan.target.y - v.position.y) * 10,
        ) / 10
      : null,
    enemies: v.visible.slice(0, 2).map((e) => ({ distance: Math.round(e.distance), hp: e.hp })),
    prompt:
      v.interactPrompt === 13
        ? "activate node"
        : v.interactPrompt === 3
          ? "boss console"
          : v.nearbyDoor
            ? "open door"
            : null,
    danger: v.takingDamage,
    boss: v.bossHealth > 0 ? { hp: v.bossHealth, ...v.bossAttack } : undefined,
  };
  const q = (instructions, criteria) => ({ type: "choice", instructions, criteria });
  if (approach === "commander")
    return {
      state,
      questions: {
        goal: q(
          "Finish the sector: collect supplies, activate node, clear enemies, use console, defeat boss, collect reward. Combat and reload are handled locally.",
          {
            progress: "Advance mission while fighting visible enemies and reloading as needed.",
            cautious: "Fight from cover and replenish ammo/health before advancing.",
            aggressive: "Push forward while fighting; prioritize a fast clear.",
          },
        ),
      },
    };
  const movement = {
    navigate: "Follow the mission route.",
    hold: "Stop movement.",
    strafe_left: "Dodge left.",
    strafe_right: "Dodge right.",
  };
  const combat = { scan: "Look for hostiles when route unavailable.", hold: "Hold fire." };
  if (v.visible.length && v.magazine > 0 && !v.reloading && !v.splashRisk)
    combat.engage = "Aim and shoot visible enemies.";
  const utility = { nothing: "Continue." };
  if (
    v.magazine < v.magazineCapacity &&
    v.reserve > 0 &&
    !v.reloading &&
    (v.magazine === 0 || (!v.visible.length && v.magazine <= v.magazineCapacity / 2))
  )
    utility.reload = "Reload.";
  if (v.nearbyDoor || [3, 13].includes(v.interactPrompt))
    utility.interact = "Use the indicated door, node or boss console now.";
  for (const g of v.inventory)
    if (g.id !== v.weapon && g.magazine + g.reserve > 0)
      utility[`equip_${g.id}`] = `Equip gun ${g.id}: ${v.weaponRoles[g.id]}.`;
  return {
    state,
    questions: {
      movement: q("Follow mission route; evade enemies without abandoning progress.", movement),
      combat: q("Engage enemies with ammo. Scan only with no route. Hold while reloading.", combat),
      ...(Object.keys(utility).length > 1
        ? {
            utility: q(
              "Select interact at activate-node or boss-console prompts. Reload empty ammo with reserve. Equip a suitable gun only when needed; otherwise continue.",
              utility,
            ),
          }
        : {}),
    },
  };
}
export function chooseAnswers(answers, schema) {
  const result = {};
  for (const [name, q] of Object.entries(schema)) {
    const a = answers?.[name];
    if (a?.type !== "choice" || !Object.hasOwn(q.criteria, a.choice))
      throw new Error(`Invalid ${name} decision`);
    result[name] = a.choice;
  }
  return result;
}
// Test baseline for the SAME public snapshot and normal-input execution, not a model substitute in results.
export function referenceChoices(v, approach) {
  if (approach === "commander") return { goal: "progress" };
  return {
    movement: "navigate",
    combat: v.visible.length && v.magazine > 0 && !v.reloading ? "engage" : "scan",
    utility:
      v.magazine === 0 && v.reserve > 0
        ? "reload"
        : v.nearbyDoor || [3, 13].includes(v.interactPrompt)
          ? "interact"
          : "nothing",
  };
}
function movementBits(relative) {
  let bits = 0;
  if (Math.cos(relative) > 0.38) bits |= INPUT.advance;
  else if (Math.cos(relative) < -0.38) bits |= INPUT.retreat;
  if (Math.sin(relative) > 0.38) bits |= INPUT.strafe_right;
  else if (Math.sin(relative) < -0.38) bits |= INPUT.strafe_left;
  return bits;
}
export function commanderUtility(h, v, m) {
  const weapons = h.inventory
    .filter((g) => g.magazine + g.reserve > 0)
    .sort(
      (a, b) =>
        Number(b.id !== 1) - Number(a.id !== 1) ||
        b.magazine + b.reserve - (a.magazine + a.reserve),
    );
  const bossTarget = v.visible[0] && v.visible[0].skin >= 12 && v.visible[0].skin <= 36;
  const preferred = weapons.find(
    (g) => g.id === (bossTarget ? 4 : v.visible[0]?.distance < 4 ? 2 : 3),
  );
  if (!h.reloading && preferred && preferred.id !== h.weapon + 1 && h.elapsedMs > m.equipUntil) {
    m.equipUntil = h.elapsedMs + 1500;
    return `equip_${preferred.id}`;
  } else if (!h.reloading && h.ammo === 0 && h.reserve === 0 && weapons.length) {
    return `equip_${weapons[0].id}`;
  } else if (
    !h.reloading &&
    h.reserve > 0 &&
    (h.ammo === 0 || (!v.visible.length && h.ammo <= v.magazineCapacity / 3))
  ) {
    return "reload";
  } else if (v.nearbyDoor || [3, 13].includes(h.prompt)) {
    return "interact";
  }
  return "nothing";
}
export function executeGoal(w, choices, m, approach, onFrame = () => {}) {
  let s = snapshot(w),
    initial = s;
  const initiallyVisible = observe(initial).visibleEnemies.length;
  const segments = [],
    interventions = {};
  let frames = 0,
    reason = "duration",
    events = 0;
  const max = approach === "compact" ? 120 : approach === "persistent" ? 720 : 1440;
  let anchor = s,
    stall = 0;
  while (frames < max && s.hud.state === 0) {
    remember(s, m);
    const v = view(s, m),
      h = s.hud,
      auto = approach === "commander";
    let combat = choices.combat,
      utility = choices.utility ?? "nothing",
      movement = choices.movement;
    const mark = (k) => (interventions[k] = (interventions[k] ?? 0) + 1);
    if (auto) {
      movement = "navigate";
      combat = v.visible.length ? "engage" : "scan";
      utility = commanderUtility(h, v, m);
      if (utility !== "nothing") mark(utility.startsWith("equip_") ? "equip" : utility);
      if (choices.goal === "cautious" && v.visible.length) movement = "hold";
    }
    let bits = 0,
      turn = 0,
      slot;
    const target = v.visible[0];
    if (
      combat === "engage" &&
      target &&
      h.ammo > 0 &&
      !h.reloading &&
      utility !== "reload" &&
      !v.splashRisk
    ) {
      turn = angle(Math.atan2(target.y - h.y, target.x - h.x) - h.yaw);
      bits |= INPUT.fire;
    } else if (v.plan.route && movement === "navigate")
      turn = angle(Math.atan2(v.plan.route.y - h.y, v.plan.route.x - h.x) - h.yaw);
    else if (combat === "scan" && !target) {
      turn = 0.18;
    }
    const delta = Math.max(-0.45, Math.min(0.45, turn)),
      facing = h.yaw + delta;
    if (movement === "navigate" && v.plan.route) {
      const bearing = Math.atan2(v.plan.route.y - h.y, v.plan.route.x - h.x);
      const d = Math.hypot(v.plan.route.x - h.x, v.plan.route.y - h.y);
      if (d > 0.1) bits |= movementBits(angle(bearing - facing));
    } else if (movement === "strafe_left") bits |= INPUT.strafe_left;
    else if (movement === "strafe_right") bits |= INPUT.strafe_right;
    if (auto && target) {
      // Orbit at medium range, retreat from close hostiles; ordinary WASD inputs.
      if (target.distance < 3.0) {
        bits &= ~15;
        bits |= INPUT.retreat;
        mark("evade");
      } else if (h.hurt > 0.1 || choices.goal !== "aggressive") {
        bits &= ~15;
        bits |= Math.floor(h.elapsedMs / 1400) % 2 ? INPUT.strafe_left : INPUT.strafe_right;
        mark("strafe");
      }
    }
    if (utility === "reload" && h.reserve > 0 && !h.reloading) {
      bits &= ~INPUT.fire;
      bits |= INPUT.reload;
    }
    if (utility === "interact") {
      // USE is edge-triggered: pulse and release it, including consecutive model choices.
      if (frames % 12 < 6) bits |= INPUT.interact;
      // A route can notice a door before USE reaches it; approach while pulsing.
      const withinDoorReach =
        v.nearbyDoor && Math.hypot(v.plan.route.x - h.x, v.plan.route.y - h.y) < 1.1;
      if (withinDoorReach || [3, 13].includes(h.prompt)) bits &= ~15;
    }
    const gun = h.inventory.find(
      (g) => `equip_${g.id}` === utility && g.id !== h.weapon + 1 && g.magazine + g.reserve > 0,
    );
    if (gun) slot = gun.id - 1;
    if (auto && bits & 15) {
      const safe = (b) => {
        let dx = 0,
          dy = 0;
        if (b & INPUT.advance) {
          dx += Math.cos(facing);
          dy += Math.sin(facing);
        }
        if (b & INPUT.retreat) {
          dx -= Math.cos(facing);
          dy -= Math.sin(facing);
        }
        if (b & INPUT.strafe_right) {
          dx -= Math.sin(facing);
          dy += Math.cos(facing);
        }
        if (b & INPUT.strafe_left) {
          dx += Math.sin(facing);
          dy -= Math.cos(facing);
        }
        const length = Math.hypot(dx, dy) || 1;
        const x = h.x + (dx / length) * 0.55,
          y = h.y + (dy / length) * 0.55;
        for (const [ox, oy] of [
          [0, 0],
          [0.25, 0],
          [-0.25, 0],
          [0, 0.25],
          [0, -0.25],
        ]) {
          const c = Math.floor(y + oy) * s.width + Math.floor(x + ox),
            k = s.map[c];
          if (![0, 10].includes(k) && !([8, 9].includes(k) && s.doors[c] >= 0.98)) return false;
        }
        return s.floor?.[Math.floor(y) * s.width + Math.floor(x)] !== 4;
      };
      if (!safe(bits & 15)) {
        const alternative = [
          INPUT.strafe_left,
          INPUT.strafe_right,
          INPUT.retreat,
          INPUT.advance,
        ].find(safe);
        bits = (bits & ~15) | (alternative ?? 0);
        mark("avoid_wall_or_hazard");
      }
    }
    const input = {
      bits,
      mx: delta / 0.0062,
      my: 0,
      ...(slot !== undefined ? { weaponSlot: slot } : {}),
    };
    const interval = step(w, input, 6, (f) => {
      onFrame(f, s);
      s = f;
      return f.hud.state === 0;
    });
    s = interval.snapshot;
    segments.push({ input, frames: interval.frames });
    frames += interval.frames;
    events |= interval.events;
    if (s.hud.elapsedMs - anchor.hud.elapsedMs >= 600) {
      if (
        (bits & 15 || (utility === "interact" && v.nearbyDoor)) &&
        Math.hypot(s.hud.x - anchor.hud.x, s.hud.y - anchor.hud.y) < 0.04
      )
        stall++;
      else stall = 0;
      anchor = s;
      if (stall >= 2 && v.plan.route) {
        const c = cellOf(s);
        m.blocked.set(`${c}:${v.plan.route.cell}`, s.hud.elapsedMs + 15000);
        m.failed.set(`cell:${v.plan.route.goal}`, s.hud.elapsedMs + 15000);
        if (v.plan.target?.id !== undefined)
          m.failed.set(`item:${v.plan.target.id}`, s.hud.elapsedMs + 15000);
        m.target = null;
        mark("reroute");
        stall = 0;
      }
    }
    if (!auto) {
      if (s.hud.ammo === 0 && initial.hud.ammo > 0) {
        reason = "empty";
        break;
      }
      if (s.hud.objective !== initial.hud.objective) {
        reason = "objective";
        break;
      }
      if (
        (initial.hud.reloading > 0 || choices.utility === "reload") &&
        frames > 6 &&
        s.hud.reloading === 0 &&
        s.hud.ammo > initial.hud.ammo
      ) {
        reason = "reloaded";
        break;
      }
      if (approach === "persistent" && !initiallyVisible && observe(s).visibleEnemies.length) {
        reason = "enemy_appeared";
        break;
      }
      if (s.hud.health <= initial.hud.health - 20) {
        reason = "damage";
        break;
      }
      if (
        approach === "compact" &&
        (s.hud.kills > initial.hud.kills ||
          (!observe(initial).visibleEnemies.length && v.visible.length))
      ) {
        reason = "combat_event";
        break;
      }
    }
    if (
      s.hud.objective !== initial.hud.objective ||
      Boolean(s.hud.bossHealth) !== Boolean(initial.hud.bossHealth) ||
      s.hud.bossPhase !== initial.hud.bossPhase
    ) {
      reason = "mission_event";
      break;
    }
  }
  return {
    snapshot: s,
    segments,
    frames,
    events,
    interventions,
    reason,
    stateHash: fingerprint(s),
  };
}
