# SR-PER-005: Season Write Protection

**Status:** Draft
**Parent:** UR-ME-001, UR-LM-002
**Priority:** Must-have

## Description

Matches cannot be recorded against a closed (ended) season. A season is closed when a new season is started (algorithm type change) or when the parent league is archived. The persistence layer enforces this constraint and returns a clear error on violation.

## Rationale

Recording matches into a closed season would corrupt the rating timeline that was finalized when the season ended. Season boundaries must be hard boundaries for data integrity.

## Acceptance Criteria

- [ ] Attempting to record a match in a season that has a non-null end_date returns a structured error
- [ ] The error message clearly indicates that the season is closed
- [ ] The check is performed within the match recording transaction before any writes occur
- [ ] Matches can still be recorded in the current (open) season of the same league
- [ ] Rating history and match data for closed seasons remain fully readable
- [ ] Archiving a league closes all its seasons and prevents further match recording

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Season Write Protection

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And league 1 exists with two seasons: season 6 (closed, end_date set) and season 7 (open, end_date null)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 1

  Scenario: Recording a match in an open season succeeds
    When record_match(pool, season_id: 7, participants: [(1,placement:1), (2,placement:2)]) is called
    Then the function returns Ok(match_result)
    And a match record exists in season 7

  Scenario: Recording a match in a closed season returns SeasonClosed error
    When record_match(pool, season_id: 6, participants: [(1,placement:1), (2,placement:2)]) is called
    Then the function returns Err(PersistenceError::SeasonClosed { season_id: 6 })
    And no match record is inserted into season 6

  Scenario: API layer translates SeasonClosed to 409 Conflict with SEASON_CLOSED error code
    Given a user "alice_op" with role "operator" is authenticated and assigned to league 1
    When "alice_op" sends POST /api/seasons/6/matches with valid participants
    Then the response status is 409 Conflict
    And the response body contains error_code "SEASON_CLOSED"
    And the response body contains season_id 6

  Scenario: Season closed check is performed before any writes occur in the transaction
    When record_match(pool, season_id: 6, ...) is called against a closed season
    Then the function returns Err immediately
    And the matches table has no new rows from this attempt
    And the match_participants table has no new rows from this attempt

  Scenario: After a new season starts, the prior season is closed and protected
    Given league 1 has open season 7 (Elo)
    When create_season(pool, league_id: 1, algorithm: Glicko2, seeding_choice: Reset) is called
    Then season 7 end_date is now non-null
    When record_match(pool, season_id: 7, ...) is called
    Then the function returns Err(PersistenceError::SeasonClosed { season_id: 7 })

  Scenario: Archiving a league closes its open season
    Given league 1 has open season 7
    When archive_league(pool, league_id: 1) is called
    Then season 7 end_date is set to a non-null timestamp

  Scenario: Rating history and match data for closed seasons remain readable
    Given season 6 is closed with 10 match records and associated rating snapshots
    When get_leaderboard(pool, season_id: 6) is called
    Then the function returns Ok(leaderboard) with the historical rankings
    When get_rating_history(pool, player_id: 1, season_id: 6) is called
    Then the function returns Ok(history) with all 10 match history entries for player 1
```
