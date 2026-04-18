# Component View — Backend Container

**ladder-rs-server (Axum backend)**
**Date:** 2026-04-15

---

## Overview

The backend container is a single Rust binary built from the `ladder-rs-server` crate with `ladder-rs-persistence` and `ladder-rs` compiled in as library dependencies. This C4 Level 3 view describes the significant components inside that binary: the Axum router, middleware layers, HTTP handler groups, and background workers.

All components in this view run in the same OS process. Communication between them is Rust function calls (no IPC, no message queues, no internal HTTP).

---

## Component Map

```
[backend container process]
│
├── [Router] ──────────────────── route table, handler dispatch
│   │
│   ├── [Auth Middleware] ──────── tower Layer: session validation, role extraction,
│   │                              forced-password-change enforcement, rate-limit check
│   │
│   ├── [Auth Handlers] ────────── login, logout, register, bootstrap, invite
│   ├── [League Handlers] ──────── league CRUD, visibility, archive
│   ├── [Season Handlers] ──────── season transitions, seeding choices
│   ├── [Player Handlers] ──────── global player CRUD, membership, search
│   ├── [Match Handlers] ───────── match recording, batch entry, admin correction
│   ├── [Leaderboard Handlers] ─── paginated ranked list
│   ├── [Rating History Handlers]── per-season chart data
│   └── [Swarm Dashboard Handlers]─ aggregate stats, active agent filter
│
├── [Background Job Poller] ────── tokio task, polls recalculation_jobs
│   └── [Recalculation Worker] ─── executes full-season replay via persistence
│
└── ──────────────────────────── calls into:
    └── [ladder-rs-persistence]  ── all DB operations (separate crate, see persistence.md)
```

---

## Components

### Router

**Technology:** `axum::Router`, nested route groups

**Responsibility:** Defines the complete HTTP API surface of the backend. Attaches middleware layers to route groups. Dispatches matched requests to the appropriate handler function.

**Key design decisions:**
- Routes are organized by domain: `/api/auth/...`, `/api/leagues/...`, `/api/players/...`, `/api/matches/...`, `/api/seasons/...`, `/api/leaderboard/...`, `/api/history/...`, `/api/swarm/...`, `/api/jobs/...`
- The Auth Middleware is applied as a tower `Layer` on all routes except the login and public-read endpoints
- All handlers receive dependencies (persistence repositories, session store) via Axum `State` extractor

**Satisfies:** All UR-* (the router is the entry point for all API interactions)

---

### Auth Middleware

**Technology:** `tower::Layer` / `tower::Service`, `tower-sessions`, custom lockout check

**Responsibility:** Intercepts every request on the authenticated route group and performs three checks:

1. **Session validation.** Reads the session cookie, looks up the session in the SQLite sessions table via `tower-sessions-sqlx-store`. Rejects with 401 if absent, expired, or invalid.

2. **Role extraction.** Loads the authenticated user's global role and any league-operator assignments from the session / DB. Attaches `AuthContext` (user_id, role, league_assignments) to the request extensions for downstream handlers.

3. **Forced-password-change enforcement.** If the user's `force_password_change` flag is set, any request other than `POST /api/auth/change-password` returns 403.

4. **Rate-limit pass-through.** The login endpoint (outside this middleware) has its own rate-limit check via the Auth Repository's `login_attempts` table. This middleware does not duplicate that check — it only validates already-issued sessions.

**Satisfies:** SR-AUTH-001, SR-AUTH-002, NFR-SEC-001 (login path), NFR-SEC-002

---

### Auth Handlers

**Technology:** Axum handler functions

**Endpoints:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| POST | `/api/auth/login` | None | Validates credentials, enforces lockout (NFR-SEC-001), issues session cookie |
| POST | `/api/auth/logout` | Any authenticated | Destroys session |
| POST | `/api/auth/register` | None | Creates new user account; requires `invite_token` in body (v1). Atomically creates the account and claims the token, linking the new user to the player record. See ADR-0008. |
| POST | `/api/auth/change-password` | Any authenticated | Changes password; clears force-change flag |
| POST | `/api/auth/admin/set-password` | Admin | Sets temporary password on any account, sets force-change flag |
| POST | `/api/auth/invites` | Admin or League Operator | Generates invite token for a player record |
| POST | `/api/auth/invites/{token}/claim` | Any authenticated | Claims an invite token; links user account to player record |

