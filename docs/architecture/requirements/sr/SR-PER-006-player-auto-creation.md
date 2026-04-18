# SR-PER-006: Player Auto-Creation

**Status:** Draft
**Parent:** UR-ME-001, UR-SW-001
**Priority:** Must-have

## Description

The library persistence API auto-creates player records on first match reference when called through the library API directly (swarm operator path). The UI path requires explicit player creation before match entry. These are two paths to the same persistence layer: the library API supports both explicit creation and implicit creation on match recording.

## Rationale

Swarm operators manage large populations of agents programmatically and cannot pre-register every agent through a UI. Auto-creation on first match reference reduces friction for programmatic use. The UI path maintains explicit creation for data quality control appropriate to human-operated leagues.

## Acceptance Criteria

- [ ] When record_match() is called with a player identifier that does not exist, the library auto-creates the player record with default rating
- [ ] Auto-created players are assigned the default rating for the current season's algorithm
- [ ] Auto-created players are assigned a default player type (non-human) when created via auto-creation
- [ ] The match recording succeeds atomically including any auto-created player records
- [ ] The backend server (UI path) validates that all players exist before calling record_match(), enforcing explicit creation
- [ ] Both auto-created and explicitly created players are stored identically in the database
- [ ] Auto-creation works correctly for both 1v1 and N-player matches with multiple unknown players

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player Auto-Creation

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And league 1 has an open Elo season (id 7, initial_rating 1000)

  Scenario: record_match auto-creates a player on first reference via library path
    Given no player named "NewAgent" exists in the database
    When record_match(pool, season_id: 7, participants: [("NewAgent", placement:1), ("KnownAgent", placement:2)]) is called via the library API
    Then the function returns Ok(match_result)
    And a player record for "NewAgent" exists in the players table
    And "NewAgent"'s initial rating snapshot has rating 1000 (the Elo default)

  Scenario: Auto-created player is assigned non-human type by default
    Given no player named "AgentX" exists
    When record_match via library API references "AgentX" as a participant
    Then the player record for "AgentX" has player_type "non-human"

  Scenario: Auto-created player has the correct default rating for the current season's algorithm
    Given league 2 "Glicko League" has an open Glicko-2 season (id 8, initial_mu 1500)
    And no player named "GlickoAgent" exists
    When record_match(pool, season_id: 8, participants: [("GlickoAgent", placement:1), ("OtherAgent", placement:2)]) is called via the library API
    Then "GlickoAgent" is created with initial rating mu = 1500 (the Glicko-2 default)

  Scenario: Auto-creation is included in the same atomic transaction as the match
    Given the database rejects player insertion (simulated failure)
    When record_match via library API is called with an unknown player
    Then the function returns Err(...)
    And no match record is created
    And no player record is created for the unknown player

  Scenario: Auto-created and explicitly created players are stored identically
    Given player "ExplicitPlayer" (id 5) was created via add_player(pool, ...)
    And player "AutoPlayer" was created via auto-creation during record_match
    When both players are queried from the players table
    Then both records have the same schema structure
    And the only distinguishing difference may be player_type (non-human for auto-created)

  Scenario: Auto-creation works for multiple unknown players in a single match
    Given no players named "AgentA", "AgentB", "AgentC" exist
    When record_match(pool, season_id: 7, participants: [("AgentA", placement:1), ("AgentB", placement:2), ("AgentC", placement:3)]) is called via the library API
    Then the function returns Ok(match_result)
    And player records for "AgentA", "AgentB", and "AgentC" all exist
    And rating snapshots for all three players exist in season 7

  Scenario: Server (UI path) validates all players exist before calling record_match
    Given player "UnknownPlayer" does NOT exist in the database
    And a user "alice_op" with role "operator" is authenticated
    When "alice_op" sends POST /api/seasons/7/matches with participant player_name "UnknownPlayer"
    Then the response status is 400 Bad Request
    And the response body indicates the player does not exist
    And no player record is auto-created via the UI path
    And no match is recorded

  Scenario: Auto-creation is idempotent under concurrent calls with the same player name
    Given two concurrent library calls both reference "ConcurrentAgent" as a new player
    When both calls execute simultaneously
    Then exactly one player record for "ConcurrentAgent" exists after both calls complete
    And both match records are created successfully
```
