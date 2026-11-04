# Design: Desired-state colony and extensible commands

## Context

Status: Rhai-first investigation within a shared Godot-first spike is approved as planning direction; colony semantics, final language/API, integration, and implementation remain unapproved. This artifact update does not authorize spike execution. See `proposal.md` for motivation and `specs/` for proposed acceptance behavior. Implementation targets this repository.

Observed: `src/sim.rs::World::step` runs each robot's shared `Instruction` program in sequence. `Robot.pc` identifies an instruction, not work; `State` mixes VM completion and physical availability. `World::run` bounds control-flow evaluation, `World::build` validates costs, and existing tests protect conservation and failed-build behavior. World fields are public, locations are integer tiles, carrying is a boolean, and IDs derive from robot count. `main` applies code by resetting the world. These are migration facts, not requirements to retain.

## Goals / Non-Goals

**Goals:** A pure authoritative tick, maintained desired state, persistent jobs, bounded player-code composition, inspectable cause/effect, and a fake executor for headless tests.

**Non-Goals:** One giant global colony enum, arbitrary native plugins, durable save/load, general autonomous planning, or automatic repair of every bad policy. World state, goal status, worker availability, and job lifecycle are separate concerns.

## Decisions

### 1. Goals describe state; jobs perform finite work

Proposed initial goal families are maintained stock at a designated inventory and desired building counts within an explicit set of player-designated sites. Placement remains an in-world decision; policy must not need integer movement commands. A small resource/hauling/workshop recipe chain is enough to exercise both. New economy rules must be explicit rather than silently importing home-only building, permanent battery death, the four-worker cap, or fixed mine coordinates.

A goal has a stable player-supplied key, kind, scope, target, and priority. An invocation updates the same key. Duplicate declarations of the same key in one evaluation are diagnostics, not implicit last-write-wins. Goals sharing the same physical inventory/site scope cannot each count the same incoming output twice: compatible minimum targets use the strongest target; incompatible constraints produce an explainable conflict. The implementation must test cross-goal attribution rather than just single-key deduplication.

Proposed ownership is full-set reconciliation: every successful policy evaluation replaces the policy-owned goal set, so omitting a prior goal withdraws it. A failed evaluation replaces nothing. Custom commands emit into that same set and do not establish hidden controllers with their own lifetime. A satisfied maintained goal remains present and can become unsatisfied again. Finite one-shot commands are deferred to avoid ambiguous repeated-call semantics.

Deficits count actual relevant inventory and attributed incoming output, subtract committed consumption, and reserve each physical item once. For construction, count completed buildings plus compatible committed sites. Lowering a target never demolishes an existing house or reverses completed work automatically.

### 2. Separate state machines with safe cancellation

Proposed job lifecycle: `queued -> running -> completed`, with explicit `blocked(reason)`, `cancelled`, and `failed` transitions. Assignment, phase/progress, goal attribution, source revision, and reservations are durable across ticks in memory. Readiness is an eligibility condition, not another mandatory UI state.

Recoverable shortages or route failures block and retry when relevant state changes; permanent invalidity is a diagnostic or explicit failure. Changed priorities affect future dispatch. Running work completes its current bounded physical operation before reassignment or cancellation; reservations not consumed are then released exactly once. Carried materials remain in world accounting and return to reachable storage through explicit executor work. Changing a helper definition affects future intent expansion, not an old job's stored executable identity.

Alternative: rebuild transient tasks every tick. Rejected because it loses progress and makes cancellation, reservations, and inspection unreliable. Alternative: expose the old VM as jobs. Rejected because numeric instruction positions cannot survive live code edits.

### 3. Deterministic authoritative tick

At a tick boundary: ingest ordered execution/player inputs; apply an accepted program revision; evaluate policy against a read-only snapshot into a temporary goal batch; validate the whole batch; reconcile retained/withdrawn goals and uncovered deficits; dispatch in stable order; commit bounded executor transitions; publish a new snapshot and structured events. The precise ordering is versioned and tested. Proposed tie-breaks are explicit priority, age, then stable ID; no hidden fairness override rescues a starving policy.