**Login rate limiting detail:** The login handler calls `Auth Repository.record_attempt()` on every attempt. On failure, it queries `Auth Repository.consecutive_failures()` — which counts failures since the most recent successful login (consecutive, not windowed). A successful login resets the counter to zero. If the count reaches >= 10, the handler returns 429. The lockout lifts automatically after 15 minutes from the 10th failure (checked against the `attempted_at` timestamp of the 10th row). No separate lockout record is needed — the state is derived from the `login_attempts` table. See the corrected lockout query in `data-architecture.md`.

**API key management (for swarm operators):**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| POST | `/api/admin/api-keys` | Admin | Generate a library API key for a swarm operator user; returns plaintext key once |
| GET | `/api/admin/api-keys` | Admin | List all API keys (key hashes not returned, only metadata) |
| DELETE | `/api/admin/api-keys/{id}` | Admin | Revoke an API key |

**Admin bootstrap:** On startup, `Auth Repository.is_users_table_empty()` is called. If true, a random username + password are generated, an admin user is inserted with `force_password_change = true`, and credentials are printed to stdout. This happens before the Axum router starts accepting connections.

**Satisfies:** SR-AUTH-001, SR-AUTH-004, SR-AUTH-005, NFR-SEC-001, NFR-SEC-002

---

### League Handlers

**Technology:** Axum handler functions

**Endpoints:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| POST | `/api/leagues` | Admin or League Operator | Create league |
| GET | `/api/leagues` | Any (visibility-filtered) | List leagues |
| GET | `/api/leagues/{id}` | Any (visibility check) | Get league detail |
| PATCH | `/api/leagues/{id}` | Admin or assigned Operator | Edit metadata, visibility |
| POST | `/api/leagues/{id}/archive` | Admin or assigned Operator | Archive league |
| POST | `/api/leagues/{id}/unarchive` | Admin or assigned Operator | Un-archive league |
| POST | `/api/leagues/{id}/algorithm` | Admin or assigned Operator | Change algorithm type (triggers season) |
| POST | `/api/leagues/{id}/operators` | Admin | Assign League Operator |
| DELETE | `/api/leagues/{id}/operators/{uid}` | Admin | Remove League Operator |

**Visibility enforcement:** All list and get endpoints filter results through `SR-AUTH-006` logic: public leagues are visible to all authenticated users; private leagues only to Admin, assigned Operators, and Player/Viewers with a `league_players` record.

**Satisfies:** UR-LM-001, SR-AUTH-003, SR-AUTH-006, SR-ALG-001, SR-ALG-002, SR-ALG-003, SR-API-002, SR-API-003

---

### Season Handlers

**Technology:** Axum handler functions

**Endpoints:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| GET | `/api/leagues/{id}/seasons` | Any (visibility check) | List seasons for a league |
| GET | `/api/seasons/{id}` | Any (visibility check) | Get season detail |
| POST | `/api/leagues/{id}/seasons` | Admin or assigned Operator | Start new season (algorithm type change) |
| PATCH | `/api/seasons/{id}/params` | Admin or assigned Operator | Update algorithm parameters (in-place, no new season) |
| POST | `/api/seasons/{id}/close` | Admin or assigned Operator | Close current season |

**Seeding choice:** When creating a new season, the request body includes `seeding_choice: "reset" | "ordinal"`. This is passed directly to `ladder-rs-persistence`'s Season Repository.

**Satisfies:** UR-LM-002, SR-ALG-003, SR-ALG-004

---

### Player Handlers

**Technology:** Axum handler functions

**Endpoints:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| POST | `/api/players` | Admin or League Operator | Create global player record |
| GET | `/api/players` | Any authenticated | List/search global players |
| GET | `/api/players/{id}` | Any authenticated | Get player detail |
| PATCH | `/api/players/{id}` | Admin or assigned Operator | Edit player name/type |
| POST | `/api/leagues/{id}/players` | Admin or assigned Operator | Add player to league (league_players join) |
| DELETE | `/api/leagues/{id}/players/{pid}` | Admin or assigned Operator | Soft-delete from league |
| GET | `/api/leagues/{id}/players` | Any (visibility check) | List players in league |
| GET | `/api/players/search` | Any authenticated | Autocomplete search by name prefix |
| POST | `/api/players/{id}/aliases` | Admin | Create alias link between two player records |
| DELETE | `/api/players/{id}/aliases/{aid}` | Admin | Remove alias link |

**Search implementation:** The `/api/players/search` endpoint accepts a `q` query parameter and delegates to `Player Repository.search_by_prefix()`. Results are limited to 20 entries. Used for autocomplete in the match entry form.

