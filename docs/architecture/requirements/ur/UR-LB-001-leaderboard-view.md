# UR-LB-001: Leaderboard View

**Status:** Draft
**Parent:** Spec Section 4.6 (Leaderboards)
**Priority:** Must-have

## Description

A League Operator can view current rankings for any league and season. The leaderboard displays rank, player name, current rating, rating deviation/uncertainty (where applicable), and match count. Columns are sortable. The leaderboard is scoped to a single season.

## Rationale

Leaderboards are the primary output of the rating system and the main reason operators use the platform. Per-season scoping ensures ratings are compared only within a coherent algorithm context.

## Acceptance Criteria

- [ ] Leaderboard displays players ordered by conservative estimate (descending) for the selected league and season: Elo uses raw rating, Glicko-2 uses mu - 2*RD, TrueSkill uses mu - 3*sigma
- [ ] Each row shows: rank, player name, current rating, rating deviation/uncertainty (for Glicko-2 and TrueSkill), and match count
- [ ] Rating deviation/uncertainty columns are shown or hidden based on the season's algorithm (not shown for Elo)
- [ ] Columns are sortable (at minimum: rank, name, rating, match count)
- [ ] Soft-deleted (inactive) players are excluded from the leaderboard
- [ ] Leaderboard is scoped to a single season; operator can select which season to view
- [ ] Leaderboard data refreshes on request (no real-time updates)
- [ ] New players with high uncertainty are ranked lower than established players with similar mean ratings due to the conservative estimate formula

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Leaderboard View

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And league 42 "Alpha League" is active
    And "alice" is assigned as operator of league 42

  Scenario: Elo leaderboard ranks players by raw rating descending
    Given league 42 has a TrueSkill season (id 7) — actually an Elo season
    And league 42 has an open Elo season (id 7)
    And players in season 7 have ratings: "Alice" 1250, "Bob" 1100, "Carol" 950
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then the response status is 200 OK
    And player "Alice" appears at rank 1 with rating 1250
    And player "Bob" appears at rank 2 with rating 1100
    And player "Carol" appears at rank 3 with rating 950

  Scenario: Glicko-2 leaderboard ranks players by (mu - 2*RD) conservative estimate
    Given league 43 "Glicko League" has an open Glicko-2 season (id 8)
    And player "Alice" has mu 1600 and RD 50 (conservative_rating = 1600 - 2*50 = 1500)
    And player "Bob" has mu 1700 and RD 150 (conservative_rating = 1700 - 2*150 = 1400)
    And "alice" is assigned to league 43 and authenticated
    When "alice" sends GET /api/seasons/8/leaderboard
    Then player "Alice" appears at rank 1 (conservative_rating 1500)
    And player "Bob" appears at rank 2 (conservative_rating 1400)
    And each row shows mu, RD, and conservative_rating

  Scenario: TrueSkill leaderboard ranks players by (mu - 3*sigma) conservative estimate
    Given league 44 "TS League" has an open TrueSkill season (id 9)
    And player "Alice" has mu 30.0 and sigma 2.0 (conservative_rating = 30.0 - 3*2.0 = 24.0)
    And player "Bob" has mu 35.0 and sigma 4.0 (conservative_rating = 35.0 - 3*4.0 = 23.0)
    And "alice" is assigned to league 44 and authenticated
    When "alice" sends GET /api/seasons/9/leaderboard
    Then player "Alice" appears at rank 1 (conservative_rating 24.0)
    And player "Bob" appears at rank 2 (conservative_rating 23.0)
    And each row shows mu, sigma, and conservative_rating

  Scenario: Leaderboard row includes rank, name, rating, deviation/uncertainty, and match count
    Given league 43 "Glicko League" has an open Glicko-2 season (id 8)
    And player "Alice" has mu 1500, RD 200, and 8 matches played
    And "alice" is assigned to league 43 and authenticated
    When "alice" sends GET /api/seasons/8/leaderboard
    Then the response body for "Alice" contains rank, name, rating (mu), deviation (RD), and match_count 8

  Scenario: Elo leaderboard does not show deviation or uncertainty columns
    Given league 42 has an open Elo season (id 7)
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then the response body does not contain "deviation" or "uncertainty" fields for any player

  Scenario: Soft-deleted players are excluded from the leaderboard
    Given player "Dave" (id 4) has been soft-deleted from league 42 season 7
    And players "Alice", "Bob", and "Carol" are active in season 7
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then the response body contains "Alice", "Bob", and "Carol"
    And the response body does not contain "Dave"

  Scenario: Leaderboard is scoped to a single season
    Given league 42 has season 7 (Elo) and season 8 (Glicko-2) with different players
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then only players with ratings in season 7 are returned
    When "alice" sends GET /api/seasons/8/leaderboard
    Then only players with ratings in season 8 are returned

  Scenario: New player with high uncertainty ranks below established player with similar mean
    Given league 44 "TS League" has an open TrueSkill season (id 9)
    And player "Veteran" has mu 25.0 and sigma 2.0 (conservative_rating = 19.0)
    And player "Newcomer" has mu 25.0 and sigma 8.333 (conservative_rating = 25.0 - 24.999 = 0.001)
    And "alice" is assigned to league 44 and authenticated
    When "alice" sends GET /api/seasons/9/leaderboard
    Then player "Veteran" appears at a higher rank than player "Newcomer"

  Scenario: Leaderboard supports sorting by name ascending
    Given league 42 has season 7 with players "Charlie", "Alice", "Bob"
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard?sort=name&direction=asc
    Then the response body lists players in alphabetical order: "Alice", "Bob", "Charlie"

  Scenario: Leaderboard supports sorting by match count
    Given league 42 has season 7 with player "Alice" (12 matches) and "Bob" (3 matches)
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard?sort=match_count&direction=desc
    Then "Alice" appears before "Bob"

  Scenario: Leaderboard data refreshes on request (not real-time)
    Given "alice" records a match that changes "Bob"'s rating
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard immediately after
    Then the response reflects Bob's updated rating

  Scenario: Leaderboard request for non-existent season returns 404
    And "alice" is authenticated
    When "alice" sends GET /api/seasons/9999/leaderboard
    Then the response status is 404 Not Found

  Scenario: Unauthenticated request to leaderboard is rejected
    When an unauthenticated client sends GET /api/seasons/7/leaderboard
    Then the response status is 401 Unauthorized
```
