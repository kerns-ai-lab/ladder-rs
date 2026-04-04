# Simplified Performance Testing

## Overview

To reduce GitHub Action minutes consumption, the performance testing infrastructure has been simplified to focus on essential WASM functionality tests only.

## What Changed

### Removed
- **Native Rust benchmarks** - These were consuming significant CI minutes
- **Criterion benchmarks** - Heavy performance testing moved to local development
- **Multi-browser testing** - Reduced to Chrome only to save minutes
- **Nightly scheduled runs** - Removed automatic daily runs
- **Performance regression checks** - Complex analysis removed from CI

### Kept
- **Basic WASM tests** - Minimal smoke tests to ensure WASM builds work
- **Single browser testing** - Chrome-only to verify basic functionality
- **PR validation** - Tests still run on pull requests

## Running Performance Tests Locally

Since benchmarks have been removed from CI, developers should run performance tests locally:

```bash
# Run native benchmarks
make bench

# Run WASM tests
make perf-test

# Generate performance baseline
make perf-baseline
```

## CI Workflow

The simplified workflow now:
1. Builds the WASM module
2. Runs minimal tests in Chrome
3. Reports success/failure

This reduces CI time from ~5 minutes to ~1-2 minutes per run.

## Future Considerations

When GitHub Action minutes are less of a concern, the full performance testing suite can be re-enabled by restoring the original `performance.yml` workflow.