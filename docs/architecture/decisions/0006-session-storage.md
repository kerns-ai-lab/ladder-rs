# ADR-0006: Session Storage via SQLite-Backed tower-sessions

**Status:** Accepted
**Date:** 2026-04-15
**Deciders:** Dustin Kerns

---

## Context

The ladder-rs platform requires session-based authentication (SR-AUTH-001). Users log in with credentials and receive a session that is validated on every subsequent request. The session must carry a user identity that the auth middleware can use to enforce RBAC (SR-AUTH-002).

Session storage requirements:
- Sessions must survive server restarts (crash recovery; NFR-REL-001)
- Sessions must be issued as HttpOnly/Secure/SameSite=Strict cookies (NFR-SEC-002)
- The platform uses Axum as the server framework (ADR-0003) and SQLite as the only persistence layer
- Session expiry must be configurable (SR-AUTH-001 acceptance criterion)
- No external infrastructure may be added for v1

The chosen HTTP framework (Axum) is built on the tower ecosystem. `tower-sessions` is the tower-native session management crate. `tower-sessions-sqlx-store` provides a sqlx-backed session store that works with SQLite.

---

## Decision

Use **`tower-sessions` with `tower-sessions-sqlx-store` backed by SQLite** for session management.

Sessions are stored in the `sessions` table in the same SQLite database used for all other platform data. Session cookies are configured as HttpOnly, Secure, and SameSite=Strict.

---

## Rationale

### Crash recovery without additional infrastructure

In-memory session stores (HashMaps, DashMap) lose all sessions when the server restarts. Every user is logged out on every deployment or crash. This violates the crash recovery spirit of NFR-REL-001 and creates a poor operator experience (admin must re-authenticate after every server update).

SQLite-backed sessions survive restarts. Sessions persist in the same database that holds all other platform data. A server restart does not invalidate active sessions; users remain logged in.

### SQLite-only infrastructure constraint

The platform is explicitly constrained to SQLite as the only persistence layer for v1. Redis is the canonical in-production session store for web applications, but adding Redis would require a third Docker container, a Redis configuration, operational credentials, and availability coupling between the server and the Redis instance. SQLite provides session persistence with zero additional infrastructure.

### tower ecosystem alignment

`tower-sessions` is a first-class tower middleware crate designed for Axum integration. `tower-sessions-sqlx-store` implements the `SessionStore` trait for sqlx connection pools. The integration is a few lines of configuration:

```rust
let session_store = SqliteStore::new(pool.clone());
let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(true)
    .with_http_only(true)
    .with_same_site(SameSite::Strict)
    .with_expiry(Expiry::OnInactivity(Duration::seconds(expiry)));
```

This is correct by construction for NFR-SEC-002. The cookie attributes are set at the library level, not as manual header manipulation. There is no risk of forgetting the `HttpOnly` attribute on a specific endpoint.

### One DB read per request

Session validation on every authenticated request requires one DB read (SELECT from the `sessions` table by session ID). At the expected scale (single-tenant, hundreds of concurrent users), this is acceptable. The session ID is the primary key; the lookup is a B-tree point read. The `sessions` table has an index on `expires_at` for background expiry cleanup.

If performance profiling reveals session lookup as a bottleneck (unexpected at this scale), a short-lived in-memory LRU cache keyed by session ID could be layered in front of the DB read. This optimization is deferred to post-v1.

### Cookie security attributes

`tower-sessions` sets HttpOnly, Secure, and SameSite=Strict by configuration, satisfying NFR-SEC-002. The session token value is not included in the API response body — it is only in the Set-Cookie header. This prevents JavaScript access to the token and CSRF exploitation.

Note: `Secure` requires HTTPS. In development (HTTP), the `Secure` flag should be disabled via `CORS_ALLOW_ORIGIN` / dev mode configuration. In production (behind nginx on port 8080), HTTPS is the operator's responsibility to configure at the nginx level or upstream.

---

## Alternatives Considered

### In-memory session store (HashMap / DashMap)

Store sessions in a `HashMap<SessionId, SessionData>` in the server process memory.

**Rejected.** Sessions are lost on every server restart or crash. Every deployment logs out all users. The admin must re-authenticate after every docker compose restart. This is unacceptable for an operational tool. NFR-REL-001 requires crash recovery; in-memory sessions are the opposite of crash-recoverable.

### Redis-backed sessions

Use Redis as the session store. Redis is the industry standard for web session storage.

**Rejected.** Adding Redis adds a third Docker container to the Compose topology, a Redis connection pool, Redis connection failure handling, and operational credentials management. The platform's constraint is SQLite-only infrastructure for v1. Redis provides session persistence + high performance, but at the cost of operational complexity not justified for single-tenant deployment at this scale.

Post-v1, if the platform scales to multi-instance deployment where session affinity becomes a concern, Redis would be the appropriate migration target.

### JWT-based stateless sessions

Issue signed JWTs as session tokens. No server-side session storage needed — the token is self-validating.

**Rejected.** JWTs cannot be invalidated server-side without a token revocation list (which reintroduces server-side storage). If a session needs to be revoked (logout, admin force-logout, security incident), a JWT-based system must maintain a blocklist. The blocklist has the same storage requirements as a session store, without the simplicity benefit.

JWTs are also larger than opaque session IDs (base64-encoded claims vs. a random 32-byte ID), consuming more cookie bandwidth on every request.

The logout requirement (SR-AUTH-001: "the system exposes a logout endpoint that invalidates the current session token") requires server-side invalidation. JWTs do not naturally support this.

### Cookie-only sessions (signed cookies, no server storage)

Store the entire session payload in the cookie, signed with `SESSION_SECRET`.

**Rejected.** Cookie-stored sessions cannot be revoked without rotating the signing secret (which invalidates all sessions simultaneously). Admin force-logout, session expiry enforcement, and the forced-password-change flag (SR-AUTH-001) all require per-session invalidation capability. Server-side session storage is required.

---

## Consequences

### Positive

- Sessions survive server restarts and crashes (users remain logged in after deployments)
- No additional infrastructure (SQLite-only, consistent with platform constraints)
- Cookie security attributes (HttpOnly, Secure, SameSite=Strict) set correctly by library configuration
- Admin-initiated force-logout is possible by deleting a session row from the `sessions` table
- Configurable session expiry via `SESSION_EXPIRY_SECONDS` environment variable

### Negative / Accepted Trade-offs

- **One DB read per authenticated request.** Every API call performs a session lookup against the `sessions` table. At hundreds of concurrent users, this is a point-key B-tree lookup and adds negligible latency. At thousands of concurrent users, this could become a consideration — but the platform is single-tenant with no requirement for that scale in v1.
- **Sessions table grows over time.** Expired sessions are cleaned up by the tower-sessions library's background sweep (indexed on `expires_at`). Without periodic cleanup, the table could grow unboundedly. The cleanup is automatic via the library, but operators should be aware that the SQLite file will grow with session history until cleanup runs.
- **`Secure` flag and HTTPS.** The `Secure` flag prevents the session cookie from being sent over HTTP. Production deployments must terminate TLS at nginx or an upstream reverse proxy. If operators deploy without TLS, the `Secure` flag must be disabled, weakening cookie security. Operators are warned in deployment documentation.
- **tower-sessions version coupling.** `tower-sessions` and `tower-sessions-sqlx-store` must stay in sync with each other and with the Axum version. Major version bumps in any of these crates may require coordinated updates.
