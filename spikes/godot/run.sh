#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
: "${GODOT:=godot}"
cargo build --locked --manifest-path "$ROOT/spikes/godot-bridge/Cargo.toml"
"$GODOT" --headless --path "$ROOT/spikes/godot" --editor --import --quit
exec "$GODOT" --path "$ROOT/spikes/godot" "$@"
