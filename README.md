# CLONK

*They're doing exactly what you told them.*

A programmable little robot colony, starting with one enthusiastic machine and twelve rocks. The direction is open-ended automation: your first shift is an onboarding milestone, not a win screen.

## Play

Run `just run` (or `cargo run`). Click **Run** to watch the starter program mine ore, build a charger and assemble a colony of four robots, or **Step** to advance every robot by one action. Each robot's battery, next source line and latest complaint appear below the world. Edit the program and click **Apply & reset** to load it. Editing pauses execution; Run and Step use the last successfully applied program, with unapplied edits clearly marked. Parse errors remain in the editor panel until a successful Apply, independently of robot messages. Reset restarts the applied program and clears the world and milestone progress, not your draft. The first-shift checklist tracks building a charger, recharging, and then delivering ore with a recharged robot; completing it does not stop execution.

Prebuilt binaries for Linux, macOS and Windows are attached to each [GitHub release](../../releases).

## The colony

Every robot runs the **same program**, each with its own place in it. Robots block each other, so two robots that want the same square will happily clonk into each other until one of them moves or runs out of power.

- **Batteries.** Robots start with 24 energy. Every action costs 1, including failed actions and each expanded move/turn, except `charge()` (also spelled `recharge()`) and `wait()`, which are free. A robot that tries to act on an empty battery powers down for good and becomes an obstacle.
- **Chargers** cost 1 ore. Facing one and calling `charge()` refills the battery.
- **Robots** cost 2 ore. The new robot appears in the square ahead, facing the same way as its builder, and starts at the top of the program on the next tick. The colony holds four robots.

## Instructions

One instruction per line. Instructions are case insensitive; blank lines, indentation and `#` comments are supported. CL-01 starts at home facing east.

- `move(5)` moves up to five squares, one action per square. Walls, chargers and robots block movement. `move()` and `MOVE` move once.
- `turn(left)`, `turn(right)` and `turn(around)` turn in place. `turn(left, 2)` turns twice; `turn(around, 2)` turns around twice. A bare `TURN` turns right once. Counts must be 1–100.
- `collect()` picks up one ore at the ore deposit. Capacity is one.
- `deposit()` stores the carried ore at home.
- `charge()` (or `recharge()`) refills the battery from the charger directly ahead, even at zero energy if the robot has not powered down. An unsuccessful attempt still advances the program but costs no energy. An empty battery leaves a paid action pending and powers the robot down; use Reset or Apply & reset to retry.
- `wait()` does nothing, for free.
- `build(charger)` and `build(robot)` spend stored ore to build in the square directly ahead. Build while standing at home, facing a free square.
- `repeat()` jumps back to the beginning. A program without it stops at the end.

### Sensors and control flow

```
if(<condition>)        while(<condition>)
  ...                    ...
else                   end
  ...
end
```

Conditions look at the square directly ahead or at the robot itself, and any of them can be negated with `not`:

| Condition | True when |
| --- | --- |
| `wall` | a wall is ahead |
| `robot` | another robot is ahead |
| `charger` | a charger is ahead |
| `blocked` | anything is ahead that would stop a move |
| `ore` / `home` | standing on the ore deposit / home |
| `carrying` | holding ore |
| `low` | battery is below 10 |
| `id(2)` | this robot is CL-02 |

Checking conditions and jumping are instant and cost no energy. A robot that loops for a whole tick without reaching an action gets lost in thought and stops.

Commands with counts expand into individual actions, so Step executes each movement or turn separately. Legacy bare commands still work.

## Development

With Rust and `just` installed, run `just` to list commands:

- `just run` launches the game.
- `just build` builds the desktop app.
- `just test` runs the tests.
- `just fmt` formats Rust source.
- `just lint` runs strict Clippy checks.
- `just check` checks formatting, runs tests, and runs strict Clippy without changing source files.

The recipes wrap standard Cargo commands, so `just` is optional. CI runs the same checks on every pull request. The simulation and parser live in `src/sim.rs`; the desktop view and controls live in `src/main.rs`.

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `ci:`, `refactor:`, …), and CI checks them. [release-please](https://github.com/googleapis/release-please) turns them into a release pull request with a version bump and changelog; merging it tags a release and attaches binaries.
