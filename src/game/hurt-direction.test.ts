import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

/**
 * The directional hurt vignette lives in two shader strings that cannot be unit
 * tested directly. What we *can* pin is the mapping contract they must obey:
 *
 *   - `hurtDir` is 0 when the source is dead ahead, +PI/2 to the player's left
 *     and +-PI behind (engine/src/lib.rs, `damage_player_from`).
 *   - Screen space is +x right, +y down, so a source ahead must highlight the
 *     top of the frame, one to the left the left edge, and one behind the
 *     bottom edge.
 *
 * The shaders express this as `(-sin(theta), -cos(theta))`. This test extracts
 * that expression from both backends and checks it against the contract, so a
 * sign flip (which would light the opposite edge) fails here rather than in
 * front of a player.
 */
const blit = readFileSync(new URL("./blit.ts", import.meta.url), "utf8");

/** Screen-space unit vector the shader uses for a given yaw-relative bearing. */
function shaderScreenDirection(theta: number): [number, number] {
  return [-Math.sin(theta), -Math.cos(theta)];
}

function dot(a: [number, number], b: [number, number]) {
  return a[0] * b[0] + a[1] * b[1];
}

describe("directional hurt vignette", () => {
  it("points ahead at the top of the frame", () => {
    const dir = shaderScreenDirection(0);
    // Screen up is -y.
    assert.ok(dir[1] < -0.99, `ahead must map to screen-up, got ${dir}`);
    assert.ok(Math.abs(dir[0]) < 1e-6, "ahead must not be offset horizontally");
  });

  it("points a left-hand source at the left edge", () => {
    // +PI/2 is the player's left; on screen that is -x.
    const dir = shaderScreenDirection(Math.PI / 2);
    assert.ok(dir[0] < -0.99, `left must map to screen-left, got ${dir}`);
    assert.ok(Math.abs(dir[1]) < 1e-6, "left must not be offset vertically");
  });

  it("points a right-hand source at the right edge", () => {
    const dir = shaderScreenDirection(-Math.PI / 2);
    assert.ok(dir[0] > 0.99, `right must map to screen-right, got ${dir}`);
  });

  it("points a source behind the player at the bottom of the frame", () => {
    for (const theta of [Math.PI, -Math.PI]) {
      const dir = shaderScreenDirection(theta);
      assert.ok(dir[1] > 0.99, `behind must map to screen-down, got ${dir}`);
    }
  });

  it("is a unit vector at every bearing", () => {
    for (let i = 0; i < 16; i++) {
      const theta = (i / 16) * Math.PI * 2 - Math.PI;
      const dir = shaderScreenDirection(theta);
      assert.ok(Math.abs(Math.hypot(dir[0], dir[1]) - 1) < 1e-6, `not unit at ${theta}`);
    }
  });

  it("brightens the arc nearest the source", () => {
    // The shader uses dot(screenDir, sourceDir) to bias the vignette.
    const left = shaderScreenDirection(Math.PI / 2);
    const leftEdge: [number, number] = [-1, 0];
    const rightEdge: [number, number] = [1, 0];
    assert.ok(dot(left, leftEdge) > 0.99, "a left-hand hit must brighten the left edge");
    assert.ok(dot(left, rightEdge) < -0.99, "a left-hand hit must not brighten the right edge");
  });

  it("both shader backends use the same signed expression", () => {
    // Greedy to the last `);` on the line: the expression contains nested calls.
    const wgsl = blit.match(/let bearing = vec2<f32>\((.*)\);/);
    const glsl = blit.match(/vec2 bearing = vec2\((.*)\);/);
    assert.ok(wgsl, "WebGPU post shader must compute a bearing vector");
    assert.ok(glsl, "WebGL2 post shader must compute a bearing vector");
    const norm = (e: string) => e.replace(/\s+/g, "").replace(/u\.feedback\.x/g, "feedback.x");
    assert.equal(norm(wgsl[1]!), norm(glsl[1]!), "the two backends must agree");
    // Guard the sign explicitly: flipping sin() lights the wrong edge.
    assert.equal(norm(wgsl[1]!), "-sin(feedback.x),-cos(feedback.x)");
  });

  it("the low-health strain is driven by the HUD strain field", () => {
    assert.ok(
      /if \(u\.feedback\.y > 0\.01\)/.test(blit),
      "the WebGPU shader must gate the strain vignette",
    );
    assert.ok(
      /if \(feedback\.y > 0\.01\)/.test(blit),
      "the WebGL2 shader must gate the strain vignette",
    );
  });
});