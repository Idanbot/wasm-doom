# BLACKSITE improvement suggestions

104 proposals across 13 aspects; highest value first within each aspect. These
have explicit Done/Undone status for this implementation pass. Unselected ideas
remain Undone. Confidence estimates
combine likely player benefit and feasibility in the current Rust/WASM browser
architecture; they are judgments, not measured probabilities. Multiplayer ideas
need a separate scope decision before adding hosting or network services.

## Performance

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Adaptive render resolution | Keep combat smooth on weaker devices. | Adjust resolution from rolling frame times, with hysteresis and a floor. | 95% | Undone |
| Active-sector residency | Reduce initial RAM and downloads. | Load current/next-sector textures; retain shared weapons and enemy families. | 92% | Undone |
| GPU texture budget | Prevent context loss on phones. | Track estimated atlas/mipmap memory; choose smaller layers before allocation. | 93% | Undone |
| Visibility-based FX budgets | Spend work on effects players see. | Cull occluded/offscreen emitters; cap particles by screen coverage. | 94% | Undone |
| Dirty atlas uploads | Avoid redundant GPU transfers. | Track changed layers and upload only those after theme/material changes. | 90% | Undone |
| Preallocated simulation buffers | Reduce busy-fight frame spikes. | Reuse entity, cue and projectile scratch arrays; profile allocations first. | 90% | Undone |
| Decouple HUD animation updates | Reduce React work during combat. | Publish discrete state changes; animate meters and timers outside rerenders. | 88% | Undone |
| Background-tab suspension | Save battery and avoid resume bursts. | Suspend cosmetic work; reset frame accumulation on visibility restoration. | 94% | Undone |

Frame scheduling reference: [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame).

## Loading and reliability

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Menu-first asset tier | Make the first screen appear sooner. | Prioritize menu art/fonts, then current sector, then remaining campaign assets. | 96% | Done |
| Versioned asset manifest | Prevent mixed-build art and engine caches. | Publish sizes/hashes and a build ID; invalidate mismatched cached entries. | 94% | Done |
| Byte-weighted progress | Better match remaining wait time. | Weight downloads by manifest bytes; reserve progress for decode/upload. | 92% | Done |
| Persistent validated cache | Speed repeat visits and flaky connections. | Cache successfully decoded versioned assets; evict by storage budget. | 89% | Done |
| Offline first-sector package | Support intermittent connectivity. | Cache a complete starter sector; visibly disable unavailable campaign sectors. | 84% | Undone |
| Device-aware preload limits | Avoid saturating mobile memory/network. | Tune a shared download/decode budget from measured load times and failures. | 88% | Undone |
| Resume partially loaded boots | Avoid repeating successful work after exhaustion. | Retain validated asset results and retry only the failed manifest entries. | 93% | Undone |
| Font recovery | Keep menus readable after font failures. | Validate local font faces; retry failed files and retain a metric-compatible fallback. | 91% | Undone |

Timeout/cancellation reference: [AbortSignal timeout](https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal/timeout_static).

## Sound

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Voice-aware music ducking | Make dialogue easier to understand. | Fade music/ambience down during the existing voice lane; restore smoothly. | 96% | Undone |
| Loudness-normalized voice assets | Remove large differences between bosses. | Measure integrated loudness/peaks offline; normalize with limiter headroom. | 95% | Undone |
| Material footsteps | Make sectors sound physically different. | Select metal, concrete, grate and liquid footsteps from floor materials. | 94% | Done |
| Room-dependent acoustics | Convey room size and separation. | Blend short/long impulse responses from authored room zones. | 86% | Undone |
| Better wall/door occlusion | Locate enemies through believable sound. | Low-pass and attenuate cues according to intervening walls/door openness. | 92% | Undone |
| Mechanical reload layers | Clarify magazine, latch and chamber actions. | Synchronize separate sounds to reload milestones for each weapon family. | 94% | Done |
| Distance-dependent gunshots | Distinguish nearby danger from remote combat. | Blend close attack, distant tail and obstruction filtering by distance. | 90% | Undone |
| Less repetitive ambience | Prevent long sectors sounding looped. | Schedule seeded rare machinery events with cooldowns and sector-specific pools. | 91% | Undone |

Audio processing reference: [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API).

