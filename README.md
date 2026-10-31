# CLONK

*They're doing exactly what you told them.*

A programmable little robot colony, starting with one enthusiastic machine and eight rocks. The direction is open-ended automation: your first shift is an onboarding milestone, not a win screen.

## Play

Run `cargo run`. Click **Run** to watch the starter program collect ore, build a charger, recharge, and deliver another ore, or **Step** to execute one action. The first-shift checklist tracks your progress; completing it does not stop execution. The next action appears below the world. Edit the program and click **Apply & reset** to load it. Editing pauses execution; Run and Step use the last successfully applied program, with unapplied edits clearly marked. Parse errors remain in the editor panel until a successful Apply, independently of robot messages. Reset restarts the applied program and clears the world and milestone progress, not your draft.

## Battery

CL-01 starts with 24 energy. Each action except `recharge()` costs one energy, including failed actions, each expanded move/turn, and `repeat()`. At zero energy, ordinary actions stop without advancing the program. `recharge()` works even at zero: stand next to the charger and face it to refill to 24. A recharge attempt uses a tick and advances the program but costs no energy, even if incorrectly positioned. If stranded, edit your route and Apply & reset, or Reset to retry the applied program. The starter has enough energy to build its charger, but needs its charging stop to finish the next trip.

## Instructions

One instruction per line. Instructions are case insensitive; blank lines and `#` comments are supported. The robot starts at home facing east.

- `move(5)` moves up to five squares, one action per square. Walls and chargers block movement. `move()` and `MOVE` move once.
- `turn(left)`, `turn(right)` and `turn(around)` turn in place. `turn(left, 2)` turns twice. A bare `TURN` turns right once. Counts must be 1–100.
- `collect()` picks up one ore at the ore deposit. Capacity is one.
- `deposit()` stores the carried ore at home.
- `build(charger)` spends one stored ore to build a charger in the square directly ahead. Build while standing at home, facing a free square. Only one charger can be built for now.
- `recharge()` refills the battery while facing an adjacent charger. Chargers block movement; do not try to stand on one.
- `repeat()` jumps back to the beginning. A program without it stops at the end.

Commands with counts expand into individual actions, so Step and the next-action display show each movement or turn separately. Legacy bare commands still work.

## Development

Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings`. The simulation and parser live in `src/sim.rs`; the desktop view and controls live in `src/main.rs`.
