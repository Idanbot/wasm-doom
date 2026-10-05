import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { keySpriteAlpha } from "./sprite-alpha.ts";

function darkSubjectWithTransparentEdge(size = 8) {
  const data = new Uint8ClampedArray(size * size * 4);
  for (let y = 1; y < size - 1; y++) {
    for (let x = 1; x < size - 1; x++) {
      const o = (y * size + x) * 4;
      data[o] = 20;
      data[o + 1] = 20;
      data[o + 2] = 20;
      data[o + 3] = 255;
    }
  }
  return data;
}

describe("enemy sprite alpha key", () => {
  it("preserves dark generated armor when the sheet edge is transparent", () => {
    const size = 8;
    const data = darkSubjectWithTransparentEdge(size);
    keySpriteAlpha(data, size, true);
    const center = (Math.floor(size / 2) * size + Math.floor(size / 2)) * 4 + 3;
    assert.equal(data[center], 255);
    assert.equal(
      data.filter((_, i) => i % 4 === 3 && data[i] > 0).length,
      (size - 2) * (size - 2),
    );
  });

  it("still removes an enclosed magenta gap inside a dark silhouette", () => {
    const size = 8;
    const data = darkSubjectWithTransparentEdge(size);
    const hole = (3 * size + 3) * 4;
    data[hole] = 255;
    data[hole + 1] = 0;
    data[hole + 2] = 255;
    keySpriteAlpha(data, size, true);
    assert.equal(data[hole + 3], 0);
    const armor = (3 * size + 4) * 4 + 3;
    assert.equal(data[armor], 255);
  });
});

import { normalizeSensitivity, readSensitivity } from "./input-settings.ts";
import { automapCells } from "./automap-data.ts";

describe("input and minimap contracts", () => {
  it("supports both sensitivity endpoints and clamps legacy saved values", () => {
    assert.equal(normalizeSensitivity("0.1"), 0.1);
    assert.equal(normalizeSensitivity(3), 3);
    assert.equal(normalizeSensitivity("3.5"), 3);
    assert.equal(normalizeSensitivity(0), 0.1);
    assert.equal(normalizeSensitivity(null), 0.5);
    assert.equal(normalizeSensitivity("bad"), 0.5);
  });
  it("preserves layout coordinates while displaying hazard floor and open doors as floor", () => {
    const map = Uint8Array.from([1, 0, 10, 8, 8, 9, 9, 2]);
    const doors = Float32Array.from([0, 0, 0, .97, 1, 0, 1, 0]);
    assert.deepEqual([...automapCells(map, doors)], [1, 0, 0, 8, 0, 9, 0, 2]);
    assert.deepEqual([...map], [1, 0, 10, 8, 8, 9, 9, 2]);
  });
});

it("migrates the old automatic sensitivity default and preserves versioned choices", () => {
  const read = (value: string | null, version: string | null = null) => readSensitivity({getItem: key => key === "blacksite-sensitivity" ? value : version});
  assert.equal(read(null), .5);
  assert.equal(read("1.4"), .5);
  assert.equal(read("1.4", "2"), 1.4);
  assert.equal(read(".8"), .8);
  assert.equal(read("3.5"), 3);
});
