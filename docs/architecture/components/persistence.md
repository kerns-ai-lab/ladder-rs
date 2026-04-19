# Component View — Persistence Crate

**ladder-rs-persistence**
**Date:** 2026-04-15

---

## Overview

`ladder-rs-persistence` is a Rust library crate — not an independently deployable container. It is linked into two distinct consumers:

1. **ladder-rs-server** — the Axum backend binary. The server calls persistence functions on every API request that touches data.
2. **Swarm Operator Process** — an external Rust process (not owned by this platform) that embeds the crate to write match data directly without going through HTTP.

Because two processes may use this crate concurrently against the same SQLite file, all public APIs in this crate are async-native (tokio). Synchronous wrappers are not provided — consumers must bring their own tokio runtime (both the server and the swarm operator are expected to run tokio).

This C4 Level 3 view describes the logical components inside `ladder-rs-persistence`.

---

## Component Map

```
[ladder-rs-persistence crate]
│
├── [Connection Pool] ─────────── sqlx SqlitePool, WAL + busy_timeout config
├── [Schema Migrations] ────────── sqlx migrate!, versioned migration files
│
├── [SwarmContext] ─────────────── authorization context for library write path (ADR-0009)
├── [League Repository] ────────── league CRUD, visibility, archive
├── [Season Repository] ────────── season create/close/seed, algorithm management
├── [Player Repository] ────────── global player CRUD, membership, search, soft-delete
├── [Match Repository] ─────────── atomic match+participant+snapshot insert, dupe check
├── [Rating Engine Bridge] ──────── calls ladder-rs math, produces RatingSnapshot values
├── [Alias Repository] ─────────── player alias link/unlink, triggers job insert
├── [Auth Repository] ──────────── users, sessions, roles, login attempts, invite tokens, api_keys
├── [Job Repository] ───────────── recalculation_jobs CRUD, atomic claim, dedup, startup recovery
└── [ApiKey Repository] ────────── api_keys CRUD, key validation, SwarmContext construction
```

---

## Components

### SwarmContext

**Technology:** Plain Rust struct

**Responsibility:** The authorization context for the library write path. Swarm operators initialize their persistence connection with an API key; the crate validates the key and constructs a `SwarmContext` containing the resolved `user_id`. All write functions that touch league-scoped data require `&SwarmContext` and check that the context's `user_id` is assigned to the target league via `League Repository.is_operator()`.

```rust
pub struct SwarmContext {
    pub user_id: i64,
}

// Constructed at startup by the ApiKey Repository:
pub async fn init_swarm_context(pool: &SqlitePool, api_key: &str) -> Result<SwarmContext>
```

Read-only functions do not require `SwarmContext`. The `ladder-rs-server` binary never constructs a `SwarmContext` — it uses `AuthContext` from the HTTP middleware instead.

**Satisfies:** ADR-0009, UR-SW-001 (multi-operator isolation)

---

### Connection Pool

**Technology:** `sqlx::SqlitePool`

**Responsibility:** Manages the pool of SQLite connections shared across all repository operations. Configures WAL mode and `busy_timeout` on pool initialization.

**Configuration applied at pool creation:**

```sql
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;   -- 5 seconds, configurable via env
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;  -- WAL mode makes this safe
```

**Pool sizing:** The pool size is configurable. Default: `max_connections = 5`. Because SQLite allows one writer at a time, the pool primarily benefits concurrent readers. All write operations serialize at the SQLite level via WAL write lock.

**Ownership:** The `SqlitePool` is the single authoritative pool for the consuming process. The server creates one pool on startup, holds it in Axum `State`, and passes it by reference to all repository calls. The swarm operator creates its own pool in its own process.

**Satisfies:** NFR-REL-001 (WAL ACID), NFR-PERF-002 (pool enables concurrent read throughput)

---

### Schema Migrations

**Technology:** `sqlx::migrate!`, SQL migration files in `migrations/`

