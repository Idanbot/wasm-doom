// Build one deterministic reachable tree per observation, shared by all goals.
// Closed ordinary doors remain routable: the input executor approaches and uses them.
export function navigationTree(s, memory) {
  const cell = (x, y) => {
    if (
      !Number.isFinite(x) ||
      !Number.isFinite(y) ||
      x < 0 ||
      y < 0 ||
      x >= s.width ||
      y >= s.height
    )
      return -1;
    return Math.floor(y) * s.width + Math.floor(x);
  };
  const start = cell(s.hud.x, s.hud.y),
    parent = new Int32Array(s.map.length).fill(-1),
    queue = [];
  if (start >= 0) {
    parent[start] = start;
    queue.push(start);
  }
  for (let n = 0; n < queue.length; n++) {
    const current = queue[n],
      x = current % s.width,
      y = Math.floor(current / s.width);
    for (const [nx, ny] of [
      [x + 1, y],
      [x, y + 1],
      [x - 1, y],
      [x, y - 1],
    ]) {
      const next = cell(nx, ny);
      if (
        next < 0 ||
        parent[next] >= 0 ||
        ![0, 8, 10].includes(s.map[next]) ||
        (memory.blocked.get(`${current}:${next}`) ?? 0) > s.hud.elapsedMs
      )
        continue;
      parent[next] = current;
      queue.push(next);
    }
  }
  return (target) => {
    const goal = cell(target.x, target.y);
    if (goal < 0 || parent[goal] < 0) return null;
    if (start === goal) return { ...target, cell: goal, goal };
    let next = goal;
    while (parent[next] !== start) next = parent[next];
    return {
      x: (next % s.width) + 0.5,
      y: Math.floor(next / s.width) + 0.5,
      cell: next,
      goal,
      mode: target.mode,
    };
  };
}
