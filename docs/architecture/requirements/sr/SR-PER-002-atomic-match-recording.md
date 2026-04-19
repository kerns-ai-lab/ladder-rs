# SR-PER-002: Atomic Match Recording

**Status:** Draft
**Parent:** UR-ME-001, UR-ME-002
**Priority:** Must-have

## Description

Match recording and rating update execute as a single atomic database transaction. If either the match insert or the rating update fails, the entire operation is rolled back. No partial state (match without ratings, or ratings without match) can exist in the database.

## Rationale

Partial writes would corrupt the rating timeline. If a match is recorded but ratings are not updated (or vice versa), the leaderboard and history views become inconsistent. Atomicity guarantees that the system is always in a valid state.

## Acceptance Criteria

- [ ] A single database transaction encompasses both the match record insertion and all associated rating snapshot updates
- [ ] If the match insert succeeds but any rating update fails, the entire transaction is rolled back
- [ ] If the rating calculation succeeds but the match insert fails, no rating snapshots are persisted
- [ ] After a successful transaction, both the match record and all updated rating snapshots are visible to subsequent queries
- [ ] After a failed transaction, the database state is identical to before the attempt
- [ ] The transaction isolation level prevents concurrent match recordings from producing inconsistent ratings for the same players

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Atomic Match Recording

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And league 1 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 1 with initial ratings 1000

  Scenario: Successful match recording persists both match and rating snapshots together
    When record_match(pool, season_id: 7, participants: [(1,placement:1), (2,placement:2)], None) is called
    Then the function returns Ok(match_result) with a match_id
    And the match record for that match_id exists in the database
    And rating snapshots for player 1 and player 2 with that match_id exist in the database
    And both the match and snapshots are visible in subsequent queries

  Scenario: If rating calculation fails after match insert, entire transaction is rolled back
    Given the rating engine is configured to fail on the next call (simulated failure)
    When record_match(pool, season_id: 7, ...) is called
    Then the function returns Err(...)
    And no match record was inserted for this attempt
    And no rating snapshots were inserted for this attempt
    And the database state is identical to before the call

  Scenario: If match insert fails, no rating snapshots are persisted
    Given the database is configured to reject the match insert (simulated constraint failure)
    When record_match(pool, season_id: 7, ...) is called
    Then the function returns Err(...)
    And no rating snapshots were inserted for player 1 or player 2 in this attempt

  Scenario: After successful transaction both match and ratings are immediately visible
    Given a second connection pool (pool_b) opened to the same database
    When record_match(pool_a, season_id: 7, ...) completes successfully
    And get_leaderboard(pool_b, season_id: 7) is called immediately after
    Then pool_b's leaderboard query returns the updated ratings for players 1 and 2

  Scenario: Concurrent match recordings for different player pairs do not corrupt ratings
    Given players "Carol" (id 3) and "Dave" (id 4) are also in season 7
    When record_match for Alice vs Bob and record_match for Carol vs Dave execute concurrently
    Then both transactions complete successfully (one may retry due to WAL write lock)
    And all 4 players have valid rating snapshots for their respective matches
    And no player's snapshot references a match_id they did not participate in

  Scenario: Failed transaction leaves database in exactly the state before the attempt
    Given Alice has rating 1000 and Bob has rating 1000 before the call
    And the match recording fails mid-transaction (simulated crash after match insert, before snapshot insert)
    When the database is queried after the failure
    Then Alice's rating is still 1000
    And Bob's rating is still 1000
    And no orphaned match record exists without associated rating snapshots

  Scenario: Schema indexes support efficient duplicate match detection
    Given the database schema is fully migrated
    When the indexes on the matches table are inspected
    Then an index exists on (season_id, recorded_at) to support duplicate detection queries
    When the indexes on the match_participants table are inspected
    Then an index exists on (match_id) to support participant lookup for duplicate checks
```
