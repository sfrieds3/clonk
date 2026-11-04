# Colony programming

## Purpose

Let players evolve the colony's governing loop and add reusable colony commands without changing the game executable or accessing engine internals.

## ADDED Requirements

### Requirement: Public desired-state interface
Player policy SHALL query a read-only colony snapshot and declare named goals and priorities through validated public operations. It SHALL NOT require tile-by-tile movement or expose direct mutable simulation or engine objects.

#### Scenario: Declare workshop supplies
- **WHEN** a policy declares a stock goal for a workshop input
- **THEN** the colony receives that desired state without the player specifying individual worker routes, facing, or carrying actions

### Requirement: Player-defined colony commands
Players SHALL define named, parameterized commands callable through the colony interface and compose them from public goal operations and other player-defined commands. Definitions SHALL be editable and usable without recompiling the game. Invalid names, built-in name collisions, duplicate registrations, and invalid arguments SHALL produce source-attributed diagnostics.

#### Scenario: Add and invoke a custom command
- **WHEN** a player defines a command named supply_workshop that composes input-stock goals and invokes it through colony with a workshop reference and quantity
- **THEN** the active command creates those named goals and repeated invocation with the same keys updates them without duplicating work

#### Scenario: Reject a name collision
- **WHEN** a player registers a custom command using a reserved built-in command name
- **THEN** candidate activation is rejected with a diagnostic identifying the conflicting definition and the existing active revision remains unchanged

### Requirement: Atomic live activation
Policy and custom-command definitions SHALL activate together at a simulation tick boundary only after successful validation and initial bounded evaluation. A failed candidate SHALL leave the active revision, accepted goals, and world unchanged. Successful activation SHALL preserve compatible jobs and committed progress rather than reset the world.

#### Scenario: Repair a policy while work continues
- **WHEN** the player applies a valid revised policy while a worker carries materials for a compatible job
- **THEN** the new revision becomes active at one boundary and the worker retains its identity, carried inventory, and committed progress

#### Scenario: Invalid candidate after valid revision
- **WHEN** candidate policy validation or its initial evaluation fails
- **THEN** no partial candidate goal updates are accepted and the last valid revision and world remain available with candidate diagnostics shown

### Requirement: Bounded transactional evaluation
Every policy evaluation, including nested custom-command calls, SHALL share finite execution and output budgets. A failed evaluation SHALL commit no partial goal changes, retain the last accepted goals, report the fault, and return control to the game. Player code SHALL have no filesystem, network, process, native-engine, or nondeterministic clock access through this interface.

#### Scenario: Recursive custom command
- **WHEN** a custom command recurses until its evaluation budget is exceeded
- **THEN** evaluation terminates with a source-attributed diagnostic, no partial goal batch is committed, and movement, inspection, and previously accepted work can continue
