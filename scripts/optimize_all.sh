#!/usr/bin/env bash
set -euo pipefail

echo "=== Optimizing Wasm Contracts ==="

WASM_DIR="target/wasm32-unknown-unknown/release"
OPT_DIR="target/wasm32-unknown-unknown/optimized"

mkdir -p "$OPT_DIR"

if ! command -v soroban &> /dev/null; then
    echo "WARNING: soroban CLI not found. Install with: cargo install soroban-cli"
    echo "Skipping Wasm optimization."
    exit 0
fi

for wasm in "$WASM_DIR"/*.wasm; do
    basename=$(basename "$wasm")
    echo "  Optimizing $basename..."
    soroban contract optimize \
        --wasm "$wasm" \
        --wasm-out "$OPT_DIR/$basename"
done

echo ""
echo "Optimized Wasm files written to $OPT_DIR"
ls -lh "$OPT_DIR"/
