# Player presence

## Purpose

Let the player inhabit and inspect the settlement directly while keeping character control separate from colony programming.

## ADDED Requirements

### Requirement: Direct character control
The player SHALL move a character directly through the settlement using movement input and a following camera, without writing colony movement commands. Solid scene obstacles SHALL prevent the character from walking through them.

#### Scenario: Walk around the workshop
- **WHEN** movement input directs the player toward and then around the workshop
- **THEN** the character is stopped by its solid boundary, can navigate around it, and remains followed by the camera

### Requirement: Nearby object inspection
The player SHALL be able to select a nearby colony object and inspect its identity and available authoritative state. Unavailable simulation details SHALL be identified as unavailable rather than fabricated.

#### Scenario: Inspect a worker at a stockpile
- **WHEN** the player approaches and inspects a worker beside a stockpile
- **THEN** the inspection identifies that worker and shows the latest provided activity and inventory information, with a clear indication for unavailable details

### Requirement: Exclusive editor input
Focused code editing SHALL consume typing and editor-navigation input rather than also controlling character movement or the camera. Closing or leaving the editor SHALL restore world input predictably.

#### Scenario: Type movement keys in code
- **WHEN** the policy editor has focus and the player types movement-key letters or scrolls its content
- **THEN** the code is edited or scrolled and the character and world camera do not move in response

### Requirement: Distinct draft and active program
The interface SHALL distinguish draft source, active source revision, and diagnostics. Editing or failed validation SHALL NOT silently activate a draft or reset the world.

#### Scenario: Inspect an invalid draft
- **WHEN** a player edits the draft into an invalid program
- **THEN** the interface marks it unapplied, displays validation diagnostics when requested, and retains the active program and world
