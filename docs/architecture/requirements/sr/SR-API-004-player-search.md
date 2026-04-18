# SR-API-004: Player Search Endpoint

**Status:** Draft
**Parent:** UR-PM-001, RQ-R3-6
**Priority:** Must-have

## Description

The system exposes a player search endpoint that supports compound filtering: name type-ahead autocomplete, player type filter (human/non-human), and an optional toggle to exclude players who are already members of a specified league. Filtering is applied server-side. Results are paginated.

## Rationale

League Operators need to find specific player records when adding them to a league. A single search box with type-ahead autocomplete reduces friction. The player type filter helps operators distinguish human competitors from AI agents. The "exclude already in league" toggle prevents accidental re-addition of existing members, which would result in a no-op or error. Server-side filtering prevents leaking the full player list to the client on every keystroke.

## Acceptance Criteria

- [ ] The system exposes a `GET /players/search` endpoint (or equivalent) accessible to authenticated users with League Operator or Admin roles
- [ ] The endpoint accepts a `q` query parameter for name prefix matching; it returns players whose name starts with or contains the supplied string (case-insensitive); an empty or absent `q` returns all players (subject to pagination)
- [ ] The endpoint accepts a `player_type` query parameter with values `human`, `non_human`, or `all` (default: `all`); results are filtered to match the specified type
- [ ] The endpoint accepts a `exclude_league_id` query parameter (a league ID); when supplied, players already belonging to that league are excluded from results
- [ ] All filtering is applied server-side before the response is sent; the client receives only the filtered subset
- [ ] Results are paginated using the platform's standard pagination mechanism (cursor-based or offset-based per SR-API-001); page size is configurable with a server-enforced maximum
- [ ] The response includes at minimum: player ID, canonical `name`, `nickname` (if set, otherwise null), `player_type`, and the list of league names/IDs the player currently belongs to
- [ ] Name matching (`q` parameter) is applied to both the `name` field and the `nickname` field (case-insensitive); a player matches if either field starts with or contains the query string
- [ ] The response displays `nickname` as the primary display label where set; consumers should apply the rule `nickname ?? name` for all display purposes
- [ ] The endpoint returns an empty result set (not an error) when no players match the supplied filters
- [ ] The endpoint returns a 400 error if `exclude_league_id` is supplied but the league ID does not exist
- [ ] Player/Viewer role users do not have access to this endpoint (returns 403)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player Search Endpoint

  Background:
    Given the system is running
    And the global player roster contains:
      | Name       | player_type | League memberships         |
      | Alice      | human       | ["League A", "League B"]   |
      | Alicia     | human       | ["League A"]               |
      | Bob        | human       | ["League B"]               |
      | RoboAlpha  | non_human   | ["League A"]               |
      | Zara       | human       | []                         |
    And league "League A" exists with id "league-a"
    And league "League C" exists with id "league-c" and no members

  Scenario: Name prefix match returns all players whose name contains the query string (case-insensitive)
    Given a League Operator is authenticated
    When a client requests GET /players/search?q=ali
    Then the response includes "Alice" and "Alicia"
    And the response does not include "Bob", "RoboAlpha", or "Zara"

  Scenario: Name prefix match is case-insensitive
    Given a League Operator is authenticated
    When a client requests GET /players/search?q=ALICE
    Then the response includes "Alice"

  Scenario: Absent q parameter returns all players (subject to pagination)
    Given a League Operator is authenticated
    When a client requests GET /players/search with no q parameter
    Then the response includes all 5 players
    And pagination is applied per the platform default page size

  Scenario: Empty q parameter returns all players
    Given a League Operator is authenticated
    When a client requests GET /players/search?q=
    Then the response includes all 5 players

  Scenario: player_type=human filters to human players only
    Given a League Operator is authenticated
    When a client requests GET /players/search?player_type=human
    Then the response includes "Alice", "Alicia", "Bob", and "Zara"
    And the response does not include "RoboAlpha"

  Scenario: player_type=non_human filters to non-human players only
    Given a League Operator is authenticated
    When a client requests GET /players/search?player_type=non_human
    Then the response includes "RoboAlpha"
    And the response does not include any human players

  Scenario: player_type=all returns all player types
    Given a League Operator is authenticated
    When a client requests GET /players/search?player_type=all
    Then the response includes all 5 players

  Scenario: exclude_league_id excludes players already in the specified league
    Given a League Operator is authenticated
    When a client requests GET /players/search?exclude_league_id=league-a
    Then the response includes "Bob" and "Zara"
    And the response does not include "Alice", "Alicia", or "RoboAlpha"

  Scenario: Combined name search and exclude_league_id filter
    Given a League Operator is authenticated
    When a client requests GET /players/search?q=ali&exclude_league_id=league-a
    Then the response does not include "Alice" or "Alicia" (both are in League A)
    And the response is empty or contains only non-League-A players matching "ali"

  Scenario: No players match the filter — returns empty array, not 404
    Given a League Operator is authenticated
    When a client requests GET /players/search?q=zzznomatch
    Then the HTTP response status is 200
    And the response body contains an empty results array

  Scenario: exclude_league_id with non-existent league ID returns 400
    Given a League Operator is authenticated
    When a client requests GET /players/search?exclude_league_id=does-not-exist
    Then the HTTP response status is 400
    And the response body contains "error_code" and "message" indicating the league was not found

  Scenario: Response includes required fields for each player
    Given a League Operator is authenticated
    When a client requests GET /players/search?q=alice
    Then each player in the response includes:
      | field       |
      | id          |
      | name        |
      | player_type |
      | leagues     |

  Scenario: Results are paginated using the platform standard pagination mechanism
    Given a League Operator is authenticated
    And the global player roster contains 200 players
    When a client requests GET /players/search?q=&page_size=50
    Then the response contains at most 50 players
    And pagination fields (next_cursor or total_count) are present in the response

  Scenario: Player/Viewer role cannot access the search endpoint
    Given a Player/Viewer is authenticated
    When the Player/Viewer requests GET /players/search?q=alice
    Then the HTTP response status is 403
    And the response body contains "error_code" indicating insufficient permissions

  Scenario: Unauthenticated request returns 401
    Given no session cookie is present
    When a client requests GET /players/search?q=alice
    Then the HTTP response status is 401

  Scenario: Filtering is applied server-side — full player list is not sent to client
    Given a League Operator is authenticated
    And the global roster has 500 players
    When a client requests GET /players/search?q=ali
    Then the response body contains only players matching "ali"
    And the response does not include any player whose name does not match the query
```
