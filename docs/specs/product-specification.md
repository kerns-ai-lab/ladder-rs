# ladder-rs Platform -- Product Specification v1.2

**Updated:** 2026-04-04

## 1. Vision & Problem Statement

Competitive gaming communities and AI agent researchers need a unified platform to track skill ratings across players and autonomous agents. Existing solutions are either closed-source black boxes (TrueSkill), language-specific libraries without operational tooling, or full SaaS platforms with unnecessary complexity.

ladder-rs solves this by providing:

- A **mathematically rigorous rating library** (Elo, Glicko-2, TrueSkill) implemented from scratch in Rust.
- An **operational web tool** for league operators to manage players, record matches, and view leaderboards.
- A **developer demo** for algorithm exploration directly in the browser via WASM.
- A **library-level integration path** for AI swarm operators who run autonomous agents at scale.

The platform serves two distinct usage patterns under one codebase: interactive league management (human operators) and programmatic high-throughput match recording (swarm agents).

## 2. Target Users

### League Operator

Manages one or more competitive leagues. Creates leagues, adds players, records match results, and monitors leaderboards. Non-technical -- interacts exclusively through the web UI. Primary needs: match entry, leaderboards, bulk import.

### Swarm Operator

Runs autonomous AI agents that compete against each other. Links directly against the `ladder-rs` library crate from their own Rust code. Writes match results to the shared database via the library's persistence layer. Needs aggregate performance dashboards (win rates, active agents, match volume). Technical -- comfortable with Rust and database tooling. **Note (RQ-R3-2):** Swarm Operators hold the League Operator role within the platform's three-role model (Admin, League Operator, Player/Viewer). No separate fourth role exists.

### Developer / Evaluator

**Eliminated from v1 (v1.2).** Originally planned as users of a static WASM demo playground. The operational frontend serves as the representative experience instead. WASM bindings remain available as a library artifact for developers who want to integrate directly.

## 3. Product Components

### 3.1 Library Crate (`ladder-rs`)

**Status: DONE (rating math). Persistence layer: NOT STARTED.**

Rust library implementing Elo, Glicko-2, and TrueSkill from scratch. Exposes a `RatingSystem` trait with associated types for ratings, outcomes, and configuration. Supports 1v1, team-based, and N-player ranked matches.

The library owns all database interaction through a persistence layer. Both the backend service and swarm operator consume the library for DB access -- there is no broker or separate write path.

Key modules: `src/elo.rs`, `src/glicko.rs`, `src/trueskill.rs`, `src/core.rs`.

Features: algorithm selection (`elo-only`, `glicko-only`, `trueskill-only`, `all-algorithms`), math backend selection (`libm-math` vs `statrs-math`), optional `rayon` parallelism.

### 3.2 WASM Bindings (`ladder-rs-wasm`)

**Status: DONE.**

WebAssembly bindings exposing all three algorithms to JavaScript/TypeScript. Located in `wasm/`. Bundle size ~266 KB uncompressed (target: 300 KB per ADR). Includes browser compatibility detection, TypeScript type definitions, and performance regression tests.

### 3.3 Server Backend

**Status: NOT STARTED.**

Thin REST wrapper over the library crate. The server has no direct DB access -- all persistence goes through the library's persistence layer.

Architecture: `Frontend -> Backend (REST) -> Library -> DB`

Responsibilities:
- League CRUD
- Player CRUD (within leagues)
- Season management (algorithm type changes trigger new seasons; parameter changes update in place)
- Match result recording (1v1 and N-player ranked events)
- Algorithm-aware input validation
- Leaderboard computation
- Rating history retrieval
- Swarm statistics aggregation
- Batch match entry (UI-driven workflow)
- Admin match correction (audited)
- Player aliasing management

API contract (v1.2):
- Both cursor-based and offset-based pagination (cursor for sequential traversal, offset for random access to rank positions)
- Structured machine-readable error responses with error codes and field-level validation messages
- Server-side filtering and sorting
- OpenAPI specification deferred to post-v1

### 3.4 Operational Frontend

**Status: NOT STARTED.**

Web application for league operators and swarm monitoring. Communicates with the server backend via REST. Framework to be determined during architecture phase.

