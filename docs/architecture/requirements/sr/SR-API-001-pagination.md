# SR-API-001: Pagination

**Status:** Draft
**Parent:** UR-LB-001, UR-SW-001
**Priority:** Must-have

## Description

The REST API supports both cursor-based and offset-based pagination for list endpoints. Cursor-based pagination is used for sequential scrolling (e.g., infinite scroll through a leaderboard). Offset-based pagination is used for random access (e.g., "jump to rank 500"). Both modes are available on the same endpoints.

## Rationale

Cursor-based pagination provides stable, performant iteration through large result sets even as data changes. Offset-based pagination enables random access which is essential for leaderboard navigation (jumping to a specific rank position). Supporting both covers the primary access patterns for league operators and swarm dashboards.

## Acceptance Criteria

- [ ] List endpoints accept cursor-based pagination parameters (cursor token, page size)
- [ ] List endpoints accept offset-based pagination parameters (offset, limit)
- [ ] Cursor-based responses include a next_cursor token for fetching subsequent pages
- [ ] Offset-based responses include total_count to support page number calculation
- [ ] Default page size is applied when no pagination parameters are provided
- [ ] Maximum page size is enforced to prevent excessively large responses
- [ ] Cursor tokens are opaque to the client (implementation detail of the server)
- [ ] Both pagination modes produce consistent results when the underlying data does not change between requests

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: API Pagination

  Background:
    Given the system is running
    And a League Operator is authenticated
    And league "Big League" has 250 players on the leaderboard

  Scenario: Cursor-based pagination returns first page with next_cursor token
    When a client requests GET /leagues/big-league/leaderboard?page_size=50
    Then the response contains exactly 50 players
    And the response body includes a "next_cursor" field with a non-empty opaque string
    And the response does not include a "total_count" field in cursor mode

  Scenario: Cursor-based pagination fetches subsequent page using next_cursor
    Given the client received next_cursor = "abc123" from the first page
    When the client requests GET /leagues/big-league/leaderboard?cursor=abc123&page_size=50
    Then the response contains the next 50 players in sequence
    And the players do not overlap with the first page results

  Scenario: Cursor-based pagination last page returns null next_cursor
    Given the client is on the last page of results
    When the client requests the next page using a cursor token
    Then the "next_cursor" field in the response is null or absent

  Scenario: Offset-based pagination returns results at specified offset
    When a client requests GET /leagues/big-league/leaderboard?offset=100&limit=50
    Then the response contains 50 players starting from position 101 in the full result set
    And the response body includes a "total_count" field with value 250

  Scenario: Offset-based pagination includes total_count for page number calculation
    When a client requests GET /leagues/big-league/leaderboard?offset=0&limit=25
    Then the response body includes "total_count": 250
    And the client can calculate that there are 10 pages of 25 results

  Scenario: Default page size is applied when no pagination parameters are provided
    When a client requests GET /leagues/big-league/leaderboard with no pagination parameters
    Then the response contains exactly the server-defined default number of results
    And the default page size is greater than 0

  Scenario: Maximum page size is enforced
    When a client requests GET /leagues/big-league/leaderboard?limit=10000
    Then the request is rejected with HTTP 400
    And the error message indicates the maximum allowed page size

  Scenario: Cursor token is opaque to the client
    When a client receives a next_cursor value in a response
    Then the cursor value is not a human-readable offset, page number, or record ID
    And decoding the cursor does not expose server-internal implementation details

  Scenario: Cursor-based pagination produces consistent results when data does not change
    Given the leaderboard data is not modified between requests
    When a client iterates through all pages using cursor-based pagination
    Then each player appears exactly once across all pages
    And the total number of results equals 250

  Scenario: Offset-based pagination produces consistent results when data does not change
    Given the leaderboard data is not modified between requests
    When a client fetches pages with offset=0 limit=100, offset=100 limit=100, offset=200 limit=100
    Then the combined results contain exactly 250 unique players with no duplicates or omissions

  Scenario: Both pagination modes are available on the same leaderboard endpoint
    When a client requests the leaderboard with cursor parameters
    Then the server responds with cursor-based pagination fields
    When a client requests the leaderboard with offset/limit parameters
    Then the server responds with offset-based pagination fields including total_count

  Scenario: Offset beyond total record count returns empty results, not an error
    When a client requests GET /leagues/big-league/leaderboard?offset=500&limit=25
    Then the response contains an empty results array
    And the response includes "total_count": 250
    And the HTTP status is 200
```
