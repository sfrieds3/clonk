# Design: Engine selection and inhabited-world migration

## Context

Status: Godot-first/Rhai-first investigation direction approved; final engine, language, integration, and migration remain unapproved. This update records a spike plan, not permission to execute it. See `proposal.md` for motivation and `specs/` for proposed acceptance behavior. Implementation targets this repository; OpenSpec artifacts are planning documents.

Observed: `src/main.rs::main` combines Macroquad startup, editor state, pacing, and simulation stepping. `src/view.rs::WorldView` reads the simulation and owns GPU state, interpolation, and tabletop input. `block` fakes lighting with face colors. `CameraRig` tests are headless but use Macroquad math. `src/sim.rs` has no Macroquad imports; its discrete stepping and conservation tests are the strongest reuse boundary. Existing releases package an executable, README, and changelog, not a general asset tree. There are uncommitted font/typography changes to preserve.

## Goals / Non-Goals

**Goals:** Select an engine by evidence; prove modern rendering, direct character control, UI input ownership, and packaged launch; maintain a one-way authoritative-state-to-presentation seam.

**Non-Goals:** Preserve the exact fixed window/tabletop camera, translate every prototype drawing call, or make engine ECS/physics state an alternative authority for colony inventory and jobs.

## Decisions

### 1. Godot-first spike before final selection

Godot is the approved first investigation candidate because integrated scene authoring, animation, character collision, navigation, and UI tooling fit the inhabited-world goal. Bevy remains the comparison alternative because direct Rust integration and an ECS-centered core could outweigh the extra authoring/plugin assembly. Neither is finally selected. Do not expand into a full renderer on Macroquad.

Build one small disposable Godot scene first: terrain, workshop, stockpile, resource deposit, an animated directly controlled character with obstacle collision and a following camera, a shadow-casting light, differentiated materials, one shader effect, and an inspection/code overlay. Add one worker and a minimal supply fixture so a player-defined colony helper can alter a maintained goal without resetting visible progress. This shares a narrow vertical slice with the runtime spike; it is not authorization to port the full economy or implement the production scheduler.

The shared Rhai spike must demonstrate one named, parameterized helper callable through colony, repeated declarations without duplicate work, failed candidate activation without partial goal changes, successful activation without resetting the world, and a runaway policy returning control while movement and inspection remain responsive. Test this first headlessly, then through the Godot bridge. The runtime design owns containment criteria; GDScript is trusted client code, not the default player-policy sandbox.

Keep a documented Godot/Bevy comparison using equivalent requirements and distinguish documentation evidence from executable results. A second hands-on Bevy scene is conditional: use it if Godot hits a hard blocker, bridge/iteration cost undermines its authoring advantage, or comparative evidence is too close to justify selection. If needed, reuse the same assets and acceptance checks. Do not require two complete prototype ports before learning whether the preferred path works.

Suggested review weights remain provisional: visual result 30%, character/scene/UI authoring workflow 25%, simulation and scripting integration 20%, desktop deployment/testability 15%, migration/reuse cost 10%. Unsupported target hardware, unusable editor input, or failure to contain policy execution are blockers rather than scores to average away.

The observed local reference machine is a MacBook Air with Apple M2 and 16 GB memory, not an agreed minimum specification. Record exact engine/binding/runtime versions and features, hardware, scene/workload, screenshots/video, sustained frame-time distribution, cold/incremental build time, memory use, package size, headless test commands, and packaged-launch results. Approve performance targets before treating measurements as pass/fail. Linux/Windows build or launch claims require actual verification and cannot be inferred from a macOS run.

#### Documentary evidence and its limits

Research on 2026-11-03 supports the Godot-first recommendation, not a measured winner. Official feature documentation describes Godot's integrated rendering, animation, collision/navigation, and GUI systems; Bevy's documentation supports its Rust/ECS/rendering strengths but does not establish equivalent integrated scene/character/navigation workflows for this project. Version-specific APIs and plugin compatibility must be rechecked and pinned before experiments.

- Godot feature inventory: https://docs.godotengine.org/en/stable/about/list_of_features.html
- Godot CLI/headless/export workflow: https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html and https://docs.godotengine.org/en/stable/tutorials/export/exporting_projects.html
- Bevy features and stability/workflow considerations: https://bevy.org/ and https://bevy.org/learn/quick-start/introduction/
- godot-rust integration/compatibility: https://github.com/godot-rust/gdext and https://godot-rust.github.io/book/toolchain/compatibility.html

