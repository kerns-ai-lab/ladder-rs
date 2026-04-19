# ADR-0002: Frontend Framework Selection

**Status:** Accepted
**Date:** 2026-04-15
**Deciders:** Dustin Kerns

---

## Context

The ladder-rs platform requires a web frontend for league management, player management, match entry, leaderboards, rating history visualization, and a swarm dashboard. The frontend communicates with the Axum backend via REST.

Key UI characteristics that influence the framework choice:
- Chart-heavy views: rating history line charts (per-player, per-season), rating distribution histograms, match volume over time, win rate by rating bucket (UR-RH-001, UR-SW-001)
- Dynamic form behavior: match entry form adapts its fields based on the selected algorithm (show/hide draw option, ranked placement UI for N-player events) (UR-ME-001)
- Pagination-heavy lists: leaderboard with cursor and offset pagination at 10K players (UR-LB-001, SR-API-001)
- Autocomplete search: player search by name prefix for match entry form (SR-API-004)
- No WASM loading: the frontend uses REST only; WASM bindings are a post-v1 capability spike

The following frameworks were evaluated: React (TypeScript), Leptos, HTMX with server-rendered templates (Askama/Tera), Vue, and Svelte.

---

## Decision

Use **React 18 with TypeScript** and **Vite** as the build tool.

---

## Rationale

### Charting ecosystem

React has the largest and most mature JavaScript charting ecosystem: Recharts, Victory, Chart.js (react-chartjs-2), Visx, and Nivo all have first-class React support. The rating history line chart and swarm dashboard histogram are central to the product. A framework with mature charting support eliminates the need to build custom visualization components.

Leptos, Vue, and Svelte have charting options, but the library breadth and documentation quality are materially lower than the React ecosystem for complex, interactive data visualizations.

### TypeScript integration

React + TypeScript is the dominant combination for typed browser applications. The TypeScript type system integrates well with Axum's JSON API responses (interfaces can be written to match the API contract). Vite provides first-class TypeScript support without configuration overhead.

### Build tooling maturity

Vite produces highly optimized production bundles (content-hashed filenames for long-term caching, tree-shaking, code splitting). The `npm run build` → `dist/` output is a simple, well-understood artifact that nginx can serve directly.

### Deployment simplicity

A static React build (`dist/`) is a folder of HTML, JS, and CSS files. It requires no server-side rendering infrastructure. nginx serves it efficiently with correct cache headers. This aligns with the two-container Docker Compose topology (ADR-0007).

### Team familiarity and ecosystem stability

React is the most widely understood JS framework. Documentation, community support, and hiring pool are considerations. The ecosystem stability of React (maintained by Meta, used by millions of projects) reduces the risk of library abandonment.

---

## Alternatives Considered

### Leptos (Rust/WASM SPA)

**Rejected.** Leptos is a compelling Rust-native frontend framework that compiles to WASM. However:
- The charting ecosystem is immature. No equivalent of Recharts or Chart.js exists for Leptos. Custom SVG chart components would be required for every chart in the swarm dashboard.
- WASM SPA adds build complexity (wasm-pack, wasm-bindgen, async WASM initialization in the browser).
- The platform's WASM bindings (`ladder-rs-wasm`) are a separate artifact not intended for the frontend. Having two WASM modules in the browser (the SPA framework + the rating library) would be confusing and hard to maintain.
- Leptos's ecosystem is smaller, with less production validation at v1 timelines.

### HTMX with Askama/Tera Server-Side Rendering

**Rejected.** HTMX with server-rendered HTML is excellent for CRUD applications with minimal client-side state. However, the platform's requirements push beyond this:
- The rating history charts and swarm dashboard histograms require JavaScript-rendered SVG or Canvas elements. HTMX would still require a JavaScript charting library — just without a framework to organize it.
- Dynamic form behavior (algorithm-aware match entry form) is easier to manage in a component model than in HTMX partial swaps.
- Server-side rendering moves chart data formatting logic to the backend, increasing coupling between the Axum backend and the frontend's visual representation.

### Vue 3

**Rejected.** Vue 3 with the Composition API is a capable framework and would be a reasonable choice. The decisive factor against Vue is the charting ecosystem: React's charting library breadth is meaningfully larger. Vue Charts (vue-chartjs, ECharts Vue wrapper) exist but have smaller communities and less documentation than their React counterparts. Given that charting is central to the swarm dashboard, this difference matters.

### Svelte / SvelteKit

**Rejected.** Svelte produces smaller bundles than React and has elegant reactivity. However, the Svelte charting ecosystem is smaller than React's. SvelteKit adds SSR complexity that is not needed for a single-tenant deployment. The team familiarity argument also applies — React's ubiquity reduces onboarding overhead.

---

## Consequences

### Positive

- Access to the full React charting ecosystem for rating history and swarm dashboard visualizations
- Vite's content-hashed builds work natively with nginx long-term caching
- TypeScript provides type safety across the API boundary (manually maintained interfaces)
- Largest developer community for debugging and library support

### Negative / Accepted Trade-offs

- **npm build step.** The frontend build requires Node.js and npm. This adds a second build toolchain to the project alongside Rust/Cargo. Both toolchains run in the Docker build stage.
- **No shared types with backend.** TypeScript interfaces for API request/response shapes must be maintained manually in the frontend. There is no code generation from Axum's route definitions in v1 (OpenAPI generation is deferred to post-v1 per the spec). Type drift between the API and the frontend types is a maintenance risk; naming conventions and API versioning discipline mitigate this.
- **Bundle size.** A React application with charting libraries will have a larger JS bundle than a Leptos WASM app or a server-rendered HTMX page. For a single-tenant internal tool, this is an acceptable trade-off.

### Post-v1 Implications

- The frontend REST interface is independent of the backend framework. If the backend changes (e.g., to add gRPC or replace Axum), the frontend is unaffected as long as the REST contract is maintained.
- If client-side rating preview becomes valuable (computing ratings in the browser before submission), the `ladder-rs-wasm` bindings can be imported as a dependency of the existing React app without changing the framework.
