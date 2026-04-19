# NFR-SCALE-001: Data Volume

**Status:** Draft
**Parent:** Spec Section 6 (Scale Expectations)
**Priority:** Must-have

## Description

The system must support the v1 scale targets: up to 100 leagues, up to 10,000 players per league, and up to 1 million total matches. The deployment model is single-tenant. SQLite is the persistence layer, using WAL mode with busy_timeout for concurrency between the server and swarm operator.

## Rationale

These scale targets define the upper bound of v1 usage. The system must perform acceptably at these limits, not just at small scale. SQLite is chosen for simplicity and is adequate for these volumes with proper configuration (WAL mode, appropriate indexing).

## Acceptance Criteria

- [ ] The system operates correctly with 100 leagues in the database
- [ ] The system operates correctly with 10,000 players in a single league
- [ ] The system operates correctly with 1 million total match records across all leagues
- [ ] Leaderboard queries at 10,000 players per league meet the NFR-PERF-002 response time target
- [ ] SQLite WAL mode is enabled for concurrent read/write access
- [ ] SQLite busy_timeout is configured to handle contention between server and swarm operator access
- [ ] No data corruption occurs under concurrent access from both the server and a swarm operator writing through the library

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Data Volume Scale Targets

  Background:
    Given SQLite is configured with WAL mode enabled
    And SQLite busy_timeout is configured to handle write contention

  Scenario: System operates correctly with 100 leagues in the database
    Given 100 leagues exist with at least one season each
    When a client requests GET /leagues
    Then the HTTP response status is 200
    And all 100 leagues are accessible (via paginated requests)
    And no errors or timeouts occur

  Scenario: System operates correctly with 10,000 players in a single league
    Given a league "Max League" contains exactly 10,000 player records each with at least one match
    When a client requests the leaderboard for "Max League"
    Then the HTTP response status is 200
    And the leaderboard returns paginated results without errors
    And all 10,000 players are accessible across pages

  Scenario: System operates correctly with 1 million total match records
    Given the database contains 1,000,000 match records distributed across leagues
    When a League Operator records a new match in any league
    Then the HTTP response status is 201
    And the match is persisted correctly with a rating snapshot
    And no query timeout or database error occurs

  Scenario: Leaderboard query at 10,000 players meets NFR-PERF-002 response time target
    Given "Max League" contains 10,000 players with computed ratings
    When a client requests the leaderboard for "Max League"
    Then the response arrives within 500 milliseconds at the 95th percentile (per NFR-PERF-002)

  Scenario: SQLite WAL mode allows concurrent read and write access
    Given the server is handling HTTP read requests
    And a swarm operator process is simultaneously writing matches via the library crate
    When both processes operate concurrently for 60 seconds
    Then no read request fails due to database locking
    And no write is lost or corrupted

  Scenario: SQLite busy_timeout handles write contention between server and swarm operator
    Given the server and a swarm operator process both attempt to write at the same instant
    When the contention occurs
    Then one writer waits and retries within the busy_timeout period
    And both writes eventually succeed without returning a database-locked error to the caller

  Scenario: No data corruption occurs under concurrent access
    Given 500 agent players in a league
    And the swarm operator's process records 1,000 matches concurrently with the server processing HTTP requests
    When all operations complete
    Then each recorded match appears exactly once in the database
    And all rating snapshots are consistent with the match history
    And no duplicate or partial match records exist
```
