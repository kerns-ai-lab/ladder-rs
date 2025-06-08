# Makefile for ladder-rs project
# Common development tasks

.PHONY: check fmt test clippy doc build clean ci-check fix

# Run all CI checks
ci-check:
	@./scripts/ci-check.sh

# Format code
fmt:
	cargo fmt

# Run tests
test:
	cargo test --all-features

# Run clippy linter
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Run clippy and fix automatically fixable warnings
clippy-fix:
	cargo clippy --all-targets --all-features --fix --allow-dirty

# Build documentation
doc:
	cargo doc --no-deps --all-features --open

# Build all targets
build:
	cargo build --all-targets --all-features

# Clean build artifacts
clean:
	cargo clean

# Fix common issues automatically
fix: fmt clippy-fix
	@echo "✅ Applied automatic fixes. Review changes before committing."

# Check everything (quick CI simulation)
check: fmt clippy test
	@echo "✅ All checks passed!"