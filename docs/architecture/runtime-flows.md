# Runtime Flows

**ladder-rs Platform**
**Date:** 2026-04-15

---

## Overview

This document traces the five critical runtime paths through the ladder-rs platform. Each flow is described as a numbered step sequence showing which container, component, and data entity is involved at each stage.

---

## Flow 1: Match Recording (Normal Path)

**Trigger:** A League Operator submits a match result via the UI.

**Preconditions:**
- User has an authenticated session (League Operator role, assigned to the league)
- The season is open (`seasons.end_date IS NULL`)
- All referenced players are members of the league

### Steps

1. **Browser** — The operator fills in the match form (select players, placements or win/loss/draw) and clicks submit. The browser sends:
   ```
   POST /api/seasons/{season_id}/matches
   Cookie: session=<session_id>
   Body: { participants: [...], score_metadata: {...} }
   ```

2. **nginx (frontend container)** — Receives the request on `:80`. The path matches `/api/*`, so nginx proxies it to `backend:3000`, preserving headers and body. No transformation.

3. **Axum Router (backend container)** — Routes the request to the Match Handler's `record_match` function.

4. **Auth Middleware** — Intercepts the request before the handler:
   - Reads the `session` cookie value
   - Queries the `sessions` table via `tower-sessions-sqlx-store` to validate and load session data
   - Loads the `AuthContext` (user_id, role, league_assignments) from the session
   - Checks `force_password_change` — not set, continues
   - Injects `AuthContext` into request extensions

5. **Match Handler** — Receives the request with `AuthContext` injected:
   - Extracts `season_id` from the path parameter
   - Calls `League Repository.is_operator(league_id, user_id)` to verify the caller is assigned to this league (or is Admin)
   - Returns 403 if not authorized
   - Validates the request body: participant count, placement values, algorithm-aware rules (e.g., draw rejected if `draw_probability = 0` for TrueSkill season)

6. **Match Repository** — Handler calls `record_match(pool, season_id, participants, score_metadata)`. The repository opens a SQLite transaction:

   a. **Season write protection check:** `SELECT end_date FROM seasons WHERE id = ?` — if non-NULL, return `PersistenceError::SeasonClosed` → handler returns 409 Conflict.

   b. **Duplicate detection check:** Queries `matches` and `match_participants` for an existing match with the same season, players, placements, draw flags, and recorded_at timestamp (to the nearest second). If found, return `PersistenceError::DuplicateMatch` → handler returns 409 Conflict.

   c. **Insert match header:** `INSERT INTO matches (season_id, recorded_at, score_metadata) RETURNING id`

   d. **Insert participants:** `INSERT INTO match_participants (match_id, player_id, placement, is_draw) VALUES ...` for each participant.

7. **Rating Engine Bridge** — Called within the same transaction. Loads the most recent `rating_snapshot` for each participant in this season. If a player has no prior snapshot (new joiner), uses the season's algorithm defaults from `seasons.algorithm_params`.

8. **ladder-rs** — The Rating Engine Bridge calls the appropriate algorithm function from the `ladder-rs` crate:
   - Elo: `elo::update_ratings(ratings, outcome)`
   - Glicko-2: `glicko2::update_ratings(ratings, outcomes)`
   - TrueSkill: `trueskill::update_ratings(ratings, ranked_teams, draw_probability)` — returns a `ConvergenceResult` indicating `Converged` or `BestApproximation`

9. **Rating Engine Bridge (continued)** — Receives the new rating values. If TrueSkill returned `BestApproximation`, sets `convergence_quality = "degraded"` on the pending match record. Computes `conservative_rating` per SR-ALG-005:
   - Elo: `conservative_rating = rating`
   - Glicko-2: `conservative_rating = rating - 2 * deviation`
   - TrueSkill: `conservative_rating = rating - 3 * uncertainty`

10. **Match Repository (continued)** — Inserts rating snapshots:
    ```sql
    INSERT INTO rating_snapshots
        (match_id, player_id, season_id, rating, deviation, uncertainty, conservative_rating, timestamp)
    VALUES ...
    ```
    If `convergence_quality = "degraded"`, updates the match record accordingly. **COMMIT** transaction.

11. **Match Handler** — Receives the `MatchResult` (new match ID, new ratings per player). Constructs the HTTP response body with the new ratings. Returns `201 Created`.

12. **nginx** — Forwards the `201` response back to the browser.

13. **Browser** — Displays the updated ratings. The leaderboard is not automatically refreshed — the operator can navigate to it to see the new standings.

**Error paths:**
- Session expired or invalid → 401 Unauthorized (step 4)
- Not assigned to league → 403 Forbidden (step 5)
- Season closed → 409 Conflict (step 6a)
- Duplicate match → 409 Conflict (step 6b)
- DB error → 500 Internal Server Error with structured JSON body (SR-API-002)