**Responsibility:** Maintains the authoritative database schema. Migrations are numbered SQL files embedded into the binary at compile time via `sqlx::migrate!`. They run automatically when the connection pool is first created (or on an explicit `migrate` call at server startup).

**Migration file naming convention:** `{NNN}_{description}.sql` — e.g., `0001_initial_schema.sql`, `0002_add_login_attempts.sql`.

**Migration guarantees:**
- Each migration runs exactly once (tracked in `_sqlx_migrations` table)
- Migrations are applied in order; out-of-order application is rejected
- Down migrations are not provided in v1 — rollback requires a database restore

**Startup integration:** `ladder-rs-server` calls `pool.migrate()` (or the migration runner) before starting the Axum router. If migrations fail, the server exits with a non-zero status.

**Satisfies:** NFR-REL-001, NFR-PORT-001 (migration files are standard SQL, portable to PostgreSQL)

---

### League Repository

**Technology:** sqlx query functions

**Responsibility:** All CRUD operations on the `leagues` table and the `league_operators` join table.

**Public API surface:**

```rust
async fn create_league(pool, name, description, algorithm, visibility, created_by) -> Result<League>
async fn get_league(pool, id) -> Result<Option<League>>
async fn list_leagues(pool, filter: LeagueFilter, viewer: &AuthContext) -> Result<Vec<League>>
async fn update_league(pool, id, patch: LeaguePatch) -> Result<League>
async fn archive_league(pool, id) -> Result<()>
async fn unarchive_league(pool, id) -> Result<()>
async fn assign_operator(pool, league_id, user_id, granted_by) -> Result<()>
async fn remove_operator(pool, league_id, user_id) -> Result<()>
async fn get_operators(pool, league_id) -> Result<Vec<LeagueOperator>>
async fn is_operator(pool, league_id, user_id) -> Result<bool>
```

**Visibility filtering:** `list_leagues` accepts an `AuthContext` and applies SR-AUTH-006 logic in the SQL WHERE clause:
- Public leagues: always included for authenticated users
- Private leagues: included only if viewer is Admin, an assigned operator, or has a `league_players` record for that league

**Satisfies:** UR-LM-001, SR-AUTH-003, SR-AUTH-006, SR-API-003

---

### Season Repository

**Technology:** sqlx query functions

**Responsibility:** Season lifecycle management: creating new seasons, closing seasons, updating algorithm parameters, and applying seeding choices at season transition.

**Public API surface:**

```rust
async fn create_season(pool, league_id, algorithm, params: AlgorithmParams, seeding_choice) -> Result<Season>
async fn get_season(pool, id) -> Result<Option<Season>>
async fn get_current_season(pool, league_id) -> Result<Option<Season>>
async fn list_seasons(pool, league_id) -> Result<Vec<Season>>
async fn close_season(pool, id) -> Result<()>
async fn update_season_params(pool, id, params: AlgorithmParams) -> Result<Season>
async fn apply_seeding(pool, from_season_id, to_season_id, choice: SeedingChoice) -> Result<()>
```

**Seeding implementation detail:** `apply_seeding` with `SeedingChoice::Ordinal` reads the final `rating_snapshots` for all players in the prior season, sorts by `conservative_rating`, and inserts initial `rating_snapshots` for the new season that map ordinal rank to the new algorithm's default-distribution range. `SeedingChoice::Reset` inserts all players at the algorithm defaults.

**Algorithm-type-change guard:** The Season Repository is not responsible for deciding when to create a new season vs. update in place — that is SR-ALG-003 logic enforced in the Season Handlers. The repository executes what it is told.

**Satisfies:** UR-LM-002, SR-ALG-003, SR-ALG-004, SR-PER-005

---

### Player Repository

**Technology:** sqlx query functions

**Responsibility:** Global player CRUD (the `players` table), league membership management (`league_players`), name-prefix search, and soft-delete.

**Public API surface:**

