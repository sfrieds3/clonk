# Proposal: An inhabited, modern 3D foundation

## Why

CLONK should be a place the player inhabits, not a grid program viewed from above. The current Macroquad renderer supplies procedural geometry and manually shaded faces; building lighting, shadows, materials, animation, and character interaction ourselves would distract from the civilization game.

## What Changes

- Investigate Godot first, following documentary research, against representative scene, authoring, simulation-boundary, and desktop packaging requirements. Keep Bevy as the comparison/fallback candidate; neither is finally selected. The detailed spike plan and conditional Bevy build criteria are in design.md.
- Record exact versions, official documentation, licensing, measured results, migration cost, and a recommendation. Obtain approval of the engine and integration approach before migration.
- **BREAKING:** Replace Macroquad application/rendering/input/UI ownership with the selected engine; the fixed tabletop layout and orbit-only viewpoint are not compatibility requirements.
- Deliver a small, cohesive, stylized settlement scene with real lights and shadows, material response, a demonstrable shader effect, and an animated player character controlled directly with a following camera.
- Let the player approach and inspect settlement objects. Keep editor input separate from movement/camera input and preserve distinct draft, applied, and diagnostic states.
- Keep authoritative colony behavior behind a headlessly testable simulation boundary. Use stable entity IDs to connect visible objects and inspection state.

### Scope and non-goals

This change selects and migrates the engine, establishes character movement and inspection, and supplies one attractive scene. It does not implement the desired-state scheduler, a full civilization economy, combat, multiplayer, or a large content pipeline. Placeholder simulation snapshots can drive inspection until the runtime proposal lands.

### Dependencies and gates

Coordinate snapshot/ID/inspection boundaries with `add-desired-state-colony-runtime`; neither headless runtime work nor engine evaluation must wait for the other's production implementation. Share a narrow Godot-first/Rhai-first spike fixture and keep the full integrated bottleneck demonstration dependent on both changes. Documentary research informs the investigation order; hands-on compatibility, containment, performance, and packaging remain unverified. Spike execution and final engine/runtime/integration selection require explicit approval before migration.

## Capabilities

### New Capabilities

- `world-presentation`: Modern 3D scene rendering, stable simulation-to-scene mapping, and packaged desktop launch.
- `player-presence`: Direct character movement, following camera, nearby inspection, and UI input ownership.

### Modified Capabilities

None. This is the first specification baseline; the prototype has no existing OpenSpec capabilities.

## Impact

Replace or restructure `src/main.rs` and `src/view.rs`, update `Cargo.toml`/lockfile or establish the chosen engine's project structure, and revise `justfile`, CI, releases, and README. Retain simulation invariants and useful tests from `src/sim.rs` rather than freezing the imperative VM as the new public API. Migration must preserve existing uncommitted typography assets/roles where applicable and package all required assets and licenses. Baseline release targets are Linux x86_64, macOS Apple Silicon, and Windows x86_64, pending review.
