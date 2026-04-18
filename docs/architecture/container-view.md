# Container View — C4 Level 2

**ladder-rs Platform**
**Date:** 2026-04-15

---

## Overview

The Container View shows the runtime deployable units inside the ladder-rs platform boundary, the technology choices for each, and how they communicate. At this level, internal crates that are not independently deployable are shown as logical components inside their host container.

---

## Deployment Topology

The platform deploys via Docker Compose as two containers sharing one named volume:

```
Host machine
├── docker-compose.yml
│
├── [Container: frontend]
│   ├── nginx:alpine
│   ├── React/TypeScript static build (dist/)
│   └── nginx.conf  (proxies /api/* to backend:3000)
│
├── [Container: backend]
│   ├── Rust binary (ladder-rs-server)
│   ├── [Crate: ladder-rs-persistence]  — library, not a separate process
│   └── [Crate: ladder-rs]              — library, not a separate process
│
└── [Volume: db_data]
    └── /data/ladder.db  (SQLite file, WAL mode)
```

Port exposed to the host: `8080 → frontend:80`. All browser traffic enters through nginx. Backend is not directly exposed to the host.

---

## Containers

### Container 1: React Frontend (nginx)

**Technology:** nginx:alpine, React 18, TypeScript, Vite

**Responsibility:** Serves the compiled React single-page application as static files and acts as a reverse proxy for all `/api/*` requests, forwarding them to the Axum backend container.

**Runtime behavior:**
- Nginx serves `index.html` and compiled JS/CSS bundles for all non-API paths (client-side routing).
- Nginx proxies `/api/*` to `backend:3000` (internal Docker network), stripping the `/api` prefix before forwarding.
- No server-side rendering. No WASM loading. Purely static file serving + proxy.

**Build process:** `npm run build` in `frontend/` produces `dist/`. The Docker image copies `dist/` into nginx's document root.

**Does NOT use:** `ladder-rs-wasm` bindings. The frontend communicates exclusively via REST. WASM usage is a post-v1 capability spike (see ADR-0002).

**Connections:**
- Inbound: Browser → nginx:80 (HTTP/HTTPS)
- Outbound: nginx → backend:3000 (HTTP, internal Docker network, for `/api/*` only)

### Container 2: Axum Backend (backend)

**Technology:** Rust, Axum, tokio, tower-sessions, sqlx

**Responsibility:** The REST API server. Handles all authenticated API requests, enforces authorization rules, drives match recording and season management, and runs the background recalculation job poller. Has no direct SQL query code — all database operations go through `ladder-rs-persistence`.

**Runtime components (crates compiled into this binary):**

- **ladder-rs-server** — Axum router definitions, HTTP handler functions, auth middleware, background task spawning. This is the executable binary's root crate.
- **ladder-rs-persistence** — The exclusively-owned database access layer. Owns the sqlx `SqlitePool`, runs schema migrations on startup, and exposes repository functions. All SQL lives here.
- **ladder-rs** — Pure rating math (Elo, Glicko-2, TrueSkill). No I/O, no async. Called by `ladder-rs-persistence`'s Rating Engine Bridge component to compute new ratings after match recording.

**Background tasks:** On startup, the server spawns a tokio background task that polls `recalculation_jobs` every 1–5 seconds. Stuck `in_progress` jobs from a prior crash are reset to `queued` before polling begins.

**Connections:**
- Inbound: nginx → backend:3000 (HTTP, `/api/*` proxied requests)
- Inbound: Swarm Operator Process → SQLite file (through shared Docker volume, not HTTP — see below)
- Outbound: backend → SQLite volume at `/data/ladder.db` (via sqlx connection pool)

**Environment variables consumed:** `DATABASE_URL`, `SESSION_SECRET`, `SESSION_EXPIRY_SECONDS`, `ADMIN_BOOTSTRAP_ENABLED`

### Volume: db_data

**Technology:** SQLite 3.x, WAL mode, `busy_timeout` configured

**Responsibility:** Durable, crash-safe storage for all platform data. Not a separate process — SQLite is a file-format database accessed directly by the process(es) holding a connection.

