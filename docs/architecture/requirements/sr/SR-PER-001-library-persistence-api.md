# SR-PER-001: Library Persistence API

**Status:** Draft
**Parent:** UR-LM-001, UR-LM-002, UR-PM-001, UR-ME-001, UR-SW-001
**Priority:** Must-have

## Description

The library crate exposes a high-level persistence API with functions including create_league(), add_player(), record_match(), and related operations. The library owns ALL database interaction. Both the backend server and swarm operators consume the library for DB access. No component accesses the database directly outside the library.

## Rationale

Centralizing persistence in the library ensures a single source of truth for data access logic, prevents inconsistencies between the server and swarm operator paths, and simplifies the server to a thin REST wrapper. This also ensures that rating calculations and persistence are always co-located.

## Acceptance Criteria

- [ ] The library crate exposes public functions for: create_league, edit_league, archive_league, unarchive_league, list_leagues
- [ ] The library crate exposes public functions for: add_player, remove_player (soft-delete), list_players, get_player_profile
- [ ] The library crate exposes public functions for: record_match, get_leaderboard, get_rating_history
- [ ] The library crate exposes public functions for: create_season, get_seasons, get_season_details
- [ ] All database reads and writes go through the library's persistence layer; the server backend has no direct DB access
- [ ] The persistence API accepts a database connection/pool as a parameter (dependency injection)
- [ ] All API functions return typed Result values with structured error types
- [ ] The API is async-native (tokio); all consumers must bring their own tokio runtime
- [ ] Write functions that touch league-scoped data require a `SwarmContext` parameter (see SR-AUTH-007); the context is validated at startup via an API key and scopes writes to the operator's assigned leagues
- [ ] Player records include both a globally unique `name` field and an optional `nickname` field; both are returned in all Player API responses; the `nickname ?? name` display rule is documented for consumers
- [ ] The library crate exposes `correct_match(pool, match_id, new_participants, reason, corrected_by: UserId) -> Result<CorrectMatchResult, PersistenceError>`; it atomically updates the match record, inserts the audit log entry, and calls `insert_job` for recalculation — all within a single transaction; it returns the job_id

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Library Persistence API

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized with WAL mode enabled
    And the database schema has been migrated to the latest version

  Scenario: Library exposes create_league and the created league is visible via list_leagues
    When the swarm operator calls create_league(pool, name: "Swarm League", algorithm: Elo, visibility: Public)
    Then the function returns Ok(league_id)
    And list_leagues(pool) includes "Swarm League" with status "active"

  Scenario: Library exposes add_player and the player has a default rating
    Given a league "Swarm League" exists with Elo season (initial_rating 1000)
    When the swarm operator calls add_player(pool, league_id, name: "Agent007", player_type: NonHuman)
    Then the function returns Ok(player_id)
    And the player record has initial_rating 1000 in the league's current season

  Scenario: Library exposes record_match and creates match + rating snapshots
    Given players "AgentA" (id 1) and "AgentB" (id 2) exist in league 1 with Elo season (id 7)
    When the swarm operator calls record_match(pool, season_id: 7, participants: [(1, placement:1), (2, placement:2)], score_metadata: None)
    Then the function returns Ok(match_result) containing the new match_id
    And a match record exists in season 7
    And rating snapshots exist for both players in season 7

  Scenario: Library exposes remove_player (soft-delete)
    Given player "AgentC" (id 3) is in league 1 with is_active = 1
    When the swarm operator calls remove_player(pool, league_id: 1, player_id: 3)
    Then the function returns Ok(())
    And player 3's is_active flag in league 1 is 0
    And the player record still exists in the database

  Scenario: Library exposes get_leaderboard returning current standings
    Given season 7 has 3 players with ratings 1200, 1100, and 900
    When the swarm operator calls get_leaderboard(pool, season_id: 7, limit: 10, offset: 0)
    Then the function returns Ok(leaderboard) containing 3 entries ordered by conservative_rating desc
    And the first entry has the highest rating

  Scenario: Library exposes get_rating_history for a player in a season
    Given player 1 has 5 rating snapshots in season 7 in chronological order
    When the swarm operator calls get_rating_history(pool, player_id: 1, season_id: 7)
    Then the function returns Ok(history) with 5 entries ordered by timestamp asc

  Scenario: Library exposes create_season and get_seasons
    Given league 1 has one open season (id 7)
    When the swarm operator calls create_season(pool, league_id: 1, algorithm: Glicko2, params: default, seeding_choice: Reset)
    Then the function returns Ok(new_season_id)
    And get_seasons(pool, league_id: 1) returns 2 seasons

  Scenario: All DB operations go through the library — no direct DB access from server
    Given the server backend code is compiled
    Then the server crate does not import "sqlx" directly
    And all persistence calls use functions from the ladder-rs-persistence crate

  Scenario: Persistence API accepts a connection pool as a dependency-injected parameter
    When the swarm operator calls any persistence function with pool_a
    Then the function operates on pool_a's database
    When the swarm operator calls the same function with pool_b (different database)
    Then the function operates on pool_b's database independently

  Scenario: Persistence API functions return typed Result values with structured error types
    Given player 9999 does not exist
    When the swarm operator calls get_player_profile(pool, player_id: 9999)
    Then the function returns Err(PersistenceError::NotFound { entity: "player", id: 9999 })
    And the error does not panic or produce an unstructured string error

  Scenario: Persistence API is callable from async context
    Given an async tokio runtime
    When the server calls record_match(pool, ...) with .await
    Then the function completes without blocking the tokio runtime
    And the result is returned as a Future<Output = Result<MatchResult, PersistenceError>>

  Scenario: Library exposes correct_match and atomically writes correction, audit entry, and recalculation job
    Given match 100 exists in season 7 with player 1 (placement 1) and player 2 (placement 2)
    When the admin calls correct_match(pool, match_id: 100, new_participants: [(2, placement:1), (1, placement:2)], reason: "wrong winner", corrected_by: admin_user_id)
    Then the function returns Ok(CorrectMatchResult { job_id })
    And match 100 has is_corrected = true in the database
    And an audit log entry exists for match 100 with actor_user_id = admin_user_id
    And a recalculation job exists with status "queued" for season 7
    And the audit log entry, match update, and job insert are committed in a single transaction

  Scenario: Server crate does not have sqlx as a direct dependency
    Given the Cargo workspace is built and metadata is available
    When `cargo metadata --no-deps` is inspected for the ladder-rs-server package
    Then "sqlx" does not appear in the direct dependencies of ladder-rs-server
    And "ladder-rs-wasm" does not appear in the direct dependencies of ladder-rs-server
    And "ladder-rs-persistence" appears as the sole persistence-layer dependency

  Scenario: Migration files execute transactionally — partial failure rolls back entire file
    Given a migration file contains CREATE TABLE followed by CREATE INDEX
    And the CREATE INDEX statement is designed to fail (syntax error)
    When the migration system applies the file
    Then the entire migration file is rolled back
    And the CREATE TABLE from that file does not persist in the schema
    And the _sqlx_migrations table does not record the migration as applied

  Scenario: Schema enforces hybrid FK cascade strategy
    Given the database schema is fully migrated
    When a match record is deleted
    Then all associated match_participants rows are automatically deleted (CASCADE)
    When a league record is deleted
    Then the delete is blocked because seasons still reference it (RESTRICT)
    When a user record is deleted
    Then associated login_attempts and sessions are deleted (CASCADE)
    But associated player_account_links are blocked (RESTRICT)

  Scenario: Schema enforces UNIQUE constraints on player_account_links
    Given user-alice is linked to player-001
    When a concurrent claim attempt tries to link user-alice to player-002
    Then the database rejects the insert with a UNIQUE constraint violation on user_id
    When a concurrent claim attempt tries to link user-bob to player-001
    Then the database rejects the insert with a UNIQUE constraint violation on player_id

  Scenario: Schema column defaults produce correct initial state
    Given a new league is inserted with only required fields (name, algorithm)
    When the row is queried
    Then is_active = 1, is_archived = 0, visibility = 'public', created_at is within 1 second of now
    Given a new recalculation job is inserted with only season_id
    When the row is queried
    Then status = 'queued' and created_at is within 1 second of now
    Given a new player is inserted with only name
    When the row is queried
    Then player_type = 'human' and is_active = 1

  Scenario: Match recorded_at is never defaulted by the database
    Given the matches table schema
    When the schema is inspected for the recorded_at column
    Then no DEFAULT constraint exists on recorded_at
    And recorded_at is NOT NULL (application must always supply it)
```
