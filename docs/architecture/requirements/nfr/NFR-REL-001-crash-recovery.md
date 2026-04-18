# NFR-REL-001: Crash Recovery

**Status:** Draft
**Parent:** Spec Section 6 (Reliability)
**Priority:** Must-have

## Description

No data loss occurs on application crash. SQLite ACID transactions ensure that committed data survives process termination. In-flight transactions that have not committed are rolled back on recovery. The system returns to a consistent state after restart without manual intervention.

## Rationale

Data integrity is paramount for a rating system. Operators and swarm operators must trust that recorded results persist. SQLite's ACID guarantees, combined with proper transaction management, provide crash recovery without additional infrastructure.

## Acceptance Criteria

- [ ] Committed match records and rating snapshots survive application crash and are present after restart
- [ ] In-flight transactions that were not committed at crash time are fully rolled back (no partial writes)
- [ ] The database is in a consistent state after restart without requiring manual repair or recovery steps
- [ ] WAL checkpointing does not lose data on crash
- [ ] The application starts successfully after an unclean shutdown without requiring manual database recovery

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Crash Recovery

  Background:
    Given the system is running with SQLite in WAL mode
    And a league "Recovery League" exists with 100 players and 500 matches

  Scenario: Committed match records survive application crash
    Given a match between Alice and Bob was successfully committed (HTTP 201 returned to client)
    When the application process is forcibly terminated (SIGKILL)
    And the application is restarted
    Then the match record for Alice vs Bob is present in the database
    And the rating snapshot for that match is present and consistent
    And the league leaderboard reflects the committed match result

  Scenario: In-flight transaction is fully rolled back on crash
    Given a match recording request is in progress and has begun writing to the database
    And the application process is forcibly terminated before the transaction is committed
    When the application is restarted
    Then the partial match record does not exist in the database
    And no partial rating snapshot exists for the in-flight transaction
    And the database is in a consistent state as if the match was never submitted

  Scenario: Database is in a consistent state after restart without manual intervention
    Given the application crashed during normal operation
    When the application is restarted
    Then the application starts successfully without reporting database corruption
    And no manual repair or SQLite recovery steps are required
    And the application accepts new requests immediately after startup

  Scenario: WAL checkpointing does not lose committed data on crash
    Given multiple transactions have been committed to the WAL log but not yet checkpointed
    When the application process crashes before the WAL checkpoint completes
    And the application is restarted
    Then all committed match records are present after restart
    And the WAL is recovered correctly by SQLite on re-open

  Scenario: Application starts successfully after unclean shutdown
    Given the application was terminated uncleanly (SIGKILL during active writes)
    When the application restarts
    Then the startup sequence completes without errors
    And no "database is locked" or "database is malformed" errors are reported
    And the API becomes available and responds to health check requests

  Scenario: No data loss for concurrent writes that committed before a crash
    Given a swarm operator committed 50 matches before the crash
    And 10 additional matches were in-flight at crash time
    When the application restarts
    Then exactly 50 matches are present (the committed ones)
    And the 10 in-flight matches are absent
    And the total match count is consistent with the pre-crash committed state
```
