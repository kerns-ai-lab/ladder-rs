#!/bin/bash
# Install git hooks for this repository
set -e
HOOK_DIR="$(git rev-parse --git-path hooks)"
SCRIPT_DIR="$(dirname "$0")"
cp "$SCRIPT_DIR/../githooks/pre-push" "$HOOK_DIR/pre-push"
chmod +x "$HOOK_DIR/pre-push"
echo "✅ pre-push hook installed to $HOOK_DIR/pre-push"
