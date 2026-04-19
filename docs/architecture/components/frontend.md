# Component View — Frontend Container

**ladder-rs React Frontend (nginx)**
**Date:** 2026-04-17

---

## Overview

The frontend container serves the compiled React single-page application as static files via nginx. This C4 Level 3 view describes the significant components inside the React application: the routing tree, the three persona-scoped page groups, shared state, and the API client layer.

All components in this view run in the browser. Communication with the backend is exclusively via REST (JSON over HTTP). There is no WASM loading, no WebSockets, and no server-side rendering in v1.

---

## Technology Choices

| Concern | Choice | Rationale |
|---------|--------|-----------|
| Routing | React Router v6 (centralized route config) | SPA routing; supports nested routes for layout composition |
| Server state | TanStack Query (React Query v5) | Handles loading/error/stale states uniformly; background refetch; pairs naturally with REST |
| Auth/session state | React Context + useReducer | Simple global state for the authenticated user; no extra dependency |
| Component library | shadcn/ui | Accessible, composable primitives; no design-system lock-in; copy-paste components |
| HTTP client | Fetch API wrapped in typed client | Thin typed wrapper over native fetch; TanStack Query manages caching |
| Build tool | Vite | Fast HMR in development; content-hashed bundles for production |

---

## Component Map

```
[browser]
│
├── [App Root]
│   ├── [AuthProvider]  ──────────── React Context: current user, role, session state
│   ├── [QueryClientProvider] ──────── TanStack Query client (cache, background refetch)
│   └── [Router]  ─────────────────── React Router v6, centralized route config
│       │
│       ├── [PublicLayout]  ────────── unauthenticated shell (login, register, claim pages)
│       │   ├── /login               LoginPage
│       │   ├── /register/:token     RegisterPage (invite-gated; token from URL)
│       │   └── /invites/:token/claim ClaimPage (for already-registered users)
│       │
│       └── [AppLayout]  ───────────── authenticated shell (nav, sidebar, persona-aware)
│           │                          Redirects to /login if no valid session
│           │
│           ├── [Admin Pages]
│           │   ├── /admin/users          UserManagementPage
│           │   ├── /admin/users/:id      UserDetailPage
│           │   └── /admin/leagues/:id/operators  OperatorAssignmentPage
│           │
│           ├── [League Operator Pages]
│           │   ├── /leagues              LeagueListPage
│           │   ├── /leagues/new          CreateLeaguePage
│           │   ├── /leagues/:id          LeagueDetailPage
│           │   ├── /leagues/:id/seasons/new   CreateSeasonPage
│           │   ├── /seasons/:id          SeasonDetailPage
│           │   ├── /seasons/:id/matches/new   RecordMatchPage
│           │   ├── /seasons/:id/matches/batch BatchMatchEntryPage
│           │   ├── /seasons/:id/leaderboard   LeaderboardPage  (shared with Player)
│           │   ├── /players              PlayerListPage
│           │   ├── /players/new          CreatePlayerPage
│           │   ├── /players/:id          PlayerDetailPage
│           │   └── /players/:id/invites  InviteGeneratorPage
│           │
│           ├── [Player / Viewer Pages]
│           │   ├── /leagues/:id/leaderboard   LeaderboardPage  (read-only view)
│           │   ├── /players/:id/history       RatingHistoryPage
│           │   └── /players/:id/seasons       SeasonOverviewPage
│           │
│           └── [Swarm Dashboard Pages]
│               ├── /leagues/:id/swarm/stats   SwarmStatsPage
│               ├── /leagues/:id/swarm/agents  SwarmAgentsPage
│               └── /leagues/:id/swarm/volume  SwarmVolumePage
```

---

## Components

### AuthProvider

**Technology:** React Context, useReducer

**Responsibility:** Holds the authenticated user's identity and role in React state. Provides `useAuth()` hook consumed by layout guards and any component that needs to know who is logged in.

**State shape:**
```typescript
type AuthState =
  | { status: "loading" }
  | { status: "unauthenticated" }
  | { status: "authenticated"; user: { id: number; username: string; role: "admin" | "operator" | "viewer"; forcePasswordChange: boolean } };
```

