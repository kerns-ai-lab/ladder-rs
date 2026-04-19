# ADR-0007: Deployment Topology — Docker Compose, Two Containers

**Status:** Accepted
**Date:** 2026-04-15
**Deciders:** Dustin Kerns

---

## Context

The product specification v1.2 (Section 8) originally described the v1 deliverable as: "Docker image: bundles server + frontend + SQLite. Single-container deployment."

During the architecture phase, the deployment strategy was re-evaluated based on:
- The React frontend's build process (`npm run build` → static files) is a natural fit for nginx as a dedicated static file server
- The choice of Axum as the backend (ADR-0003) means the backend has no built-in static file serving capability equivalent to nginx's
- Embedding React static assets in the Rust binary (via `rust-embed` or similar) would require additional crate dependencies in `ladder-rs-server` and a custom static file handler
- nginx provides superior static asset serving: gzip/brotli compression, long-term caching headers, `try_files` for SPA routing — all for free, without custom code
- Separating frontend and backend containers enables independent updates to each

The original single-container approach was a simplification assumption made before the frontend framework was selected (ADR-0002). With React + Vite chosen, a two-container topology is the right match.

---

## Decision

Deploy the ladder-rs platform as **two Docker containers orchestrated by Docker Compose**, sharing one named Docker volume:

1. **`frontend` container:** nginx:alpine serving the React static build. Proxies `/api/*` to the backend.
2. **`backend` container:** Rust binary (`ladder-rs-server`). Exposes the REST API on port 3000 (internal Docker network only).
3. **`db_data` volume:** Named Docker volume mounting the SQLite file at `/data/ladder.db` in the backend container.

The host exposes port 8080 → frontend container port 80. The backend is not directly accessible from outside the Docker network.

The product specification Section 8 is updated to reflect this topology.

---

## Rationale

### nginx is purpose-built for static file serving

nginx handles React SPA deployment requirements correctly out of the box:
- `try_files $uri $uri/ /index.html` handles client-side routing (React Router navigations to deep URLs)
- `expires 1y; add_header Cache-Control "public, immutable"` for content-hashed JS/CSS bundles (Vite generates content-hashed filenames)
- `gzip_static` / `brotli_static` for pre-compressed asset delivery

Reproducing these capabilities in a Rust HTTP handler (`rust-embed` + tower) would require custom code, ongoing maintenance, and testing for correct cache header behavior. nginx eliminates this entire problem space.

### nginx as a reverse proxy eliminates CORS in production

With nginx proxying `/api/*` to the backend, all browser requests appear to come from the same origin (the nginx host). The browser has no cross-origin concern. The backend sees requests arriving from `127.0.0.1:PORT` inside the Docker network — no CORS configuration needed on the Axum backend for production deployments.

Without this separation (single binary serving both static files and API), CORS would still not be an issue, but the API/static routing logic would need to be custom-implemented in the Rust binary.

### Two-container separation enables independent updates

If the frontend needs a UI change, only the `frontend` image needs to be rebuilt and redeployed. The `backend` container and the `db_data` volume are untouched. Conversely, if the Axum server gets a bug fix, only the `backend` image is updated. The React app is unaffected.

In a single-binary deployment, every frontend change requires a full Rust recompile and redeploy of the entire binary.

### Named volume ensures data persistence

The `db_data` named volume decouples the SQLite file's lifecycle from the container lifecycle. `docker compose down` does not delete the volume (only `docker compose down -v` would). Container image updates (`docker compose up --build`) replace the container but mount the same volume with the existing data. This is the correct operational model for a database-backed application.

### Extensibility

The two-container topology is the natural starting point for future evolution:
- If PostgreSQL is adopted (NFR-PORT-001 enables this), the `db_data` volume is replaced by a `postgres` container — no changes to the frontend
- If the backend needs horizontal scaling, load balancing can be added in front of multiple backend containers sharing a PostgreSQL database — the frontend container is unaffected
- If the swarm operator needs a dedicated container, it can be added to the Compose file as a third service mounting the same volume

A single-binary deployment would require more refactoring to reach the same extensibility.

---

## Alternatives Considered

### Single binary with embedded React assets (`rust-embed`)

Embed the React `dist/` output into the Rust binary at compile time using `rust-embed`. The Axum server handles both API requests and static file serving.

**Rejected.** This approach has several disadvantages:
- Every frontend change requires a Rust recompile (slow — Rust compile times are measured in seconds to minutes, not milliseconds)
- Static file serving logic (cache headers, SPA routing, compression) must be implemented in Rust rather than configured in nginx
- The binary becomes large (embeds the full React bundle including charting libraries)
- The single binary cannot independently serve the frontend while the backend is restarting for a migration
- Loss of nginx's optimized static file serving and caching behavior

The simplicity benefit of a single binary is real but does not outweigh these costs.

### Single Docker container with both nginx and the Rust binary (supervisord)

Package nginx and the Rust binary in a single container, using a process supervisor like `supervisord` to run both.

**Rejected.** This gives up the operational independence of two containers while adding process supervisor complexity. It also violates the "one process per container" Docker convention. Log separation (frontend access logs vs. backend application logs) becomes harder. The health check model becomes muddier (is the container healthy if nginx is up but Rust is down?). There is no meaningful advantage over the two-container approach.

### Kubernetes

Deploy on Kubernetes with Deployments for frontend and backend, a PersistentVolumeClaim for SQLite, and a Service for ingress.

**Rejected.** Kubernetes is significant operational overhead for a single-tenant v1 deployment. Docker Compose is the appropriate tool for a self-hosted, single-organization deployment. Kubernetes would be considered if the platform needs to support multi-tenant deployments, high availability, or cloud-native scale — none of which are v1 requirements.

### Pure frontend-only deployment (Netlify/Vercel + backend API)

Host the frontend on a CDN (Netlify, Vercel, GitHub Pages) and deploy only the backend server.

**Rejected.** The platform's v1 deliverable is a self-hosted deployment for an organization that controls its own infrastructure. Hosting the frontend on an external CDN introduces external dependencies and may not be acceptable to organizations with data residency requirements. The Docker Compose topology keeps everything self-contained.

---

## Consequences

### Positive

- nginx provides correct SPA routing, cache headers, and compression without custom code
- Frontend and backend containers can be updated independently
- Named volume provides durable, container-lifecycle-independent SQLite storage
- No CORS configuration needed in production (same-origin via nginx proxy)
- Natural extensibility path toward PostgreSQL, horizontal scaling, additional services

### Negative / Accepted Trade-offs

- **Two build pipelines.** Building the platform requires both `npm run build` (Node.js) and `cargo build --release` (Rust). CI must have both Node.js and Rust toolchains available. The Docker build handles this via multi-stage builds.
- **Spec update required.** The product specification Section 8 described a single-container deployment. This decision reverses that assumption. The spec is updated accordingly.
- **Docker Compose required.** Operators must have Docker and Docker Compose installed. A standalone single-binary deployment is not provided in v1. For developers running without Docker, separate manual processes are used (as described in the deployment documentation).
- **Container orchestration overhead.** Docker Compose adds a `docker-compose.yml` deliverable and nginx configuration to the project. These are simple files but represent an additional surface to maintain.
- **Swarm Operator volume access.** The swarm operator process must have filesystem access to the SQLite volume path (`/data/ladder.db` or the Docker volume host path). This requires either running the swarm operator in a container on the same Compose network with the volume mounted, or mounting the Docker volume path directly on the host. Deployment documentation must explain both options.
