# CLONK

*They're doing exactly what you told them.*

A programmable little robot colony, starting with one enthusiastic machine and eight rocks.

## Play

Install Rust, then run `cargo run`. The window opens with a working ore delivery program. Click **Run** to watch it, or **Step** to execute one instruction. The next instruction appears below the world.

Edit the program and click **Apply & reset** to load it and restart the world. Editing pauses execution; Run and Step use the last successfully applied program. **Reset** restarts that program. The speed button toggles normal and fast execution.

## Instructions

- `MOVE`: advance one square; walls block movement.
- `TURN`: rotate clockwise by 90 degrees.
- `COLLECT`: pick up one ore at the ore deposit; capacity is one.
- `DEPOSIT`: unload at the blue home square.
- `REPEAT`: jump back to the start of the program.

Instructions are case insensitive. Blank lines and `# comments` are supported. The robot starts facing east. A program without REPEAT stops at its end.

## Development

`cargo test` checks ore conservation, the demo delivery loop, wall collisions, program termination, and parsing. `cargo fmt --check` checks formatting. `cargo clippy --all-targets -- -D warnings` checks lints.

The simulation lives in `src/sim.rs`, independent of rendering. The desktop view and controls live in `src/main.rs`.

## Next experiments

- Conditions, sensors, and named loops.
- Inventory and program inspection, with execution history.
- Multiple robots, shared resources, and increasingly unfortunate traffic.
- Batteries, charging stations, factories, and construction.
- Deterministic replay and a debugger that explains each CLONK.
