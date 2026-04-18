# Theory of Operation

**ladder-rs Platform**
**Date:** 2026-04-15

---

## Purpose of This Document

The Theory of Operation explains how the ladder-rs platform works as a coherent system — not just what each part does, but why the architecture is shaped the way it is, how the pieces fit together under load, and what operational behavior to expect. This is the narrative document that ties the product vision to the technical design.

---

## The Core Insight: One Codebase, Two Access Patterns

The platform serves two fundamentally different usage patterns from the same codebase:

**Interactive league management** — A human League Operator opens a browser, records a match result, and checks who is at the top of the leaderboard. This interaction requires a web UI, a REST API, authentication, and a responsive user experience. Latency matters because a human is waiting.

**Programmatic swarm recording** — An autonomous AI agent experiment generates hundreds of match results per second in a tight loop. The swarm operator writes these directly to the database via a linked library crate, bypassing HTTP entirely. Throughput matters because the operator does not want an HTTP server to be the bottleneck.

The architectural decision that enables both patterns simultaneously is the placement of the `ladder-rs-persistence` crate. It lives between the math library and the HTTP server, providing a stable Rust API that any tokio-native process can call directly. The HTTP server is a consumer of this library — not the exclusive gatekeeper to the database.

This means the swarm operator's Rust process and the Axum backend process are peers. They share the SQLite file through WAL mode concurrency, and both use the same match recording code path (with all its business rule checks: duplicate detection, season write protection, player auto-creation). The REST API adds authentication and HTTP framing but adds no logic that is unavailable to the library consumer.

---

## Rating Math as a Pure Library

The `ladder-rs` crate is the foundation. It implements Elo, Glicko-2, and TrueSkill from scratch in Rust with no I/O, no async, no network dependencies. It is a pure computation library that takes ratings and match outcomes and returns new ratings.

This purity is intentional and permanent. The library can be used from a web server, a CLI tool, a WASM module, a batch processor, or a background job — without any changes. It can be tested in isolation with zero infrastructure. Its correctness does not depend on anything that can fail at runtime.

The persistence crate calls the library synchronously within database transactions. The match recording transaction opens, loads the current ratings, calls the library to compute new ratings, inserts the results, and commits — all in one atomic unit. If the library computation fails (which it should not for Elo and Glicko-2; TrueSkill may not fully converge), the transaction still commits with a `convergence_quality = "degraded"` flag. No match is ever rejected for non-convergence.

---

## Seasons as Rating Contexts

The season model is central to how the platform organizes ratings. A season is not just a time period — it is a complete rating context defined by a specific algorithm and its parameters. Ratings in one season are not directly comparable to ratings in another season (the algorithms and scales may differ).

When a league operator changes the algorithm type (e.g., from Elo to TrueSkill), the current season ends and a new one begins. The operator can choose to seed the new season from the old season's ordinal rankings (preserving who was better than whom without carrying over raw values) or to reset everyone to defaults. Mid-season joiners always start at defaults — seeding only applies at season transitions.

This model means the leaderboard always shows ratings that are internally consistent. A TrueSkill leaderboard shows TrueSkill ratings for a TrueSkill season. There is no mixed-algorithm leaderboard. The rating history chart shows a player's progression within a single season, and the season overview shows their final rating per season — never a cross-season combined chart, because the scales are incompatible.

---

## The Leaderboard and Conservative Estimation

A naive rating system ranks players by their raw mean rating. This is problematic for new or inactive players: a brand-new player with a high initial mean but enormous uncertainty should not outrank an established player who has played hundreds of games and consistently performed well.

The platform uses algorithm-specific conservative estimates as the ranking metric:
- Elo: raw rating (Elo has no uncertainty component)
- Glicko-2: `mu - 2 * RD` (penalizes high rating deviation)
- TrueSkill: `mu - 3 * sigma` (penalizes high standard deviation)

These values are pre-computed and stored in the `rating_snapshots.conservative_rating` column at the time of each match update. The leaderboard query simply retrieves the most recent snapshot per player and sorts by `conservative_rating DESC`. This avoids per-query arithmetic over potentially millions of rows, keeping leaderboard queries fast even at 10,000 players.

The practical effect: new players start with very high uncertainty and rank near the bottom even if their initial mean rating is at the default. As they play matches and their uncertainty decreases, they climb or fall to their true position. The leaderboard rewards demonstrated, consistent performance.

---

## Match Corrections and Eventual Consistency

Match corrections are the most architecturally complex feature in the platform. When an Admin corrects a match, the ratings for everyone in that season must be recomputed from scratch — because rating algorithms are stateful, and a change to an earlier match propagates through all subsequent matches.

Doing this synchronously would block the HTTP response for potentially seconds (hundreds or thousands of matches to replay) and risk timeouts. The platform instead records the correction immediately (atomic DB write, audit log), queues a background recalculation job, and returns 202 Accepted to the caller.

The background job poller runs in the same backend process as a tokio task, checking the `recalculation_jobs` table every few seconds. It claims one job at a time using an atomic SQL UPDATE with a WHERE clause. When it picks up a job, the Recalculation Worker replays every match in the season in chronological order, treating aliased player records as one player, computing new ratings at each step, and accumulating the results. At the end, a single SQLite transaction atomically replaces all the old rating snapshots with the new ones and marks the job as completed.

During the window between the correction being recorded and the recalculation completing, the leaderboard serves stale (pre-correction) ratings. The UI should display a "recalculation in progress" indicator so users understand the data is temporarily inconsistent. If the recalculation fails for any reason, the stale ratings remain intact — the system is never left without a leaderboard.