---

## Flow 2: Leaderboard Query

**Trigger:** A user (any role, including unauthenticated for public leagues — v1 requires authentication) navigates to a league leaderboard page.

**Preconditions:**
- User has an authenticated session
- League exists and is visible to the user (public, or user is Admin/operator/member for private)

### Steps

1. **Browser** — Navigates to the leaderboard view. React component mounts and sends:
   ```
   GET /api/seasons/{season_id}/leaderboard?limit=50&offset=0
   Cookie: session=<session_id>
   ```

2. **nginx** — Proxies `/api/*` to `backend:3000`.

3. **Axum Router** — Routes to `Leaderboard Handler`.

4. **Auth Middleware** — Validates session, extracts `AuthContext`. Continues on success.

5. **Leaderboard Handler**:
   - Extracts `season_id`, `limit`, `offset` (and optional `cursor`) from query parameters
   - Loads the season's `league_id` from `seasons` table
   - Calls `League Repository.get_league(league_id)` to check `visibility`
   - For private leagues: checks if the user is Admin, an assigned operator (`league_operators`), or a league member (`league_players`) — returns 404 (not 403, to avoid leaking existence) if not permitted (SR-AUTH-006)

6. **Rating Snapshot Query** — Handler calls a leaderboard query on the persistence layer:
   ```sql
   SELECT
       rs.player_id,
       p.name,
       rs.rating,
       rs.deviation,
       rs.uncertainty,
       rs.conservative_rating,
       COUNT(mp2.match_id) as match_count,
       ROW_NUMBER() OVER (ORDER BY rs.conservative_rating DESC) as rank
   FROM rating_snapshots rs
   JOIN players p ON p.id = rs.player_id
   JOIN league_players lp ON lp.league_id = ? AND lp.player_id = rs.player_id AND lp.is_active = 1
   JOIN match_participants mp2 ON mp2.player_id = rs.player_id
       JOIN matches m2 ON m2.id = mp2.match_id AND m2.season_id = rs.season_id
   WHERE rs.season_id = ?
     AND rs.match_id = (
         SELECT MAX(rs2.match_id) FROM rating_snapshots rs2
         WHERE rs2.player_id = rs.player_id AND rs2.season_id = rs.season_id
     )
   ORDER BY rs.conservative_rating DESC
   LIMIT ? OFFSET ?;
   ```
   The `idx_rating_snapshots_season_player_match` index enables the subquery to efficiently find the latest snapshot per player without a full scan.

7. **Leaderboard Handler** — Constructs the response:
   ```json
   {
     "season_id": 42,
     "algorithm": "trueskill",
     "players": [
       { "rank": 1, "player_id": 7, "name": "Alice", "rating": 32.1,
         "uncertainty": 2.4, "conservative_rating": 24.9, "match_count": 47 },
       ...
     ],
     "pagination": { "total": 312, "limit": 50, "offset": 0, "next_cursor": "..." }
   }
   ```

8. **Browser** — Renders the leaderboard table sorted by rank.

**Performance target:** < 500ms for 10,000 players (NFR-PERF-002), achieved by the indexed `conservative_rating` pre-computation and the compound index on `(season_id, player_id, match_id DESC)`.

---

## Flow 3: Admin Match Correction

**Trigger:** An Admin identifies an incorrect match result and submits a correction.

**Preconditions:**
- User has Admin role
- Match exists and belongs to an open or closed season

### Steps

1. **Browser** — Admin navigates to the match detail view, edits the match, and submits:
   ```
   PATCH /api/matches/{match_id}
   Cookie: session=<session_id>
   Body: { participants: [updated placements...], reason: "entered wrong winner" }
   ```

2. **nginx** → **Axum Router** — Routes to `Match Handler` (`correct_match` function).

3. **Auth Middleware** — Validates session, extracts `AuthContext`. Confirms `role = 'admin'`. Non-admin returns 403.

4. **Match Handler**:
   - Loads current match state from `matches` + `match_participants` as `before_state` (JSON snapshot)
   - Validates the correction payload (valid participant IDs, valid placements)

5. **Match Repository** — Handler calls `correct_match(pool, match_id, correction, changed_by)`:

   a. Opens a SQLite transaction.

   b. `UPDATE matches SET is_corrected = 1 WHERE id = ?`

   c. `UPDATE match_participants SET placement = ?, is_draw = ? WHERE match_id = ? AND player_id = ?` for each changed participant.

   d. Captures `after_state` as a JSON snapshot of the updated match + participants.

   e. `INSERT INTO match_audit_log (match_id, changed_by, changed_at, before_state, after_state) VALUES (?)`

   f. **COMMIT** transaction.

