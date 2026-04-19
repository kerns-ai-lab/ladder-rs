# Cross-Cutting Concerns

**ladder-rs Platform**
**Date:** 2026-04-15

---

## Overview

Cross-cutting concerns are architectural properties that affect every layer of the system rather than belonging to a single component. This document defines how error handling, observability, configuration, input sanitization, rate limiting, CORS, and database migrations are implemented uniformly across the platform.

---

## Error Handling

### Error Type Hierarchy

Errors propagate upward through three layers, each converting to the next:

```
ladder-rs (pure math)
    └── RatingError { NonConvergence, InvalidParticipants, ... }
            (treated as a result, not an error; non-convergence is flagged, not rejected)

ladder-rs-persistence
    └── PersistenceError {
            Database(sqlx::Error),
            NotFound { entity, id },
            DuplicateMatch,
            SeasonClosed { season_id },
            DuplicateUser { field },
            InvalidToken,
            TokenExpired,
            TokenAlreadyClaimed,
            AlgorithmMismatch,
            PlayerLocked,
        }

ladder-rs-server
    └── AppError {
            Persistence(PersistenceError),
            Unauthorized,
            Forbidden { reason },
            BadRequest { field, message },
            NotFound { resource },
            Conflict { reason },
            InternalError,
        }
        impl IntoResponse for AppError  // Axum trait
```

### HTTP Mapping

`AppError` implements Axum's `IntoResponse` trait. The mapping from error variants to HTTP status codes is:

| AppError Variant | HTTP Status | Structured Body |
|------------------|------------|-----------------|
| `Unauthorized` | 401 | `{"error": "UNAUTHORIZED", "message": "..."}` |
| `Forbidden` | 403 | `{"error": "FORBIDDEN", "message": "..."}` |
| `BadRequest` | 400 | `{"error": "VALIDATION_ERROR", "fields": [{"field": "...", "message": "..."}]}` |
| `NotFound` | 404 | `{"error": "NOT_FOUND", "resource": "..."}` |
| `Conflict` (DuplicateMatch) | 409 | `{"error": "DUPLICATE_MATCH"}` |
| `Conflict` (SeasonClosed) | 409 | `{"error": "SEASON_CLOSED", "season_id": 42}` |
| `Conflict` (DuplicateUser) | 409 | `{"error": "DUPLICATE_USER", "field": "email"}` |
| `InternalError` (Database error) | 500 | `{"error": "INTERNAL_ERROR"}` (no internal detail exposed) |

**Design rule:** Internal error details (SQL messages, stack traces) are never included in HTTP responses. They are logged at ERROR level server-side. The response body contains only a machine-readable error code and a human-readable message. This satisfies SR-API-002.

### Persistence Error Mapping

`PersistenceError` variants map to `AppError` variants in the server layer:

```rust
impl From<PersistenceError> for AppError {
    fn from(e: PersistenceError) -> Self {
        match e {
            PersistenceError::NotFound { entity, id } =>
                AppError::NotFound { resource: format!("{entity}/{id}") },
            PersistenceError::DuplicateMatch =>
                AppError::Conflict { reason: "duplicate_match".into() },
            PersistenceError::SeasonClosed { season_id } =>
                AppError::Conflict { reason: format!("season_closed:{season_id}") },
            PersistenceError::Database(db_err) => {
                tracing::error!("database error: {db_err}");
                AppError::InternalError
            }
            // ... other variants
        }
    }
}
```

### Non-Convergence Handling

TrueSkill's factor graph inference returns a `ConvergenceResult` that may indicate `BestApproximation` rather than full convergence. This is not an error — the result is used and stored with `convergence_quality = "degraded"`. The API response includes the `convergence_quality` field so the caller can take note. Match recording is never rejected for non-convergence.

---

## Observability

### Logging

**Crate:** `tracing` with `tracing-subscriber` for output formatting.

**Format:** Structured JSON in production (`tracing_subscriber::fmt::json()`). Human-readable in development (`tracing_subscriber::fmt::pretty()`). Controlled by the `RUST_LOG` environment variable.

**Log levels and what triggers them:**

| Level | Events |
|-------|--------|
| ERROR | Unrecoverable DB errors, recalculation job failures, startup failures |
| WARN | Auth failures (invalid sessions, failed logins, lockout triggers), constraint violations, non-convergence on TrueSkill calculations |
| INFO | Every HTTP request (method, path, status, duration), server startup, migration completion, admin bootstrap, job poller claims |
| DEBUG | Detailed SQL query parameters (development only; not emitted in production by default) |
| TRACE | Rating math intermediate values (development only) |

**Request logging:** Axum's `tower_http::trace::TraceLayer` provides per-request INFO log entries:
```
INFO ladder_rs_server::middleware: GET /api/seasons/42/leaderboard status=200 duration=47ms
INFO ladder_rs_server::middleware: POST /api/auth/login status=401 duration=3ms
```

**Sensitive data policy:** Passwords, session tokens, invite tokens, and SESSION_SECRET must never appear in log output. Logging code must not log request bodies for auth endpoints.

### Metrics

