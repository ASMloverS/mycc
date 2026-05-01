#!/usr/bin/env bash
# Build dispatch-rs in release mode and deploy binaries
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$(cd "$DIR/.." && pwd)"
TOOLS_DIR="$DIR/tools"
DISPATCH_BIN_DIR="$DIR/bin"

case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) EXT=".exe" ;;
    *)                    EXT=""    ;;
esac

echo "Building dispatch and harness-install in release mode..."
cargo build --release --bin dispatch --bin harness-install --manifest-path "$DIR/Cargo.toml"

# Deploy dispatch (symlink in ../bin/)
SRC_DISPATCH="$DIR/target/release/dispatch${EXT}"
LINK_DISPATCH="$BIN_DIR/dispatch${EXT}"

if [[ ! -f "$SRC_DISPATCH" ]]; then
    echo "ERROR: built binary not found at $SRC_DISPATCH" >&2
    exit 1
fi

echo "Removing stale dispatch entries in $BIN_DIR..."
rm -f "$BIN_DIR/dispatch" "$BIN_DIR/dispatch.exe"

echo "Creating symlink $LINK_DISPATCH -> $SRC_DISPATCH"
ln -s "$SRC_DISPATCH" "$LINK_DISPATCH"

# Deploy harness-install to dispatch-rs/tools/
SRC_INSTALL="$DIR/target/release/harness-install${EXT}"
DEST_INSTALL="$DISPATCH_BIN_DIR/harness-install${EXT}"

if [[ ! -f "$SRC_INSTALL" ]]; then
    echo "ERROR: built binary not found at $SRC_INSTALL" >&2
    exit 1
fi

mkdir -p "$DISPATCH_BIN_DIR"
echo "Copying $SRC_INSTALL -> $DEST_INSTALL"
cp -f "$SRC_INSTALL" "$DEST_INSTALL"

echo "Done."
echo "  dispatch:        $(du -h "$SRC_DISPATCH" | cut -f1)  $LINK_DISPATCH"
echo "  harness-install: $(du -h "$SRC_INSTALL" | cut -f1)  $DEST_INSTALL"
