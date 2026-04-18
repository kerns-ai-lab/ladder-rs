# ADR-0005: Asynchronous Recalculation via DB-Backed Poll Loop

**Status:** Accepted
**Date:** 2026-04-15
**Deciders:** Dustin Kerns

---

## Context

Two platform operations trigger full-season rating recalculations:

1. **Admin match correction** (UR-ADM-001): When an Admin corrects a historical match, all subsequent matches in the season must be replayed in order to produce correct ratings for all players.

2. **Player alias operations** (UR-PM-002): Linking or unlinking two player records causes the platform to treat their match histories as belonging to one player. All seasons in which either player participated must be replayed.

A full-season recalculation involves: loading all matches in the season in chronological order, iterating through each match, calling the rating algorithm for each one, and accumulating new rating snapshots. For a season with thousands of matches and hundreds of players, this can take seconds.

If this work were performed synchronously (blocking the HTTP handler), the API response would be delayed for the full recalculation duration. This violates the responsiveness requirement (a human is waiting for the 202/correction confirmation) and risks HTTP timeouts.

The platform must also remain crash-recoverable. A recalculation job that was in progress when the server crashed must not be silently lost — it must be detected and re-queued on the next startup.

Constraints:
- The platform uses SQLite as its only infrastructure (ADR-0007). No Redis, no RabbitMQ, no external job queue.
- The server runs as a single binary in Docker Compose. Multi-process coordination is not a requirement for v1.

---

## Decision

Implement asynchronous recalculation using a **DB-backed poll loop**: a `recalculation_jobs` table in SQLite acts as a persistent job queue. A long-lived tokio background task polls this table every 1–5 seconds, claims one job at a time via an atomic SQL UPDATE, and executes the recalculation.

Key properties:
- Jobs are persisted immediately when a correction or alias operation is submitted.
- The HTTP handler returns 202 Accepted with a job ID before the recalculation begins.
- On server startup, `in_progress` jobs are reset to `queued` (crash recovery).
- During recalculation, old rating snapshots remain visible to readers (eventual consistency).
- On completion, a single SQLite transaction atomically replaces old snapshots with new ones.

---

## Rationale

### Crash recovery is required

A recalculation job that fails mid-flight must not silently disappear. The `recalculation_jobs` table gives the job a durable existence that survives server crashes. On startup, the poller calls:

```sql
UPDATE recalculation_jobs SET status = 'queued', started_at = NULL
WHERE status = 'in_progress';
```

This is the only crash recovery mechanism needed. No distributed coordination, no lock expiry, no heartbeat. The job is simply re-queued and replayed from scratch.

### No external infrastructure

The platform's deployment constraint is Docker Compose with SQLite (ADR-0007). Introducing Redis, RabbitMQ, Postgres, or any external queue would require a third container, operational credentials, and deployment complexity. All of this complexity is avoided by using the existing SQLite database as the job store.

SQLite's WAL mode serializes writes, which means the atomic job claim (UPDATE ... WHERE status = 'queued' ... RETURNING) is safe without explicit locking — only one claim can succeed per UPDATE execution.

### Polling overhead is negligible

Polling a single-row query against an indexed column (`idx_recalc_jobs_status`) every 3 seconds adds negligible load. The query executes in microseconds. At expected usage rates (corrections are rare events, not high-frequency operations), the job queue is almost always empty, and the poll is a no-op index scan.

### One job at a time eliminates race conditions

The poller processes one job at a time. It claims the oldest queued job, executes it to completion, then polls for the next. This serial processing simplifies the recalculation logic: the worker does not need to coordinate with other workers, handle partial results, or worry about two recalculations modifying the same season concurrently.

If multiple corrections are queued for the same season, they run sequentially in trigger order. Each recalculation replays the entire season from scratch, so later recalculations automatically incorporate the results of earlier corrections.

### Status visibility for the UI

