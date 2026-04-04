# WASM Bundle Size

## Current Size

| Build | Uncompressed | Estimated gzip | Estimated brotli |
|-------|-------------|----------------|------------------|
| Release (`wasm-opt -Oz`) | ~266 KB | ~130 KB | ~110 KB |

## Target

**300 KB uncompressed** (`MAX_BUNDLE_SIZE=307200` in `scripts/bundle_size_check.sh`)

This gives a comfortable ~11% margin over the current build and aligns with
industry norms for Rust-compiled WASM libraries of comparable complexity.

## Rationale

The original 200 KB target was set when the WASM module only exposed the Elo
algorithm. It was never revised after these additions:

- **Task 1.3.3/1.3.4** — Glicko and TrueSkill WASM bindings
- **Task 1.4.3** — Browser compatibility detection module (`browser_compat.rs`)

### Industry context

There is no published WASM-specific size guideline from web.dev, MDN, or the
Chrome team. The Rust WASM community notes that bundles "often exceed 300 KB
uncompressed" for non-trivial libraries.

| Library | Uncompressed WASM |
|---------|------------------|
| Rust hello world (after `wasm-opt`) | ~17 KB |
| Single crypto algorithm (e.g. argon2id) | ~30–50 KB |
| **ladder-rs** (4 algorithms + bindings) | **~266 KB** |
| Stockfish chess (simpler build) | ~400 KB |
| SQLite WASM (`@sqlite.org/sqlite-wasm`) | ~822 KB – 3.5 MB |

### Wire-size and load time

The compressed wire size matters more than the raw binary size. At ~130 KB
gzipped, the module downloads in:

| Connection | Transfer time |
|------------|--------------|
| Slow 3G (~400 Kbps) | ~2.6 s |
| Fast 3G (~1.6 Mbps) | ~0.65 s |
| 4G LTE (avg ~20 Mbps) | ~52 ms |
| WiFi / 5G | <25 ms |

With `WebAssembly.instantiateStreaming()`, download and compilation are
pipelined, so on 4G+ the module is effectively invisible to the user. After
the first load, the browser caches the compiled module — parse/compile cost is
zero on subsequent visits.

## Optimization history

| Date | Size | Change |
|------|------|--------|
| Task 1.1.1 (Elo only) | ~72 KB | Initial elo-only build |
| Task 1.3.4 + 1.4.3 (all algorithms + browser compat) | ~272 KB | Added TrueSkill, Glicko, browser-compat |
| Task 1.1.3 (libm replacement) | ~266 KB | Replaced `statrs`+`nalgebra` with `libm` for Gaussian math |

### Why libm instead of statrs?

`statrs` was used solely for `Normal(0,1).cdf(x)` and `Normal(0,1).pdf(x)` —
eight call sites in `src/trueskill.rs`. `statrs` depends on `nalgebra` (a full
linear algebra library). Although `wasm-opt -Oz` dead-code-eliminates much of
`nalgebra`, replacing `statrs` with direct `libm::erfc` calls removes the
transitive dependency entirely and produces a cleaner, more auditable
dependency graph.

The math is equivalent:
- `pdf(x)  = exp(-x²/2) / √(2π)`
- `cdf(x)  = 0.5 × erfc(-x / √2)`

## Running the check locally

```bash
./scripts/bundle_size_check.sh
# or via Make:
make ci-check
```
