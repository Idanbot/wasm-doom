/** World-space cosmetic layers. Never sampled from the HUD or weapon framebuffer.
 * Both GPU backends use the same masks, eight-step scattering and 18% reflection cap.
 * Reflections are ceiling/light probes, rather than traced scene geometry.
 */
import { T_SECTOR_SURFACE } from "./atlas-slots.ts";

// Welding, frost, biomass, circuits, ions, phase stains, moisture, polished vaults.
export const SURFACE_FAMILIES = [0, 1, 2, 3, 4, 5, 1, 3, 0, 6, 7, 0, 6, 7, 4, 3, 2, 4, 6, 5, 4, 6, 5, 1, 7] as const;

export const MATERIAL_WGSL = `
fn surface_mask(p: vec2<f32>, level: f32) -> vec3<f32> {
  let cell = floor(p);
  let seed = fract(dot(cell, vec2<f32>(0.1031, 0.11369)) + level * 0.137);
  let q = fract(p) - vec2<f32>(0.48 + seed * 0.1, 0.48 - seed * 0.1);
  let coverage = 1.0 - smoothstep(0.045, 0.19, dot(q * vec2<f32>(1.0, 1.7), q * vec2<f32>(1.0, 1.7)));
  let families = array<i32, 25>(${SURFACE_FAMILIES.join(",")});
  let family = families[clamp(i32(level), 0, 24)];
  let moisture = select(0.6, 1.0, family == 6 || family == 7);
  let wet = coverage * smoothstep(0.60, 0.90, seed) * moisture;
  let grime = coverage * (1.0 - smoothstep(0.15, 0.48, seed)) * select(1.0, 0.45, family == 1 || family == 3);
  var stripe = (1.0 - smoothstep(0.025, 0.045, abs(fract((p.x + p.y) * 5.0) - 0.5))) * step(0.84, seed) * step(0.33, abs(q.y));
  if (family == 1) { stripe = pow(max(0.0, 1.0 - abs(q.x + q.y * 0.5) * 22.0), 2.0) * coverage * 0.55; }
  if (family == 2) { stripe = pow(max(0.0, sin(p.x * 24.0) * sin(p.y * 21.0)), 8.0) * coverage * 0.7; }
  if (family == 3 || family == 7) { stripe = (1.0 - smoothstep(0.01, 0.025, min(abs(q.x), abs(q.y)))) * coverage * 0.65; }
  return vec3<f32>(wet, grime, stripe);
}
fn surface_finish(base: vec3<f32>, p: vec2<f32>, dist: f32, floor_surface: bool, style: f32) -> vec3<f32> {
  let mask = surface_mask(p, view.v[3].z);
  let tint = arena_color(i32(view.v[3].z));
  var rgb = base * (1.0 - mask.y * 0.22);
  // Cosmetic paint stays off the objective and damaging arena markers.
  if (floor_surface && style < 1.5) { rgb = mix(rgb, tint * 0.48, mask.z * 0.38); }
  if (floor_surface && style < 1.5 && mask.x > 0.001) {
    let ripples = vec2<f32>(sin(p.y * 18.0 + view.v[1].w * 0.8), cos(p.x * 16.0 + view.v[1].w * 0.6)) * 0.012;
    let probe = p + ripples;
    let ceiling = sample_atlas(${T_SECTOR_SURFACE} + i32(view.v[3].z) * 5 + 4, probe.x, probe.y).rgb;
    let light = max(light_at(probe.x, probe.y), vec3<f32>(0.0));
    let fresnel = 0.04 + 0.14 * dist / (dist + 4.0);
    rgb = mix(rgb, clamp(ceiling * 0.20 + light * 0.55, vec3<f32>(0.0), vec3<f32>(1.0)), mask.x * fresnel);
  }
  if (floor_surface) {
    let cell=vec2<i32>(floor(p));
    if (all(cell>=vec2<i32>(0)) && all(cell<vec2<i32>(48,32))) {
      let data=textureLoad(surface_tex,cell,0);
      let sp=clamp(p-vec2<f32>(0.5),vec2<f32>(0.0),vec2<f32>(46.0,30.0));
      let si=vec2<i32>(floor(sp));let sf=fract(sp);
      let sa=mix(textureLoad(surface_tex,si,0).x,textureLoad(surface_tex,si+vec2<i32>(1,0),0).x,sf.x);
      let sb=mix(textureLoad(surface_tex,si+vec2<i32>(0,1),0).x,textureLoad(surface_tex,si+vec2<i32>(1,1),0).x,sf.x);
      rgb *= 1.0-mix(sa,sb,sf.y);
      let kind=i32(floor(data.y));
      if (kind>0 && style<1.5) {
        let q=fract(p)-data.zw;
        let stain=pow(max(0.0,1.0-dot(q,q)/0.18),2.0)*fract(data.y)*0.7;
        var color=vec3<f32>(18.0,13.0,11.0)/255.0;
        if(kind==2){color=vec3<f32>(61.0,91.0,18.0)/255.0;}
        if(kind==3){color=vec3<f32>(93.0,91.0,86.0)/255.0;}
        rgb=mix(rgb,color,stain);
      }
    }
  }
  return clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
}
fn lit_air(rgb: vec3<f32>, start: vec2<f32>, end: vec2<f32>, height: f32) -> vec3<f32> {
  let distance = min(length(end - start), 24.0);
  var scatter = vec3<f32>(0.0);
  for (var i = 1; i <= 8; i++) {
    let reach = start + (end - start) * min(1.0, 24.0 / max(length(end - start), 0.001));
    let p = mix(start, reach, f32(i) / 8.0);
    let cell = vec2<i32>(clamp(floor(p), vec2<f32>(0.0), vec2<f32>(47.0, 31.0)));
    let light = max(textureLoad(light_tex, cell, 0).rgb, vec3<f32>(0.0));
    // Narrow beams are fixed to ceiling slats in world space, never camera space.
    let band = 1.0 - smoothstep(0.02, 0.10, abs(fract(p.x * 0.5 + p.y * 0.11) - 0.5));
    let slat = band * band;
    let dust = 0.65 + 0.35 * fract(p.x * 0.13 + p.y * 0.31 + view.v[1].w * 0.02);
    scatter += light * (0.003 + slat * 0.016 * (0.5 + height * 0.5)) * dust;
  }
  let extinction = exp(-distance * 0.004);
  return clamp(rgb * extinction + min(scatter * distance / 8.0, vec3<f32>(0.14)), vec3<f32>(0.0), vec3<f32>(1.0));
}
`;

