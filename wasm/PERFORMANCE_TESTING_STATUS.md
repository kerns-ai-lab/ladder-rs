# WASM Performance Testing Status

## Current Status: All Algorithms Available

As of this milestone, all rating algorithms (Elo, Glicko, TrueSkill) are enabled
in WASM builds via `features = ["all-algorithms"]`.

## Background: Resolved rayon/WASI False Alarm

### The Misdiagnosis

A previous change locked the WASM crate to `features = ["elo-only"]`, excluding
TrueSkill from WASM builds. The belief was that `statrs` (used by TrueSkill's math)
pulled in `rayon`, which fails to compile for `wasm32-unknown-unknown` targets.

### Investigation Results

This belief was incorrect. Verified via `cargo tree -e features -i rayon`:

- `statrs` v0.16.1 has **no rayon dependency**. Its deps are: `approx`, `lazy_static`,
  `nalgebra`, `num-traits`, `rand`
- `rayon` only enters the dependency tree via ladder-rs's `full-deps` feature flag,
  which is never enabled in WASM builds
- `cargo tree -e features -i rayon --manifest-path wasm/Cargo.toml` returns
  "no package found" - confirming zero rayon in WASM builds

### Root Cause

The original breakage was caused by using `features = ["elo-only", "trueskill-only",
"statrs"]` - a broken/conflicting feature combination. The "fix" changed it to
`features = ["elo-only"]`, which happened to work but over-restricted the build.
It was fixing a bad feature combo, not a real rayon/WASI incompatibility.

### Actual Fix Applied

1. **`wasm/Cargo.toml`**: Changed `features = ["elo-only"]` to `features = ["all-algorithms"]`
   - `all-algorithms` enables `rand + statrs` (needed for TrueSkill and Glicko)
   - Does NOT enable `full-deps` (rayon/chrono/serde_json), maintaining WASI safety

2. **`src/glicko.rs`**: The `glicko.rs` implementation had an unconditional
   `use rayon::prelude::*` import and `.par_iter()` calls. These were gated behind
   `#[cfg(feature = "full-deps")]` with `.iter()` fallbacks for non-rayon builds.

## Testing Status

### Performance Regression Tests

Location: `wasm/tests/performance_regression_tests.rs`

Covers:
- Elo sequential match performance (200 matches)
- Elo batch player performance (50 players, round-robin)
- TrueSkill system creation in WASM
- TrueSkill 1v1 match processing
- TrueSkill sequential match stability (50 iterations)
- TrueSkill batch processing (10 players, 5 concurrent matches)
- TrueSkill leaderboard generation
- TrueSkill match quality calculation
- Cross-algorithm rating stability regression tests
- TrueSkill serialization round-trip precision
- TrueSkill win probability normalization

These tests require `wasm-bindgen-test` and a browser/wasm runner environment.
They are NOT run in GitHub Actions CI (no browser available in CI by default).

### Running Performance Tests Locally

```bash
# Install wasm-pack
cargo install wasm-pack

# Run tests in a headless Chrome browser
wasm-pack test --headless --chrome wasm/

# Or in Firefox
wasm-pack test --headless --firefox wasm/
```

### CI Integration

The `cargo build --target wasm32-unknown-unknown` step in CI confirms that the
WASM binary compiles cleanly with all algorithms. The bundle size check ensures
the binary stays under 200KB.

Note: Do NOT add benchmark targets to GitHub Actions workflows, as they consume
too many CI minutes. Run benchmarks locally only.

## WASM Bundle Size

Bundle size limits enforced in CI:
- Target: under 200KB for the release WASM binary

The addition of TrueSkill (via statrs/nalgebra) does increase binary size. If
the bundle grows too large, the CI bundle size check will fail with instructions
for optimization.