## Visuals and effects

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Stronger sector color hierarchy | Make all 25 sectors recognizable. | Give each sector a dominant light color plus one restrained contrast color. | 95% | Done |
| Material-responsive muzzle lighting | Connect weapons to surrounding geometry. | Project short light pulses onto nearby visible surfaces with distance falloff. | 90% | Done |
| Local contact shadows | Ground sprites and dropped feeds. | Add depth-tested floor shadows scaled by height and nearby light direction. | 93% | Done |
| Selective wet-floor reflections | Add depth without full ray tracing. | Reflect low-detail emissive geometry only in authored wet floor patches. | 79% | Undone |
| Consistent impact residue | Show where combat happened. | Place capped scorch, acid and chip decals aligned to impacted surfaces. | 94% | Done |
| Directional smoke motion | Avoid stationary FX cutouts. | Advect smoke along sector ventilation vectors, with wall-aware fading. | 85% | Undone |
| Better distant sprite readability | Reduce noisy silhouettes in firefights. | Use distance-specific contrast/outline treatment and mip-safe alpha borders. | 90% | Undone |
| Cinematic boss reveal lighting | Give each introduction a focal point. | Sequence existing entrance FX with light falloff, silhouettes and delayed reveal. | 93% | Done |

## Weapons and gunplay

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Frame-aligned reload ejection | Make handling feel physically connected. | Emit each feed when its removal frame crosses the matching animation milestone. | 96% | Done |
| Material-specific feed landings | Match the new physical debris artwork. | Choose metal/polymer/glass landing samples and bounce damping by feed type. | 94% | Undone |
| Controlled recoil recovery | Reward bursts without unpredictable aim. | Apply weapon-specific recoil curves; recover smoothly after a short delay. | 90% | Undone |
| Clear weapon range identities | Give every gun a useful role. | Tune spread/falloff and show concise range guidance in the arsenal. | 95% | Undone |
| Enemy-material impact feedback | Make hits easier to interpret. | Pair sparks, fragments and audio with armor, flesh and shield contacts. | 95% | Done |
| Safe explosive targeting cue | Reduce accidental self-damage confusion. | Briefly mark nearby unsafe impact points using distance and line of sight. | 87% | Undone |
| Stateful boss-weapon displays | Explain complex ammunition at a glance. | Drive liquid, cell charge and cassette indicators from actual ammo state. | 91% | Undone |
| Optional aim-down-sights support | Add precision without losing FPS silhouettes. | Add a separate aligned pose/FOV blend for suitable conventional weapons. | 80% | Undone |

## Enemies and bosses

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Attack readability audit | Make deaths feel fair rather than sudden. | Check every boss tell, windup, projectile speed and recovery window. | 97% | Done |
| Local squad coordination | Reduce enemies bunching in doorways. | Reserve approach slots; limit simultaneous aggressive pushes per group. | 93% | Undone |
| Distinct retreat behavior | Make ranged enemies feel deliberate. | Add distance-band movement and escape routes constrained by line of sight. | 92% | Undone |
| Boss recovery windows | Create tactical moments for counterattacks. | Expose short vulnerabilities after missed heavy attacks or interrupted charges. | 94% | Undone |
| Minion wave ceilings | Avoid unreadable endless boss crowds. | Cap active supports; tie spawn waves to phases and explicit telegraphs. | 96% | Undone |
| Cover destruction reactions | Make destruction change combat behavior. | Recalculate nearby movement/cover goals when relevant props break. | 85% | Undone |
| Audio-only awareness rules | Give unseen movement believable consequences. | Let enemies investigate recent nearby shots without knowing player coordinates. | 86% | Undone |
| Animation transition blending | Reduce abrupt pose jumps. | Crossfade compatible state transitions with depth-tested sprite rendering. | 81% | Undone |

## Maps and exploration

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Sector landmark navigation | Reduce wandering through similar corridors. | Place large theme-specific landmarks visible from multiple approach routes. | 96% | Undone |
| Optional encounter branches | Add replay value without blocking progress. | Author side arenas with distinct rewards and reconnecting exit routes. | 94% | Undone |
| Destructible shortcut walls | Give explosives exploration utility. | Tag clearly signaled weak panels; reveal alternate routes when destroyed. | 90% | Undone |
| Environmental secret clues | Make hidden rewards discoverable through observation. | Match cable routes, light failures and sound leaks to secret entrances. | 94% | Undone |
| Elevated sightline variety | Vary fights beyond corridor length. | Add supported platforms/windows and navigation links after collision validation. | 76% | Undone |
| Connected machinery puzzles | Give sector objects a gameplay purpose. | Route power through switches to open shortcuts or disable a hazard. | 90% | Undone |
| Safe-room usefulness | Let players assess threats and supplies. | Add readable map hints and predictable low-risk resupply before major encounters. | 93% | Undone |
| Sector revisit transformations | Make later loops feel meaningfully changed. | Change route blockers and hazard schedules while preserving authored topology. | 84% | Undone |

