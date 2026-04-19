# SR-PER-003: Player Soft-Delete

**Status:** Draft
**Parent:** UR-PM-001
**Priority:** Must-have

## Description

Player removal is implemented as a soft-delete. The player record is marked as inactive but remains in the database. Soft-deleted players are excluded from active leaderboards and player listings but their match history and rating snapshots are fully preserved.

## Rationale

Hard-deleting player records would create orphaned match records and break rating history integrity. Soft-delete preserves the complete historical record while removing the player from active views.

## Acceptance Criteria

- [ ] Removing a player sets an inactive/deleted flag on the player record rather than deleting the row
- [ ] Soft-deleted players are excluded from leaderboard queries by default
- [ ] Soft-deleted players are excluded from active player listing queries by default
- [ ] All match records involving a soft-deleted player are fully preserved
- [ ] All rating snapshots for a soft-deleted player are fully preserved
- [ ] Soft-deleted player data is accessible through player profile/history queries
- [ ] Matches cannot be recorded against soft-deleted players

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player Soft-Delete

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And league 1 has an open Elo season (id 7)
    And player "Carol" (id 3) is in league 1 with is_active = 1
    And player "Carol" has 4 matches and 4 rating snapshots in season 7

  Scenario: Removing a player sets inactive flag instead of deleting the row
    When remove_player(pool, league_id: 1, player_id: 3) is called
    Then the function returns Ok(())
    And the player record with id 3 still exists in the players table
    And the league_players row for (league_id: 1, player_id: 3) has is_active = 0

  Scenario: Soft-deleted player is excluded from leaderboard queries by default
    Given player 3 has been soft-deleted from league 1
    When get_leaderboard(pool, season_id: 7) is called
    Then the returned leaderboard does not include player id 3

  Scenario: Soft-deleted player is excluded from active player listing by default
    Given player 3 has been soft-deleted from league 1
    When list_players(pool, league_id: 1) is called (default active-only)
    Then the returned list does not include player id 3

  Scenario: Soft-deleted player's match records are fully preserved
    Given player 3 has been soft-deleted from league 1
    When the match_participants table is queried for player_id 3
    Then 4 match_participant rows exist for player 3

  Scenario: Soft-deleted player's rating snapshots are fully preserved
    Given player 3 has been soft-deleted from league 1
    When the rating_snapshots table is queried for player_id 3 and season_id 7
    Then 4 rating snapshot rows exist for player 3

  Scenario: Soft-deleted player's history is accessible via profile/history query
    Given player 3 has been soft-deleted from league 1
    When get_rating_history(pool, player_id: 3, season_id: 7) is called
    Then the function returns Ok(history) containing 4 entries

  Scenario: Recording a match against a soft-deleted player is rejected
    Given player 3 has been soft-deleted from league 1
    And player "Alice" (id 1) is active in league 1
    When record_match(pool, season_id: 7, participants: [(1, placement:1), (3, placement:2)]) is called
    Then the function returns Err(PersistenceError::PlayerLocked) or equivalent
    And no match record is created in the database

  Scenario: Soft-delete of a player who is not in the league returns an appropriate error
    Given player 99 does not belong to league 1
    When remove_player(pool, league_id: 1, player_id: 99) is called
    Then the function returns Err(PersistenceError::NotFound { entity: "league_player", ... })
```