```rust
async fn create_player(pool, name, player_type) -> Result<Player>
async fn get_player(pool, id) -> Result<Option<Player>>
async fn get_or_create_player(pool, name, player_type) -> Result<(Player, bool)>  // for swarm auto-creation
async fn list_players(pool, league_id, filter: PlayerFilter) -> Result<Vec<Player>>
async fn update_player(pool, id, patch: PlayerPatch) -> Result<Player>
async fn soft_delete_from_league(pool, league_id, player_id) -> Result<()>
async fn add_to_league(pool, league_id, player_id) -> Result<()>
async fn search_by_prefix(pool, q: &str, limit: usize) -> Result<Vec<Player>>
async fn link_account(pool, player_id, user_id, created_by) -> Result<()>
```

**Auto-creation (`get_or_create_player`):** Implements SR-PER-006. On the swarm operator path, a player name reference in a match that has no existing player record results in a new player being created with `player_type = "non-human"`. The boolean return value indicates whether the player was created (true) or already existed (false). Uses INSERT OR IGNORE + SELECT to be race-safe. Because `players.name` is globally unique (case-insensitive), the lookup is unambiguous — there can never be two players with the same name to confuse the result.

**Nickname:** `players.nickname` is an optional display name with no uniqueness constraint. The Player Repository exposes it in all `Player` return types. Display logic (`nickname ?? name`) is the responsibility of the server and frontend layers.

**Soft-delete semantics:** `soft_delete_from_league` sets `is_active = false` on the `league_players` join record (not on the global `players` table). The player's historical data and rating snapshots are unaffected. The player is excluded from leaderboard queries and cannot participate in new matches for that league.

**Search:** `search_by_prefix` issues `SELECT ... WHERE name LIKE ? || '%'`. The `players.name` column has a B-tree index for prefix-scan efficiency. Results are limited to `limit` entries (default 20). Case-insensitive via SQLite's default case folding.

**Satisfies:** UR-PM-001, SR-PER-003, SR-PER-006, SR-API-004

---

### Match Repository

**Technology:** sqlx query functions, SQLite transactions

**Responsibility:** The most complex repository. Records a complete match atomically: the match header, all participants, the rating computation call, and the resulting rating snapshots. Also provides duplicate detection and season write protection.

**Public API surface:**

```rust
async fn record_match(pool, season_id, participants: Vec<MatchParticipant>, score_metadata: Option<Json>) -> Result<MatchResult>
async fn record_match_batch(pool, season_id, entries: Vec<BatchEntry>) -> Result<Vec<BatchEntryResult>>
async fn get_match(pool, id) -> Result<Option<Match>>
async fn list_matches(pool, season_id, filter: MatchFilter) -> Result<Vec<Match>>
async fn correct_match(pool, match_id, correction: MatchCorrection, changed_by: UserId) -> Result<()>
async fn is_duplicate(pool, season_id, participants: &[MatchParticipant], recorded_at: DateTime) -> Result<bool>
async fn is_season_closed(pool, season_id) -> Result<bool>
```

**Atomic match recording transaction (record_match):**

```
BEGIN TRANSACTION
  1. Check is_season_closed(season_id) — reject if closed (SR-PER-005)
  2. Check is_duplicate(...) — reject if duplicate (SR-PER-004)
  3. INSERT INTO matches (...) RETURNING id
  4. INSERT INTO match_participants (...) for each participant
  5. Call Rating Engine Bridge → compute new RatingSnapshot values
  6. INSERT INTO rating_snapshots (...) for each participant
COMMIT
```

If any step fails, the transaction rolls back. No partial match state is committed to the database.

**Duplicate detection criteria (SR-PER-004):** A match is a duplicate if there exists a match in the same season with the same set of player_ids, the same placements, the same `is_draw` flags, and the same `recorded_at` timestamp (to the nearest second). The check uses a hash of the canonical participant representation.

**Admin correction:** `correct_match` updates the match record, inserts an audit log entry, but does NOT recompute ratings. Rating recomputation is handled by the Recalculation Worker after a job is queued via the Job Repository.

