# ADR-0003: Server Framework Selection

**Status:** Accepted
**Date:** 2026-04-15
**Deciders:** Dustin Kerns

---

## Context

The ladder-rs platform requires an HTTP server layer that:
- Exposes a REST API consumed by the React frontend
- Enforces authentication via session cookies
- Enforces role-based access control per request
- Manages sessions backed by SQLite
- Applies login rate limiting
- Spawns and manages a long-lived background job poller (tokio task)
- Is thin: it wraps `ladder-rs-persistence` rather than containing business logic

The server must be async-native (the persistence layer uses tokio). The primary Rust async HTTP server candidates are Axum and Actix-web.

---

## Decision

Use **Axum** as the HTTP server framework.

---

## Rationale

### tower ecosystem alignment

Axum is built on the `tower` ecosystem (`tower::Service`, `tower::Layer`). The platform's middleware requirements map directly to tower components:

- **Session management:** `tower-sessions` + `tower-sessions-sqlx-store` — a first-class tower middleware layer for session management backed by sqlx stores. This is the primary integration point for HttpOnly/Secure/SameSite=Strict cookie sessions (NFR-SEC-002). These crates are maintained alongside Axum and are designed to compose with Axum's middleware stack.
- **Rate limiting:** `tower-governor` or a custom tower layer for login rate limiting (NFR-SEC-001). Tower middleware composes cleanly into Axum's layer stack.
- **Request tracing:** `tower-http::trace::TraceLayer` provides per-request structured logging out of the box.
- **CORS:** `tower-http::cors::CorsLayer` for development CORS configuration.

Using Actix-web would require bridging to the tower ecosystem or finding Actix-specific equivalents for each of these crates. The `tower-sessions-sqlx-store` crate in particular does not have an Actix-web equivalent — we would need to implement session storage from scratch.

### tokio-native design

Axum is built on top of hyper and uses tokio as its runtime. The `ladder-rs-persistence` crate is tokio-native (all async functions assume a tokio runtime). There is no impedance mismatch. Axum's `State`, `Extension`, and extractor system integrate cleanly with tokio's task model for the background job poller.

### Extractor ergonomics

Axum's extractor system (`State<T>`, `Path<T>`, `Query<T>`, `Json<T>`, `Extension<T>`) enables clean, testable handler functions with explicit dependencies:

```rust
async fn record_match(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthContext>,
    Path(season_id): Path<i64>,
    Json(payload): Json<MatchRequest>,
) -> Result<impl IntoResponse, AppError> { ... }
```

Each handler declares exactly what it needs. Testing is straightforward — inject test values for each extractor. Middleware that injects `AuthContext` into request extensions composes cleanly with this pattern.

### IntoResponse for structured errors

Axum's `IntoResponse` trait enables the `AppError` type to convert directly to HTTP responses with structured JSON bodies (SR-API-002). This is clean and type-safe:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, json!({...})),
            // ...
        };
        (status, Json(body)).into_response()
    }
}
```

This requires less boilerplate than Actix-web's `ResponseError` trait and integrates more naturally with the `?` operator in handler functions.

---

## Alternatives Considered

### Actix-web

**Rejected** for this project, for the following reasons:

**Actor model overhead is unnecessary here.** The original Actix framework was built on an actor model. While Actix-web 4.x has moved away from the actor requirement for handlers, the legacy actor overhead and design patterns still surface in parts of the ecosystem. The platform does not need actors — it needs a thin HTTP wrapper over a library.

**Weaker tower ecosystem alignment.** `tower-sessions` and `tower-sessions-sqlx-store` do not have Actix-web equivalents. Session management would require implementing a custom session store backed by SQLite, adding complexity and maintenance burden.

**Thread-safety constraints.** Actix-web historically required `Send + Sync` bounds in ways that were more restrictive than Axum's tokio-native model. While this has improved in recent versions, Axum's model aligns more naturally with the persistence crate's async API.

**Ecosystem considerations.** `tower-http` utilities (TraceLayer, CorsLayer) do not compose with Actix-web middleware. Equivalent utilities exist but come from different crates with different APIs.

Actix-web is a capable framework and would produce a working server. The decision comes down to middleware ecosystem fit: tower's ecosystem (tower-sessions, tower-http) is the right match for this project's specific requirements.

### Other Frameworks (Warp, Tide, Rocket)

Not seriously considered. Warp's composition model has ergonomic issues at scale. Tide is less actively maintained. Rocket requires a nightly Rust compiler (or significant configuration overhead in stable Rust). None of these have the tower ecosystem alignment that is the primary driver of the Axum choice.

---

## Consequences

### Positive

- `tower-sessions-sqlx-store` provides the SQLite-backed session store without custom implementation
- `tower-http` utilities (TraceLayer, CorsLayer, CompressionLayer) work natively
- Handler functions are clean, testable, and composable via extractors
- `IntoResponse` makes structured error responses natural
- tokio alignment throughout: no runtime bridging with the persistence crate

### Negative / Accepted Trade-offs

- **Axum version lock.** Axum's API has changed significantly between versions (0.6, 0.7). Upgrading Axum may require migrating middleware and extractor usage. The `tower-sessions` ecosystem must be kept in sync with the Axum version.
- **Tower middleware learning curve.** tower's service/layer abstraction is powerful but has a non-trivial learning curve compared to simpler middleware models. New contributors must understand `Layer`, `Service`, and how to compose them.
- **Less built-in than some frameworks.** Axum is deliberately minimal. Features like request body size limits, multipart form handling, and file upload are provided by tower-http extensions or community crates, not Axum core. For this project's use cases (JSON REST API, no file uploads), this is not a limitation.
