# ADR-0009: Library API Keys for Swarm Operator Authorization

**Status:** Accepted
**Date:** 2026-04-17
**Deciders:** Dustin Kerns

---

## Context

The `ladder-rs-persistence` crate is consumed by two types of processes:

1. **`ladder-rs-server`** — the Axum backend. All writes go through HTTP handlers that enforce RBAC via auth middleware. A League Operator can only write to leagues they are assigned to.

2. **Swarm Operator Process** — an external Rust process that links `ladder-rs-persistence` directly and writes match results to the SQLite database without going through HTTP.

The direct-library path bypasses the HTTP auth middleware entirely. Without additional controls, any process with a valid `DATABASE_URL` and the `ladder-rs-persistence` crate can write match records to any league or season — regardless of which leagues the operator is supposed to manage.

For a single operator running a single swarm, this is fine by convention. For a deployment with multiple swarm operators managing different leagues, one operator could (accidentally or intentionally) write data to another operator's league.

---

## Decision

`ladder-rs-persistence` introduces a **`SwarmContext`** type required by all write functions:

```rust
pub struct SwarmContext {
    pub user_id: i64,
}
```

Swarm operators initialize the persistence crate with an **API key** (a cryptographically random token stored as a hash in a new `api_keys` table). The crate's initialization function validates the key against the database and constructs a `SwarmContext` containing the associated `user_id`.

All write functions on the persistence API that create or modify league-scoped data accept a `&SwarmContext` parameter:

```rust
async fn record_match(
    pool: &SqlitePool,
    season_id: i64,
    participants: Vec<MatchParticipant>,
    score_metadata: Option<Json>,
    ctx: &SwarmContext,
) -> Result<MatchResult>

async fn get_or_create_player(
    pool: &SqlitePool,
    name: &str,
    player_type: PlayerType,
    league_id: i64,
    ctx: &SwarmContext,
) -> Result<(Player, bool)>
```

The repository checks `is_operator(league_id, ctx.user_id)` before executing the write. If the swarm context user is not assigned to the target league, the call returns `PersistenceError::Unauthorized`.

Read-only functions (`get_league`, `list_seasons`, `get_leaderboard`, etc.) do not require a `SwarmContext` — they apply the same visibility filtering as the HTTP layer based on league `visibility` column.

### API Key lifecycle

- **Creation:** Admin calls `POST /api/admin/api-keys` with a description and a `user_id` (which must have operator role). The server generates a 32-byte random key, stores its SHA-256 hash in `api_keys`, and returns the plaintext key exactly once.
- **Revocation:** Admin calls `DELETE /api/admin/api-keys/:id`. The row is deleted; any swarm process using that key will fail to initialize on its next startup.
- **No expiry in v1:** API keys do not expire automatically. Revocation is explicit.

### `api_keys` table

```sql
CREATE TABLE api_keys (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash    TEXT NOT NULL UNIQUE,   -- SHA-256 of the plaintext key
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    created_by  INTEGER NOT NULL REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

### `ladder-rs-server` is unaffected

The Axum backend does not use `SwarmContext`. It passes authorization through the `AuthContext` extracted by the HTTP middleware. The `SwarmContext` parameter is only present on the public persistence API; the server crate does not construct one.

---

## Rationale

### The trust boundary is the application, not just the filesystem

A multi-operator deployment where different swarm operators manage different leagues requires per-operator write isolation. Relying on filesystem access control (who has the `DATABASE_URL`) is insufficient when multiple trusted parties share the same host — each should only be able to write to their own leagues.

### Minimal surface change

Adding `SwarmContext` to write function signatures is a small, localized change. The context is constructed once at process startup from the API key. All downstream write calls thread it through without significant boilerplate.

### The HTTP path's RBAC and the library path's RBAC use the same underlying data

Both paths call `is_operator(league_id, user_id)` in the League Repository. The RBAC rules are not duplicated — both paths share the same enforcement function with the same data. This ensures that revoking a user's operator assignment immediately affects both the REST API and any live library consumers on their next write.

---

## Alternatives Considered

### Trust by convention (no API keys)

Rely on filesystem-level access control. Only give the SQLite file path to trusted processes.

**Rejected** because it provides no enforcement at the application layer, does not support multi-operator deployments, and produces no audit trail for library-path writes.

### Per-call authentication token (stateless)

Pass a signed JWT or HMAC token on every write call rather than at initialization.

**Rejected** as over-engineering for v1. The library is a long-lived process, not a stateless HTTP client. Initializing once with an API key and holding the resulting `SwarmContext` is simpler and has equivalent security properties for this use case.

---

## Consequences

### Positive

- Multi-operator deployments are supported: each swarm operator has their own API key scoped to their leagues
- Revocation is immediate: deleting an API key prevents new initializations; the running process fails on the next startup
- Both write paths (HTTP and library) enforce RBAC using the same underlying `is_operator` data
- API key usage can be logged (key ID included in match audit metadata)

### Negative / Accepted Trade-offs

- The `ladder-rs-persistence` public API now includes a `SwarmContext` parameter on write functions. External consumers (swarm operators) must update their integration to construct this context at startup.
- Admin must issue API keys as part of swarm operator onboarding — a new operational step.
- `api_keys` table adds a new schema migration.
- Running processes are not immediately invalidated on key revocation — they continue until they restart. For immediate revocation, the operator must also terminate the swarm process.