Use monotonic non-reused IDs. Revalidate resources at dispatch, atomically reserve, and define the exact phase at which costs and output commit. A headless deterministic executor proves invariants without a renderer. The engine adapter supplies routing/spatial execution results as ordered inputs; render interpolation never completes authoritative work. Fixed simulation ticks are independent of rendering.

### 4. Player commands extend the public colony interface

Commands are named player-authored functions with validated arguments and access only to read-only snapshot queries and public goal operations. They can invoke other custom commands under the same budget. No engine recompilation is required. Registration rejects built-in collisions and duplicates; editor discovery/help should include the accepted command names and parameters.

Conceptual pseudocode, not a language selection:

```python
@colony.command("supply_workshop")
def supply_workshop(colony, workshop, amount):
    colony.keep_stock(
        key=f"{workshop.id}:ore",
        inventory=workshop.inputs,
        resource="ore",
        minimum=amount,
    )

def govern(colony):
    colony.supply_workshop(colony.workshop("foundry"), amount=20)
    colony.ensure_buildings(key="housing", kind="house", count=5,
                            sites=colony.designated_sites("housing"))
```

The decorator, f-string, lookup helpers, and method binding are illustrative Python-like pseudocode, not promised Rhai syntax. The approved investigation order is Rhai first, with pinned PUC Lua through mlua as a fallback if Rhai cannot meet the agreed containment or authoring requirements. Neither is selected for production. Trusted GDScript can implement client/UI behavior; ordinary engine scripting is not automatically a player-code sandbox.

#### Rhai-first containment spike

Research on 2026-11-03 identifies Rhai's operation budget and call-depth/collection limits as useful starting points, not a complete security boundary. Its documented string/array/map limits do not establish an aggregate heap quota. Pin runtime version/features and keep unchecked execution disabled. Validate a minimal allowlisted engine with imports/file resolvers and ambient capabilities unavailable; expose only bounded read-only snapshot queries and capped goal-batch construction. No Godot object, arbitrary mutable Rust object, filesystem/network/process access, or nondeterministic clock reaches scripts.

One evaluation owns an irrevocable shared budget across nested helpers, dynamic command dispatch, callback reentry, output conversion, and diagnostics. Native callbacks need separate bounded work/allocation because a script-operation counter cannot preempt arbitrary host work. Bound source/compile complexity, allocations and retained state, number/size/depth of emitted goals, and logs before expensive copying. Spike evidence must state the allocation threat model and explain whether available structural limits suffice; an unproven aggregate bound is a blocker to approval, not an implied guarantee. If a hard byte quota is required and unavailable, evaluate the Lua fallback or isolation rather than weakening the specification silently.

#### Minimal shared vertical slice

Start headlessly with a fixed colony snapshot, a tiny maintained-stock reconciler/fake executor, and a temporary goal batch. Prove that a player can define supply_workshop with parameters, call it through colony, invoke another helper under the same budget, and re-evaluate the same keyed goals without duplicate jobs. Reject reserved/duplicate names and invalid arguments. Prove compile/initial-evaluation failure, runtime failure after partial emission, runaway loops/recursion, oversized output/allocation attempts, and forbidden capabilities leave accepted goals and world intact and return control. Then use this same harness through a minimal trusted Rust GDExtension in the engine spike, preserving visible worker progress on successful edits and character/inspection responsiveness on failure. Record actual supported Rhai registration/call syntax before claiming the colony method interface works; do not equate calling a global function by name with proving the desired interface.

This is a disposable contract fixture, not the full scheduler, economy, or production editor. The engine tasks own the scene/bridge and measurements; runtime tasks own VM, helper-binding, and transactional tests. Both refer to the same artifacts/results rather than duplicate implementations.

#### Alternatives and documentary sources

Lua via mlua offers instruction hooks and a Lua-state allocation limit, but host allocations, coroutine/hook coverage, catchable errors, recursion, and capability removal still need explicit proof. Safe Rust bindings or a library named safe do not imply an untrusted-policy sandbox. If the fallback is needed, pin backend/features and test one shared fuel ledger and denied IO/OS/package/debug/file-loading capabilities. A watchdog detecting a frozen main thread is not containment in either runtime.

