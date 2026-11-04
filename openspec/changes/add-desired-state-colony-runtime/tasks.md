# Tasks

Draft plan. Semantics and language/runtime remain proposed; gate approval and evidence precede the corresponding implementation. All work targets the CLONK code repository, not this store.

## 1. Approve semantics and the extension boundary

- [ ] 1.1 Review first goal families, goal keys/ownership/withdrawal, overlapping-goal rules, priority tie-breaks, cancellation phases, and the minimal economy with the user; verify accepted choices and illustrative API examples are recorded in design.md.
- [ ] 1.2 Agree stable snapshot IDs, ordered executor inputs, and inspection payloads with the engine proposal; verify an engine-free fixture describes a stockpile, worker, workshop, and blocked supply goal.
- [ ] 1.3 After explicit spike execution authorization, build the Rhai-first headless fixture with pinned runtime/features, immutable snapshot, minimal stock-goal reconciler/fake executor, and temporary goal batch; verify parameterized custom calls through colony, nested helpers, stable keyed declarations, registration/argument diagnostics, and document actual supported syntax rather than the illustrative Python example.
- [ ] 1.4 Test Rhai's irrevocable shared execution budget, recursion and source/compile bounds, allocation strategy, capped output/logs/conversion, denied ambient/engine capabilities, and atomic activation/evaluation; verify runaway code and late failures preserve accepted goals/progress and return control. Integrate this same fixture with the engine task's Godot bridge and record responsiveness; evaluate Lua/mlua only if Rhai fails agreed criteria, explicitly distinguishing structural limits from a proven aggregate heap bound.
- [ ] 1.5 Present Rhai/bridge findings and any Lua fallback evidence for final language/runtime/API/integration approval, then refine production tasks; verify the decision records exact versions, real player-facing syntax, containment limitations, and no implementation beyond the disposable spike before that gate.

## 2. Implement maintained state and reconciliation

- [ ] 2.1 Establish the engine-independent colony model, non-reused IDs, immutable snapshots, and fake executor; verify headless tick tests and document the authoritative boundary and test command in the same change.
- [ ] 2.2 Implement maintained stock/building targets and policy-owned goal-set replacement; verify repeated declarations, withdrawn goals, completed-plus-committed house counts, and satisfied-then-consumed stock scenarios.
- [ ] 2.3 Implement shared-output attribution and atomic reservations; verify competing final-resource claims, overlapping targets under different keys, cancellation release, and inventory conservation.
- [ ] 2.4 Implement persistent queued/running/blocked/completed/cancelled/failed jobs and bounded physical phases; verify shortages resume with stable identity, reprioritization preserves compatible progress, and cancellation retains carried resources.
- [ ] 2.5 Implement fixed-tick stable scheduling and replay; verify identical ordered inputs produce identical authoritative assignments, events, and inventories across different render schedules, documenting ordering and priority semantics.

## 3. Implement player policy and custom commands

- [ ] 3.1 Implement read-only queries and validated public desired-state operations in the approved runtime; verify no direct mutable world/engine access and source-attributed argument/key errors, documenting the public contract with runnable examples.
- [ ] 3.2 Implement named parameterized custom-command registration and calls through colony, including nested composition; verify a player adds supply_workshop without recompilation, repeated keyed calls do not duplicate work, and duplicate/reserved names are rejected.
- [ ] 3.3 Implement combined policy/command revisions, initial validation/evaluation, and tick-boundary activation; verify invalid drafts preserve the active revision and accepted goals while valid edits preserve compatible work and world state.
- [ ] 3.4 Enforce whole-evaluation execution/allocation/output budgets and transactional batch commit; verify recursion, runaway loops, oversized outputs, forbidden capabilities, and late evaluation failures terminate without partial mutations or freezing player control.
- [ ] 3.5 Add command discovery/help and draft/active diagnostics to the chosen editor integration; verify custom signatures are discoverable and documented examples run under the actual approved syntax.

## 4. Explain and execute work

- [ ] 4.1 Publish structured desired/actual/in-flight quantities, blocking reasons, assignment, tick/revision, and command call-chain provenance; verify inspection payload tests preserve old revision/source attribution after helper edits.
- [ ] 4.2 Implement the minimal gathering/hauling/workshop executor and engine adapter; verify route failure blocks without losing identity, dispatch/completion cannot double-spend, and visible motion cannot independently complete jobs.
- [ ] 4.3 Integrate nearby object inspection in the engine client; verify the player can trace an idle workshop through missing material to a goal and its custom-command source, documenting the debugging loop.

## 5. Prove the playable loop and retire the old interface

- [ ] 5.1 Run the shared observe/inspect/revise/recover scenario; verify policy repair redirects eligible work and resumes workshop output with no world reset, retaining committed progress.
- [ ] 5.2 Verify a valid but unwise priority policy remains unwise without hidden rescue; confirm the bottleneck and its provenance remain visible while hard resource invariants hold.
- [ ] 5.3 Replace the movement-program UI/docs deliberately; verify old fixtures are retained as reference or intentionally retired, explicit reset remains separate, and the packaged desktop slice passes the documented headless/integration checks.

## Workflow follow-up

- Request review before applying any implementation phase.
- Archive only after acceptance and reconcile colony-reconciliation/colony-programming/colony-observability specs into the store's main specifications.
