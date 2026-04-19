# UR-PM-002: Player Aliasing

**Status:** Draft
**Parent:** Spec Section 4.3 (Player CRUD), RQ3a decision
**Priority:** Should-have

## Description

A League Operator can link two player records as aliases of the same competitor. Both player records persist after linking (additive alias system). Aliasing triggers a full rating recalculation to merge the match histories. Aliases can be removed, which triggers another recalculation. True destructive merge (combining records into one) is deferred to post-v1.

## Rationale

Players may register under different names or identities across sessions. Rather than requiring operators to prevent this upfront, the alias system lets them correct it after the fact. The additive approach preserves data integrity while ensuring ratings reflect the combined match history. Recalculation is necessary because merging histories changes the sequence of opponents and outcomes.

## Acceptance Criteria

- [ ] Operator can link two player records within the same league as aliases of each other
- [ ] After linking, both original player records remain in the database (no records are deleted)
- [ ] Linking aliases triggers a full rating recalculation for all affected matches in the current season
- [ ] After recalculation, the aliased players share a single unified rating
- [ ] Operator can remove an alias link between two previously linked players
- [ ] Removing an alias triggers a full rating recalculation reverting to separate histories
- [ ] The alias relationship is visible in the player profile view
- [ ] Matches recorded against either alias name are attributed to the unified player for rating purposes

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player Aliasing

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And league 42 "Alpha League" is active with an open Elo season (id 7)
    And "alice" is assigned as operator of league 42
    And player "Alice_old" with id 10 is in league 42 with 5 matches in season 7
    And player "Alice_new" with id 11 is in league 42 with 3 matches in season 7

  Scenario: Operator links two players as aliases
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players/10/aliases with alias_player_id 11
    Then the response status is 202 Accepted
    And the response body contains a job_id
    And both player records 10 and 11 still exist in the database

  Scenario: Both original player records persist after aliasing
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players/10/aliases with alias_player_id 11
    Then the response status is 202 Accepted
    And sending GET /api/players/10 returns 200 OK
    And sending GET /api/players/11 returns 200 OK

  Scenario: Aliasing triggers asynchronous full rating recalculation
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players/10/aliases with alias_player_id 11
    Then the response status is 202 Accepted
    And the response body contains status "queued"
    And a recalculation job is created with status "queued" for season 7
    And while the job is running the leaderboard serves pre-alias ratings

  Scenario: After alias recalculation completes, aliased players share unified rating
    Given player "Alice_old" (id 10) and player "Alice_new" (id 11) are linked as aliases
    And the recalculation job for season 7 has status "completed"
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then player 10 and player 11 reflect the same unified rating computed from all 8 combined matches

  Scenario: Alias relationship is visible in player profile
    Given players 10 and 11 are linked as aliases
    And "alice" is authenticated
    When "alice" sends GET /api/players/10
    Then the response body contains an "aliases" field that includes player id 11
    When "alice" sends GET /api/players/11
    Then the response body contains an "aliases" field that includes player id 10

  Scenario: Matches against either alias are attributed to the unified player for rating
    Given players 10 (Alice_old) and 11 (Alice_new) are linked as aliases
    And the recalculation job for season 7 has status "completed"
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then the leaderboard entry for player 10 reflects a match_count of 8 (5 + 3 combined)

  Scenario: Operator removes an alias link between two players
    Given players 10 and 11 are linked as aliases
    And "alice" is authenticated
    When "alice" sends DELETE /api/leagues/42/players/10/aliases/11
    Then the response status is 202 Accepted
    And the response body contains a new job_id for a recalculation
    And the alias link between 10 and 11 is removed from the database

  Scenario: Removing an alias triggers recalculation reverting to separate histories
    Given players 10 and 11 were linked as aliases but the alias link is now removed
    And the recalculation job for the removal has status "completed"
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then player 10 rating is based only on their own 5 matches
    And player 11 rating is based only on their own 3 matches

  Scenario: Viewer cannot create an alias link
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends POST /api/leagues/42/players/10/aliases with alias_player_id 11
    Then the response status is 403 Forbidden

  Scenario: Operator cannot alias players in a league they are not assigned to
    Given a user "bob_op" with role "operator" is authenticated and NOT assigned to league 42
    When "bob_op" sends POST /api/leagues/42/players/10/aliases with alias_player_id 11
    Then the response status is 403 Forbidden

  Scenario: Aliasing a player to themselves is rejected
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players/10/aliases with alias_player_id 10
    Then the response status is 400 Bad Request
    And the response body contains a validation error indicating self-alias is not permitted

  Scenario: Aliasing a player not enrolled in the league is rejected with 400
    Given player 11 exists in the global roster but is NOT a member of league 42
    And "alice" is authenticated and assigned to league 42
    When "alice" sends POST /api/leagues/42/players/10/aliases with alias_player_id 11
    Then the HTTP response status is 400
    And the response body contains error_code "VALIDATION_ERROR"
    And the message indicates player 11 is not a member of league 42

  Scenario: Aliasing a third player expands the equivalence group transitively
    Given players 10 (A) and 11 (B) are linked as aliases in league 42
    And the recalculation for that alias has status "completed"
    And player 12 (C) is a member of league 42 with 2 matches in season 7
    When "alice" sends POST /api/leagues/42/players/10/aliases with alias_player_id 12
    Then the response status is 202 Accepted
    And a new recalculation job is queued for season 7
    And after the job completes, players 10, 11, and 12 all share the same unified rating
    And the leaderboard entry for player 10 reflects a match_count combining all three players' matches

  Scenario: All members of an alias group are visible in each member's profile
    Given players 10 (A), 11 (B), and 12 (C) are all in the same alias group (A→B and A→C established)
    When "alice" sends GET /api/players/11
    Then the response body contains an "aliases" field including both player id 10 and player id 12
    When "alice" sends GET /api/players/12
    Then the response body contains an "aliases" field including both player id 10 and player id 11
```
