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

import { DEFAULT_BINDINGS, INPUT_BITS, assignBinding, controlBits, controlForCode, normalizeBindings, actionLabel } from './controls.ts';
import { DEFAULT_GFX, effectIntensity } from './types.ts';

describe('custom controls and comfort settings',()=>{
  it('preserves legacy defaults and exposes all 33 direct weapon choices',()=>{
    assert.equal(controlBits(DEFAULT_BINDINGS,['KeyW','KeyR','Space']),INPUT_BITS.W|INPUT_BITS.RELOAD|INPUT_BITS.FIRE);
    assert.equal(controlForCode(DEFAULT_BINDINGS,'Digit1')?.slot,0);
    assert.equal(Object.keys(DEFAULT_BINDINGS).filter(id=>id.startsWith('weapon')).length,33);
    assert.equal(DEFAULT_BINDINGS.pause?.includes('KeyP'),true);
  });
  it('replaces a key, rejects conflicts and keeps old keys inactive',()=>{
    const result=assignBinding(DEFAULT_BINDINGS,'reload',0,'KeyF');
    assert.ok(result.bindings);
    assert.equal(controlBits(result.bindings,['KeyR']),0);
    assert.equal(controlBits(result.bindings,['KeyF']),INPUT_BITS.RELOAD);
    assert.equal(actionLabel(result.bindings,'reload'),'F');
    assert.ok(assignBinding(result.bindings,'interact',0,'KeyF').error);
    assert.ok(assignBinding(result.bindings,'reload',0,'Escape').error);
    assert.ok(assignBinding(result.bindings,'unknown',0,'KeyF').error);
    assert.ok(assignBinding(result.bindings,'reload',3,'KeyF').error);
    assert.deepEqual(DEFAULT_BINDINGS.reload,['KeyR']);
  });
  it('validates saved bindings, preserves unbound actions and removes duplicates',()=>{
    const value=normalizeBindings({forward:['KeyI','KeyI','Escape',17],reload:[],fire:['Mouse2'],pause:['Mouse2'],unknown:['KeyZ']});
    assert.deepEqual(value.forward,['KeyI']);
    assert.deepEqual(value.reload,[]);
    assert.deepEqual(value.fire,['Mouse2']);
    assert.deepEqual(value.pause,[]);
    assert.equal(value.unknown,undefined);
    assert.deepEqual(normalizeBindings(null),DEFAULT_BINDINGS);
  });
  it('clamps effects without accepting corrupt persisted values',()=>{
    assert.equal(effectIntensity(0),0);assert.equal(effectIntensity(.4),.4);
    assert.equal(effectIntensity(-1),0);assert.equal(effectIntensity(4),1);
    for(const bad of [NaN,Infinity,'0',null,undefined])assert.equal(effectIntensity(bad),1);
    assert.equal(DEFAULT_GFX.flash,1);assert.equal(DEFAULT_GFX.shake,1);assert.equal(DEFAULT_GFX.debris,1);
  });
});
