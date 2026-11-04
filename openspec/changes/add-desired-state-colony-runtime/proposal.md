# Proposal: A programmable desired-state colony

## Why

The player should rewrite the civilization's governing logic, then walk through the settlement to understand its consequences. The existing shared per-robot movement program cannot express maintained goals, persistent work, or reusable player-defined colony commands without exposing the wrong level of detail.

## What Changes

- **BREAKING:** Introduce a colony-level desired-state policy loop instead of making `move`/`turn` programs the primary interface. The simulation handles routing, hauling, and work execution; policy selects goals and priorities.
- Reconcile named goals such as maintaining stock or ensuring a building count against actual state and in-flight work. Repeated declarations update the same intent rather than enqueueing another copy.
- Add persistent intents and jobs with explicit lifecycle, ownership, progress, reservations, blocking reasons, and deterministic resource accounting.
- Let players define named commands callable on `colony`, with arguments and nested composition of public desired-state operations. Adding a command must not require editing the engine or recompiling the game.
- Validate and activate policy plus command definitions together at a tick boundary without resetting the settlement. Failed edits retain the last valid revision and show diagnostics.
- Expose inspectable explanations connecting goals, worker activity, shortages, and player source. Honor valid but poor priorities; do not secretly optimize away policy mistakes.

### Scope and non-goals

The first slice includes maintained stock, desired building counts at player-designated sites, a minimal resource-to-workshop production chain, and composable custom commands. Jobs persist across ticks and policy edits; durable save/load, distributed controllers, arbitrary native plugins, a complete standard library, and the full civilization economy are outside this change. Examples are conceptual; scripting language and concrete syntax remain open.

### Dependencies and gates

Agree the snapshot/ID/execution boundary with `select-and-migrate-3d-engine`. Headless reconciliation and command tests can proceed independently; the walk-up inspection demonstration needs the engine slice. Before implementing the script-facing interface, approve the language/runtime, command registration syntax, goal ownership semantics, and resource/priority rules. The design records proposed defaults rather than treating these as already settled.

## Capabilities

### New Capabilities

- `colony-reconciliation`: Desired-state identity, deficit calculation, scheduling, persistent jobs, and conservation.
- `colony-programming`: Bounded policy evaluation, safe activation, public goal operations, and player-defined commands.
- `colony-observability`: Structured goal/job explanations and source provenance consumable by in-world inspection.

### Modified Capabilities

None. This is the first specification baseline; the prototype has no existing OpenSpec capabilities.

## Impact

Introduce an engine-independent simulation/runtime boundary rather than extending `Op` with another growing list of built-ins. `src/sim.rs` supplies reusable cost/conservation rules and test cases, but `Robot.pc`, shared programs, ahead-facing actions, and grid-only assumptions are not the new contract. Replace the editor's `Apply & reset` behavior for colony policies while keeping explicit world reset separate. Integrate scene execution and inspection through adapters, never mutable engine objects exposed to player code.
