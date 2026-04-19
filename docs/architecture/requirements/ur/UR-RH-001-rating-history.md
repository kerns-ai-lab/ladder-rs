# UR-RH-001: Rating History

**Status:** Draft
**Parent:** Spec Section 4.3 (Player CRUD - rating history), Spec Section 4.6, RQ5 decision
**Priority:** Must-have

## Description

A League Operator can view a player's rating history. This includes per-season detail charts showing match-by-match rating progression, and a season overview showing the final rating per season (table or card format). There is no cross-season combined chart because different algorithms produce incomparable rating scales. Navigation between seasons uses a season picker.

## Rationale

Rating history gives operators and players visibility into competitive trajectory. Per-season scoping is essential because algorithm changes create incomparable scales. The season overview provides a high-level summary without forcing the operator to drill into each season individually.

## Acceptance Criteria

- [ ] Operator can view a per-season detail chart showing match-by-match rating progression for a selected player and season
- [ ] The detail chart plots rating after each match in chronological order
- [ ] Operator can view a season overview showing the final rating achieved in each season (displayed as a table or card layout)
- [ ] No cross-season combined chart is presented (scales are incomparable across algorithm changes)
- [ ] A season picker allows navigation between seasons for the detail view
- [ ] Rating history is accessible from the player profile view
- [ ] Rating deviation/uncertainty is shown alongside rating where applicable (Glicko-2, TrueSkill)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Rating History

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And league 42 "Alpha League" is active
    And "alice" is assigned as operator of league 42
    And player "Alice" (id 1) is in league 42
    And player "Alice" has played 5 matches in Elo season 7 with ratings [1000, 1016, 1029, 1044, 1057] after each match

  Scenario: Operator views per-season detail chart for a player
    Given "alice" is authenticated
    When "alice" sends GET /api/players/1/seasons/7/history
    Then the response status is 200 OK
    And the response body contains 5 rating history entries in chronological order
    And the first entry has rating 1016 (after match 1)
    And the fifth entry has rating 1057 (after match 5)

  Scenario: Rating history entries are returned in chronological match order
    Given "alice" is authenticated
    When "alice" sends GET /api/players/1/seasons/7/history
    Then the response body entries are ordered by match timestamp ascending
    And no entry's timestamp is later than the entry that follows it

  Scenario: Season overview shows final rating achieved in each season
    Given player "Alice" (id 1) has participated in season 7 (Elo, final rating 1057) and season 8 (Glicko-2, final mu 1530)
    And "alice" is authenticated
    When "alice" sends GET /api/players/1/seasons
    Then the response status is 200 OK
    And the response body contains an entry for season 7 with final_rating 1057
    And the response body contains an entry for season 8 with final_rating 1530

  Scenario: No cross-season combined chart is presented
    Given player "Alice" has history in both Elo season 7 and Glicko-2 season 8
    And "alice" is authenticated
    When "alice" sends GET /api/players/1/seasons/7/history
    Then the response body contains only season 7 data
    When "alice" sends GET /api/players/1/seasons/8/history
    Then the response body contains only season 8 data
    And there is no endpoint that returns combined cross-season rating data

  Scenario: Glicko-2 rating history includes deviation alongside rating
    Given league 43 "Glicko League" has Glicko-2 season (id 8)
    And player "Alice" (id 1) has 3 matches in season 8 with (mu, RD) pairs: (1500, 350), (1520, 300), (1545, 260)
    And "alice" is assigned to league 43 and authenticated
    When "alice" sends GET /api/players/1/seasons/8/history
    Then each entry contains both "rating" (mu) and "deviation" (RD) values
    And the first entry has rating 1520 and deviation 300
    And the third entry has rating 1545 and deviation 260

  Scenario: TrueSkill rating history includes uncertainty (sigma) alongside rating
    Given league 44 "TS League" has TrueSkill season (id 9)
    And player "Alice" (id 1) has 2 matches in season 9 with (mu, sigma) pairs: (25.5, 7.1), (26.2, 6.3)
    And "alice" is assigned to league 44 and authenticated
    When "alice" sends GET /api/players/1/seasons/9/history
    Then each entry contains both "rating" (mu) and "uncertainty" (sigma) values

  Scenario: Elo rating history does not include deviation or uncertainty
    Given league 42 has Elo season (id 7)
    And "alice" is authenticated
    When "alice" sends GET /api/players/1/seasons/7/history
    Then the response body entries do not contain "deviation" or "uncertainty" fields

  Scenario: Rating history is accessible from the player profile view
    Given "alice" is authenticated
    When "alice" sends GET /api/players/1
    Then the response body contains a link or reference to the player's season history endpoint

  Scenario: Rating history for a non-existent player returns 404
    Given "alice" is authenticated
    When "alice" sends GET /api/players/9999/seasons/7/history
    Then the response status is 404 Not Found

  Scenario: Rating history for a season with no matches returns an empty list
    Given player "NewPlayer" (id 20) was just added to league 42 season 7 and has played 0 matches
    And "alice" is authenticated
    When "alice" sends GET /api/players/20/seasons/7/history
    Then the response status is 200 OK
    And the response body contains an empty entries array

  Scenario: Soft-deleted player's rating history is still accessible
    Given player "Carol" (id 3) has been soft-deleted from league 42
    And "Carol" has 4 matches in season 7
    And "alice" is authenticated
    When "alice" sends GET /api/players/3/seasons/7/history
    Then the response status is 200 OK
    And the response body contains 4 rating history entries

  Scenario: Rating history is accessible via the season-centric URL alias
    Given "alice" is authenticated
    When "alice" sends GET /api/seasons/7/players/1/history
    Then the response status is 200 OK
    And the response body is identical to GET /api/players/1/seasons/7/history

  Scenario: Season overview is accessible via the player-centric URL
    Given "alice" is authenticated
    When "alice" sends GET /api/players/1/seasons
    Then the response status is 200 OK
    And the response body contains an entry for each season player 1 has participated in
```