6. **Match Handler** — After the match is corrected, inserts a recalculation job:
   - Calls `Job Repository.insert_job(pool, season_id, triggered_by = user_id)`
   - Returns the `job_id`

7. **Match Handler** — Returns `202 Accepted`:
   ```json
   { "job_id": 17, "status": "queued", "message": "Correction recorded. Ratings will be recalculated asynchronously." }
   ```
   The HTTP response returns immediately — no rating recalculation happens synchronously.

8. **Browser** — Shows "Recalculation queued" status. The UI may poll `GET /api/jobs/{job_id}` to monitor progress.

--- (asynchronous continuation) ---

9. **Background Job Poller** — Running in the backend process on a 1–5 second poll cycle. Calls `Job Repository.claim_next_job()`:
   ```sql
   UPDATE recalculation_jobs SET status = 'in_progress', started_at = ...
   WHERE id = (SELECT id FROM recalculation_jobs WHERE status = 'queued' ORDER BY triggered_at ASC LIMIT 1)
   RETURNING *;
   ```
   Receives the job for season `season_id`. Passes it to the Recalculation Worker.

10. **Recalculation Worker** — Executes the full-season replay:

    a. Loads all matches for the season ordered by `recorded_at ASC` (SR-PER-008).

    b. Calls `Alias Repository.resolve_alias_group(player_id)` for each player in the season to get full alias groups. Aliased players are treated as one during replay.

    c. Iterates matches in order, calling the Rating Engine Bridge for each. The bridge calls `ladder-rs` math and produces new `RatingSnapshot` values.

    d. Accumulates all new snapshots in memory.

    e. Opens a SQLite transaction:
       - `DELETE FROM rating_snapshots WHERE season_id = ?` — removes all stale snapshots for the season
       - `INSERT INTO rating_snapshots (...)` — inserts all recalculated snapshots in match order
       - `UPDATE recalculation_jobs SET status = 'completed', completed_at = ? WHERE id = ?`
    f. **COMMIT** transaction. The new ratings are now atomically visible to all readers.

11. **If recalculation fails** — The transaction is rolled back. Job status is set to `'failed'` with the error message. The pre-correction rating snapshots remain intact (stale but available).

**Eventual consistency window:** Between step 7 (202 returned) and step 10f (commit), the leaderboard serves the pre-correction ratings. The Swarm Dashboard UI should display a "recalculation in progress" indicator when `Job Repository.is_pending_for_season(season_id)` returns true.

---

## Flow 4: First Startup / Admin Bootstrap

**Trigger:** The backend container starts for the first time against an empty (or newly initialized) SQLite database.

### Steps

1. **backend container starts** — The `ladder-rs-server` binary starts. The tokio runtime initializes.

2. **Connection Pool initialization** — `ladder-rs-persistence` creates the `SqlitePool`. Applies `PRAGMA journal_mode = WAL`, `busy_timeout`, `foreign_keys = ON`, `synchronous = NORMAL`.

3. **Schema Migrations** — `sqlx::migrate!` runs all pending migration files from `migrations/`. On a brand-new database, all migrations run in order, creating all tables and indexes. On subsequent startups with no new migrations, this is a no-op (checked via `_sqlx_migrations` table).

4. **Startup recovery** — Before the Axum router starts, `Job Repository.reset_stuck_jobs()` runs. On first startup, there are no `in_progress` jobs, so this is a no-op.

5. **Admin bootstrap check** — The server calls `Auth Repository.is_users_table_empty()`:
   ```sql
   SELECT COUNT(*) = 0 FROM users;
   ```

6. **Bootstrap detected** — If the table is empty:

   a. Generate a cryptographically random username (e.g., `admin`) and password (16+ character random string).

   b. Hash the password with argon2id.

   c. `INSERT INTO users (username, email, password_hash, role, force_password_change) VALUES ('admin', 'admin@localhost', '<hash>', 'admin', 1)`

   d. Print to stdout (plaintext, visible in `docker logs backend`):
   ```
   =========================================
   FIRST-RUN BOOTSTRAP — ADMIN CREDENTIALS
   Username: admin
   Password: <plaintext_password>
   
   You MUST change this password on first login.
   =========================================
   ```

7. **Axum router starts** — The router binds to `0.0.0.0:3000`. The background job poller task is spawned.

8. **Admin first login** — Admin navigates to the UI, logs in with the printed credentials. The Auth Handler detects `force_password_change = 1` in the session data. All API requests return 403 until `POST /api/auth/change-password` is called. On successful password change, `Auth Repository.clear_force_change(user_id)` is called.