**Mount points:**
- backend container: `/data/ladder.db`
- Swarm Operator Process (external): mounts the same file path, or more commonly runs on the host and accesses the volume path directly, depending on deployment topology

**Concurrency model:** SQLite in WAL mode allows one writer and multiple concurrent readers. The backend's sqlx connection pool and the Swarm Operator's connection pool can both operate concurrently. Writers queue behind each other using `busy_timeout`. This is not a remote database — both processes must have filesystem access to the SQLite file.

---

## External Process: Swarm Operator

**Technology:** Rust (external; not owned by this platform), `ladder-rs-persistence` (linked as a library)

**Responsibility:** The Swarm Operator's own Rust process, embedding `ladder-rs-persistence` to write match results directly. This process maintains its own `SqlitePool` pointing at the same SQLite file as the backend container.

This is not a container in the platform deployment — it is an external actor that depends on a platform crate. It is shown here because it shares the SQLite volume and influences concurrency requirements.

**Connection:** Swarm Operator Process → SQLite file (direct filesystem access, WAL concurrent reader/writer)

---

## Communication Map

| From | To | Protocol | Path / Note |
|------|----|----------|-------------|
| Browser | nginx (frontend) | HTTP/HTTPS | Port 8080 on host |
| nginx (frontend) | Axum (backend) | HTTP (Docker internal) | `/api/*` proxy |
| Axum (backend) | SQLite (volume) | sqlx (file I/O) | `/data/ladder.db` |
| Swarm Operator Process | SQLite (volume) | sqlx (file I/O) | Shared volume path |
| `ladder-rs-server` | `ladder-rs-persistence` | Rust function calls (in-process) | Library dependency |
| `ladder-rs-persistence` | `ladder-rs` | Rust function calls (in-process) | Library dependency |

---

## Cargo Workspace Structure

```
ladder-rs/                  (workspace root)
├── ladder-rs/              — pure rating math (Elo, Glicko-2, TrueSkill), no DB deps, DONE
├── ladder-rs-persistence/  — sqlx + SQLite, all DB interaction, NEW
├── ladder-rs-server/       — Axum REST API, auth middleware, NEW
├── ladder-rs-wasm/         — WASM bindings, not used by frontend in v1, DONE
└── frontend/               — React/TypeScript/Vite, NEW
```

The `ladder-rs-wasm` crate remains in the workspace as a library artifact. It is built and kept current but is not linked into any runtime container in v1.

---

## Key Architecture Constraints

1. **No direct DB access in server crate.** `ladder-rs-server` imports `ladder-rs-persistence` and calls repository functions. It does not import `sqlx` directly.

2. **No HTTP in persistence crate.** `ladder-rs-persistence` has no Axum or HTTP dependency. It is a pure library crate usable from any async Rust context (server, swarm operator, CLI).

3. **ladder-rs stays pure.** The `ladder-rs` crate has no DB, no HTTP, no async dependencies. It is a pure computation library callable synchronously from any context.

4. **WASM not used by frontend.** The React frontend communicates with the backend via REST only. There is no WASM loading on the frontend in v1.

5. **WAL mode is mandatory.** Any deployment where a Swarm Operator process and the backend server access the same SQLite file requires WAL mode and `busy_timeout` to prevent serialization failures.

---

## Requirements Traceability

| Requirement | Container / Decision |
|-------------|----------------------|
| SR-PER-001 | `ladder-rs-persistence` as shared library; both server and swarm operator use it |
| SR-PER-009 | Background job poller in backend container |
| SR-AUTH-001 | Auth middleware in backend container; tower-sessions backed by SQLite volume |
| NFR-PERF-002 | SQLite WAL mode + indexed queries enable <500ms leaderboard at 10K players |
| NFR-REL-001 | SQLite ACID transactions; job poller crash recovery via startup reset |
| NFR-PORT-001 | sqlx used throughout persistence crate; queries written for SQLite/PostgreSQL portability |
| NFR-SEC-001 | Login rate limiting in backend auth middleware layer |
| NFR-SEC-002 | Session cookies HttpOnly/Secure/SameSite=Strict via tower-sessions |
| ADR-0003 | Axum chosen for backend; see component view |
| ADR-0004 | Four-crate workspace structure |
| ADR-0007 | Docker Compose two-container topology |