**Satisfies:** UR-ME-001, UR-ME-002, SR-PER-002, SR-PER-004, SR-PER-005, SR-PER-008, SR-ADM-001

---

### Rating Engine Bridge

**Technology:** Synchronous calls into `ladder-rs` rating functions, wrapped in `spawn_blocking` if needed

**Responsibility:** The seam between the persistence crate and the pure rating math crate. Takes a set of current player ratings and a match outcome, calls the appropriate `ladder-rs` algorithm, and returns the new ratings as `RatingSnapshot` values ready to insert into the DB.

**Convergence quality:** TrueSkill factor graph inference may return a "best approximation" result when the iteration does not fully converge. The bridge inspects the `ladder-rs` result type and sets `convergence_quality = "degraded"` on the match record when this occurs. The match is never rejected for non-convergence.

**Per-algorithm output:**

| Algorithm | Rating | Deviation | Uncertainty | Conservative Rating |
|-----------|--------|-----------|-------------|---------------------|
| Elo | rating | NULL | NULL | rating |
| Glicko-2 | mu | RD | NULL | mu - 2*RD |
| TrueSkill | mu | NULL | sigma | mu - 3*sigma |

The `conservative_rating` column is pre-computed here and stored in `rating_snapshots` to avoid per-query arithmetic in leaderboard queries.

**Satisfies:** SR-ALG-005, NFR-PERF-001

---

### Alias Repository

**Technology:** sqlx query functions

**Responsibility:** Manages the `player_aliases` table. Creating or removing an alias immediately inserts a `recalculation_jobs` record for all seasons in which either player has match history.

**Public API surface:**

```rust
async fn create_alias(pool, primary_player_id, alias_player_id, created_by) -> Result<Vec<JobId>>
async fn remove_alias(pool, primary_player_id, alias_player_id) -> Result<Vec<JobId>>
async fn get_aliases(pool, player_id) -> Result<Vec<PlayerAlias>>
async fn resolve_alias_group(pool, player_id) -> Result<Vec<PlayerId>>
```

**Job insertion:** Both `create_alias` and `remove_alias` query `SELECT DISTINCT season_id FROM matches JOIN match_participants ON ... WHERE player_id IN (primary_id, alias_id)` and insert one `recalculation_jobs` row per affected season. Returns the job IDs so the caller can include them in the API response.

**Recalculation usage:** The Recalculation Worker calls `resolve_alias_group` to get the full set of player IDs that should be treated as one player for rating purposes in a given season.

**Satisfies:** UR-PM-002, SR-PER-007, SR-PER-009

---

### Auth Repository

**Technology:** sqlx query functions, `argon2` for password hashing

**Responsibility:** All authentication and authorization data: users, sessions (managed by tower-sessions-sqlx-store, but read here for admin operations), roles, league-operator assignments, login attempt tracking, lockouts, and invite tokens.

**Public API surface:**

```rust
// Users
async fn create_user(pool, username, email, password_hash, role) -> Result<User>
async fn get_user_by_username(pool, username) -> Result<Option<User>>
async fn get_user_by_email(pool, email) -> Result<Option<User>>
async fn get_user(pool, id) -> Result<Option<User>>
async fn set_password(pool, user_id, password_hash, force_change: bool) -> Result<()>
async fn clear_force_change(pool, user_id) -> Result<()>
async fn is_users_table_empty(pool) -> Result<bool>

// Roles
async fn get_user_role(pool, user_id) -> Result<Role>
async fn get_league_assignments(pool, user_id) -> Result<Vec<LeagueId>>

// Login rate limiting
async fn record_attempt(pool, user_id, success: bool) -> Result<()>
async fn consecutive_failures(pool, user_id, window: Duration) -> Result<u32>
async fn is_locked_out(pool, user_id) -> Result<bool>

// Invite tokens
async fn create_invite_token(pool, player_id, created_by) -> Result<InviteToken>
async fn claim_invite_token(pool, token_hash, claiming_user_id) -> Result<Option<PlayerId>>
async fn get_invite_token(pool, token_hash) -> Result<Option<InviteToken>>
```

