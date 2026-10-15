#!/bin/bash
# CI Check Script - Runs all checks that CI will run
# This helps catch issues before pushing

set -e

echo "🔍 Running CI checks locally..."
echo "================================"

# The root Cargo.toml is both a package and the workspace, with no
# default-members, so cargo commands without --workspace (or --all for fmt)
# act on the root `ladder-rs` crate only. Every step below covers all members:
# ladder-rs, ladder-rs-persistence, ladder-rs-server and ladder-rs-wasm.

# 1. Format check
echo "📋 Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Format check failed!"
    echo "Run 'cargo fmt --all' to fix formatting issues."
    exit 1
fi
echo "✅ Format check passed!"

# 2. Clippy check with all targets and features
echo ""
echo "🔍 Running clippy on all targets..."
if ! cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy check failed!"
    echo "Fix the warnings above before pushing."
    exit 1
fi
echo "✅ Clippy check passed!"

# 3. Run all tests. cargo-nextest when installed (CI installs it): it runs
# test binaries in parallel. It does not run doctests, so those run separately.
echo ""
echo "🧪 Running all tests..."
if cargo nextest --version >/dev/null 2>&1; then
    if ! cargo nextest run --workspace --all-features --no-fail-fast \
        || ! cargo test --workspace --all-features --doc; then
        echo "❌ Tests failed!"
        echo "Fix the failing tests before pushing."
        exit 1
    fi
elif ! cargo test --workspace --all-features; then
    echo "❌ Tests failed!"
    echo "Fix the failing tests before pushing."
    exit 1
fi
echo "✅ All tests passed!"

# 4. Check documentation. Clippy already type-checks every target, so this
# step only adds value with rustdoc warnings (broken links, bad HTML) denied.
# A plain `cargo build --all-targets` step used to follow; it was redundant
# with clippy --all-targets and has been dropped.
echo ""
echo "📚 Checking documentation..."
if ! RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features; then
    echo "❌ Documentation check failed!"
    echo "Fix documentation errors before pushing."
    exit 1
fi
echo "✅ Documentation check passed!"

# 5. WASM bundle size check
echo ""
echo "📦 Verifying WASM bundle size..."
if ! "$(dirname "$0")/bundle_size_check.sh"; then
    echo "❌ Bundle size check failed!"
    exit 1
fi
echo "✅ Bundle size within limits!"

echo ""
echo "================================"
echo "✅ All CI checks passed! Safe to push."