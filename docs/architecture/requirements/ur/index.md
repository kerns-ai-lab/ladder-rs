# User Requirements Index

| ID | Title | Priority | Spec Section |
|---|---|---|---|
| [UR-LM-001](UR-LM-001-league-management.md) | League Management | Must-have | 4.1 |
| [UR-LM-002](UR-LM-002-season-management.md) | Season Management | Must-have | 4.2 |
| [UR-PM-001](UR-PM-001-player-management.md) | Player Management | Must-have | 4.3 |
| [UR-PM-002](UR-PM-002-player-aliasing.md) | Player Aliasing | Should-have | 4.3, RQ3a |
| [UR-ME-001](UR-ME-001-match-entry.md) | Match Entry | Must-have | 4.4 |
| [UR-ME-002](UR-ME-002-batch-match-entry.md) | Batch Match Entry | Should-have | 4.5, RQ6 |
| [UR-LB-001](UR-LB-001-leaderboard-view.md) | Leaderboard View | Must-have | 4.6 |
| [UR-RH-001](UR-RH-001-rating-history.md) | Rating History | Must-have | 4.3, 4.6, RQ5 |
| [UR-SW-001](UR-SW-001-swarm-dashboard.md) | Swarm Dashboard | Must-have | 4.7 |
| [UR-ADM-001](UR-ADM-001-admin-match-correction.md) | Admin Match Correction | Should-have | RQ7 |
| [UR-AUTH-001](UR-AUTH-001-user-accounts.md) | User Accounts | Must-have | RQ-R2-1, RQ-R3-1, RQ-R3-4 |
| [UR-AUTH-002](UR-AUTH-002-role-based-access.md) | Role-Based Access Control | Must-have | RQ-R2-1, RQ-R2-1a, RQ-R3-2 |
| [UR-AUTH-003](UR-AUTH-003-player-invite.md) | Player-to-Account Linking via Invite | Must-have | RQ-R3-3 |

## Traceability

### By Persona

**Admin:** UR-AUTH-001, UR-AUTH-002, UR-ADM-001, UR-LM-001, UR-LM-002, UR-PM-001, UR-PM-002, UR-ME-001, UR-ME-002, UR-LB-001, UR-RH-001, UR-SW-001

**League Operator:** UR-AUTH-001, UR-LM-001, UR-LM-002, UR-PM-001, UR-PM-002, UR-ME-001, UR-ME-002, UR-LB-001, UR-RH-001

**Player/Viewer:** UR-AUTH-001, UR-AUTH-003, UR-LB-001, UR-RH-001

**Swarm Operator:** UR-SW-001 (read-only dashboard; writes via library crate covered by SRs)

### Round 3 Scope Changes

- **League visibility (public/private) added** (UR-LM-001 updated) — RQ-R3-7.
- **Admin temporary password** added to UR-AUTH-001 — RQ-R3-4. Self-service forgot-password deferred to post-v1.
- **Swarm Operators confirmed as League Operators** — no fourth role. Three-role model is complete (RQ-R3-2).
- **Player-to-account linking via invite added** (UR-AUTH-003) — RQ-R3-3.
- **Active agent threshold made configurable** per-league — UR-SW-001 updated (RQ-R3-8). Heartbeat-based connectivity deferred to post-v1.
- **Self-service password reset deferred** to post-v1 (RQ-R3-4).
- **Agent heartbeat connectivity tracking deferred** to post-v1 (RQ-R3-8).

### Round 2 Scope Changes

- **User accounts and RBAC added to v1 scope** (UR-AUTH-001, UR-AUTH-002) -- reverses previous "no user accounts, no RBAC" out-of-scope decision.
- **Anomaly detection deferred to post-v1** -- removed from UR-SW-001 (RQ-R2-8).
- **Players are global** -- updated in UR-PM-001 (RQ-R2-6).
- **Batch entry is 1v1 only** -- updated in UR-ME-002 (RQ-R2-9).
- **League editing vs. algorithm change are separate actions** -- updated in UR-LM-001 (RQ-R2-7).
- **Leaderboard uses conservative estimates** -- updated in UR-LB-001 (RQ-R2-5).
- **Recalculation is asynchronous** -- updated in UR-ADM-001 (RQ-R2-3).

### Eliminated from v1

**Developer/Evaluator persona and Developer Demo (RQ8):** WASM bindings remain as a library artifact but no dedicated demo deliverable in v1.

**Anomaly detection (RQ-R2-8):** Deferred to post-v1. Swarm dashboard surfaces raw data only.