## Game logic, progression and balance

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Combat budget director | Keep late-game difficulty rising gradually. | Allocate threats by enemy roles, health and attack pressure, not count alone. | 94% | Undone |
| Supply fairness checks | Prevent avoidable unwinnable resource states. | Validate expected ammunition/health against encounters and owned weapon roles. | 95% | Undone |
| Objective transition invariants | Prevent stalled exits or duplicate rewards. | Enforce one-way boss/reward/sector transitions with deterministic regression scenarios. | 97% | Undone |
| Weapon-role encounter tuning | Encourage varied loadouts. | Mix ranges, armor and movement counters without requiring a single specific gun. | 92% | Undone |
| New Game Plus modifiers | Make sectors beyond 25 distinct. | Introduce bounded role/hazard modifiers before increasing raw enemy health. | 93% | Undone |
| Optional mutator challenges | Add repeatable goals cheaply. | Seed limited-ammo, no-armor or hazard-heavy runs with separate score categories. | 90% | Undone |
| Lightweight upgrade choices | Give boss victories lasting decisions. | Offer two bounded weapon/utility upgrades at selected sector milestones. | 82% | Undone |
| Encounter pacing validation | Avoid long empty stretches followed by spikes. | Measure combat/rest intervals over fixed runs and redistribute spawn groups. | 93% | Undone |

## UI, onboarding and accessibility

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Full control rebinding | Accommodate keyboards and accessibility needs. | Centralize action mappings and validate conflicts in settings. | 96% | Undone |
| Separate effects intensity sliders | Avoid overwhelming flashes and shakes. | Expose bloom, flash, shake and debris amounts without changing combat logic. | 97% | Undone |
| Subtitle presentation controls | Keep dialogue readable across devices. | Add size, contrast backing and placement presets alongside the existing toggle. | 95% | Undone |
| Contextual objective guidance | Reduce confusion without constant arrows. | Reveal concise guidance after prolonged inactivity or repeated failed interactions. | 92% | Undone |
| First-run combat tutorial room | Teach reload and boss mechanics safely. | Offer a short optional room using real input and simulation rules. | 95% | Undone |
| Color-independent threat cues | Improve readability for color vision differences. | Pair colors with shape, motion and distinct hazard icons. | 97% | Undone |
| Gamepad and gyro presets | Improve couch and mobile play. | Add dead zones, acceleration curves and optional gyro look with calibration. | 83% | Undone |
| Legible weapon comparisons | Help players choose unfamiliar boss drops. | Show role, ammo source and one tactical strength in compact pickup previews. | 93% | Undone |

## Multiplayer and social features

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Shareable deterministic challenge seeds | Enable competition without a server. | Encode sector, modifiers and seed in a URL; compare local completion metrics. | 94% | Undone |
| Input-replay ghost runs | Make personal improvement visible. | Record timestamped actions; replay a noninteractive ghost with version checks. | 89% | Undone |
| Two-player cooperative prototype | Test whether teamwork improves boss fights. | Extend the simulation to two players before implementing networking. | 67% | Undone |
| Peer co-op networking experiment | Explore low-infrastructure shared play. | Prototype host-authoritative input/state exchange over WebRTC; plan signaling and NAT relay explicitly. | 58% | Undone |
| Asynchronous squad operations | Share decisions without real-time synchronization. | Exchange signed/versioned mission results and seeded follow-up scenarios. | 70% | Undone |
| Spectator replay viewer | Make runs reviewable and shareable. | Render recorded snapshots/actions with scrub controls outside live gameplay. | 84% | Undone |
| Challenge result export | Encourage comparison while retaining local saves. | Export a compact run report and optional verified replay bundle. | 91% | Undone |
| Collaborative map authoring | Increase content variety. | Add local editor import/export with strict layout and asset validation. | 79% | Undone |

