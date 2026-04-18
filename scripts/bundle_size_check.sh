#!/bin/bash
# Bundle Size Check Script
# Builds the WASM package and reports the bundle size.
#
# Governance model (see docs/architecture/decisions/0001-wasm-bundle-size-target.md):
#   - SOFT_TARGET (300 KB): informational; logged as GitHub Actions ::warning::
#     when exceeded so growth is visible on every PR without blocking CI.
#   - HARD_CAP    (500 KB): panic threshold; CI fails only if exceeded. Intended
#     to catch accidental regressions (e.g. forgotten dev profile, unintended
#     dependency) rather than steady-state growth.
#
# Active enforcement of the 300 KB target is deliberately deferred; see the
# tracking bead referenced in ADR-0001 to re-engage optimisation work before
# public release.

set -e

SOFT_TARGET=307200  # 300 KB — documented target from ADR-0001
HARD_CAP=512000     # 500 KB — panic threshold to catch runaway growth

WASM_DIR="$(dirname "$0")/../wasm"

cd "$WASM_DIR"

# Build without the build script's inline size check to avoid duplicate output.
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
soft_kb=$((SOFT_TARGET / 1024))
hard_kb=$((HARD_CAP / 1024))

if [ "$size" -gt "$HARD_CAP" ]; then
  # GitHub Actions ::error:: annotation — also fails the job via exit 1.
  echo "::error::WASM bundle ${size_kb} KB exceeds HARD CAP of ${hard_kb} KB — investigate before merging."
  echo "Bundle size governance: docs/architecture/decisions/0001-wasm-bundle-size-target.md"
  exit 1
elif [ "$size" -gt "$SOFT_TARGET" ]; then
  # GitHub Actions ::warning:: annotation — surfaces in PR UI but does not fail.
  echo "::warning::WASM bundle ${size_kb} KB exceeds soft target of ${soft_kb} KB (hard cap ${hard_kb} KB). Optimisation deferred — see ADR-0001."
  echo "✅ Bundle size check passed (advisory warning only)."
else
  echo "✅ Bundle size ${size_kb} KB within ${soft_kb} KB soft target."
fi
