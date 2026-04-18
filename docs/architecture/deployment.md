# Deployment View

**ladder-rs Platform**
**Date:** 2026-04-15

---

## Overview

The ladder-rs platform deploys as two Docker containers orchestrated by Docker Compose, sharing one named volume for the SQLite database. A host port exposes the frontend container to users; the backend container is not directly accessible from outside the Docker network.

---

## Docker Compose Topology

```
Host machine (port 8080 exposed)
│
│  docker-compose.yml
│
├── [network: ladder_net (bridge)]
│   │
│   ├── [service: frontend]
│   │   Image: custom nginx:alpine build
│   │   Ports: "8080:80" (host:container)
│   │   Volumes: (static build baked into image at build time)
│   │   Config: nginx.conf with /api proxy to backend:3000
│   │   Health check: GET http://localhost/ → 200
│   │
│   └── [service: backend]
│       Image: custom Rust binary build
│       Ports: (not exposed to host; only accessible via ladder_net)
│       Volumes: db_data:/data
│       Env: DATABASE_URL, SESSION_SECRET, SESSION_EXPIRY_SECONDS, ADMIN_BOOTSTRAP_ENABLED
│       Health check: GET http://localhost:3000/health → 200
│
└── [volume: db_data]
    Mounted at: /data/ladder.db (backend only)
    Driver: local (host filesystem)
```

---

## `docker-compose.yml`

```yaml
services:
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "8080:80"
    depends_on:
      backend:
        condition: service_healthy
    networks:
      - ladder_net
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost/"]
      interval: 10s
      timeout: 5s
      retries: 3

  backend:
    build:
      context: .
      dockerfile: Dockerfile.backend
    volumes:
      - db_data:/data
    environment:
      DATABASE_URL: "sqlite:///data/ladder.db"
      SESSION_SECRET: "${SESSION_SECRET}"
      SESSION_EXPIRY_SECONDS: "${SESSION_EXPIRY_SECONDS:-86400}"
      ADMIN_BOOTSTRAP_ENABLED: "${ADMIN_BOOTSTRAP_ENABLED:-true}"
      HTTPS_ENABLED: "${HTTPS_ENABLED:-true}"
    networks:
      - ladder_net
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

volumes:
  db_data:
    driver: local

networks:
  ladder_net:
    driver: bridge
```

---

## Service Definitions

### `frontend` Service

**Base image:** `nginx:alpine`

**Build process:** The frontend Dockerfile performs a two-stage build:

```dockerfile
# Stage 1: build the React app
FROM node:20-alpine AS builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ .
RUN npm run build

# Stage 2: serve with nginx
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf
```

**`nginx.conf`:**