No metrics infrastructure (Prometheus, StatsD, etc.) in v1. Observability in v1 is log-based only. If throughput analysis is needed, operators parse structured JSON logs. Metrics infrastructure is a post-v1 operational concern.

### Health Check

`GET /health` returns `200 OK` with `{"status":"ok"}`. This is a pure liveness probe — it does not check database connectivity. The endpoint bypasses all auth middleware. Used by Docker health checks and any upstream load balancer.

---

## Configuration

### Runtime Configuration (Environment Variables)

All runtime configuration is supplied via environment variables. There is no configuration file at runtime. The complete set of variables is documented in `deployment.md`. Key variables:

| Variable | Effect |
|----------|--------|
| `DATABASE_URL` | SQLite file path |
| `SESSION_SECRET` | Session signing key (must be stable across restarts) |
| `SESSION_EXPIRY_SECONDS` | Cookie and DB session lifetime |
| `ADMIN_BOOTSTRAP_ENABLED` | Enable/disable first-run admin creation |
| `HTTPS_ENABLED` | `true` (default): sets `Secure` on session cookies. `false` (development): omits `Secure` so cookies work over HTTP localhost |
| `RUST_LOG` | Log level and filter |
| `BUSY_TIMEOUT_MS` | SQLite busy_timeout |
| `JOB_POLL_INTERVAL_SECS` | Background poller interval |

### Compile-Time Configuration (Cargo Features)

The `ladder-rs` crate uses Cargo features for algorithm and math backend selection:

| Feature | Effect |
|---------|--------|
| `elo-only` | Compile only Elo algorithm |
| `glicko-only` | Compile only Glicko-2 |
| `trueskill-only` | Compile only TrueSkill |
| `all-algorithms` (default) | All three algorithms |
| `libm-math` | Use libm for floating-point math (default, minimal deps) |
| `rayon` | Enable parallel rating computation for batch operations |

The server binary is built with `all-algorithms` (the default). The `ladder-rs-wasm` crate is built separately with wasm-compatible features.

### No Secrets in Code

All secrets (`SESSION_SECRET`, admin passwords, invite tokens) must be externally provided or generated at runtime. The codebase must contain no hardcoded credentials, tokens, or secrets. CI must validate this (e.g., via `gitleaks` or similar).

---

## Input Sanitization

**Requirement:** SR-API-002, NFR-SEC-003 require all user-supplied strings to be sanitized before persistence.

### SQL Injection Prevention

All database queries use sqlx's parameterized query API (`sqlx::query!` or `query_as!` macros). No string interpolation is used to build SQL. This is enforced by convention and code review.

### HTML/Script Injection Prevention

User-supplied text fields (player names, league names, descriptions) pass through a sanitization function before any persistence or display. The sanitizer:
1. Strips HTML tags (removes `<`, `>`, and tag content between them)
2. Escapes `&`, `"`, `'`, `<`, `>` as HTML entities for display contexts
3. Rejects strings exceeding defined maximum lengths (enforced at the handler level before sanitization)

**Implementation:** A dedicated `sanitize_string(input: &str) -> String` function in `ladder-rs-server`. Called in handlers before passing values to persistence. Not called in the persistence crate (the persistence crate is not responsible for sanitization — it trusts its callers).

**Field length limits (enforced at handler level):**

| Field | Max Length |
|-------|-----------|
| username | 64 characters |
| email | 254 characters (RFC 5321) |
| player name | 100 characters |
| league name | 200 characters |
| league description | 2000 characters |
| match reason (audit) | 500 characters |

### Algorithm Parameter Validation

Algorithm parameter JSON is validated against per-algorithm schemas at the handler level (SR-ALG-002). Parameters outside the min/max guardrail ranges are rejected with a 400 Bad Request before reaching the persistence layer.

---

## Login Rate Limiting

**Requirement:** NFR-SEC-001

### Mechanism

Rate limiting is implemented at the application layer using the `login_attempts` table, not in-memory or via an external service. This ensures lockout state survives server restarts.

**Algorithm:**
1. Every call to `POST /api/auth/login` records an attempt via `Auth Repository.record_attempt(user_id, success: bool)`.
2. On failure, `Auth Repository.consecutive_failures(user_id)` counts consecutive failed attempts since the most recent success using a subquery:
   ```sql
   SELECT COUNT(*) FROM login_attempts
   WHERE user_id = ?
     AND success = 0
     AND id > (SELECT COALESCE(MAX(id), 0) FROM login_attempts WHERE user_id = ? AND success = 1)
   ```
   Only an unbroken run of failures since the last successful login counts toward the threshold.
3. If count >= 10, the account is placed into a `locked` state with a `locked_until` timestamp (now + 15 minutes) stored in the `users` table.
4. While locked, every login attempt (including with correct credentials) returns `429 Too Many Requests` with an approximate remaining duration. The lockout lifts automatically when `locked_until` is in the past.
5. An Admin endpoint (`POST /admin/users/:id/unlock`) clears `locked_until` and resets the consecutive failure count.

**Timing safety:** The login handler always performs the full credential verification process, then records the attempt. This prevents timing-based username enumeration via attempt-recording timing differences.

