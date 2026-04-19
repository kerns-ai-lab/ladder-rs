# System Context — C4 Level 1

**ladder-rs Platform**
**Date:** 2026-04-15

---

## Overview

The System Context view defines the boundary of the ladder-rs platform and identifies every external actor and system that interacts with it. Anything inside the boundary is owned and operated by the platform; anything outside is either a user persona or an external dependency.

---

## The System: ladder-rs Platform

The ladder-rs Platform is the totality of software and infrastructure deployed to serve one organization's competitive league management and AI swarm monitoring needs. It encompasses:

- A React-based web frontend served by nginx
- An Axum REST backend with embedded background workers
- A SQLite database on a named Docker volume
- The `ladder-rs-persistence` crate as the exclusive database access layer

The platform is a single-tenant system. One deployment serves one organization. Multi-tenancy is out of scope for v1.

---

## External Actors

### Admin

A privileged human user with global authority across the platform. The Admin interacts with the system exclusively through a web browser. The Admin can:

- Bootstrap the platform on first startup (credentials are printed to stdout and forced-change on first login)
- Create and manage all user accounts, including setting temporary passwords
- Assign League Operators to leagues
- Perform audited match corrections
- Manage all leagues, seasons, players, and matches without restriction

The Admin persona maps to the `Admin` role in the RBAC model (SR-AUTH-002). There is at most one Admin role; it is seeded at bootstrap.

### League Operator

A human user responsible for managing one or more specific leagues. The League Operator interacts with the system through a web browser. The League Operator can:

- Create and edit leagues assigned to them
- Manage seasons within their leagues (start, close, configure algorithm and parameters)
- Add and remove players from their leagues
- Record match results (1v1 and N-player ranked)
- View leaderboards and rating history for their leagues
- Generate player invite links for account linking

The League Operator role is league-scoped (SR-AUTH-003). A user may be a League Operator for multiple leagues. Swarm Operators hold this role within the platform — there is no separate fourth role (RQ-R3-2).

### Player / Viewer

A human user who participates in leagues or views league data. The Player/Viewer interacts with the system through a web browser. The Player/Viewer can:

- View public league leaderboards and rating history
- View their own player profile and match history
- Claim an account via an invite link, linking their user account to their player record

The Player/Viewer has read-only access to public leagues and their own player data. Access to private leagues requires league membership (SR-AUTH-006).

### Swarm Operator

A technical user who runs autonomous AI agent experiments. The Swarm Operator interacts with the platform in two distinct ways:

1. **Library path (primary write path):** The Swarm Operator links `ladder-rs-persistence` as a library crate directly in their Rust process. All match recording and player registration for swarm experiments goes through this path — no HTTP involved. The Swarm Operator's process and the Axum backend share the same SQLite file through SQLite's WAL mode concurrent access.

2. **Browser path (read-only dashboard):** The Swarm Operator views the Swarm Dashboard through a web browser using the same frontend as other users. This is a read-only interface surfacing aggregate statistics from data written by their library process.

The Swarm Operator holds the League Operator role within the platform's RBAC model for any leagues they manage.

---

## External Systems

### Web Browser (User Agent)

All four human actor personas interact with the platform exclusively through a web browser. The browser is not a distinct system — it is the runtime environment for the React frontend application. It communicates with the platform's nginx container over HTTPS (or HTTP in development).

### Swarm Operator Process

The Swarm Operator's own Rust application. This is a software system external to the platform that embeds `ladder-rs-persistence` as a linked library. It is not an HTTP client — it accesses the SQLite database file directly via a connection pool. The platform must accommodate this concurrent access pattern through SQLite WAL mode configuration.

---

## System Boundary Summary

| Element | Inside Platform Boundary | Notes |
|---------|--------------------------|-------|
| React Frontend (nginx) | Yes | Served as static files |
| Axum REST Backend | Yes | REST API + background workers |
| SQLite Database | Yes | Named Docker volume |
| `ladder-rs-persistence` crate | Yes | Owned DB access library |
| `ladder-rs` crate | Yes | Pure rating math |
| `ladder-rs-wasm` crate | Yes (artifact only) | Not used by frontend in v1 |
| Web Browser | No | User agent / client runtime |
| Swarm Operator Process | No | External Rust process using the persistence crate |

---

## Key Interactions

| From | To | Protocol / Mechanism | Purpose |
|------|----|----------------------|---------|
| Admin (browser) | ladder-rs Platform | HTTPS / REST | League management, user admin, match correction |
| League Operator (browser) | ladder-rs Platform | HTTPS / REST | Match entry, player management, leaderboards |
| Player/Viewer (browser) | ladder-rs Platform | HTTPS / REST | Leaderboard viewing, profile access |
| Swarm Operator (browser) | ladder-rs Platform | HTTPS / REST | Dashboard viewing (read-only) |
| Swarm Operator Process | ladder-rs Platform | Rust FFI (library link) | Direct DB writes via `ladder-rs-persistence` |

---

## Requirements Traceability

| Requirement | Addressed by this view |
|-------------|------------------------|
| UR-AUTH-001 | Admin and League Operator actors identified; user accounts are internal to platform |
| UR-AUTH-002 | Three actor roles defined: Admin, League Operator, Player/Viewer |
| UR-AUTH-003 | Player/Viewer invite claim path identified |
| UR-SW-001 | Swarm Operator dual-path interaction documented |
| SR-PER-001 | Swarm Operator library path documented as peer DB access |
| NFR-REL-001 | SQLite WAL mode enables concurrent access across two processes |
