#!/bin/bash
# CI Check Script - Runs all checks that CI will run
# This helps catch issues before pushing

set -e

echo "🔍 Running CI checks locally..."
echo "================================"

# 1. Format check
echo "📋 Checking code formatting..."
if ! cargo fmt -- --check; then
    echo "❌ Format check failed!"
    echo "Run 'cargo fmt' to fix formatting issues."
    exit 1
fi
echo "✅ Format check passed!"

# 2. Clippy check with all targets and features
echo ""
echo "🔍 Running clippy on all targets..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy check failed!"
    echo "Fix the warnings above before pushing."
    exit 1
fi
echo "✅ Clippy check passed!"

# 3. Run all tests
echo ""
echo "🧪 Running all tests..."
if ! cargo test --all-features; then
    echo "❌ Tests failed!"
    echo "Fix the failing tests before pushing."
    exit 1
fi
echo "✅ All tests passed!"

# 4. Check documentation
echo ""
echo "📚 Checking documentation..."
if ! cargo doc --no-deps --all-features; then
    echo "❌ Documentation check failed!"
    echo "Fix documentation errors before pushing."
    exit 1
fi
echo "✅ Documentation check passed!"

# 5. Build check
echo ""
echo "🔨 Checking if project builds..."
if ! cargo build --all-targets --all-features; then
    echo "❌ Build failed!"
    echo "Fix build errors before pushing."
    exit 1
fi
echo "✅ Build check passed!"

echo ""
echo "================================"
echo "✅ All CI checks passed! Safe to push."