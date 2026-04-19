# SR-API-003: Server-Side Filtering and Sorting

**Status:** Draft
**Parent:** UR-LB-001, UR-SW-001, UR-LM-001
**Priority:** Must-have

## Description

The REST API supports server-side filtering and sorting on list endpoints. Clients can specify filter criteria (e.g., league status, player type, season) and sort order (e.g., rating descending, name ascending) as query parameters. Filtering and sorting are applied at the database level, not in application code.

## Rationale

Client-side filtering and sorting requires transferring the full dataset to the client, which does not scale to 10,000-player leaderboards. Server-side processing reduces bandwidth, improves response times, and keeps the frontend simple. Database-level execution leverages indexes for performance.

## Acceptance Criteria

- [ ] League list endpoint supports filtering by status (active, archived)
- [ ] Player list endpoint supports filtering by player type (human, non-human) and active/inactive status
- [ ] Leaderboard endpoint supports sorting by any displayed column (rating, name, match count)
- [ ] Sort direction (ascending/descending) is configurable per field
- [ ] Filter and sort parameters are passed as query parameters
- [ ] Invalid filter or sort parameters return a structured error response (not silently ignored)
- [ ] Filtering and sorting are performed at the database query level using appropriate indexes

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Server-Side Filtering and Sorting

  Background:
    Given the system is running
    And a League Operator is authenticated
    And the system has 3 active leagues and 2 archived leagues
    And league "Alpha League" has 50 players including 30 human and 20 non-human, of which 5 are inactive

  Scenario: League list filtered by status=active returns only active leagues
    When a client requests GET /leagues?status=active
    Then the response contains exactly 3 leagues
    And all returned leagues have status "active"
    And no archived leagues appear in the response

  Scenario: League list filtered by status=archived returns only archived leagues
    When a client requests GET /leagues?status=archived
    Then the response contains exactly 2 leagues
    And all returned leagues have status "archived"

  Scenario: Player list filtered by player_type=human returns only human players
    When a client requests GET /leagues/alpha-league/players?player_type=human
    Then the response contains exactly 30 players
    And all returned players have player_type = "human"
    And no non-human players appear in the response

  Scenario: Player list filtered by player_type=non_human returns only non-human players
    When a client requests GET /leagues/alpha-league/players?player_type=non_human
    Then the response contains exactly 20 players
    And all returned players have player_type = "non_human"

  Scenario: Player list filtered by active status excludes inactive players
    When a client requests GET /leagues/alpha-league/players?active=true
    Then the response contains exactly 45 players
    And all returned players have active status = true

  Scenario: Leaderboard sorted by rating descending returns highest-rated player first
    Given "Alpha League" uses Elo and player "Alice" has the highest rating 1400
    When a client requests GET /leagues/alpha-league/leaderboard?sort=rating&direction=desc
    Then the first player in the response is "Alice" with rating 1400
    And ratings decrease monotonically through the response

  Scenario: Leaderboard sorted by rating ascending returns lowest-rated player first
    Given "Alpha League" uses Elo and player "Zara" has the lowest rating 900
    When a client requests GET /leagues/alpha-league/leaderboard?sort=rating&direction=asc
    Then the first player in the response is "Zara" with rating 900
    And ratings increase monotonically through the response

  Scenario: Leaderboard sorted by name ascending returns alphabetical order
    When a client requests GET /leagues/alpha-league/leaderboard?sort=name&direction=asc
    Then the returned players are ordered alphabetically by name A-Z

  Scenario: Leaderboard sorted by match_count ascending returns players with fewest matches first
    When a client requests GET /leagues/alpha-league/leaderboard?sort=match_count&direction=asc
    Then the first player in the response has the lowest match count

  Scenario: Invalid sort parameter returns structured error, not silent fallback
    When a client requests GET /leagues/alpha-league/leaderboard?sort=invalid_column
    Then the HTTP response status is 400
    And the response body contains "error_code" and "message" identifying "sort" as the invalid parameter

  Scenario: Invalid filter parameter returns structured error, not silent fallback
    When a client requests GET /leagues?status=unknown_status
    Then the HTTP response status is 400
    And the response body contains "error_code" indicating the invalid filter value

  Scenario: Invalid sort direction returns structured error
    When a client requests GET /leagues/alpha-league/leaderboard?sort=rating&direction=sideways
    Then the HTTP response status is 400
    And the response body contains "error_code" and "message" identifying "direction" as invalid

  Scenario: Filtering and sorting are applied together in a single request
    When a client requests GET /leagues/alpha-league/players?player_type=human&sort=name&direction=asc
    Then the response contains only human players
    And those human players are ordered alphabetically by name

  Scenario: Filtering is enforced server-side — response does not contain excluded records
    When a client requests GET /leagues/alpha-league/players?player_type=human
    Then the response body contains no non-human player records
    And the client cannot derive non-human player data from the response
```