**Alias triggers:** Creating or removing an alias triggers a recalculation job via the Job Repository. The handler returns 202 Accepted with the job ID.

**Satisfies:** UR-PM-001, UR-PM-002, SR-PER-003, SR-PER-006, SR-PER-007, SR-PER-009, SR-API-002, SR-API-004

---

### Match Handlers

**Technology:** Axum handler functions

**Endpoints:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| POST | `/api/seasons/{id}/matches` | Admin or assigned Operator | Record a single match |
| POST | `/api/seasons/{id}/matches/batch` | Admin or assigned Operator | Batch match entry (sequential) |
| GET | `/api/seasons/{id}/matches` | Any (visibility check) | List matches in season |
| GET | `/api/matches/{id}` | Any (visibility check) | Get match detail |
| PATCH | `/api/matches/{id}` | Admin only | Admin correction (audited) |

**Single match recording flow:** Handler validates the request (algorithm-aware: draws disallowed if `draw_probability=0`), calls `Match Repository.record_match()` which atomically inserts the match record, participants, calls the Rating Engine Bridge to compute new ratings, and inserts rating snapshots. Returns the new ratings in the response body.

**Batch entry:** Matches are processed sequentially in submission order. Each match is individually validated. Non-convergence does not abort the batch — the affected match is flagged with `convergence_quality = "degraded"` and processing continues. Errors are returned per-entry in the response.

**Admin correction:** Only Admin role may call `PATCH /api/matches/{id}`. The handler updates the match record, writes a `match_audit_log` entry with before/after state, and inserts a `recalculation_jobs` record with `status = "queued"`. Returns 202 Accepted with the job ID. The actual recalculation is asynchronous.

**Satisfies:** UR-ME-001, UR-ME-002, UR-ADM-001, SR-PER-002, SR-PER-004, SR-PER-005, SR-PER-008, SR-ADM-001, SR-PER-009, SR-API-002

---

### Leaderboard Handlers

**Technology:** Axum handler functions

**Endpoint:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| GET | `/api/seasons/{id}/leaderboard` | Any (visibility check) | Ranked player list for a season |

**Pagination:** Supports both cursor-based and offset-based pagination (SR-API-001). Query parameters: `limit`, `offset`, `cursor`. Cursor tokens encode the `conservative_rating` and `player_id` of the last-seen row for stable sequential traversal.

**Sort key:** The ranking metric is the algorithm-specific conservative estimate (SR-ALG-005):
- Elo: `rating`
- Glicko-2: `rating - 2 * deviation`
- TrueSkill: `rating - 3 * uncertainty`

The `conservative_rating` column in `rating_snapshots` stores the pre-computed sort key, updated on every rating change. This avoids per-query arithmetic on 10K rows.

**Response fields per player:** rank, player_id, player_name, raw rating, deviation/uncertainty (where applicable), conservative_rating, match_count.

**Satisfies:** UR-LB-001, SR-ALG-005, SR-API-001, SR-API-003

---

### Rating History Handlers

**Technology:** Axum handler functions

**Endpoints:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| GET | `/api/seasons/{sid}/players/{pid}/history` | Any (visibility check) | Per-season match-by-match rating progression |
| GET | `/api/players/{pid}/seasons` | Any authenticated | Season overview (final rating per season) |

**Chart data format:** The per-season history endpoint returns an ordered list of `{match_id, recorded_at, rating, deviation, uncertainty, conservative_rating}` entries for the player in that season. The frontend renders this as a line chart.

**Season overview:** Returns a list of `{season_id, algorithm, start_date, end_date, final_rating, final_conservative_rating, match_count}` per season the player participated in. No cross-season combined chart — scales are incompatible.

**Satisfies:** UR-RH-001

---

### Swarm Dashboard Handlers

**Technology:** Axum handler functions

**Endpoints:**

| Method | Path | Role Required | Description |
|--------|------|---------------|-------------|
| GET | `/api/leagues/{id}/swarm/stats` | Any (visibility check) | Aggregate swarm statistics |
| GET | `/api/leagues/{id}/swarm/agents` | Any (visibility check) | Per-agent breakdown |
| GET | `/api/leagues/{id}/swarm/volume` | Any (visibility check) | Match volume over time |

**Active agent filter:** All swarm endpoints accept an optional `active_threshold_days` query parameter. If absent, the league's configured `active_agent_threshold_days` is used as the default. Players (agents) without a match within the threshold window are excluded from "active" counts but included in aggregate totals.