**Initialization:** On mount, `AuthProvider` calls `GET /api/auth/me` (a lightweight session check endpoint). If the session cookie is valid, the response populates `user`. If not, status becomes `unauthenticated` and the router redirects to `/login`.

**Session expiry:** If any TanStack Query request returns `401`, a global query error handler dispatches `{ type: "LOGOUT" }` to the AuthProvider reducer, clearing state and redirecting to `/login`.

---

### API Client (`src/lib/api-client.ts`)

**Technology:** Native `fetch`, typed TypeScript wrapper

**Responsibility:** A thin typed wrapper over `fetch` that:
- Sets `credentials: "include"` on every request (session cookie)
- Sets `Content-Type: application/json` on mutation requests
- Throws a typed `ApiError` on non-2xx responses (preserving the structured error body from the backend)
- Provides typed request/response functions per endpoint group

**Pattern:**
```typescript
// Each endpoint group is a module of typed async functions
export async function recordMatch(seasonId: number, body: RecordMatchBody): Promise<MatchResult> {
  return apiPost(`/api/seasons/${seasonId}/matches`, body);
}
```

TanStack Query `queryFn` and `mutationFn` functions call these typed functions directly. No raw `fetch` calls appear in page components.

---

### PublicLayout

**Technology:** React Router `<Outlet>`, minimal shell (logo, no nav)

**Responsibility:** Wraps unauthenticated pages. Redirects to `/` (AppLayout home) if the user is already authenticated. Contains no sidebar or navigation — just a centered card layout for the auth forms.

**Routes:**
- `/login` — `LoginPage`: email/password form → `POST /api/auth/login`. On success, AuthProvider populates user; router redirects to `/leagues` (or `/admin/users` for admin role). If `forcePasswordChange` is true, redirects to change-password instead.
- `/register/:token` — `RegisterPage`: username/email/password form + the invite token from the URL path. Submits to `POST /api/auth/register` with `{ invite_token }`. On success, the user is registered and linked to the player record in one step.
- `/invites/:token/claim` — `ClaimPage`: shown to already-authenticated users who open an invite URL. Confirms the link with a single button → `POST /api/auth/invites/:token/claim`.

---

### AppLayout

**Technology:** React Router nested route layout, `useAuth()` guard

**Responsibility:** The authenticated application shell. Renders a top navigation bar (league switcher, user menu) and a role-aware sidebar. Guards the entire subtree: if `status !== "authenticated"`, redirects to `/login`.

**Role-aware navigation:** The sidebar renders links based on `user.role`:
- `admin`: shows User Management, all league management actions, swarm dashboard
- `operator`: shows league management actions for assigned leagues only, swarm dashboard
- `viewer`: shows leaderboard and rating history links for public leagues and their own player profile

**Forced password change redirect:** If `user.forcePasswordChange` is true, AppLayout renders only the change-password form regardless of the current route.

---

### LeaderboardPage

**Technology:** TanStack Query, shadcn/ui Table, cursor pagination

**Responsibility:** Fetches and renders the paginated ranked player list for a season. Supports both operator (editable context) and viewer (read-only) modes — determined by `useAuth()`.

**Key behaviors:**
- Cursor-based pagination: "Load more" button appends next page using the cursor from the previous response
- Polling: re-fetches every 10 seconds (configurable) to pick up rating updates
- Recalculation indicator: if `GET /api/jobs?season_id=X&status=queued,in_progress` returns results, shows a banner: "Rating recalculation in progress — results may be temporarily stale."
- Display name: shows `player.nickname ?? player.name` in all rows

---

### RecordMatchPage

**Technology:** TanStack Query mutation, shadcn/ui Form, player search autocomplete

**Responsibility:** The primary match entry form. Allows a League Operator to record a single match.

**Form fields:**
- Player search: autocomplete using `GET /api/players/search?q=...` (debounced, 300ms). Displays `nickname ?? name` in results. Supports adding N participants.
- Placement or outcome: rank ordering (drag-and-drop) or win/loss/draw selection depending on algorithm
- Submit: `POST /api/seasons/:id/matches`. On success, invalidates the leaderboard query cache.

---

### BatchMatchEntryPage

**Technology:** TanStack Query mutation, dynamic row list