**Lockout response:** Returns `429 Too Many Requests` with a generic message. The message does not reveal whether the username exists or whether it is specifically the rate limit vs. invalid credentials causing the rejection.

---

## CORS

### Production

In production, all traffic enters through the `frontend` container (nginx) on port 8080. The browser sees all requests (both for static assets and for API calls proxied to the backend) as originating from the same host (`http://your-host:8080`). From the browser's perspective, all requests are same-origin. **No CORS headers are required on the backend in production.**

### Development

In development, the React dev server runs on `:5173` and the Axum backend runs on `:3000`. These are different origins. CORS is required for the frontend dev server to call the backend directly.

The backend sets CORS headers when `CORS_ALLOW_ORIGIN` is set:
```
Access-Control-Allow-Origin: <CORS_ALLOW_ORIGIN>
Access-Control-Allow-Methods: GET, POST, PATCH, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Cookie
Access-Control-Allow-Credentials: true
```

**Implementation:** `tower_http::cors::CorsLayer` configured conditionally based on the presence of `CORS_ALLOW_ORIGIN`. If the variable is not set, no CORS layer is added (production behavior).

Note: The Vite dev server also provides a `/api` proxy in `vite.config.ts`, which eliminates the CORS requirement even in development. Both mechanisms are available; the Vite proxy is recommended for simplicity.

---

## Database Migrations

### Mechanism

`sqlx::migrate!` macro embeds migration SQL files from `ladder-rs-persistence/migrations/` into the binary at compile time. On startup, the migration runner compares the embedded migrations against the `_sqlx_migrations` table in the database and applies any that have not yet been run.

### File Naming Convention

```
migrations/
  0001_initial_schema.sql
  0002_add_login_attempts.sql
  0003_add_invite_tokens.sql
  ...
```

Files are prefixed with a zero-padded sequence number. The migration runner applies them in lexicographic order. Gaps in sequence numbers are not permitted — the runner validates that applied migration hashes match the embedded ones and fails if there is a mismatch.

### Migration Rules

- Migrations are forward-only. No `DOWN` migration files are provided in v1.
- Each migration file is content-hashed by sqlx. If a deployed migration file is modified after deployment, the server will fail to start on the next deployment (hash mismatch detected). This enforces immutability of applied migrations.
- New columns or tables are added in new migration files, never by editing existing ones.
- All SQL in migration files follows the same portability rules as production queries (standard SQL, compatible with both SQLite and PostgreSQL per NFR-PORT-001).

### Startup Failure

If migration fails (syntax error, constraint violation, hash mismatch), the server logs the error at ERROR level and exits with a non-zero status code. Docker Compose will restart the container per its restart policy. The operator must resolve the migration issue before the server starts successfully.

---

## Session Management

**Library:** `tower-sessions` + `tower-sessions-sqlx-store`

Sessions are stored in the `sessions` SQLite table. `tower-sessions-sqlx-store` manages the session store implementation. `ladder-rs-server` configures the session layer with:

```rust
let https_enabled: bool = std::env::var("HTTPS_ENABLED")
    .unwrap_or_else(|_| "true".into())
    .parse()
    .unwrap_or(true);

let session_store = SqliteStore::new(pool.clone());
let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(https_enabled)  // Secure flag: true in production, false for HTTP dev
    .with_http_only(true)        // HttpOnly flag always set (no JS access)
    .with_same_site(SameSite::Strict)  // SameSite always set
    .with_expiry(Expiry::OnInactivity(Duration::seconds(session_expiry)));
```

**`HTTPS_ENABLED` behavior:** When `false`, the `Secure` attribute is omitted from the `Set-Cookie` header, allowing session cookies to be transmitted over plain HTTP. This is the development configuration. `HttpOnly` and `SameSite=Strict` are always set regardless of `HTTPS_ENABLED`.

**Session expiry:** Sessions expire after `SESSION_EXPIRY_SECONDS` of inactivity (configurable; default 24 hours). tower-sessions handles expiry enforcement and cleanup of expired rows.

**Session content:** The session stores the authenticated `user_id`. The `AuthContext` (including role and league assignments) is loaded from the DB on each request using the `user_id` from the session. Role and assignments are not cached in the session to ensure changes (e.g., revoking operator status) take effect immediately.

---

## Requirements Traceability

| Requirement | Cross-Cutting Element |
|-------------|----------------------|
| SR-API-002 | Error handling (structured JSON responses, HTTP mapping) |
| SR-AUTH-001 | Session management (tower-sessions configuration) |
| NFR-PERF-001 | No cross-cutting overhead on rating math path |
| NFR-REL-001 | Migrations (forward-only, hash-verified); WAL mode (in deployment) |
| NFR-PORT-001 | Migration SQL portability rules |
| NFR-SEC-001 | Login rate limiting (login_attempts table, 10 failures / 15 min) |
| NFR-SEC-002 | Session cookie attributes (HttpOnly, Secure, SameSite=Strict) |
| NFR-SEC-003 | Input sanitization (sanitize_string, field length limits, parameterized SQL) |
