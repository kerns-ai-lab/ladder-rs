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
# Performance testing targets
.PHONY: bench bench-wasm perf-test perf-baseline perf-compare

# Run all benchmarks
bench:
	@echo "Running native benchmarks..."
	cargo criterion

# Run WASM benchmarks
bench-wasm:
	@echo "Running WASM benchmarks..."
	cd wasm && cargo criterion

# Run WASM performance tests
perf-test:
	@echo "Running WASM performance tests..."
	cd wasm && wasm-pack test --headless --chrome --test performance_regression_tests

# Generate performance baseline
perf-baseline:
	@echo "Generating performance baseline..."
	cargo criterion --message-format=json > baseline-results.json
	cd wasm && cargo criterion --message-format=json > ../wasm-baseline-results.json

# Compare performance against baseline
perf-compare: bench
	@echo "Comparing performance against baseline..."
	@if [ -f baseline-results.json ]; then \
		python3 scripts/check_performance_regression.py \
			--current benchmark-results.json \
			--baseline baseline-results.json \
			--threshold 10; \
	else \
		echo "No baseline found. Run 'make perf-baseline' first."; \
	fi