export const MATERIAL_GL = `
vec3 surfaceMask(vec2 p, float level) {
  vec2 cell = floor(p);
  float seed = fract(dot(cell, vec2(0.1031, 0.11369)) + level * 0.137);
  vec2 q = fract(p) - vec2(0.48 + seed * 0.1, 0.48 - seed * 0.1);
  float coverage = 1.0 - smoothstep(0.045, 0.19, dot(q * vec2(1.0, 1.7), q * vec2(1.0, 1.7)));
  const int families[25] = int[25](${SURFACE_FAMILIES.join(",")});
  int family = families[clamp(int(level), 0, 24)];
  float moisture = family == 6 || family == 7 ? 1.0 : .6;
  float wet = coverage * smoothstep(0.60, 0.90, seed) * moisture;
  float grime = coverage * (1.0 - smoothstep(0.15, 0.48, seed)) * (family == 1 || family == 3 ? .45 : 1.0);
  float stripe = (1.0 - smoothstep(0.025, 0.045, abs(fract((p.x + p.y) * 5.0) - 0.5))) * step(0.84, seed) * step(0.33, abs(q.y));
  if (family == 1) stripe = pow(max(0.0, 1.0 - abs(q.x + q.y * .5) * 22.0), 2.0) * coverage * .55;
  if (family == 2) stripe = pow(max(0.0, sin(p.x * 24.0) * sin(p.y * 21.0)), 8.0) * coverage * .7;
  if (family == 3 || family == 7) stripe = (1.0 - smoothstep(.01, .025, min(abs(q.x), abs(q.y)))) * coverage * .65;
  return vec3(wet, grime, stripe);
}
vec3 surfaceFinish(vec3 base, vec2 p, float dist, bool floorSurface, float style) {
  vec3 mask = surfaceMask(p, view3.z);
  vec3 tint = arenaColor(int(view3.z));
  vec3 rgb = base * (1.0 - mask.y * 0.22);
  if (floorSurface && style < 1.5) rgb = mix(rgb, tint * 0.48, mask.z * 0.38);
  if (floorSurface && style < 1.5 && mask.x > .001) {
    vec2 ripples = vec2(sin(p.y * 18.0 + view1.w * 0.8), cos(p.x * 16.0 + view1.w * 0.6)) * 0.012;
    vec2 probe = p + ripples;
    vec3 ceiling = sampleAtlas(${T_SECTOR_SURFACE} + int(view3.z) * 5 + 4, probe).rgb;
    vec3 light = max(lightAt(probe), vec3(0.0));
    float fresnel = 0.04 + 0.14 * dist / (dist + 4.0);
    rgb = mix(rgb, clamp(ceiling * 0.20 + light * 0.55, vec3(0.0), vec3(1.0)), mask.x * fresnel);
  }
  if (floorSurface) {
    ivec2 cell=ivec2(floor(p));
    if(all(greaterThanEqual(cell,ivec2(0))) && all(lessThan(cell,ivec2(48,32)))) {
      vec4 data=texelFetch(surfaceTex,cell,0);
      vec2 sp=clamp(p-.5,vec2(0.0),vec2(46.0,30.0));ivec2 si=ivec2(floor(sp));vec2 sf=fract(sp);
      float sa=mix(texelFetch(surfaceTex,si,0).x,texelFetch(surfaceTex,si+ivec2(1,0),0).x,sf.x);
      float sb=mix(texelFetch(surfaceTex,si+ivec2(0,1),0).x,texelFetch(surfaceTex,si+ivec2(1,1),0).x,sf.x);
      rgb*=1.0-mix(sa,sb,sf.y);
      int kind=int(floor(data.y));
      if(kind>0 && style<1.5) {
        vec2 q=fract(p)-data.zw;
        float stain=pow(max(0.0,1.0-dot(q,q)/.18),2.0)*fract(data.y)*.7;
        vec3 color=kind==2?vec3(61.0,91.0,18.0)/255.0:kind==3?vec3(93.0,91.0,86.0)/255.0:vec3(18.0,13.0,11.0)/255.0;
        rgb=mix(rgb,color,stain);
      }
    }
  }
  return clamp(rgb, vec3(0.0), vec3(1.0));
}
vec3 litAir(vec3 rgb, vec2 start, vec2 end, float height) {
  float distance = min(length(end - start), 24.0);
  vec3 scatter = vec3(0.0);
  for (int i = 1; i <= 8; i++) {
    vec2 reach = start + (end - start) * min(1.0, 24.0 / max(length(end - start), 0.001));
    vec2 p = mix(start, reach, float(i) / 8.0);
    ivec2 cell = ivec2(clamp(floor(p), vec2(0.0), vec2(47.0, 31.0)));
    vec3 light = max(texelFetch(lightTex, cell, 0).rgb, vec3(0.0));
    float band = 1.0 - smoothstep(.02, .10, abs(fract(p.x * .5 + p.y * .11) - .5));
    float slat = band * band;
    float dust = .65 + .35 * fract(p.x * .13 + p.y * .31 + view1.w * .02);
    scatter += light * (0.003 + slat * 0.016 * (0.5 + height * 0.5)) * dust;
  }
  float extinction = exp(-distance * 0.004);
  return clamp(rgb * extinction + min(scatter * distance / 8.0, vec3(0.14)), vec3(0.0), vec3(1.0));
}
`;