- Rhai operation accounting: https://rhai.rs/book/safety/max-operations.html
- Rhai limits and host API: https://rhai.rs/book/safety/index.html and https://docs.rs/rhai/latest/rhai/struct.Engine.html
- mlua hooks, memory limits, and library capabilities: https://docs.rs/mlua/latest/mlua/struct.Lua.html and https://docs.rs/mlua/latest/mlua/struct.StdLib.html
- Godot/Rust bridge compatibility: https://godot-rust.github.io/book/toolchain/compatibility.html

These are documentary findings and moving reference URLs, not tested pins or sandbox certification. Syntax familiarity and Rust reuse do not override containment and usability. Final runtime/API approval follows measured spike results.

### 5. Transactional evaluation and live changes

Compile/validate policy and definitions as one revision, evaluate against a snapshot into a temporary batch, and activate at a tick boundary only on success. Preserve old source revisions for provenance while jobs reference them. At runtime, an evaluation fault discards the entire new batch, retains last accepted goals, reports the fault, and lets existing work/player interaction continue. Retrying or applying a replacement must not commit earlier partial emissions. Execution/output limits apply across the whole call tree.

Alternative: live patch mutable global methods directly. Rejected because partially replaced helpers and uncontrolled side effects cannot supply atomic activation or deterministic replay. Custom persistent mutable script state is deferred; use snapshot queries and goal keys first.

### 6. Inspectable explanations are core output

Publish actual/desired quantities, incoming commitments, job lifecycle and assignment, reason codes, priority, tick, goal key, and source revision/call chain. Rendering turns reason codes into prose; tests assert structured facts, not humorous message strings. Inspection in the engine uses the same stable IDs and records as the headless debugger.

## Risks / Trade-offs

- Helpful scheduler erases gameplay → Automate mechanics such as routing and carrying, not unstated economic priorities or goals.
- Overproduction from overlapping goals → Test attribution across keys and scopes, counted incoming work, and shared reservations.
- Goal withdrawal destroys progress → Cancel only at bounded safe phases; retain physical outputs and carried inventory.
- Script extensions escape containment → Prove limits and capability restrictions before approving a language/runtime.
- Integration turns physics timing into nondeterminism → Record ordered executor inputs and test the pure core independently.
- Replacing the prototype accidentally drops valuable invariants → Retain resource/cost/failed-action cases as new domain tests; explicitly retire facing/program-counter assumptions.

## Migration Plan

1. Review the proposed goal ownership, priority, overlap, cancellation, and first-economy semantics. Agree the cross-proposal boundary.
2. After explicit spike execution authorization, prove Rhai and the colony command binding in the shared Godot-first slice; evaluate Lua only if the documented fallback criteria apply. Record actual syntax, pinned versions, resource bounds, and bridge results, then stop for final runtime/API/integration approval before production implementation.
3. Introduce the headless state/reconciler and fake executor beside the prototype, porting invariant tests rather than mutating `Op` into a scheduler.
4. Add custom commands, atomic revisions, diagnostics, and structured inspection; connect the engine adapter when available.
5. Demonstrate shortage -> inspection -> policy repair -> resumed production without reset. Retire the old movement editor/API deliberately and update documentation. Prototype programs are reference fixtures, not a promised compatibility format.

Rollback keeps the old client available in version control while the new core remains independently testable; no save migration is required in this slice.

## Approval Gates and Evidence Still Needed

- Review first goal families, full-set withdrawal semantics, overlapping-goal merge/conflict rules, and safe cancellation behavior.
- Obtain spike execution authorization, then prove Rhai's helper registration/call syntax, shared execution ledger, allocation strategy, output bounds, denied capabilities, transactional edits, and Godot bridge. State failures/unknowns explicitly and compare Lua if needed; obtain final runtime/API/integration approval before production implementation.
- Agree the minimal economy/worker physical rules; battery, food, and population systems are not automatically carried forward or added.
- Confirm headless tests and real in-world bottleneck repair before replacing the old API.

## Open Questions

Exact numerical execution budgets, retry interval, diagnostics wording, and inspector layout can be tuned once the approved runtime and first fixture provide measurements. Durable save formats and additional command families require future proposals.