**Password hashing:** Argon2id with a unique salt per user. The hash is stored in `users.password_hash`. The `verify_password(candidate, hash)` function is synchronous CPU-intensive work; it is called inside `spawn_blocking` to avoid blocking the tokio executor.

**Invite token:** The token is a cryptographically random 32-byte value. The Auth Repository stores the SHA-256 hash of the token (not the plaintext). The plaintext token is returned to the API handler exactly once (at creation time) for display to the operator. `claim_invite_token` checks expiry and `claimed_at IS NULL` before recording the claim.

**Login lockout:** `is_locked_out` queries `login_attempts WHERE user_id = ? AND success = false AND attempted_at > (NOW - 15 minutes)` and counts rows. If count >= 10, the user is locked. No separate lockout table is needed — the lockout state is derived from the `login_attempts` table on every check.

**Satisfies:** UR-AUTH-001, UR-AUTH-002, UR-AUTH-003, SR-AUTH-001, SR-AUTH-002, SR-AUTH-003, SR-AUTH-004, SR-AUTH-005, NFR-SEC-001, NFR-SEC-002

---

### Job Repository

**Technology:** sqlx query functions

**Responsibility:** The `recalculation_jobs` table: inserting new jobs (with deduplication), atomically claiming one job for execution, updating job status, and recovering stuck jobs on startup.

**Public API surface:**

```rust
async fn insert_job(pool, season_id, triggered_by) -> Result<JobId>
// Returns existing queued job ID if one already exists for this season (deduplication).
// Only inserts a new job if no queued job exists. In-progress jobs do not block insertion.
async fn claim_next_job(pool) -> Result<Option<RecalculationJob>>
async fn mark_completed(pool, job_id) -> Result<()>
async fn mark_failed(pool, job_id, error_message) -> Result<()>
async fn get_job(pool, job_id) -> Result<Option<RecalculationJob>>
async fn reset_stuck_jobs(pool) -> Result<u32>  // returns count of reset jobs
async fn is_pending_for_season(pool, season_id) -> Result<bool>
```

**Job deduplication:** `insert_job` checks `is_pending_for_season(season_id)` before inserting. If a `queued` job exists for the season, the existing job ID is returned without a new insert. If the existing job is `in_progress`, a new job is inserted (the in-progress job will not include the triggering correction). This prevents redundant sequential replays when multiple corrections arrive before the poller fires.

**Atomic claim:** `claim_next_job` uses a single SQL statement:

```sql
UPDATE recalculation_jobs
SET status = 'in_progress', started_at = CURRENT_TIMESTAMP
WHERE id = (
    SELECT id FROM recalculation_jobs
    WHERE status = 'queued'
    ORDER BY triggered_at ASC
    LIMIT 1
)
RETURNING *;
```

This is safe under SQLite's serialized write model. Two concurrent callers cannot both claim the same job because SQLite serializes writes.

**Startup recovery:** `reset_stuck_jobs` runs before the poller loop starts:

```sql
UPDATE recalculation_jobs
SET status = 'queued', started_at = NULL
WHERE status = 'in_progress';
```

**Season serialization:** The poller processes one job at a time. If multiple jobs are queued for the same season, they are processed in `triggered_at` order. The `is_pending_for_season` function is used by the Swarm Dashboard UI indicator.

**Satisfies:** SR-PER-009, NFR-REL-001, ADR-0005

---

### ApiKey Repository

**Technology:** sqlx query functions

**Responsibility:** Manages the `api_keys` table. Validates an API key string (hashes it, looks it up), constructs a `SwarmContext`, and supports Admin CRUD for key management.

**Public API surface:**