**`ADMIN_BOOTSTRAP_ENABLED` environment variable:** If set to `false`, step 5-6 is skipped even if the users table is empty. This allows controlled deployments where credentials are provisioned externally.

---

## Flow 5: Swarm Operator Writes

**Trigger:** A Swarm Operator's Rust process records a batch of agent match results by calling `ladder-rs-persistence` functions directly (no HTTP involved).

**Preconditions:**
- Swarm Operator process has `ladder-rs-persistence` as a Cargo dependency
- The `DATABASE_URL` pointing to the same SQLite file is configured in the Swarm Operator's environment
- The Swarm Operator has a valid user account in the platform and is an assigned operator for their league

### Steps

1. **Swarm Operator Process** — The external process initializes its own `SqlitePool` via `ladder-rs-persistence`'s `Connection Pool` component:
   ```rust
   let pool = persistence::connect(&database_url).await?;
   ```
   This opens a separate connection pool to the same `/data/ladder.db` file.

2. **SQLite WAL mode** — Both the Axum backend's pool and the Swarm Operator's pool have WAL mode enabled. They share the WAL log file. Readers (backend leaderboard queries) are not blocked by swarm writes. Concurrent writes serialize at the SQLite level via the WAL write lock.

3. **Player auto-creation** — The Swarm Operator's code references agent names in match entries. The `Player Repository.get_or_create_player(pool, agent_name, PlayerType::NonHuman)` function is called for each agent. New agents are auto-created (SR-PER-006). This is an atomic INSERT OR IGNORE + SELECT, safe against concurrent invocations.

4. **Match recording** — The Swarm Operator calls `Match Repository.record_match(pool, season_id, participants, None)` directly. This is the same function called by the Axum backend in Flow 1. The atomic transaction (duplicate check, insert match, insert participants, compute ratings, insert snapshots) executes identically whether called from the server or from the Swarm Operator process.

5. **Concurrent backend activity** — While the Swarm Operator is writing, the Axum backend may be serving leaderboard queries concurrently. In WAL mode:
   - Backend read transactions proceed against the last committed WAL snapshot — they see all matches committed before the current swarm write, but not the in-progress transaction
   - After the swarm write commits, the next backend read sees the new data

6. **`busy_timeout` handling** — If the backend is in the middle of a write (e.g., recording a manually entered match) when the Swarm Operator also attempts a write, one write will wait up to `busy_timeout` (5 seconds) for the WAL write lock. If the lock is not released within that window, the operation returns `SQLITE_BUSY`, which sqlx converts to a `sqlx::Error::Database` error. The Swarm Operator process is responsible for handling this error and retrying.

7. **Swarm Dashboard (read path)** — After writing, the Swarm Operator can view their agents' performance on the Swarm Dashboard via the browser (Flow 2 variant). The dashboard reads from the same `rating_snapshots` and `matches` tables written in step 4.

**Key invariant:** The Swarm Operator process does not bypass any business logic. It calls the same `Match Repository.record_match` function that the backend uses, so all checks (duplicate detection, season write protection, player soft-delete guard) apply equally.

---

## Requirements Traceability

| Requirement | Flow(s) |
|-------------|---------|
| SR-PER-002 | Flow 1 (atomic transaction), Flow 5 (same path) |
| SR-PER-004 | Flow 1 (duplicate check in step 6b) |
| SR-PER-005 | Flow 1 (season closed check in step 6a) |
| SR-PER-006 | Flow 5 (auto-creation in step 3) |
| SR-PER-007 | Flow 3 (recalc after correction), triggered by alias ops |
| SR-PER-008 | Flow 3 (recalculation replays in recorded_at order in step 10a) |
| SR-PER-009 | Flow 3 (202 Accepted + async recalculation) |
| SR-AUTH-001 | All flows (session validation in Auth Middleware) |
| SR-AUTH-002 | Flows 1, 2, 3 (role checks) |
| SR-AUTH-004 | Flow 4 (admin bootstrap) |
| SR-AUTH-006 | Flow 2 (visibility check in step 5) |
| SR-ALG-005 | Flows 1, 2 (conservative_rating computation and sort) |
| SR-ADM-001 | Flow 3 (audit log in step 5e) |
| NFR-PERF-001 | Flow 1, step 8 (ladder-rs math < 10ms for TrueSkill) |
| NFR-PERF-002 | Flow 2 (leaderboard < 500ms for 10K players) |
| NFR-REL-001 | Flow 4 (WAL mode, crash recovery), Flow 3 (stale ratings preserved on failure) |
| NFR-SEC-001 | Flows 1-3 (Auth Middleware, login rate limiting in Auth Handlers) |
| NFR-SEC-002 | All flows (HttpOnly session cookies) |
