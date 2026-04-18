# ADR-0004: Cargo Workspace Crate Structure

**Status:** Accepted
**Date:** 2026-04-15
**Deciders:** Dustin Kerns

---

## Context

The ladder-rs project has grown from a single library crate into a multi-component platform. The architecture must accommodate:

1. **Pure rating math** (`ladder-rs`) — already implemented. Must remain free of database, HTTP, or async dependencies. Used by: persistence layer, WASM bindings, future CLI tools.

2. **Database access** (`ladder-rs-persistence`) — all SQLite operations. Must be async-native (tokio). Must be usable by two distinct consumers without modification: the Axum backend and swarm operator processes. Swarm operators link this crate directly from their own Rust code.

3. **HTTP server** (`ladder-rs-server`) — thin Axum wrapper. Depends on the persistence layer; must not import sqlx or write SQL directly.

4. **WASM bindings** (`ladder-rs-wasm`) — already implemented. Must remain WASM-compatible (no tokio, no sqlx, no native filesystem access). Depends only on `ladder-rs`.

The challenge: these four components have incompatible dependency requirements. A single crate cannot simultaneously be:
- WASM-compatible (no tokio, no sqlx)
- async-native with tokio (for DB access)
- HTTP-server-capable (Axum, tower)

An additional challenge: the persistence layer must serve two async consumers (server + swarm operator), but the swarm operator may want to call it from either an async or a sync context depending on their own architecture.

---

## Decision

Structure the project as a **Cargo workspace with four crates**:

```
ladder-rs/                  (workspace root, Cargo.toml with [workspace])
├── ladder-rs/              — pure math crate (existing, no changes)
├── ladder-rs-persistence/  — DB access crate (new, async-native, tokio)
├── ladder-rs-server/       — Axum HTTP server crate (new)
└── ladder-rs-wasm/         — WASM bindings crate (existing, no changes)
```

The `frontend/` directory is outside the Cargo workspace (it is a Node.js project):
```
frontend/                   — React/TypeScript/Vite (outside Cargo workspace)
```

**Dependency graph (arrows = "depends on"):**

```
ladder-rs-server
    └── ladder-rs-persistence
            └── ladder-rs

ladder-rs-wasm
    └── ladder-rs

(swarm operator — external)
    └── ladder-rs-persistence
            └── ladder-rs
```

No crate has a reverse dependency (no cycles). `ladder-rs` and `ladder-rs-persistence` are the only crates that swarm operators depend on. `ladder-rs-server` and `ladder-rs-wasm` are never depended on by external consumers.

---

## Rationale

### Core library stays pure

`ladder-rs` compiles without any runtime dependencies (it uses only `std` and optionally `libm`, `rayon`). This means it can be compiled to WASM, used in `no_std` environments, benchmarked without I/O overhead, and tested without infrastructure. Adding DB or async dependencies to this crate would contaminate all its use cases.

The four-crate structure maintains this purity permanently. `ladder-rs-persistence` takes on the async and DB dependencies; `ladder-rs` never needs to know they exist.

### Persistence as a shared library

The critical architectural insight is that both the Axum backend and the swarm operator need DB access. If DB access lived only in `ladder-rs-server`, the swarm operator would have no way to write to the DB without going through HTTP — which would bottleneck swarm throughput and create an operational dependency.

By extracting DB access into `ladder-rs-persistence`, the swarm operator links the crate directly. No HTTP roundtrip. No authentication overhead. Direct DB writes at library speed. This matches the product spec's intent: "The library owns all database interaction through a persistence layer. Both the backend service and swarm operator consume the library for DB access — there is no broker or separate write path."

### Server is intentionally thin

`ladder-rs-server` imports `ladder-rs-persistence` and calls repository functions. It does not import `sqlx` directly. It does not write SQL. The server crate is responsible for: HTTP routing, auth middleware, handler functions (authorization checks + repository calls), session management, and background task spawning. All data logic is in the persistence crate.