```rust
async fn create_api_key(pool, user_id, description, created_by) -> Result<(ApiKeyId, String)>
// Returns the key ID and the plaintext key (returned to caller exactly once)
async fn validate_api_key(pool, plaintext_key: &str) -> Result<SwarmContext>
// SHA-256-hashes the key, looks it up, returns SwarmContext { user_id }
// Returns PersistenceError::InvalidToken if not found
async fn list_api_keys(pool, user_id: Option<i64>) -> Result<Vec<ApiKey>>
// Lists keys; if user_id is Some, filters to that user's keys
async fn delete_api_key(pool, key_id) -> Result<()>
```

**Satisfies:** ADR-0009, UR-SW-001

---

## Crate Dependencies

```
ladder-rs-persistence
├── ladder-rs          (workspace dependency, pure math)
├── sqlx               (async DB, SQLite feature)
├── tokio              (async runtime, features: rt, rt-multi-thread)
├── argon2             (password hashing, Auth Repository)
├── sha2               (token hashing for invite tokens)
├── serde              (JSON serialization for algorithm_params, score_metadata)
├── serde_json         (JSON support)
├── time / chrono      (datetime handling; align with sqlx's chrono feature)
└── thiserror          (error type derivation)
```

No HTTP, no Axum, no tower dependencies. The crate is purely a data access library.

---

## Error Type Hierarchy

```rust
pub enum PersistenceError {
    Database(sqlx::Error),
    NotFound { entity: &'static str, id: String },
    DuplicateMatch,
    SeasonClosed { season_id: i64 },
    DuplicateUser { field: &'static str },
    InvalidToken,
    TokenExpired,
    TokenAlreadyClaimed,
    AlgorithmMismatch,
    PlayerLocked,  // account lockout
}
```

The server crate maps `PersistenceError` variants to HTTP status codes and structured JSON error responses (SR-API-002).

---

## Requirements Traceability

| Requirement | Component(s) |
|-------------|--------------|
| SR-PER-001 | All repositories (persistence crate is the library API) |
| SR-PER-002 | Match Repository (atomic transaction) |
| SR-PER-003 | Player Repository (soft-delete) |
| SR-PER-004 | Match Repository (duplicate detection) |
| SR-PER-005 | Match Repository (season closed check) |
| SR-PER-006 | Player Repository (get_or_create_player) |
| SR-PER-007 | Alias Repository (recalc job on alias change) |
| SR-PER-008 | Match Repository (recorded_at ordering in batch) |
| SR-PER-009 | Job Repository (job lifecycle), Alias Repository (job insertion) |
| SR-AUTH-001 | Auth Repository (user CRUD, password hashing, session) |
| SR-AUTH-002 | Auth Repository (role lookup) |
| SR-AUTH-003 | League Repository (is_operator), Auth Repository (league assignments) |
| SR-AUTH-004 | Auth Repository (is_users_table_empty, bootstrap insert) |
| SR-AUTH-005 | Auth Repository (invite token create/claim) |
| SR-AUTH-006 | League Repository (visibility filter) |
| SR-ALG-004 | Season Repository (apply_seeding) |
| SR-ALG-005 | Rating Engine Bridge (conservative_rating pre-computation) |
| SR-ADM-001 | Match Repository (correct_match, audit log insert) |
| SR-SW-001 | Player Repository, Match Repository (active threshold queries) |
| NFR-PERF-001 | Rating Engine Bridge (calls ladder-rs synchronous math) |
| NFR-PERF-002 | Connection Pool (WAL + pool), indexed queries |
| NFR-REL-001 | Connection Pool (WAL mode), Job Repository (startup recovery) |
| NFR-PORT-001 | Schema Migrations (standard SQL), all repositories (sqlx portable syntax) |
| NFR-SEC-001 | Auth Repository (consecutive_failures, is_locked_out) |
| NFR-SEC-002 | Auth Repository (session management via tower-sessions) |
| NFR-SEC-003 | All repositories (parameterized queries prevent injection) |
