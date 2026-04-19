# SR-PER-004: Duplicate Match Rejection

**Status:** Draft
**Parent:** UR-ME-001
**Priority:** Must-have

## Description

The persistence layer rejects duplicate match submissions. A duplicate is defined as a match with the same set of players, the same outcome, and the same timestamp. The rejection returns a clear error indicating the duplicate was detected.

## Rationale

Duplicate matches would inflate match counts and distort ratings. This is especially important for the swarm operator path where programmatic submission may retry on transient errors. The duplicate check provides idempotency protection.

## Acceptance Criteria

- [ ] A match submission with identical players, identical outcome, and identical timestamp as an existing match is rejected
- [ ] The rejection returns a structured error identifying it as a duplicate
- [ ] Matches with the same players and outcome but different timestamps are accepted (not duplicates)
- [ ] Matches with the same players and timestamp but different outcomes are accepted (not duplicates)
- [ ] The duplicate check is performed within the atomic match recording transaction
- [ ] The duplicate check works correctly for both 1v1 and N-player matches

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Duplicate Match Rejection

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And league 1 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 1
    And a match was recorded at timestamp T with participants: player 1 placement 1, player 2 placement 2

  Scenario: Submitting an identical match is rejected with a DUPLICATE_MATCH error
    When record_match(pool, season_id: 7, participants: [(1,placement:1), (2,placement:2)], recorded_at: T) is called again
    Then the function returns Err(PersistenceError::DuplicateMatch)
    And only one match record exists for that timestamp and participant combination

  Scenario: API layer translates DuplicateMatch error to 409 Conflict
    Given a user "alice_op" with role "operator" is authenticated
    When "alice_op" sends POST /api/seasons/7/matches with the same participants, outcome, and timestamp as an existing match
    Then the response status is 409 Conflict
    And the response body contains error_code "DUPLICATE_MATCH"

  Scenario: Same players and outcome but different timestamp is not a duplicate
    When record_match(pool, season_id: 7, participants: [(1,placement:1), (2,placement:2)], recorded_at: T+60s) is called
    Then the function returns Ok(match_result)
    And a second match record is created with timestamp T+60s

  Scenario: Same players and timestamp but different outcome is not a duplicate
    When record_match(pool, season_id: 7, participants: [(2,placement:1), (1,placement:2)], recorded_at: T) is called
    Then the function returns Ok(match_result)
    And a second match record is created with the reversed outcome

  Scenario: Duplicate check is performed within the atomic transaction before any writes
    When a duplicate record_match call is made
    Then the DuplicateMatch error is returned
    And the match_participants table has no new rows from the rejected attempt
    And the rating_snapshots table has no new rows from the rejected attempt

  Scenario: Duplicate check works correctly for 1v1 matches
    Given a 1v1 match at timestamp T with player 1 winning over player 2
    When the identical 1v1 match is submitted again at timestamp T
    Then the function returns Err(PersistenceError::DuplicateMatch)

  Scenario: Duplicate check works correctly for N-player ranked matches
    Given a 3-player ranked match at timestamp T with players 1 (1st), 2 (2nd), 3 (3rd)
    When the identical 3-player match is submitted again at timestamp T
    Then the function returns Err(PersistenceError::DuplicateMatch)
    When a 3-player match is submitted at timestamp T with player 2 in 1st place (different outcome)
    Then the function returns Ok(match_result) (not a duplicate)
```