**Responsibility:** Allows entry of multiple matches in sequence. Renders a dynamic list of match rows, each identical to the single match form. On submit, sends `POST /api/seasons/:id/matches/batch`. Per-entry errors are displayed inline.

---

### SwarmStatsPage / SwarmAgentsPage / SwarmVolumePage

**Technology:** TanStack Query, recharts (or similar) for charts

**Responsibility:** Read-only swarm dashboard views. Fetch from `/api/leagues/:id/swarm/stats`, `/agents`, and `/volume` respectively. Support the `active_threshold_days` query parameter via a filter control in the UI.

**Chart types:**
- Stats: rating distribution histogram, top-N / bottom-N agent tables
- Agents: per-agent table (first match, match count, last match, active status, win rate)
- Volume: time-series line chart (match count per hour/day/week bucket, switchable)

---

### RatingHistoryPage

**Technology:** TanStack Query, recharts line chart

**Responsibility:** Displays a player's rating progression within a single season as a line chart. Fetches from `GET /api/seasons/:sid/players/:pid/history`. X-axis: match sequence (or `recorded_at`). Y-axis: `conservative_rating`. Tooltip shows raw rating, deviation/uncertainty, and match ID.

---

### PlayerDetailPage

**Technology:** TanStack Query, shadcn/ui Card

**Responsibility:** Shows a player's global profile: display name (`nickname ?? name`), type (human/non-human), linked account (if any), and a summary of seasons participated in (`GET /api/players/:pid/seasons`). Admin-only section shows alias management controls and invite link generator.

---

## State Management Summary

| State category | Storage | Why |
|----------------|---------|-----|
| Authenticated user identity | React Context (AuthProvider) | Needed everywhere; changes rarely; single source of truth |
| Server data (leagues, seasons, matches, leaderboard) | TanStack Query cache | Handles loading/error/stale/refetch automatically |
| Form state | React controlled inputs (local component state) | Forms are ephemeral; no global form state needed |
| UI state (sidebar open, active filters) | Local useState | Component-local; not shared |

---

## Error Handling

- **Network / API errors:** TanStack Query surfaces errors via `isError` / `error` state. Components render an inline error card using the structured `ApiError` type (which carries the backend's `error` code and `message`).
- **401 responses:** Global TanStack Query `onError` handler dispatches logout to AuthProvider → redirect to `/login`.
- **Validation errors (400):** Mutation error responses include `fields` array. Form components map field errors to inline field-level messages.
- **Non-convergence flag:** Successful match recording responses include `convergence_quality`. If `"degraded"`, the UI shows a non-blocking info toast: "TrueSkill did not fully converge for this match. Results are best approximations."

---

## Requirements Traceability

| Requirement | Component(s) |
|-------------|--------------|
| UR-LM-001 | LeagueListPage, LeagueDetailPage, CreateLeaguePage |
| UR-LM-002 | SeasonDetailPage, CreateSeasonPage |
| UR-PM-001 | PlayerListPage, PlayerDetailPage, CreatePlayerPage |
| UR-PM-002 | PlayerDetailPage (alias management), InviteGeneratorPage |
| UR-ME-001 | RecordMatchPage |
| UR-ME-002 | BatchMatchEntryPage |
| UR-LB-001 | LeaderboardPage |
| UR-RH-001 | RatingHistoryPage, SeasonOverviewPage |
| UR-SW-001 | SwarmStatsPage, SwarmAgentsPage, SwarmVolumePage |
| UR-ADM-001 | LeaderboardPage (recalculation indicator), AdminPages |
| UR-AUTH-001 | LoginPage, RegisterPage, AppLayout (forced change redirect) |
| UR-AUTH-002 | AppLayout (role-aware nav), AuthProvider |
| UR-AUTH-003 | ClaimPage, RegisterPage, InviteGeneratorPage |
| SR-AUTH-004 | LoginPage (bootstrap credential entry → forced change redirect) |
| SR-AUTH-006 | AppLayout (private league visibility guard) |
| SR-API-001 | LeaderboardPage (cursor pagination), SwarmDashboard pages |
| SR-ALG-005 | LeaderboardPage (conservative_rating sort key display) |
| NFR-SEC-002 | API client (credentials: "include" on all requests) |
