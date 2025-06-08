# Development Workflow Guide

## Before Pushing Code

To ensure your code will pass CI checks, follow these steps:

### 1. Quick Check (Recommended before every commit)
```bash
make check
```
This runs formatting, clippy, and tests.

### 2. Full CI Check (Recommended before pushing)
```bash
make ci-check
# or
./scripts/ci-check.sh
```
This runs exactly what CI will run on GitHub.

### 3. Fix Common Issues Automatically
```bash
make fix
```
This will:
- Format your code with `cargo fmt`
- Fix auto-fixable clippy warnings

### 4. Manual Fixes
For issues that can't be automatically fixed:
- **Unused imports**: Remove them or add `#[allow(unused_imports)]` if intentional
- **Dead code**: Remove it or add `#[allow(dead_code)]` if needed for future use
- **Clippy warnings**: Follow the suggestions in the error messages

## Pre-Push Hook

A pre-push hook is installed that will:
1. Check formatting
2. Run clippy on all targets
3. Run tests

Install the hook by running:
```bash
./scripts/install-hooks.sh
```

If any of these fail, the push will be blocked.

## VS Code Setup

If using VS Code, the project includes settings that will:
- Format on save
- Run clippy checks in the background
- Show warnings inline

## Common Commands

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Build everything
cargo build --all-targets --all-features

# Generate docs
cargo doc --no-deps --all-features --open
```

## Troubleshooting

### "error: failed to push some refs"
This means the pre-push hook found issues. Run:
```bash
make ci-check
```
to see what needs to be fixed.

### Clippy warnings in dependencies
Focus on warnings in your code (src/, tests/, benches/, examples/).
Warnings in target/ or dependencies can be ignored.

### Too many warnings to fix at once
1. Fix critical errors first (compilation errors)
2. Then fix clippy errors (marked with `-D warnings`)
3. Finally, address clippy warnings if time permits