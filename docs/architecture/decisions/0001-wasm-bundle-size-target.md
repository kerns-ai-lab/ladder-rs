# ADR-0001: WASM Bundle Size Target

**Status:** Accepted  
**Date:** 2026-04-04  
**Deciders:** Dustin Kerns  

---

## Context

The ladder-rs WASM module exposes Elo, Glicko-2, and TrueSkill rating algorithms
to JavaScript via wasm-bindgen. When the project began, the WASM bundle only
included the Elo algorithm and a 200 KB uncompressed size limit was set as an
internal constraint in `scripts/bundle_size_check.sh`.

Over subsequent tasks the module grew significantly:

| Task | Addition |
|------|----------|
| 1.3.3 / 1.3.4 | Glicko and TrueSkill WASM bindings |
| 1.4.3 | Browser compatibility detection module (`browser_compat.rs`) using `web-sys` |

The size limit was not revised at those milestones. By the time task 1.4.4 was
completed the bundle had grown to ~272 KB — 36% over the original 200 KB
target — and the limit had been bumped twice as a workaround without documented
rationale.

During task 1.1.3 (bundle size optimisation) we investigated the growth,
profiled the binary, and researched industry norms before settling on a new
target.

## Decision

**Set the WASM bundle size ceiling at 300 KB uncompressed.**

The limit is enforced by `MAX_BUNDLE_SIZE=307200` in
`scripts/bundle_size_check.sh`, which is called by `scripts/ci-check.sh` on
every CI run.

## Rationale

### No authoritative external standard exists

Web.dev, MDN, and the Chrome team publish no WASM-specific size guideline.
Their performance budgets address total page weight and critical-path JavaScript,
not compiled WASM modules loaded as a library.

### Comparable libraries exceed 200 KB

| Library | Uncompressed WASM |
|---------|------------------|
| Single crypto algorithm (e.g. argon2id) | ~30–50 KB |
| **ladder-rs** (3 algorithms + bindings) | **~266 KB** |
| Stockfish chess (simpler build) | ~400 KB |
| SQLite (`@sqlite.org/sqlite-wasm`) | ~822 KB – 3.5 MB |

The Rust WASM community explicitly notes bundles "often exceed 300 KB
uncompressed" for non-trivial libraries. Our 266 KB sits comfortably within
that norm.

### Wire size is what users experience

WASM compresses well. The 266 KB binary delivers ~130 KB over the wire via
gzip and ~110 KB via brotli. At those sizes the module downloads in:

| Connection | Transfer time (gzip ~130 KB) |
|------------|------------------------------|
| Slow 3G (~400 Kbps) | ~2.6 s |
| Fast 3G (~1.6 Mbps) | ~0.65 s |
| 4G LTE (avg ~20 Mbps) | ~52 ms |
| WiFi / 5G | <25 ms |

With `WebAssembly.instantiateStreaming()`, download and compilation pipeline
together. On any 4G+ connection the module is imperceptible to users. The
browser caches the compiled module after the first visit — subsequent loads
have zero parse or compile cost.

### The 200 KB target was not revisited as scope grew

The original target was set for a single-algorithm (Elo-only) build. It was
never re-evaluated when TrueSkill, Glicko, and browser-compat were added.
Continuing to enforce it would require removing features rather than fixing a
genuine performance problem.

### 300 KB leaves meaningful headroom

The current build is ~266 KB — 11% under the new ceiling. This provides room
for future algorithm additions or API expansion without immediately re-opening
the size conversation.

## Alternatives Considered

### Keep 200 KB target, remove features to comply

Would require dropping TrueSkill WASM bindings or the browser-compat module.
Rejected: these are core deliverables from tasks 1.3.4 and 1.4.3 and removing
them would regress functionality without meaningful user benefit.

### Accept unbounded growth (no limit)

Rejected: a ceiling still serves as a canary for unintended dependency bloat.
The limit enforces intentionality — any future growth past 300 KB requires a
documented decision.

### Set target at 256 KB (a round binary boundary)

Considered. Rejected because it would require trimming `web-sys` features or
the `browser_compat.rs` API surface to achieve without removing algorithms,
which was assessed as optimisation for its own sake at this stage.

## Consequences

- `scripts/bundle_size_check.sh` enforces 300 KB on every CI run.
- `wasm/docs/bundle-size.md` documents the size history, rationale, and
  wire-size analysis for future maintainers.
- Any future addition that pushes the bundle past 300 KB requires a new ADR
  or an amendment to this one before the limit is raised.
- The `statrs` + `nalgebra` dependency was replaced with `libm` during this
  work (saving ~6 KB and removing a heavy transitive dependency), independently
  of the target change.