The `recalculation_jobs` table exposes job status to the API (`GET /api/jobs/{id}`). The UI can poll this endpoint after a correction to display "recalculation in progress" or "complete" state. `Job Repository.is_pending_for_season(season_id)` enables the leaderboard view to show a staleness warning.

---

## Alternatives Considered

### `tokio::spawn` only (in-memory task)

Spawn a tokio task when a recalculation is triggered. The task runs asynchronously without persisting any job state.

**Rejected:** This approach is not crash-recoverable. If the server crashes or is restarted during recalculation, the in-progress work is lost. The correction is recorded in the DB (the match and audit log are committed), but the recalculation is never retried. The leaderboard shows stale, incorrect ratings indefinitely. This violates the crash recovery requirement (NFR-REL-001) and SR-PER-009's acceptance criterion that "upon successful completion, the recalculated ratings atomically replace the stale ratings" — which implies recovery when successful completion does not occur.

An in-memory approach also has no status visibility — the API cannot tell callers whether a recalculation is running.

### External job queue (Redis, RabbitMQ, Celery, etc.)

Use an external message queue for job dispatch.

**Rejected:** This introduces a third infrastructure component into the Docker Compose deployment. Redis or RabbitMQ would need to be configured, monitored, and backed up. If the queue loses the job (crash, misconfiguration), the same crash recovery problem exists but is now harder to solve. The operational overhead is not justified for the expected job frequency (corrections are rare admin operations, not high-throughput events).

The platform's explicit constraint is SQLite-only infrastructure for v1 (product spec Section 6). An external queue contradicts this.

### Synchronous recalculation (blocking HTTP response)

Execute the full recalculation within the HTTP request handler and return when complete.

**Rejected:** For large seasons (thousands of matches), this could take seconds. HTTP timeouts (typically 30–60 seconds in nginx + browser) could expire before completion. The user experience is unacceptable — the operator sees a spinning UI for several seconds with no feedback. There is also no mechanism to report progress or allow the user to navigate away and check back.

Synchronous recalculation is not feasible at scale and contradicts the asynchronous recalculation requirement (SR-PER-009).

### Database trigger + SQLite WAL hook

Use a SQLite trigger on `match_audit_log` insertion to insert a job record, combined with a WAL hook to wake the poller immediately.

**Rejected:** SQLite triggers fire in the writing transaction context. A trigger that inserts into `recalculation_jobs` is fine (already done in the Match Handler / Job Repository). However, SQLite WAL hooks are a C-level API not exposed by sqlx. The polling approach already provides an adequate response time (1–5 second latency) for a user-initiated admin operation. Adding WAL hook complexity is not warranted.

---

## Consequences

### Positive

- Crash recovery is automatic and requires no distributed coordination
- No external infrastructure (SQLite-only, consistent with platform constraints)
- The HTTP handler returns immediately with a job ID (responsive user experience)
- Job status is queryable by the UI for progress indication
- Multiple pending corrections for the same season are serialized automatically

### Negative / Accepted Trade-offs

- **Eventual consistency window.** Between the 202 Accepted response and the recalculation commit, the leaderboard serves stale (pre-correction) ratings. This window is bounded by the polling interval (1–5 seconds) plus the recalculation duration. For large seasons, this could be tens of seconds. The UI must display a staleness warning during this window.
- **Duplicate recalculation on restart.** If the server crashes during a recalculation, the job is reset to `queued` on the next startup and the entire recalculation runs again from scratch. This is safe (idempotent — the final atomic snapshot replacement is correct) but wastes computation for an already-partially-completed job. For v1 recalculation frequencies (rare admin events), this is acceptable.
- **Polling idle overhead.** A 3-second poll interval means the poller makes ~28,800 DB queries per day even when no jobs are queued. Each is a fast indexed read, but it is not zero. Post-v1, a condition variable or notification mechanism could reduce idle polling.
- **Single worker.** One job runs at a time. If a season has an enormous number of matches, recalculations for other seasons in the queue must wait. This is a reasonable simplification for v1 where recalculations are rare.
