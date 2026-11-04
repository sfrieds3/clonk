# Disposable Godot / Rhai playable spike

This is an experiment, **not an engine selection, production client, complete economy, or security sandbox**. The Rust worker hosts the same adjacent `colony-runtime` fixture; Godot displays snapshots and never completes jobs or changes stock. Fixture stock grows by one every five fake progress steps, and the first job drives the single illustrated worker. Multiple goal keys are accepted by the fixture but this visualization shows only the first job/goal; it is not routing, carrying, conservation, real workshop production, or a scheduler. A crate appears for each of the first three stock units; the inspection number remains authoritative. Worker, stockpile and workshop IDs are fixture IDs. Inventory details, routes and provenance are unavailable.

## Run on the tested macOS arm64 setup

The tested engine is a **locally patched source build**, not the official binary. Set `GODOT` to that executable; the official unpatched 4.7.2 binary still crashes on a fresh extension editor import. The patched build uses 4.7.2 commit `ed1daf0bf001b61586d9930840f2f1394092c079` plus upstream fix `5017b3b63302c0e99497a93bd468f019401668bf` and differs from the optimized official binary. See the external change's `spike-evidence.md` and the local scratch source-validation report for provenance and limits. The extension library path currently targets the adjacent macOS debug build; this is not a portable package.

From the repository root:

```sh
export GODOT=/path/to/patched-godot
spikes/godot/run.sh
cargo test --locked --manifest-path spikes/colony-runtime/Cargo.toml
cargo test --locked --manifest-path spikes/godot-bridge/Cargo.toml
cargo build --locked --manifest-path spikes/godot-bridge/Cargo.toml
"$GODOT" --headless --path spikes/godot --editor --import --quit
"$GODOT" --headless --path spikes/godot --script res://tests/integration.gd
```

WASD walks; F2 focuses the editor; Escape leaves it; E inspects the nearest worker, stockpile, or workshop within range. Tab remains an editor key. Click Apply or press Ctrl+Enter while editing. Draft text remains separate from the last accepted source; a busy request does not submit, and a rejection retains the active revision and diagnostic while later ticks continue. Change the sample's `20` to another integer to revise the maintained target. To try atomic rejection, replace the source with `colony.keep_stock("new", 4); throw "late failure";`; to try the operation limit, use `loop { }`. Reapply the original sample afterwards. The fixture disallows `try` and uses an opaque constant receiver; prior concerns about mutable `__host` and catchable errors do not describe the current implementation. This does not prove an aggregate heap bound or a complete capability allowlist. Never expose it to untrusted scripts.

Inspection shows unavailable until the first worker reply, including when the first activation fails. Leaving the finite floor returns the player to the start without resetting the simulation. A disconnected worker is reported as unavailable and is not automatically retried; this does not cancel or time out a stuck native callback. On successful ticks, the accepted goals are reconciled against the resulting stock without re-running the policy.

The procedural geometry and shader are original to the spike and require no downloaded artwork. The UI uses bundled Lato Regular and Source Code Pro Regular, copied from the repository assets; their copyright notices and SIL Open Font License 1.1 are in `fonts/LICENSE.css`. Godot is MIT-licensed, godot-rust 0.5.5 is MPL-2.0, and Rhai 1.27.0 and transitive dependencies need a redistribution audit before packaging. No recorded walkthrough, user visual acceptance, performance measurements, exported package, Linux or Windows verification is claimed.