Views:
- League management (create, edit, archive, un-archive)
- Season management (view seasons within a league, start new season on algorithm/parameter change)
- Player management (add, remove, view per-league)
- Match entry form (adapts to selected algorithm: show/hide draw option, ranked placement UI for N-player events)
- Leaderboard (current rankings per league per season, sortable)
- Rating history charts (per-player over time, scoped to season)
- Swarm dashboard (see Section 4.7)
- Batch match entry workflow (UI-driven, not file upload)

### 3.5 Developer Demo

**Status: ELIMINATED (v1.2).**

Originally planned as a minimal static site loading WASM bindings for algorithm exploration. Eliminated during requirements elicitation -- the operational frontend (Section 3.4) serves as the representative experience. WASM bindings (Section 3.2) remain as a library artifact but do not require a standalone UI.

## 4. Functional Scope -- v1

### 4.1 League CRUD

- Create a league with a name, description, selected rating algorithm (Elo, Glicko-2, or TrueSkill), and a visibility setting (public or private; default: public). Algorithm selection provides sensible default parameters (presets) with the ability to customize within guardrailed min/max ranges. *(Added v1.2; visibility added RQ-R3-7)*
- Edit league metadata (including visibility setting).
- Archive a league (frozen -- no new matches or seasons can be created; all data remains readable).
- Un-archive a league (resumes write access).
- List leagues with status filtering (active/archived). Visibility rules apply: public leagues are visible to all authenticated users; private leagues are visible only to Admins, assigned League Operators, and Player/Viewers whose player record is a member. *(RQ-R3-7)*
- Seasons inherit archive state from their parent league.
- Archived league data is included in aggregate stats and dashboards. Archiving prevents writes, not reads.

### 4.2 Seasons

- A league contains one or more seasons. Creating a league starts the first season.
- Each season records: algorithm type, algorithm parameters, start date, and optional end date.
- Changing a league's algorithm **type** ends the current season and starts a new one. Changing parameters within the same algorithm type updates the current season in place without creating a new season. *(Revised v1.2: parameter-only changes no longer trigger season breaks.)*
- On season transition, the operator chooses: **(A)** reset all players to the new algorithm's defaults, or **(B)** seed initial ratings from the prior season's ordinal rankings (preserving relative ordering without raw value carryover). *(Added v1.2)*
- Mid-season joiners always start at the algorithm's defaults, regardless of whether existing players were seeded or reset. *(Added v1.2)*
- Each season has its own rating timeline and leaderboard. Players carry over across seasons.
- The league is the continuous container; seasons are rating-coherent units within it.

### 4.3 Player CRUD

