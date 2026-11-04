# Colony reconciliation

## Purpose

Turn player-authored desired state into persistent, explainable work without duplicate construction, lost progress, or invented resources.

## ADDED Requirements

### Requirement: Stable maintained goals
A repeated declaration with the same goal key SHALL update one maintained goal. Reconciliation SHALL account for existing state and compatible in-flight work before creating work for a deficit. Building-count goals SHALL count completed buildings and committed construction in their declared scope.

#### Scenario: Repeated housing target
- **WHEN** the policy repeatedly requests five houses under one key and the scoped settlement contains three completed houses and two committed house constructions
- **THEN** the target remains five and no additional house construction is created

#### Scenario: Stock is consumed after satisfaction
- **WHEN** a maintained stock target has been satisfied and later consumption creates a new deficit
- **THEN** the same goal becomes unsatisfied and requests only uncovered replacement work

### Requirement: Persistent job lifecycle
Jobs SHALL retain identity and committed progress across ticks and compatible policy updates. Inspectable job states SHALL distinguish queued, running, blocked, completed, cancelled, and failed work. Recoverable missing prerequisites SHALL block work with a reason rather than silently discard it.

#### Scenario: Missing input becomes available
- **WHEN** a workshop job lacks an input and that input later becomes available
- **THEN** the job reports the missing input while blocked and can resume under the same job identity without repeating committed work

### Requirement: Conservative resource commitments
Dispatch and completion SHALL reserve and spend each resource at most once, prevent negative inventory, and account for carried and committed resources. Cancelling work SHALL release unused reservations without undoing completed physical work or deleting carried resources.

#### Scenario: Contend for the last material
- **WHEN** two eligible jobs require the last available unit of a material
- **THEN** at most one receives its reservation, the other reports a shortage, and total resource accounting remains conserved

#### Scenario: Lower a target during hauling
- **WHEN** a policy lowers a goal while an associated worker carries materials
- **THEN** obsolete work is stopped at a safe execution boundary and the carried materials remain accounted for rather than disappearing or being spent twice

### Requirement: Reproducible scheduling
Given the same initial authoritative state and ordered policy/input revisions, headless simulation SHALL produce the same job creation, assignments, resource changes, and outcomes independently of render frame rate.

#### Scenario: Replay at different render rates
- **WHEN** the same accepted inputs are replayed through identical simulation ticks with different presentation frame rates
- **THEN** authoritative job and resource outcomes match

### Requirement: Faithful policy consequences
The runtime SHALL enforce validity and physical accounting but SHALL NOT silently replace valid player goals or priorities with a more productive policy.

#### Scenario: Player prioritizes the wrong output
- **WHEN** a valid policy allocates available workers to a low-value stock target ahead of a workshop supply target
- **THEN** the colony follows that priority and exposes the workshop bottleneck instead of silently reallocating workers against policy