These moving documentation URLs are research leads, not a dependency lock. Record pinned release/crate/API evidence in the spike results. No scene, performance, binding compatibility, or standalone package was tested during research. A final recommendation must return to the user for engine/version/integration approval before migration.

### 2. Preserve the simulation boundary, not necessarily its language

The default migration preference is to retain a pure headless core and its tested invariants. First prove a minimal trusted Rust GDExtension hosting Rhai behind snapshot/goal-batch adapters, without exposing Godot objects to policy code. godot-rust is a community integration rather than built-in Rust support: pin the released binding crate and API target, verify compatibility with the chosen Godot release, record license obligations, and account for native artifacts on each target. Failure or disproportionate packaging/iteration costs reopen the integration decision; the bridge is not already proven.

Bevy offers direct Rust embedding as the fallback comparison. A Godot-native simulation with a separately contained VM is another option if it materially simplifies the boundary. Do not adopt FFI solely to preserve a small obsolete VM or rewrite a working core solely for engine uniformity. The new colony contract owns goals/jobs; the engine owns rendering, player collision, and spatial execution services.

Snapshots carry stable identities, transforms, inspectable activity, and revision/tick information. Engine objects render them; movement/execution feedback crosses a defined adapter and is accepted by the simulation before resources or completion change. Cosmetic interpolation does not decide outcomes. Deterministic navigation/execution feedback must be represented as ordered simulation input; hardware-dependent physics does not silently enter replay state.

### 3. Player-first presentation

Proposed camera is a controllable third-person following camera rather than a permanent management view. The player directly moves and inspects; optional overview controls are deferred. Use one cohesive stylized art direction: warm directional light, readable soft shadows, restrained material palette, and one subtle effect such as vegetation motion. Bloom alone is not the visual target. Choose visual references and review a moving scene before scaling content.

The editor captures typing, pointer, and scrolling while focused. Body text remains proportional and code monospaced; package fonts and their licenses. Stub inspection reports unavailable fields explicitly until the real runtime adapter lands.

### 4. Parallel tracks with one integration checkpoint

Engine evaluation and a fixture-based scene can advance independently of the headless desired-state runtime. Agree stable IDs, snapshots, execution feedback, and inspection payloads first. Only the integrated observe/edit/recover demonstration depends on both changes. Avoid requiring the obsolete movement VM to become the permanent engine API.

## Risks / Trade-offs

- Engine preference overwhelms gameplay needs → Score the same scene and authoring tasks; obtain approval before migration.
- Rendering success hides release failures → Test packaged launch outside the checkout; explicitly record each target as verified, blocked, or not yet tested.
- ECS/physics duplicates simulation authority → Keep mutations and resource commits behind the simulation boundary and test replay against ordered inputs.
- Migration discards working behavior unintentionally → Classify old tests as retained invariants, adapter-specific tests, or intentionally retired grid rules before removal.
- Visual scope expands indefinitely → One scene, one character, one shader effect, user visual review, then stop.

## Migration Plan

1. Record the current baseline and preserve working-tree typography changes; do not reset or overwrite them.
2. After explicit spike execution authorization, agree measurement criteria and run the shared Godot-first/Rhai-first slice. Evaluate Bevy hands-on only when the documented fallback criteria apply. Record results and stop for final engine/runtime/integration approval before migration.
3. Establish the selected project/build structure, snapshot fixture adapter, and a headless core test path.
4. Implement the scene, player, inspection, editor focus, and asset packaging. Connect the new runtime only when its public boundary is ready.
5. Update CI/releases/README and retire Macroquad after acceptance. Keep the prototype available through version control or an isolated comparison target during migration, not as a permanent double-maintained client.

Rollback is returning to the last working client while retaining independently tested core work; no existing save format migration is promised by this prototype.

## Approval Gates and Evidence Still Needed

- Confirm desktop baseline, camera/art references, performance targets, and evaluation weights; use the observed M2 Air/16 GB as the initial local reference, not an invented minimum hardware target.
- Obtain explicit authorization to execute the spike; this artifact update approves investigation direction only.
- Pin engine/binding/runtime versions and verify bridge compatibility, containment, scene quality, measured performance, and target packages. Documentary research is available; hands-on evidence is not.
- Present the Godot result, Bevy comparison/fallback findings, and Rhai limitations for final engine/runtime/integration approval, then refine migration tasks.
- Review visual quality in motion and standalone packages before removing the old client.

## Open Questions

Exact input bindings, shader-effect subject, art asset sources, and inspection-panel layout can be refined during the spike without changing the public behavior contracts. Any proposed platform or simulation-boundary change returns to proposal review.
