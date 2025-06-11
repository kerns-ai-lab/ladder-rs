# Performance Testing Guide

This document describes the performance testing infrastructure implemented for the ladder-rs WASM module.

## Overview

The performance testing system consists of three main components:

1. **Browser-based performance tests** - Using `wasm-bindgen-test` for in-browser testing
2. **Native benchmarks** - Using Criterion for pre-WASM performance measurement
3. **CI/CD integration** - Automated performance regression detection in GitHub Actions

## Running Performance Tests

### Local Development

```bash
# Run native benchmarks
make bench

# Run WASM-specific benchmarks
make bench-wasm

# Run browser-based performance tests
make perf-test

# Generate performance baseline
make perf-baseline

# Compare against baseline
make perf-compare
```

### Continuous Integration

Performance tests run automatically on:
- Every push to `main` branch
- Every pull request
- Nightly at 2 AM UTC
- Manual workflow dispatch

## Test Structure

### Browser Performance Tests

Located in `wasm/tests/performance_regression_tests.rs`:

- **Initialization tests** - Measure system creation overhead
- **Rating update tests** - Benchmark rating calculations
- **Serialization tests** - Test JSON conversion performance
- **Memory usage tests** - Track memory allocation patterns
- **Batch operation tests** - Measure bulk processing performance
- **Cross-algorithm tests** - Compare different rating systems
- **Real-world scenarios** - Simulate tournament usage patterns

### Native Benchmarks

Located in `wasm/benches/wasm_performance.rs`:

- **Serialization benchmarks** - JSON encoding/decoding performance
- **Interface pattern benchmarks** - WASM-style API performance
- **Memory pattern benchmarks** - Allocation and collection patterns
- **Algorithm adaptation benchmarks** - Batch processing at various scales
- **Critical path benchmarks** - Performance-sensitive operations

## Performance Tracking

The `performance_tracking` module provides:

- Metric collection and storage
- Regression detection with configurable thresholds
- HTML and Markdown report generation
- Baseline management

### Usage Example

```rust
use ladder_rs_wasm::PerformanceTracker;

let mut tracker = PerformanceTracker::new("browser-chrome".to_string());

// Record metrics
tracker.record_metric("elo_update", 1.5, 1000);
tracker.record_metric_with_memory("batch_processing", 150.0, 100, 1024 * 1024);

// Check for regressions
if !tracker.check_regressions() {
    println!("Performance regressions detected!");
    println!("{}", tracker.generate_report());
}
```

## Performance Baselines

Default performance baselines are defined for common operations:

| Operation | Min Ops/Second | Max Duration (ms) |
|-----------|----------------|-------------------|
| create_elo_system | 10,000 | 0.1 |
| create_trueskill_system | 5,000 | 0.2 |
| elo_1v1_update | 1,000 | 1.0 |
| trueskill_1v1_update | 100 | 10.0 |
| serialize_100_players | 100 | 10.0 |
| batch_100_matches | 10 | 100.0 |

## Regression Detection

The system detects three severity levels:

- **Minor** - Less than 10% regression
- **Major** - 10-50% regression  
- **Critical** - More than 50% regression

## CI/CD Integration

### GitHub Actions Workflow

The `.github/workflows/performance.yml` workflow:

1. Runs native and WASM benchmarks
2. Executes browser-based tests in Chrome and Firefox
3. Compares results against baseline (for PRs)
4. Posts results as PR comments
5. Stores baselines for future comparisons

### Performance Analysis Script

The `scripts/check_performance_regression.py` script:

- Parses Criterion benchmark results
- Compares against baseline
- Generates Markdown reports
- Exits with error on regression

## Best Practices

1. **Run benchmarks before optimization** - Establish baseline performance
2. **Test in multiple browsers** - Performance can vary significantly
3. **Use realistic data sizes** - Test with production-scale data
4. **Monitor memory usage** - WASM has different memory characteristics
5. **Set appropriate thresholds** - Balance between noise and real regressions

## Troubleshooting

### Common Issues

1. **Benchmarks timing out**
   - Reduce iteration count
   - Check for infinite loops
   - Verify WASM module loads correctly

2. **Inconsistent results**
   - Run benchmarks multiple times
   - Check for background processes
   - Use performance isolation if available

3. **CI failures**
   - Review workflow logs
   - Check browser compatibility
   - Verify WASM build succeeds

## Future Improvements

- Add flame graph generation
- Implement memory profiling
- Add WebAssembly-specific metrics
- Create performance dashboard
- Add statistical analysis for noise reduction