---

## Authentication and the Role Model

Authentication in the platform follows a conventional session-cookie model: the user logs in, receives a session cookie, and presents the cookie on every subsequent request. The session is stored in the SQLite database — not in memory — so it survives server restarts.

The three-role model (Admin, League Operator, Player/Viewer) is deliberately simple. Admin is a global superpower role. League Operator is a scoped role — a user can be an operator for league A but not league B. Player/Viewer is read-only and is the default for all new accounts. Swarm Operators hold the League Operator role for the leagues they manage; there is no separate technical role.

The first time the server starts against an empty database, it bootstraps an admin account and prints the plaintext credentials to stdout (visible via `docker logs backend`). This is the only time plaintext credentials exist in the system. The admin is forced to change this password on first login. All subsequent accounts are created through normal registration or admin-set temporary passwords.

Player-to-account linking is done through one-time invite tokens. A League Operator generates a token for a player record; the token is displayed as a copy-paste URL. The player opens the URL, logs in (or registers), and their account is linked to the player record. This is the only mechanism for account-to-player linking in v1 — no email delivery infrastructure is needed.

---

## SQLite as the Right Choice

SQLite is the platform's only storage layer in v1. This is a deliberate constraint, not a limitation:

- **Single-process access pattern fits well.** The backend and swarm operator are the only two writers. SQLite's WAL mode handles this with minimal contention.
- **No operational overhead.** No separate database server to deploy, monitor, or back up. The database is a file on a Docker volume.
- **Adequate scale.** At 100 leagues, 10,000 players per league, and 1 million total matches, SQLite performs well within its design envelope. The leaderboard query target of < 500ms at 10K players is achievable with the indexed `conservative_rating` column.
- **Portability preserved.** All queries are written using `sqlx` with standard SQL syntax. The persistence crate does not use any SQLite-specific syntax that would prevent migration to PostgreSQL in the future. When scale eventually demands a more capable database, the migration path is a `DATABASE_URL` change and a schema port.

The `busy_timeout` setting (default 5 seconds) is the safety valve. If two writers collide, one waits up to 5 seconds for the write lock. At the expected write rates for v1, this timeout should almost never be reached.

---

## Deployment Philosophy

The deployment topology (two containers + one named volume) is simpler than a single binary with embedded assets and more extensible than a monolithic single-container approach.

Nginx handles static file serving efficiently — it is purpose-built for this and provides correct cache headers, gzip compression, and `try_files` routing for the React SPA. Trying to replicate this from a Rust binary would require additional complexity with no benefit.

The Axum backend container holds no state other than in-flight requests. All persistent state is in the SQLite volume. This means the backend container can be stopped, updated (with a new binary running migrations on startup), and restarted without losing data. The volume is the durable unit.

The two-container split also makes the future clearer: if the platform ever needs to scale the backend independently, or replace SQLite with PostgreSQL, or add a second backend instance, the architectural pieces are already separated. A single-container monolith would require more surgery to evolve.

---

## What Is Not in v1 and Why

Several features are explicitly deferred and their absences are architecturally intentional:

**No email infrastructure.** Invite links are copy-paste URLs. Adding email requires an SMTP server or SaaS email API, external credentials to manage, deliverability configuration, and HTML email templates. None of this is essential to the core value proposition of rating management. Post-v1, when account self-service is needed, email can be added as an independent layer.

**No WebSockets / real-time updates.** Leaderboards refresh on request. Adding real-time updates would require either server-sent events or WebSockets, connection management, and state synchronization logic. At the current scale, polling on navigation is adequate. Post-v1, if live leaderboard updates become important, this can be layered on top of the existing REST infrastructure.

**No WASM in the frontend.** The React frontend uses REST only. The WASM bindings exist as a library artifact and a future capability spike. Using WASM in the frontend for client-side rating preview would require bundling the WASM module, managing async loading, and ensuring browser compatibility. The REST API provides the same computed results without this complexity. Post-v1, if client-side rating preview is valuable, the WASM path is ready.

**No PostgreSQL.** SQLite is adequate for v1 scale. The portability requirement (NFR-PORT-001) and the sqlx abstraction ensure that PostgreSQL adoption later is a migration, not a rewrite.

---

## Operational Invariants

These properties must hold at all times in a correctly operating deployment:

1. **Every match in the database has a rating snapshot for every participant.** The atomic transaction in `Match Repository.record_match` ensures this. Partial state (match recorded, no snapshot) is impossible under correct operation.

2. **Every player's most recent rating snapshot reflects their actual performance.** This invariant is temporarily violated during the window between a match correction and the completion of asynchronous recalculation. The `recalculation_jobs.status` field tracks this window.

3. **The `conservative_rating` column is always consistent with `rating`, `deviation`, and `uncertainty`.** It is computed at insert time in the Rating Engine Bridge using the same formula as the leaderboard sort.

4. **No in-progress recalculation job survives a server restart.** The startup recovery step resets all `in_progress` jobs to `queued`. The next poll cycle picks them up and reruns them. This may result in duplicate recalculation work but never in lost jobs.

5. **The admin user always exists.** Once bootstrapped, the admin account is never automatically deleted. Admin account management is a manual operator concern.

6. **A closed season accepts no new matches.** The season write protection check in `Match Repository` enforces this. Matches can only be corrected (by Admin), not added.
