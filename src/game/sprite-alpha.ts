export function keySpriteAlpha(data: Uint8ClampedArray, size: number, preserveDark = false) {
  // Generated BLACKSITE sheets already carry alpha from the offline key.
  // The optional flag lets them keep near-black armor that is valid subject
  // detail rather than treating it as an old procedural background key.
  const n = size * size;
  const seen = new Uint8Array(n);
  const q: number[] = [];
  const isKey = (i: number) => {
    const o = i * 4;
    const r = data[o];
    const g = data[o + 1];
    const b = data[o + 2];
    const a = data[o + 3];
    if (a < 16) return true;
    if (r > 220 && b > 170 && g < 100) return true;
    const mx = r > g ? (r > b ? r : b) : g > b ? g : b;
    const mn = r < g ? (r < b ? r : b) : g < b ? g : b;
    return !preserveDark && (mx < 28 || (mx < 42 && mx - mn < 10));
  };
  const seed = (x: number, y: number) => {
    const i = y * size + x;
    if (isKey(i)) q.push(i);
  };
  for (let x = 0; x < size; x++) {
    seed(x, 0);
    seed(x, size - 1);
  }
  for (let y = 0; y < size; y++) {
    seed(0, y);
    seed(size - 1, y);
  }
  while (q.length) {
    const i = q.pop()!;
    if (seen[i]) continue;
    seen[i] = 1;
    if (!isKey(i)) continue;
    data[i * 4 + 3] = 0;
    const x = i % size;
    const y = (i / size) | 0;
    if (x > 0) q.push(i - 1);
    if (x + 1 < size) q.push(i + 1);
    if (y > 0) q.push(i - size);
    if (y + 1 < size) q.push(i + size);
  }
  // A model can leave a magenta island inside a closed silhouette (between
  // an arm and the torso, under a weapon, etc.). Those pixels are background
  // too, even though they are not connected to the canvas edge.
  for (let i = 0; i < n; i++) {
    const o = i * 4;
    const r = data[o];
    const g = data[o + 1];
    const b = data[o + 2];
    if (r > 220 && b > 170 && g < 100) data[o + 3] = 0;
  }
  for (let i = 0; i < n; i++) {
    if (data[i * 4 + 3] === 0) continue;
    const x = i % size;
    const y = (i / size) | 0;
    let edge = false;
    if (x > 0 && data[(i - 1) * 4 + 3] === 0) edge = true;
    if (x + 1 < size && data[(i + 1) * 4 + 3] === 0) edge = true;
    if (y > 0 && data[(i - size) * 4 + 3] === 0) edge = true;
    if (y + 1 < size && data[(i + size) * 4 + 3] === 0) edge = true;
    if (!edge) continue;
    const r = data[i * 4];
    const g = data[i * 4 + 1];
    const b = data[i * 4 + 2];
    if (!preserveDark && r + g + b < 90) data[i * 4 + 3] = 0;
  }
}
