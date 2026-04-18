# System Requirements Index

## Authentication & Authorization Subsystem (AUTH)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [SR-AUTH-001](SR-AUTH-001-authentication.md) | Authentication | Must-have | UR-AUTH-001 |
| [SR-AUTH-002](SR-AUTH-002-authorization.md) | Authorization | Must-have | UR-AUTH-002 |
| [SR-AUTH-003](SR-AUTH-003-league-scoped-roles.md) | League-Scoped Roles | Must-have | UR-AUTH-002 |
| [SR-AUTH-004](SR-AUTH-004-admin-bootstrap.md) | Admin Bootstrap | Must-have | UR-AUTH-001 |
| [SR-AUTH-005](SR-AUTH-005-player-invite-linking.md) | Player Invite Linking | Must-have | UR-AUTH-003 |
| [SR-AUTH-006](SR-AUTH-006-league-visibility.md) | League Visibility Enforcement | Must-have | UR-LM-001 |
| [SR-AUTH-007](SR-AUTH-007-library-api-keys.md) | Library API Key Management | Must-have | UR-SW-001, UR-AUTH-002 |
| [SR-AUTH-008](SR-AUTH-008-invite-gated-registration.md) | Invite-Gated Registration | Must-have | UR-AUTH-001, UR-AUTH-003 |

## Persistence Subsystem (PER)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [SR-PER-001](SR-PER-001-library-persistence-api.md) | Library Persistence API | Must-have | UR-LM-001, UR-LM-002, UR-PM-001, UR-ME-001, UR-SW-001 |
| [SR-PER-002](SR-PER-002-atomic-match-recording.md) | Atomic Match Recording | Must-have | UR-ME-001, UR-ME-002 |
| [SR-PER-003](SR-PER-003-player-soft-delete.md) | Player Soft-Delete | Must-have | UR-PM-001 |
| [SR-PER-004](SR-PER-004-duplicate-match-rejection.md) | Duplicate Match Rejection | Must-have | UR-ME-001 |
| [SR-PER-005](SR-PER-005-season-write-protection.md) | Season Write Protection | Must-have | UR-ME-001, UR-LM-002 |
| [SR-PER-006](SR-PER-006-player-auto-creation.md) | Player Auto-Creation | Must-have | UR-ME-001, UR-SW-001 |
| [SR-PER-007](SR-PER-007-player-alias-recalc.md) | Player Alias Recalculation | Should-have | UR-PM-002 |
| [SR-PER-008](SR-PER-008-match-timestamp-ordering.md) | Match Timestamp Ordering | Must-have | UR-ME-001, UR-ME-002 |
| [SR-PER-009](SR-PER-009-async-recalculation.md) | Asynchronous Recalculation | Must-have | UR-ADM-001, UR-PM-002 |
| [SR-PER-010](SR-PER-010-job-deduplication.md) | Recalculation Job Deduplication | Should-have | UR-ADM-001, UR-PM-002 |

## Algorithm Subsystem (ALG)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [SR-ALG-001](SR-ALG-001-algorithm-presets.md) | Algorithm Parameter Presets | Must-have | UR-LM-001 |
| [SR-ALG-002](SR-ALG-002-parameter-guardrails.md) | Parameter Guardrails | Must-have | UR-LM-001 |
| [SR-ALG-003](SR-ALG-003-season-trigger-rules.md) | Season Trigger Rules | Must-have | UR-LM-002 |
| [SR-ALG-004](SR-ALG-004-season-transition-seeding.md) | Season Transition Seeding | Must-have | UR-LM-002 |
| [SR-ALG-005](SR-ALG-005-leaderboard-ranking-metric.md) | Leaderboard Ranking Metric | Must-have | UR-LB-001 |

## API Subsystem (API)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [SR-API-001](SR-API-001-pagination.md) | Pagination | Must-have | UR-LB-001, UR-SW-001 |
| [SR-API-002](SR-API-002-structured-errors.md) | Structured Error Responses | Must-have | UR-ME-001, UR-LM-001, UR-PM-001 |
| [SR-API-003](SR-API-003-server-side-filtering.md) | Server-Side Filtering and Sorting | Must-have | UR-LB-001, UR-SW-001, UR-LM-001 |
| [SR-API-004](SR-API-004-player-search.md) | Player Search Endpoint | Must-have | UR-PM-001 |

## Swarm Subsystem (SW)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [SR-SW-001](SR-SW-001-active-agent-threshold.md) | Configurable Active Agent Threshold | Must-have | UR-SW-001 |

## Admin Subsystem (ADM)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [SR-ADM-001](SR-ADM-001-audit-log.md) | Audit Log | Should-have | UR-ADM-001 |

## Non-Functional Requirements (NFR)

See [NFR index](../nfr/).

| ID | Title | Priority |
|---|---|---|
| [NFR-PERF-001](../nfr/NFR-PERF-001-rating-calculation-latency.md) | Rating Calculation Latency | Must-have |
| [NFR-PERF-002](../nfr/NFR-PERF-002-api-response-time.md) | API Response Time | Must-have |
| [NFR-SCALE-001](../nfr/NFR-SCALE-001-data-volume.md) | Data Volume | Must-have |
| [NFR-REL-001](../nfr/NFR-REL-001-crash-recovery.md) | Crash Recovery | Must-have |
| [NFR-PORT-001](../nfr/NFR-PORT-001-db-portability.md) | Database Portability | Should-have |
| [NFR-SEC-001](../nfr/NFR-SEC-001-login-rate-limiting.md) | Login Rate Limiting | Must-have |
| [NFR-SEC-002](../nfr/NFR-SEC-002-session-security.md) | Session Security | Must-have |
| [NFR-SEC-003](../nfr/NFR-SEC-003-input-sanitization.md) | Input Sanitization | Must-have |

## Coverage Matrix

Every UR maps to at least one SR:

| UR | SRs |
|---|---|
| UR-LM-001 | SR-PER-001, SR-ALG-001, SR-ALG-002, SR-API-002, SR-API-003, SR-AUTH-006 |
| UR-LM-002 | SR-PER-001, SR-PER-005, SR-ALG-003, SR-ALG-004 |
| UR-PM-001 | SR-PER-001, SR-PER-003, SR-API-002, SR-API-004 |
| UR-PM-002 | SR-PER-007, SR-PER-009 |
| UR-ME-001 | SR-PER-001, SR-PER-002, SR-PER-004, SR-PER-005, SR-PER-006, SR-PER-008, SR-API-002 |
| UR-ME-002 | SR-PER-002, SR-PER-008 |
| UR-LB-001 | SR-ALG-005, SR-API-001, SR-API-003 |
| UR-RH-001 | SR-PER-001 |
| UR-ADM-001 | SR-ADM-001, SR-PER-009, SR-PER-010, SR-AUTH-002 |
| UR-AUTH-001 | SR-AUTH-001, SR-AUTH-004, SR-AUTH-008 |
| UR-AUTH-002 | SR-AUTH-002, SR-AUTH-003, SR-AUTH-007 |
| UR-AUTH-003 | SR-AUTH-005, SR-AUTH-008 |
| UR-SW-001 | SR-PER-001, SR-PER-006, SR-API-001, SR-API-003, SR-SW-001, SR-AUTH-007 |