**Aggregate stats returned:** Rating distribution histogram (bucket counts), top-N agents, bottom-N agents, match volume per time bucket (hour/day/week), per-agent win rate by rating bucket, agent lifecycle summary (first match, match count, last match, active status).

**Satisfies:** UR-SW-001, SR-SW-001, SR-API-001, SR-API-003

---

### Background Job Poller

**Technology:** `tokio::spawn`, looping async task

**Responsibility:** A long-lived background tokio task started when the server initializes. Polls the `recalculation_jobs` table every 1–5 seconds (configurable; exponential backoff on idle). Claims one job at a time using an atomic claim operation (UPDATE with WHERE clause and RETURNING). Delegates claimed jobs to the Recalculation Worker.

**Startup recovery:** Before the polling loop starts, the poller calls `Job Repository.reset_stuck_jobs()`, which sets all `in_progress` jobs back to `queued`. This handles the case where the server crashed mid-recalculation.

**Serialization:** Only one job runs at a time per server instance. If multiple corrections are queued for the same season, they run sequentially. The claim operation's WHERE clause (`status = 'queued'`) ensures at-most-one execution.

**Satisfies:** SR-PER-009, NFR-REL-001, ADR-0005

---

### Recalculation Worker

**Technology:** Rust async function, called by Background Job Poller

**Responsibility:** Executes one full-season recalculation for a claimed job. Steps:

1. Load all matches for the season, ordered by `recorded_at` (SR-PER-008)
2. Load all alias groups for players in those matches
3. Iterate matches in order, calling the Rating Engine Bridge (via persistence crate) to compute ratings treating aliased player records as one
4. Accumulate new `RatingSnapshot` values
5. Within a single SQLite transaction: delete all existing `rating_snapshots` for the season, insert the newly computed snapshots, update the job `status` to `completed`

**Failure handling:** If any step fails, the transaction rolls back, job status is set to `failed` with the error message, and the pre-recalculation snapshots remain intact (SR-PER-009 acceptance criterion: retain stale ratings on failure).

**Satisfies:** SR-PER-007, SR-PER-009, SR-ADM-001

---

## Request / Response Lifecycle

A typical authenticated API request flows through these components in order:

1. HTTP request arrives at Axum router
2. `Auth Middleware` validates session cookie, extracts `AuthContext`
3. Axum router matches route, extracts path/query parameters
4. Handler function is called with `State`, `AuthContext`, and extractors
5. Handler performs authorization check (role from `AuthContext` vs. required role)
6. Handler calls one or more repository functions on `ladder-rs-persistence`
7. Handler constructs HTTP response from repository results
8. Response passes back through middleware tower (no significant outbound middleware in v1)
9. Axum sends response

---

## Requirements Traceability

| Requirement | Component(s) |
|-------------|--------------|
| UR-LM-001 | League Handlers, Auth Middleware |
| UR-LM-002 | Season Handlers |
| UR-PM-001 | Player Handlers |
| UR-PM-002 | Player Handlers (alias endpoints) |
| UR-ME-001 | Match Handlers |
| UR-ME-002 | Match Handlers (batch endpoint) |
| UR-LB-001 | Leaderboard Handlers |
| UR-RH-001 | Rating History Handlers |
| UR-SW-001 | Swarm Dashboard Handlers |
| UR-ADM-001 | Match Handlers (correction), Background Job Poller, Recalculation Worker |
| UR-AUTH-001 | Auth Handlers, Auth Middleware |
| UR-AUTH-002 | Auth Middleware (role enforcement) |
| UR-AUTH-003 | Auth Handlers (invite endpoints) |
| SR-AUTH-004 | Auth Handlers (bootstrap logic on startup) |
| SR-AUTH-006 | League Handlers, Season Handlers, Leaderboard Handlers, Rating History Handlers |
| SR-PER-009 | Background Job Poller, Recalculation Worker |
| SR-ALG-005 | Leaderboard Handlers |
| SR-API-001 | Leaderboard Handlers |
| SR-API-002 | All handler groups |
| SR-API-003 | Leaderboard Handlers, Swarm Dashboard Handlers |
| SR-API-004 | Player Handlers (search endpoint) |
| SR-SW-001 | Swarm Dashboard Handlers |
| SR-ADM-001 | Match Handlers (correction), Recalculation Worker |
| NFR-SEC-001 | Auth Handlers (login rate limiting) |
| NFR-SEC-002 | Auth Handlers (session cookie configuration) |
| NFR-SEC-003 | All handler groups (input sanitization via cross-cutting layer) |
| NFR-REL-001 | Background Job Poller (startup recovery) |