- Add a player to a league (initializes default rating per the current season's algorithm).
- Remove a player from a league: soft-delete. Player becomes inactive (hidden from leaderboard, cannot participate in new matches), but all match history and rating snapshots are preserved. Full deletion is an admin-only edge case. *(Revised v1.2: clarifies soft-delete semantics.)*
- View player profile: current rating, match count, season overview (final rating per season as table/card), per-season detail chart (match-by-match progression). No cross-season combined chart -- scales are incompatible. *(Revised v1.2: clarifies rating history display.)*
- List players in a league with current ratings.
- Player type flag: `human` or `non-human` (swarm agents are regular players with a type flag).
- **Player aliasing (v1.2):** Two player records can be linked as aliases (additive -- both records persist). Aliasing triggers a full rating recalculation treating all aliased matches as one player's history. Aliases can be removed (triggers another recalc). True destructive merge is post-v1.
- **Auto-creation:** The library auto-creates players on first match reference (for swarm operator path). The UI requires explicit player creation before match entry. *(Added v1.2)*
- Player identity management lives in the application layer, not the library. The library handles persistence but not identity semantics.

### 4.4 Match Entry

- Manual match result submission via the UI: select league, select participants, record outcome.
- **1v1 matches:** win/loss/draw.
- **N-player ranked events:** 2-N participants with ranked placement outcomes. TrueSkill handles N-player directly; Elo/Glicko decompose into pairwise results internally.
- **Algorithm-aware validation:** Drawing in a TrueSkill league configured with `draw_probability=0` is rejected at the application layer. UI adapts to the selected algorithm (show/hide draw, show ranked placement for N-player).
- Match entry triggers immediate rating recalculation for involved players.
- Match results are append-only for normal users. An administrator-level audited override exists to correct submission errors. All corrections are logged (who, what, when). *(Revised v1.2: replaces strict immutability.)*
- **Duplicate rejection:** Matches with identical participants, outcome, and timestamp are rejected to prevent accidental double-submission. *(Added v1.2)*
- **Convergence quality:** TrueSkill approximation may not fully converge. The library returns a result type distinguishing "converged" from "best approximation." Non-converged results are recorded with a `convergence_quality` flag (degraded confidence) in the DB. Matches are never rejected for non-convergence.
- **Score metadata:** Match records include an optional score field for display purposes. Scores are not used in rating calculations for v1.

### 4.5 Batch Match Entry

*(Revised v1.2: UI-driven workflow replaces raw CSV/JSON file upload.)*

- UI-driven workflow for entering multiple matches in a single operation.
- The UI handles player resolution, validation feedback, and confirmation before committing.
- Errors are reported per-entry with interactive correction (not a silent per-row report).
- Non-convergence on individual matches does not abort the batch -- flagged and continued.
- Matches are processed sequentially in entry order (order matters for rating calculation).
- The specific UI mechanism (multi-match form, paste-from-spreadsheet, etc.) is determined during architecture/design.

### 4.6 Leaderboards

- Current rankings per league per season, ordered by rating (descending).
- Display: rank, player name, current rating, rating deviation/uncertainty where applicable, match count.
- Sortable columns.

### 4.7 Swarm Dashboard

- **Rating distribution histogram:** distribution of current ratings across agents.
- **Rating velocity:** rate of rating change over time per agent.
- **Match volume over time:** matches recorded per time period (hour/day/week).
- **Top/bottom N agents:** highest and lowest rated agents.
- **Agent lifecycle:** when agents started competing, total matches played, current status.
- **Win rate by rating bucket:** win percentage grouped by rating ranges.
- **Anomaly detection:** flag unusual patterns (sudden rating spikes/drops, abnormal match volumes).
- "Active agent" is derived from match timestamps using a configurable per-league recency threshold (operator-set, with a server-defined default). An agent is active if it has at least one match within the threshold window. The threshold is surfaced as a UI filter. Heartbeat-based connectivity tracking is a post-v1 spike. *(RQ-R3-8)*
- Read-only view -- swarm operators write data via the library crate, not via this UI.

### 4.8 Developer Demo

**Eliminated (v1.2).** See Section 3.5.

## 5. Data Model (Key Entities)

| Entity | Key Fields |
|---|---|
| League | id, name, description, status (active/archived), created_at |
| Season | id, league_id, algorithm, algorithm_params, start_date, end_date (nullable) |
| Player | id, name, player_type (human/non-human), created_at |
| LeaguePlayer | league_id, player_id, joined_at |
| Match | id, season_id, recorded_at, convergence_quality (converged/degraded), score_metadata (optional JSON), is_corrected (boolean) |
| MatchAuditLog | id, match_id, changed_by, changed_at, before_state (JSON), after_state (JSON) |
| PlayerAlias | primary_player_id, alias_player_id, created_at |
| MatchParticipant | match_id, player_id, placement (1-N for ranked, 1/2 for win/loss), is_draw (boolean) |
| RatingSnapshot | id, match_id, player_id, season_id, rating, deviation, uncertainty, timestamp |

The match model supports both 1v1 and N-player ranked events through the MatchParticipant table with a `placement` field.

## 6. Non-Functional Requirements

### Performance

- Single match rating update: < 1ms for Elo, < 5ms for Glicko-2, < 10ms for TrueSkill (library crate, measured on commodity hardware).
- REST API response time: < 100ms for single-entity operations, < 500ms for leaderboard queries with up to 10,000 players.
- Bulk import: process at least 1,000 matches/second.

### Bundle Size (WASM)

- WASM binary: <= 300 KB uncompressed (per established ADR).
- Compressed wire size: ~130 KB gzip, ~110 KB brotli.

### Scale Expectations (v1)

- Single-tenant deployment.
- Up to 100 leagues, 10,000 players per league, 1M total matches.
- SQLite is the persistence layer -- adequate for this scale.
- Concurrency: SQLite WAL mode + `busy_timeout`. No broker. Both server and swarm operator access DB through the library's persistence layer.

### Database Portability

- Persistence layer uses `sqlx` (supports SQLite and PostgreSQL).
- No full repository pattern -- thin abstraction only.
- All queries written to be portable between SQLite and PostgreSQL from v1, even though only SQLite is supported in v1.

### Reliability

- No data loss on crash -- SQLite provides ACID transactions.
- Graceful error reporting for malformed input (API returns structured error responses).

### Portability

- Server: Linux primary, macOS for development.
- WASM demo: Chrome, Firefox, Safari, Edge (latest two versions).
- Library crate: any platform Rust targets.

## 7. Out of Scope for v1

- ~~**Multi-tenancy and authentication.** Single deployment = single org. No user accounts, no RBAC.~~ **Reversed in v1.2 (RQ-R2-1):** User accounts and RBAC are now in scope for v1. Three roles: Admin (global), League Operator (league-scoped), Player/Viewer (read-only). Multi-tenancy remains out of scope.
- **gRPC API.** REST only.
- **Tournament brackets / swiss-system pairings.** Matches are recorded, not scheduled.
- **Matchmaking queue.** The library computes ratings; it does not pair players for future matches.
- **Educational content.** The developer demo is a playground, not a tutorial.
- **Destructive player merge.** Aliasing is v1; true merge (deleting provisional records) is post-v1.
- **OpenAPI/Swagger specification.** Deferred to post-v1.
- **Real-time updates (WebSockets).** Leaderboards refresh on request.
- **PostgreSQL deployment.** Queries are portable but only SQLite is supported for v1.
- **Mobile-specific UI.** Responsive design is acceptable but no native mobile app.
- **Internationalization.**
- **Self-service forgot-password / email-based password reset.** Deferred to post-v1 (RQ-R3-4). Admin can set a temporary password for any account; that user must change it on next login.
- **Agent heartbeat / connectivity tracking.** Real-time agent connectivity status derived from heartbeats rather than match timestamps is deferred to post-v1 (RQ-R3-8). The v1 "active agent" definition is match-recency-based with a configurable per-league threshold.
- **Score-based rating calculations.** Scores are stored as metadata for display only. Score-weighted rating extensions are a future research spike.
- **Public publishing.** No crates.io, npm, or hosted demo for v1.
- **In-app email delivery.** Invite links (player-to-account linking) and password reset notifications are copy-paste URLs only. No SMTP or email SaaS infrastructure in v1. Deferred to post-v1.
- **Client-side rating preview via WASM.** The React frontend uses REST only. Computing ratings in the browser before submission using the `ladder-rs-wasm` bindings is a post-v1 capability spike. *(Added v1.3 — see ADR-0002)*

## 8. v1 Deliverables

- **GitHub release:** compiled binary + README.
- **Docker Compose:** frontend container (nginx + React build), backend container (Axum + SQLite volume). Deployment via `docker compose up`. *(Revised v1.3 — see ADR-0007)*
- **Library crate:** consumed internally by the server and swarm operators. Not published to crates.io.
- **WASM bindings:** library artifact, not deployed as a standalone demo. *(Revised v1.2)*

## 9. Open Decisions

These items are deferred to the architecture phase or later:

1. **CLI role.** Unclear if a CLI is needed for v1. The frontend covers league management; the swarm operator uses the library directly. If included, the CLI would use the library's persistence layer. Architecture phase should evaluate.

2. **WASM PlayerManager fate.** Resolved (v1.2): With the developer demo eliminated, the PlayerManager remains as a WASM binding utility but is not a required component of the operational architecture.

3. **"Active agent" definition.** RESOLVED (RQ-R3-8): Match-based recency threshold, configurable per league by the operator. Default value determined by architecture. Heartbeat-based connectivity is post-v1.

## 10. Open Questions for Architecture

All architecture questions are now resolved. *(Updated v1.3 — architecture phase complete)*

1. **Frontend framework selection.** RESOLVED (ADR-0002): React 18 + TypeScript + Vite. Largest charting ecosystem for swarm dashboard and rating history views; mature build tooling; deploys as static files served by nginx.

2. **DB schema design.** RESOLVED: Full schema defined in `docs/architecture/data-architecture.md`. Conservative rating pre-computed in `rating_snapshots.conservative_rating` column; compound indexes for leaderboard and history queries at 10K player / 1M match scale.

3. **Server framework selection.** RESOLVED (ADR-0003): Axum. tower ecosystem alignment (tower-sessions, tower-http), tokio-native, extractor ergonomics, IntoResponse for structured errors.

4. **Deployment strategy.** RESOLVED (ADR-0007): Docker Compose, two containers (nginx frontend + Axum backend) + one named volume (SQLite). `docker compose up` is the deployment command. Single-container assumption from v1.1 is reversed.

5. **Developer demo hosting.** RESOLVED (v1.2): Developer demo eliminated. No hosting decision needed.