```nginx
server {
    listen 80;
    root /usr/share/nginx/html;
    index index.html;

    # Serve static assets with cache headers
    location ~* \.(js|css|png|svg|ico|woff2?)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Proxy API requests to backend
    location /api/ {
        proxy_pass http://backend:3000/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_read_timeout 30s;
    }

    # Client-side routing: all non-API, non-asset paths return index.html
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

**Responsibilities:**
- Serve the React SPA for all non-API paths
- Proxy `/api/*` to the backend container on the internal Docker network
- Apply cache headers for static assets (JS/CSS bundles are content-hashed by Vite)
- Handle client-side routing by returning `index.html` for unknown paths

**CORS:** In production, all frontend requests and API requests share the same origin (both served via the host on port 8080). nginx's proxy makes this same-origin from the browser's perspective. No CORS headers are needed on the backend in production. In development (where the React dev server runs on a different port from the backend), the backend sets permissive CORS headers gated on a `CORS_ALLOW_ORIGIN` environment variable.

### `backend` Service

**Base image:** `debian:bookworm-slim` (or `scratch` for minimal binary, requires musl target)

**Build process:** The backend Dockerfile builds the Rust binary:

```dockerfile
# Stage 1: build the Rust binary
FROM rust:1.77-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY ladder-rs/ ./ladder-rs/
COPY ladder-rs-persistence/ ./ladder-rs-persistence/
COPY ladder-rs-server/ ./ladder-rs-server/
# ladder-rs-wasm is excluded from server build
RUN cargo build --release --bin ladder-rs-server

# Stage 2: runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ladder-rs-server /usr/local/bin/
CMD ["ladder-rs-server"]
```

**Binary behavior on startup:**
1. Read `DATABASE_URL` environment variable
2. Initialize `SqlitePool` with WAL pragmas
3. Run sqlx migrations (`sqlx::migrate!`)
4. Call `Job Repository.reset_stuck_jobs()` (crash recovery)
5. Check `ADMIN_BOOTSTRAP_ENABLED`; if true and `users` table is empty, run bootstrap sequence
6. Spawn background job poller tokio task
7. Bind Axum router to `0.0.0.0:3000`
8. Log `INFO: server ready on :3000`

**Health check endpoint:** `GET /health` returns `200 OK` with body `{"status":"ok"}`. This endpoint bypasses auth middleware and serves as the Docker health check and load balancer health probe.

---

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | SQLite path, e.g. `sqlite:///data/ladder.db` |
| `SESSION_SECRET` | Yes | — | 32+ byte random secret for session signing; must persist across restarts |
| `SESSION_EXPIRY_SECONDS` | No | `86400` | Session lifetime in seconds (default: 24 hours) |
| `ADMIN_BOOTSTRAP_ENABLED` | No | `true` | If `false`, skip bootstrap even on empty users table |
| `HTTPS_ENABLED` | No | `true` | Controls the `Secure` attribute on session cookies. Set to `false` in development to allow session cookies over HTTP. `HttpOnly` and `SameSite=Strict` are always set regardless of this value. |
| `CORS_ALLOW_ORIGIN` | No | — | If set, the backend allows CORS from this origin (development use) |
| `BUSY_TIMEOUT_MS` | No | `5000` | SQLite `busy_timeout` in milliseconds |
| `JOB_POLL_INTERVAL_SECS` | No | `3` | Background job poller interval |
| `RUST_LOG` | No | `info` | Log level filter (uses `tracing` crate format) |

**Secret management:** `SESSION_SECRET` must be set via the host environment or a Docker secrets mechanism. It must not be committed to version control. The `docker-compose.yml` references it as `${SESSION_SECRET}` — operators must set this in their shell or in a `.env` file (excluded from git via `.gitignore`).

---

## Volume Management

### `db_data`

The named volume stores the SQLite database file at `/data/ladder.db`. Docker manages the volume lifecycle independently from the container lifecycle. The file persists across container restarts and image updates.

**Backup:** Volume backup is not automated in v1. Operators should use `sqlite3 /data/ladder.db .backup` or copy the file while the server is stopped, or use SQLite's online backup API. A backup cronjob is a post-v1 operational concern.

**Upgrade path:** When updating the backend image, the new binary runs sqlx migrations on startup. Migrations are forward-only in v1. If a migration fails, the server exits and the old data is intact.

---

## Build Process

### Development

```bash
# Start backend with hot-reload (cargo-watch)
cargo watch -x 'run --bin ladder-rs-server'

# Start frontend dev server (Vite HMR)
cd frontend && npm run dev

# Frontend dev server proxies /api to localhost:3000 via vite.config.ts proxy setting
```

In development, the frontend Vite dev server runs on `:5173` and the backend on `:3000`. Vite's proxy forwards `/api/*` to `:3000`. The `CORS_ALLOW_ORIGIN=http://localhost:5173` environment variable enables CORS on the backend.

### Production Build

```bash
# Build and start both containers
docker compose up --build

# Or build separately
docker compose build frontend
docker compose build backend
docker compose up -d
```

### CI Artifacts

CI (GitHub Actions) builds both images and, optionally, pushes them to a container registry. The `ladder-rs-wasm` crate is built separately in CI as a library artifact. Benchmarks are excluded from CI (per project convention — benchmarks run locally only).

---

## Deployment Scenarios

### Minimum Viable (Single Host)

The default `docker compose up` on a single Linux host. Suitable for a single organization with moderate traffic (hundreds of users, tens of thousands of matches).

```
Host: 1x Linux VM (2 CPU, 4 GB RAM)
  ├── Docker Engine
  ├── docker-compose.yml
  └── db_data volume (local disk)
```

### Development

```
Developer workstation
  ├── cargo run --bin ladder-rs-server (port 3000)
  └── npm run dev in frontend/ (port 5173, proxies /api to 3000)
```

No Docker required for development. The developer uses a local SQLite file specified by `DATABASE_URL`.

### With Swarm Operator

The Swarm Operator's process runs on the same host (most common) or a networked host with filesystem access to the SQLite volume:

```
Host
  ├── Docker Compose (frontend + backend containers)
  ├── db_data volume → /var/lib/docker/volumes/db_data/_data/ladder.db
  └── Swarm Operator process (host native or separate container)
      └── DATABASE_URL=sqlite:////var/lib/docker/volumes/db_data/_data/ladder.db
```

If the Swarm Operator runs in a separate container in the same Compose network, it must mount `db_data` at the same path and use the same `DATABASE_URL`.

---

## Requirements Traceability

| Requirement | Deployment Element |
|-------------|-------------------|
| NFR-REL-001 | `db_data` named volume (data survives container restart), WAL mode |
| NFR-PORT-001 | Single `DATABASE_URL` env var; switching to PostgreSQL changes only this var |
| NFR-SEC-001 | Backend container only; not exposed to host |
| NFR-SEC-002 | `SESSION_SECRET` via environment variable |
| SR-PER-001 | `ladder-rs-persistence` compiled into backend binary |
| SR-AUTH-004 | Admin bootstrap on first startup (step 5 of binary startup sequence) |
| SR-PER-009 | Background job poller spawned at startup (step 6) |
| ADR-0007 | Two-container Docker Compose topology |
