# World presentation

## Purpose

Make the settlement a coherent modern 3D place whose visible activity accurately represents authoritative simulation state.

## ADDED Requirements

### Requirement: Lit and materially coherent settlement
The game SHALL render the representative settlement with real-time lighting, cast shadows, differentiated material response, and at least one shader-driven environmental effect. Visual acceptance SHALL include user review of the scene in motion, not only successful compilation.

#### Scenario: Review the representative scene
- **WHEN** the player walks through the acceptance scene containing terrain, a resource deposit, a stockpile, a workshop, and animated characters
- **THEN** light direction and shadows are consistent, material differences are visible, and the selected environmental effect is visible without obscuring activity or text

### Requirement: Stable scene identity
Visible colony objects SHALL map to stable simulation identities. Rendering and interpolation SHALL NOT independently spend resources, complete jobs, or change colony goals.

#### Scenario: Observe one worker across updates
- **WHEN** successive snapshots update a worker's position and job while render frames arrive at a different rate
- **THEN** the same visible worker and inspection target retain their identity and display the newest authoritative activity

### Requirement: Self-contained desktop distribution
The release package SHALL contain the assets, shaders, fonts, licenses, and runtime dependencies needed to launch on the agreed desktop target without a developer checkout or engine editor installation.

#### Scenario: Launch outside the repository
- **WHEN** a tester extracts the release package on each agreed desktop target and launches it from an unrelated working directory
- **THEN** the settlement, UI, fonts, animation, and shader effect load without missing-file errors or dependence on development paths
