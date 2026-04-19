# UR-LM-002: Season Management

**Status:** Draft
**Parent:** Spec Section 4.2 (Seasons)
**Priority:** Must-have

## Description

A League Operator can manage seasons within a league. Seasons are rating-coherent units: each season records its algorithm type, parameters, start date, and optional end date. A new season is triggered ONLY when the algorithm TYPE changes (not parameter changes). When a new season starts, the operator chooses whether to reset all players to defaults or seed from prior season rankings.

## Rationale

Different algorithms produce incomparable rating scales, so an algorithm type change requires a clean break. Parameter changes within the same algorithm keep ratings comparable, so they do not force a new season. Season transition seeding gives operators flexibility to either start fresh or preserve relative ordering from the prior season.

## Acceptance Criteria

- [ ] Each league has at least one season, created automatically when the league is created
- [ ] Each season records: algorithm type, algorithm parameters, start date, and optional end date
- [ ] Changing the algorithm TYPE (e.g., Elo to Glicko-2) ends the current season and starts a new one
- [ ] Changing algorithm PARAMETERS within the same type does NOT create a new season
- [ ] When a new season is created due to algorithm type change, the operator is presented with two options: (A) reset all players to the new algorithm's defaults, or (B) seed from prior season rankings using ordinal ranking mapped to initial ratings with spread
- [ ] Players carry over across seasons; their ratings are reset or seeded per the operator's choice
- [ ] Players who join mid-season always start at the algorithm's default rating regardless of seeding choice
- [ ] Seasons inherit archive state from their parent league
- [ ] Each season has its own leaderboard and rating timeline
- [ ] A season's end date is set when a new season begins or the league is archived

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Season Management

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And league 42 "Alpha League" exists with status "active"
    And "alice" is assigned as operator of league 42

  Scenario: Creating a league automatically starts the first season
    Given "alice" is authenticated
    When "alice" sends POST /api/leagues with name "New League" and algorithm "elo"
    Then the response status is 201 Created
    And a season is automatically created for the new league with algorithm "elo"
    And the new season has a non-null start_date
    And the new season has a null end_date

  Scenario: Season record stores algorithm type, parameters, start date, and null end date
    Given league 42 has a current open season with id 7
    When "alice" sends GET /api/leagues/42/seasons/7
    Then the response status is 200 OK
    And the response body contains algorithm "elo"
    And the response body contains a non-null start_date
    And the response body contains a null end_date
    And the response body contains algorithm_params with at least k_factor and initial_rating

  Scenario: Changing algorithm type from Elo to Glicko-2 closes current season and opens new one
    Given league 42 current season id 7 uses algorithm "elo" and has null end_date
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/change-algorithm with algorithm "glicko2" and seeding_choice "reset"
    Then the response status is 200 OK
    And season 7 now has a non-null end_date
    And a new season exists for league 42 with algorithm "glicko2" and null end_date

  Scenario: Changing Elo K-factor does NOT create a new season
    Given league 42 current season id 7 uses algorithm "elo" with k_factor 32
    And "alice" is authenticated
    When "alice" sends PATCH /api/leagues/42/seasons/7 with algorithm_params k_factor 24
    Then the response status is 200 OK
    And league 42 still has exactly one open season with id 7
    And season 7 algorithm_params k_factor is 24
    And season 7 end_date remains null

  Scenario: Changing Glicko-2 tau does NOT create a new season
    Given league 43 "Glicko League" current season id 8 uses algorithm "glicko2" with tau 0.5
    And "alice" is assigned as operator of league 43 and authenticated
    When "alice" sends PATCH /api/leagues/43/seasons/8 with algorithm_params tau 0.3
    Then the response status is 200 OK
    And league 43 still has exactly one open season with id 8
    And season 8 algorithm_params tau is 0.3

  Scenario: Season transition with reset seeding initializes all players to default rating
    Given league 42 current season id 7 uses algorithm "elo" with initial_rating 1000
    And players "Alice", "Bob", and "Carol" are in league 42 with ratings 1200, 1100, and 900
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/change-algorithm with algorithm "glicko2" and seeding_choice "reset"
    Then the new season is created with algorithm "glicko2"
    And players "Alice", "Bob", and "Carol" in the new season each have rating equal to glicko2 default initial_mu 1500

  Scenario: Season transition with ordinal seeding preserves relative ordering
    Given league 42 current season id 7 uses algorithm "elo"
    And player "Alice" has final rating 1400 (rank 1) and "Bob" has final rating 1100 (rank 2) in season 7
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/change-algorithm with algorithm "glicko2" and seeding_choice "ordinal"
    Then the new season is created with algorithm "glicko2"
    And player "Alice"'s initial rating in the new season is higher than player "Bob"'s initial rating
    And the seeding values are not all identical (meaningful spread exists)

  Scenario: Mid-season joiner always starts at default rating regardless of seeding choice
    Given league 42 current season id 8 uses algorithm "glicko2" with seeding_choice "ordinal"
    And the season is currently open
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/players with name "Dave" and type "human"
    Then the response status is 201 Created
    And "Dave"'s initial rating in season 8 is the glicko2 default initial_mu 1500

  Scenario: Seasons inherit archive state from parent league
    Given league 42 "Alpha League" is active with season 7 open
    When "alice" sends POST /api/leagues/42/archive
    Then the response status is 200 OK
    And season 7 end_date is set to a non-null timestamp
    And season 7 is effectively closed

  Scenario: Each season has its own leaderboard scoped to that season
    Given league 42 has season 7 (elo) and season 8 (glicko2)
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then the response body contains only ratings from season 7
    When "alice" sends GET /api/seasons/8/leaderboard
    Then the response body contains only ratings from season 8

  Scenario: A season's end_date is set when a new season begins
    Given league 42 current season id 7 uses algorithm "elo" with null end_date
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/change-algorithm with algorithm "trueskill" and seeding_choice "reset"
    Then season 7 has a non-null end_date
    And the end_date value is approximately the current server timestamp

  Scenario: Players carry over across seasons (the player roster persists)
    Given league 42 season 7 with players "Alice" and "Bob"
    And "alice" is authenticated
    When "alice" sends POST /api/leagues/42/change-algorithm with algorithm "trueskill" and seeding_choice "reset"
    Then the new season exists
    And players "Alice" and "Bob" are visible in league 42's player roster

  Scenario: Viewer cannot change algorithm type or season parameters
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends POST /api/leagues/42/change-algorithm with algorithm "trueskill" and seeding_choice "reset"
    Then the response status is 403 Forbidden

  Scenario: Operator cannot manage seasons in a league they are not assigned to
    Given a user "bob" with role "operator" is authenticated and NOT assigned to league 42
    When "bob" sends PATCH /api/leagues/42/seasons/7 with algorithm_params k_factor 16
    Then the response status is 403 Forbidden
```
