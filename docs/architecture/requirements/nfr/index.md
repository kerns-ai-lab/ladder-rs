# Non-Functional Requirements Index

## Performance (PERF)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [NFR-PERF-001](NFR-PERF-001-rating-calculation-latency.md) | Rating Calculation Latency | Must-have | UR-ME-001 |
| [NFR-PERF-002](NFR-PERF-002-api-response-time.md) | API Response Time | Must-have | UR-LB-001, UR-SW-001 |

## Scale (SCALE)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [NFR-SCALE-001](NFR-SCALE-001-data-volume.md) | Data Volume | Must-have | UR-ME-001, UR-LB-001, UR-SW-001 |

## Reliability (REL)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [NFR-REL-001](NFR-REL-001-crash-recovery.md) | Crash Recovery | Must-have | UR-ME-001 |

## Portability (PORT)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [NFR-PORT-001](NFR-PORT-001-db-portability.md) | Database Portability | Should-have | UR-LM-001, UR-ME-001 |

## Security (SEC)

| ID | Title | Priority | Parent UR(s) |
|---|---|---|---|
| [NFR-SEC-001](NFR-SEC-001-login-rate-limiting.md) | Login Rate Limiting | Must-have | UR-AUTH-001 |
| [NFR-SEC-002](NFR-SEC-002-session-security.md) | Session Security | Must-have | UR-AUTH-001 |
| [NFR-SEC-003](NFR-SEC-003-input-sanitization.md) | Input Sanitization | Must-have | UR-AUTH-001, UR-LM-001, UR-PM-001 |

## Round 3 Additions

- **NFR-SEC-001, NFR-SEC-002, NFR-SEC-003** added per RQ-R3-5: login rate limiting, session cookie security, and input sanitization are required security properties for the authentication and data entry subsystems.
