# CLONK

*They're doing exactly what you told them.*

A programmable little robot colony, starting with one enthusiastic machine and eight rocks.

## Play

Run `cargo run`. Click **Run** to watch the starter program collect ore and build a charger, or **Step** to execute one action. The next action appears below the world. Edit the program and click **Apply & reset** to load it. Editing pauses execution; Run and Step use the last successfully applied program. Reset restarts that program.

## Instructions

One instruction per line. Instructions are case insensitive; blank lines and `#` comments are supported. The robot starts at home facing east.

- `move(5)` moves up to five squares, one action per square. Walls and chargers block movement. `move()` and `MOVE` move once.
- `turn(left)`, `turn(right)` and `turn(around)` turn in place. `turn(left, 2)` turns twice; `turn(around, 2)` turns around twice. A bare `TURN` turns right once. Counts must be 1–100.
- `collect()` picks up one ore at the ore deposit. Capacity is one.
- `deposit()` stores the carried ore at home.
- `build(charger)` spends one stored ore to build a charger in the square directly ahead. Build while standing at home, facing a free square. Only one charger can be built for now; it is decorative until batteries are added.
- `repeat()` jumps back to the beginning. A program without it stops at the end.

Commands with counts expand into individual actions, so Step executes each movement or turn separately. The next-action display shows the source line being executed. Legacy bare commands still work.

## Development

Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings`. The simulation and parser live in `src/sim.rs`; the desktop view and controls live in `src/main.rs`.
