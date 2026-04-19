# NFR-PERF-002: API Response Time

**Status:** Draft
**Parent:** Spec Section 6 (Performance)
**Priority:** Must-have

## Description

The REST API must respond within defined latency targets: less than 100ms for single-entity operations (create, read, update on individual resources) and less than 500ms for leaderboard queries with up to 10,000 players. Bulk import must process at least 1,000 matches per second.

## Rationale

Responsive API performance ensures a smooth UI experience for league operators and adequate throughput for swarm operators. The 100ms target for single operations keeps individual interactions feeling instant. The 500ms leaderboard target accounts for the query complexity of ranking 10,000 players with sorting and pagination. The bulk import target ensures that large batches complete in reasonable time.

## Acceptance Criteria

- [ ] Single-entity CRUD operations (create league, add player, record match, get player) respond in less than 100ms (p95)
- [ ] Leaderboard queries for a season with up to 10,000 players respond in less than 500ms (p95)
- [ ] Bulk match processing achieves at least 1,000 matches per second throughput
- [ ] Response times are measured end-to-end at the HTTP layer (request received to response sent)
- [ ] Performance targets are validated under realistic load conditions, not just single-request benchmarks

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: API Response Time

  Background:
    Given the system is running on commodity hardware
    And response times are measured end-to-end at the HTTP layer (request received to response sent)

  Scenario: Single league creation responds within 100ms at p95
    Given the database is in a steady state with 50 existing leagues
    When a League Operator sends a POST /leagues request with valid data
    Then the response arrives within 100 milliseconds at the 95th percentile

  Scenario: Single player retrieval responds within 100ms at p95
    Given a league with 5,000 players exists
    When a client sends a GET /leagues/{id}/players/{player_id} request
    Then the response arrives within 100 milliseconds at the 95th percentile

  Scenario: Single match recording responds within 100ms at p95
    Given a league with 1,000 players and 50,000 match records exists
    When a League Operator sends a POST /leagues/{id}/matches request with valid data
    Then the response arrives within 100 milliseconds at the 95th percentile

  Scenario: Leaderboard query for 10,000 players responds within 500ms at p95
    Given a season with exactly 10,000 players each having at least one recorded match
    And the league uses the TrueSkill algorithm with conservative rankings computed
    When a client sends a GET /leagues/{id}/leaderboard?page_size=100 request
    Then the response arrives within 500 milliseconds at the 95th percentile
    And the response contains 100 correctly ranked players

  Scenario: Leaderboard query for 10,000 Glicko-2 players responds within 500ms at p95
    Given a season with 10,000 players using the Glicko-2 algorithm
    When a client requests the leaderboard
    Then the response arrives within 500 milliseconds at the 95th percentile

  Scenario: Bulk match processing achieves at least 1,000 matches per second
    Given a league with 500 agent players
    When 10,000 matches are submitted to the bulk match import endpoint in a single batch
    Then the total processing time is no more than 10 seconds (1,000 matches/second throughput)
    And all 10,000 matches are recorded with correct rating updates

  Scenario: API response times do not degrade significantly at maximum supported scale
    Given the database contains 100 leagues, 10,000 players in the target league, and 1 million total matches
    When a client sends a GET /leagues/{id}/leaderboard request
    Then the response still arrives within 500 milliseconds at the 95th percentile
```