Networking reference: [RTCDataChannel](https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel).
WebRTC still requires connection setup/signaling; a static Pages build alone does
not provide that infrastructure. Treat peer co-op as an experiment, not a promised
hosting-free production feature.

## Experimental game concepts

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Spent-feed power puzzles | Turn new reload debris into deliberate utility. | Optional sockets accept matching depleted cells to reroute low-power machinery. | 73% | Undone |
| Evidence-shaped sector reconstruction | Make discovery alter later exploration. | Found records select authored room/hazard variants on the next campaign loop. | 74% | Undone |
| Boss weapon signatures in the world | Link trophies to environmental storytelling. | Specific reward weapons temporarily reveal matching scars, signals or concealed routes. | 84% | Undone |
| Silence as a tactical resource | Create tension without another ammo pool. | Heavy machinery masks footsteps/shots only during readable timed cycles. | 83% | Undone |
| Sound-memory machinery | Make loud combat affect subsequent routes. | Devices record nearby weapon families and visibly alter their next hazard cycle. | 71% | Undone |
| Two-layer minimap diagnostics | Make navigation an active choice. | Toggle structural layout versus inferred powered circuits, with clearly marked unknowns. | 82% | Undone |
| Recoverable failed-run traces | Give defeats a narrative consequence. | Leave a local deterministic echo containing a clue or limited salvage next run. | 77% | Undone |
| Boss-counter circuit construction | Add optional preparation before the fight. | Combine sector-specific salvage into one temporary, telegraphed arena countermeasure. | 78% | Undone |

## Testing and diagnostics

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| Fault-injected loading matrix | Catch transient failures before release. | Test timeouts, corrupt responses, retries, cancellation and Pages paths in CI. | 98% | Done |
| Every-sector reachability checks | Prevent trapped spawns or objectives. | Flood-fill valid walkable cells for all authored sectors and spawn variants. | 98% | Done |
| Boss-phase scenario suite | Protect distinctive encounter mechanics. | Use fixed engine inputs around each attack, interrupt, death and reward transition. | 97% | Undone |
| Visual atlas sampling fixtures | Catch neighbor-frame bleed across renderers. | Sample diagnostic sheets in CPU/WebGL/WebGPU at multiple scales and rotations. | 96% | Undone |
| Long-session memory soak | Catch gradual resource accumulation. | Loop sector transitions and reloads; inspect heap/GPU growth after warmup. | 94% | Undone |
| Mobile performance gates | Avoid desktop-only quality claims. | Track frame times and memory under throttled CPU, narrow viewports and low FX tiers. | 94% | Undone |
| Failed-run debug bundles | Make reported bugs reproducible. | Export seed, engine hash, settings and bounded recent input/state history locally. | 96% | Undone |
| Player-visible commander evaluation | Improve autonomous QA without cheating. | Score completion, reload decisions and stuck recovery from legitimate observations. | 90% | Undone |

## Asset production and maintenance

| Suggestion | Why | How | Confidence | Status |
|---|---|---|---|---|
| One canonical live-asset manifest | Prevent catalogs and runtime paths drifting. | Generate texture slots, catalogs and preload inventories from approved records. | 97% | Undone |
| Sprite-sheet safe-area contracts | Prevent repeat crop defects. | Record per-frame bounds/gutters and verify approved poses before publishing art. | 98% | Done |
| Remove provably unused runtime art | Reduce download/build footprint safely. | Compare references and manifests; retain licensed source masters outside public output. | 95% | Undone |
| Lossless sprite compression audit | Lower transfer cost without alpha damage. | Benchmark PNG/WebP alternatives against decoded pixel and border tests. | 93% | Undone |
| Voice script consistency pass | Improve character identity and intelligibility. | Review vocabulary, delivery, accents and event suitability before regeneration. | 94% | Undone |
| Contact-sheet approval workflow | Catch visual drift before integration. | Show all weapon/enemy states with frame indices, silhouettes and muzzle anchors. | 97% | Undone |
| Shared material palette | Make unrelated generations look coherent. | Reuse approved metal, plastic, emissive and contamination references per theme. | 94% | Undone |
| License provenance gate | Keep future replacements safe to publish. | Require source, author, license and attribution fields for every external asset. | 98% | Undone |
