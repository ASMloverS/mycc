#!/usr/bin/env bash
# Build dispatch-rs in release mode and create a symlink in ../
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$(cd "$DIR/.." && pwd)"

case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) EXT=".exe" ;;
    *)                    EXT=""    ;;
esac

SRC="$DIR/target/release/dispatch${EXT}"
LINK="$BIN_DIR/dispatch${EXT}"

echo "Building dispatch in release mode..."
cargo build --release --manifest-path "$DIR/Cargo.toml"

if [[ ! -f "$SRC" ]]; then
    echo "ERROR: built binary not found at $SRC" >&2
    exit 1
fi

echo "Removing stale entries in $BIN_DIR..."
rm -f "$BIN_DIR/dispatch" "$BIN_DIR/dispatch.exe"

echo "Creating symlink $LINK -> $SRC"
ln -s "$SRC" "$LINK"

echo "Done. Target size: $(du -h "$SRC" | cut -f1)"
ls -l "$LINK"
