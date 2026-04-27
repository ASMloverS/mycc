#!/usr/bin/env bash
# Build release binary and deploy to bin/dispatch.exe
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$DIR/../bin"

echo "Building dispatch in release mode..."
cargo build --release --manifest-path "$DIR/Cargo.toml"

echo "Deploying to $BIN_DIR/dispatch.exe..."
cp "$DIR/target/release/dispatch.exe" "$BIN_DIR/dispatch.exe"
echo "Done. Size: $(du -h "$BIN_DIR/dispatch.exe" | cut -f1)"
