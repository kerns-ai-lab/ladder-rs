# UR-PM-001: Player Management

**Status:** Draft
**Parent:** Spec Section 4.3 (Player CRUD)
**Priority:** Must-have

## Description

A League Operator can add, remove, and view players within a league through the web UI. Players must be explicitly created in the UI before matches can be recorded against them. Each player has a name and a type flag (human or non-human). Removing a player is a soft-delete: the player becomes inactive, is hidden from the leaderboard, but all match history is preserved.

## Rationale

Players are the core participants in competitive activity. Operators need to manage the player roster explicitly to maintain data quality. The UI requires explicit creation (unlike the library API which auto-creates) because operators need to control player identity. Soft-delete preserves historical integrity while keeping inactive players out of active views.

## Acceptance Criteria

- [ ] Operator can add a player to a league by providing a name and selecting a type (human or non-human)
- [ ] Adding a player initializes their rating to the default for the current season's algorithm
- [ ] Operator can view a player's profile: current rating, match count, and rating history across seasons
- [ ] Operator can list all players in a league with their current ratings
- [ ] Operator can remove a player from a league, which soft-deletes them (marks as inactive)
- [ ] Soft-deleted players are hidden from the active leaderboard
- [ ] Soft-deleted players' match history is fully preserved and accessible
- [ ] In the UI, matches cannot be recorded against players who have not been explicitly created
- [ ] Player listing displays player type (human/non-human)
- [ ] Players are global entities: a player is created once and can belong to multiple leagues with independent ratings per league
- [ ] When adding a player to a league, the operator can select from existing global players or create a new player
- [ ] A player's profile shows all leagues they belong to, with per-league rating information
- [ ] A Player/Viewer user account can be linked to a player record to identify it as "theirs"

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player Management

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And league 42 "Alpha League" is active with an open Elo season (k_factor 32, initial_rating 1000)
    And "alice" is assigned as operator of league 42

  Scenario: Operator adds a human player to a league
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players with name "Charlie" and type "human"
    Then the response status is 201 Created
    And the response body contains player name "Charlie"
    And the response body contains player_type "human"
    And "Charlie"'s initial rating in league 42 is 1000 (the Elo default)

  Scenario: Operator adds a non-human player to a league
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players with name "AgentBot" and type "non-human"
    Then the response status is 201 Created
    And the response body contains player_type "non-human"

  Scenario: Adding an existing global player to a second league
    Given global player "Dave" with id 5 already exists (created in league 10)
    And "alice" is authenticated and assigned to league 42
    When "alice" sends POST /api/leagues/42/players with existing player_id 5
    Then the response status is 201 Created
    And player "Dave" is now a member of league 42 with its own initial rating of 1000

  Scenario: Operator views a player profile
    Given player "Charlie" with id 7 is in league 42 with rating 1150 and 12 matches played
    And "alice" is authenticated
    When "alice" sends GET /api/players/7
    Then the response status is 200 OK
    And the response body contains name "Charlie"
    And the response body contains current rating 1150
    And the response body contains match_count 12
    And the response body contains per-league rating information for league 42

  Scenario: Operator lists all players in a league with current ratings
    Given players "Alice", "Bob", and "Carol" are in league 42 with ratings 1200, 1000, and 900
    And "alice" is authenticated
    When "alice" sends GET /api/leagues/42/players
    Then the response status is 200 OK
    And the response body contains "Alice" with rating 1200
    And the response body contains "Bob" with rating 1000
    And the response body contains "Carol" with rating 900
    And each player entry includes player_type

  Scenario: Operator soft-deletes a player from a league
    Given player "Carol" with id 9 is in league 42 and is active
    And "alice" is authenticated
    When "alice" sends DELETE /api/leagues/42/players/9
    Then the response status is 200 OK
    And player "Carol"'s is_active flag in league 42 is 0
    And player "Carol"'s global player record still exists in the database

  Scenario: Soft-deleted player is excluded from the leaderboard
    Given player "Carol" with id 9 has been soft-deleted from league 42
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then the response body does not contain player name "Carol"

  Scenario: Soft-deleted player's match history is fully preserved
    Given player "Carol" with id 9 has been soft-deleted from league 42
    And "Carol" had 5 matches in season 7
    And "alice" is authenticated
    When "alice" sends GET /api/players/9/seasons/7/history
    Then the response status is 200 OK
    And the response body contains 5 rating history entries for "Carol"

  Scenario: Match cannot be recorded against a soft-deleted player via UI path
    Given player "Carol" with id 9 has been soft-deleted from league 42
    And player "Bob" with id 8 is active in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with participant player_id 9 (winner) and player_id 8 (loser)
    Then the response status is 400 Bad Request
    And the response body indicates player 9 is inactive

  Scenario: Player listing shows player_type field for each player
    Given players "Alice" (human) and "AgentBot" (non-human) are in league 42
    And "alice" is authenticated
    When "alice" sends GET /api/leagues/42/players
    Then the response body contains "Alice" with player_type "human"
    And the response body contains "AgentBot" with player_type "non-human"

  Scenario: Player is a global entity and can belong to multiple leagues
    Given global player "Dave" with id 5 is in league 42 with Elo rating 1050
    And league 43 "Beta League" uses TrueSkill and "alice" is assigned to it
    When "alice" sends POST /api/leagues/43/players with existing player_id 5
    Then the response status is 201 Created
    And player "Dave" in league 43 has a TrueSkill rating initialized to TrueSkill defaults
    And player "Dave" in league 42 still has Elo rating 1050 (unchanged)

  Scenario: Player profile shows all leagues the player belongs to
    Given player "Dave" with id 5 is in league 42 and league 43
    And "alice" is authenticated
    When "alice" sends GET /api/players/5
    Then the response body contains league 42 membership with its rating
    And the response body contains league 43 membership with its rating

  Scenario: Viewer cannot add a player to a league
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends POST /api/leagues/42/players with name "Hacker" and type "human"
    Then the response status is 403 Forbidden

  Scenario: Operator cannot manage players in a league they are not assigned to
    Given a user "bob_op" with role "operator" is authenticated and NOT assigned to league 42
    When "bob_op" sends POST /api/leagues/42/players with name "Sneaky" and type "human"
    Then the response status is 403 Forbidden

  Scenario: Adding a player to a non-existent league returns 404
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues/9999/players with name "Nobody" and type "human"
    Then the response status is 404 Not Found
```