This constraint means the server can be tested against a test version of the persistence layer (or a test SQLite database) without needing to mock SQL. It also means the persistence layer can be used by future tools (CLI, admin scripts) without any server dependency.

### WASM stays isolated

`ladder-rs-wasm` depends only on `ladder-rs`. It never sees tokio, sqlx, or Axum. This is required for WASM compatibility. The workspace structure enforces this by keeping WASM as a separate crate — if someone accidentally adds a tokio dependency to `ladder-rs`, the WASM build will fail at compile time.

### Async-only persistence crate

The persistence crate is async-only (no synchronous wrapper API). Both known consumers (Axum server and swarm operator) bring their own tokio runtime. A sync wrapper would require an embedded runtime, which creates runtime-within-runtime problems when the caller already has a runtime.

Swarm operators who want a synchronous API can use `tokio::runtime::Runtime::block_on()` in their own code. This is a clear and well-understood pattern. Providing a sync wrapper in the persistence crate would add complexity and maintenance burden for a use case that can be solved by the caller.

---

## Alternatives Considered

### Monolithic single crate

Put everything in `ladder-rs`: rating math, DB access, HTTP server, WASM bindings.

**Rejected.** A monolithic crate cannot be simultaneously WASM-compatible (no tokio, no sqlx) and server-capable (tokio, Axum). Cargo feature flags could theoretically gate the dependencies, but:
- Feature flags that change the public API surface are fragile and hard to test
- The WASM target and the server target cannot share a crate build artifact — different compilation targets require separate builds regardless
- The swarm operator would depend on the entire server codebase just to get DB access

### Two crates: library + server

Merge `ladder-rs-persistence` into `ladder-rs-server`. The server owns all DB access; the swarm operator writes through HTTP.

**Rejected.** This forces swarm operators through HTTP, creating:
- Authentication overhead on every swarm match write (session lookup, role check)
- Network roundtrip latency for high-throughput agents
- An operational dependency (server must be running for swarm to write)
- A single write path bottleneck

The product spec explicitly calls this out: "Both the backend service and swarm operator consume the library for DB access — there is no broker or separate write path."

### Async + sync dual API in the persistence crate

Provide both `async fn` and synchronous wrappers in `ladder-rs-persistence`.

**Rejected.** Maintaining two API surfaces doubles the test burden. The sync API would embed a tokio runtime or use `futures::executor::block_on()`, which conflicts with callers that already have a runtime. The known consumers all use tokio, so there is no demand for a sync API at present.

---

## Consequences

### Positive

- `ladder-rs` remains pure and WASM-compilable without modification
- Swarm operators link `ladder-rs-persistence` directly for zero-HTTP-overhead DB writes
- `ladder-rs-server` is thin and testable without needing to mock SQL
- Cargo workspace enables shared `Cargo.lock` and consistent dependency versions across all crates
- `cargo build --workspace` builds everything; `cargo test --workspace` tests everything
- Each crate has a focused set of dependencies; unused crates incur no compilation overhead for the other crates' builds

### Negative / Accepted Trade-offs

- **Four-crate workspace increases navigation overhead.** Contributors must understand which crate owns which responsibility. The boundary rules (no SQL in server, no HTTP in persistence, no async in core library) must be enforced by convention and code review.
- **Swarm operators depend on `ladder-rs-persistence`.** This means changes to the persistence crate's public API are breaking changes for external swarm operator codebases. The persistence crate's API must be designed for stability. Semver discipline is required.
- **Workspace build time.** All four crates are compiled together. This is slower than compiling only what is needed, but the workspace `Cargo.lock` ensures consistent dependency resolution and is the standard Rust multi-crate project approach.
- **No automatic type sharing with the frontend.** TypeScript interfaces for API request/response types must be maintained manually. This is a consequence of the REST boundary between the server and the React frontend, not of the crate structure.
