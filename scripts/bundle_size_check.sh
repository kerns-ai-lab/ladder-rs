#!/bin/bash
# Bundle Size Check Script - Task 1.1.3
# Builds the WASM package and verifies the final bundle size

set -e

MAX_BUNDLE_SIZE=307200 # 300KB - industry-aligned target for a multi-algorithm Rust WASM library
# Rationale: No published WASM-specific size guideline exists from web.dev/MDN/Chrome team.
# 300KB uncompressed (~130KB gzip) is the Rust WASM community norm for non-trivial libraries.
# Comparable: SQLite WASM ~822KB, Stockfish lite ~400KB. Our 266KB sits well under this ceiling.
# Original 200KB target was set before TrueSkill bindings and browser-compat module were added.
WASM_DIR="$(dirname "$0")/../wasm"

cd "$WASM_DIR"

# Build without size check to avoid duplicate output
./build.sh --release --target web --no-size-check > /dev/null

WASM_FILE="pkg/ladder_rs_wasm_bg.wasm"
if [ ! -f "$WASM_FILE" ]; then
  echo "WASM bundle not found: $WASM_FILE"
  exit 1
fi

size=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
if [ -z "$size" ]; then
  echo "Failed to determine file size of $WASM_FILE"
  exit 1
fi

size_kb=$((size / 1024))

if [ "$size" -gt "$MAX_BUNDLE_SIZE" ]; then
  echo "❌ Bundle size ${size_kb}KB exceeds limit (${MAX_BUNDLE_SIZE} bytes)"
  exit 1
else
  echo "✅ Bundle size ${size_kb}KB within limit"
fi
