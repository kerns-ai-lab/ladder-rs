# UR-ME-001: Match Entry

**Status:** Draft
**Parent:** Spec Section 4.4 (Match Entry)
**Priority:** Must-have

## Description

A League Operator can record match results through the web UI. The UI adapts to the selected algorithm: showing/hiding the draw option, presenting ranked placement UI for N-player events. After recording a match, the user returns to the ratings list/leaderboard. Match recording and rating update are atomic. Matches are timestamped at submission and the timestamp determines processing order.

## Rationale

Match entry is the primary data input mechanism for league operators. Algorithm-aware validation prevents invalid data (e.g., draws in a zero-draw-probability TrueSkill league). Atomic transactions ensure data consistency. Returning to the leaderboard after entry gives immediate feedback on rating changes.

## Acceptance Criteria

- [ ] Operator can select a league, select participants, and record a match outcome
- [ ] For 1v1 matches, outcomes are win/loss/draw
- [ ] For N-player ranked events, outcomes are ranked placements (1st through Nth)
- [ ] The UI adapts to the selected algorithm: draw option is hidden when algorithm configuration makes draws impossible (e.g., TrueSkill with draw_probability=0)
- [ ] Match entry triggers immediate rating recalculation for all involved players
- [ ] Match recording and rating update execute as a single atomic transaction
- [ ] Matches are timestamped at the moment of submission
- [ ] After successful match recording, the UI navigates to the leaderboard view
- [ ] TrueSkill matches that do not fully converge are recorded with a convergence_quality flag (degraded confidence) and are never rejected
- [ ] Match records include an optional score field for display purposes (scores are not used in rating calculations)
- [ ] Matches cannot be recorded in a closed (ended) season
- [ ] There is no upper bound on the number of participants (N) in a ranked event; the system accepts any N >= 2
- [ ] NFR-PERF-001 latency targets apply to 1v1 matches; larger N-player events are expected to take proportionally longer

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Match Entry

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And league 42 "Alpha League" is active
    And "alice" is assigned as operator of league 42

  Scenario: Operator records a 1v1 win/loss match in an Elo league
    Given league 42 has an open Elo season (id 7, k_factor 32, initial_rating 1000)
    And players "Alice" (id 1, rating 1000) and "Bob" (id 2, rating 1000) are in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with participants: player 1 placement 1, player 2 placement 2
    Then the response status is 201 Created
    And the response body contains new ratings for both players
    And player 1's new rating is greater than 1000 (winner gains)
    And player 2's new rating is less than 1000 (loser loses)
    And a rating snapshot exists for player 1 in season 7 after this match
    And a rating snapshot exists for player 2 in season 7 after this match

  Scenario: Operator records a 1v1 draw in an Elo league
    Given league 42 has an open Elo season (id 7)
    And players "Alice" (id 1, rating 1000) and "Bob" (id 2, rating 1000) are in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with participants: player 1 is_draw true, player 2 is_draw true
    Then the response status is 201 Created
    And both players' ratings remain approximately 1000 (draw with equal ratings produces no change)

  Scenario: Match recording and rating update are atomic
    Given league 42 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with valid participants
    Then the response status is 201 Created
    And the match record exists in the database
    And rating snapshots exist for both players linked to the same match_id

  Scenario: Match timestamp is set at submission time
    Given league 42 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with valid participants at time T
    Then the match record has a recorded_at timestamp within 1 second of T

  Scenario: After recording a match, the UI navigates to the leaderboard
    Given league 42 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 42
    And "alice" is authenticated
    When "alice" submits a match via the UI for season 7
    Then the UI navigates to the leaderboard view for season 7

  Scenario: Operator records a 5-player ranked TrueSkill event
    Given league 43 "TS League" has an open TrueSkill season (id 8, draw_probability 0.1)
    And players with ids 1, 2, 3, 4, 5 are in league 43
    And "alice" is assigned to league 43 and authenticated
    When "alice" sends POST /api/seasons/8/matches with 5 participants each assigned a distinct placement 1-5
    Then the response status is 201 Created
    And 5 rating snapshots are created, one per player in season 8

  Scenario: TrueSkill draw rejection when draw_probability is 0
    Given league 43 "TS League" has an open TrueSkill season (id 8, draw_probability 0.0)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 43
    And "alice" is assigned to league 43 and authenticated
    When "alice" sends POST /api/seasons/8/matches with both players marked is_draw true
    Then the response status is 400 Bad Request
    And the response body contains a validation error indicating draws are not allowed for this TrueSkill configuration

  Scenario: TrueSkill non-converged match is recorded with degraded convergence_quality
    Given league 43 "TS League" has an open TrueSkill season (id 8)
    And the TrueSkill algorithm returns BestApproximation for this match
    And "alice" is assigned to league 43 and authenticated
    When "alice" sends POST /api/seasons/8/matches with valid TrueSkill participants
    Then the response status is 201 Created
    And the match record has convergence_quality "degraded"
    And the response body includes convergence_quality "degraded"

  Scenario: Match in a closed season is rejected
    Given season 6 in league 42 has end_date set (season is closed)
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/6/matches with valid participants
    Then the response status is 409 Conflict
    And the response body contains error_code "SEASON_CLOSED"

  Scenario: Match with optional score metadata is accepted
    Given league 42 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with participants and score_metadata "{'set_score': '6-3'}"
    Then the response status is 201 Created
    And the match record stores the score_metadata as-is

  Scenario: Match entry with N=2 minimum participants is accepted
    Given league 42 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with 2 participants each with valid placements
    Then the response status is 201 Created

  Scenario: Match entry with N=10 participants is accepted
    Given league 42 has an open Elo season (id 7)
    And 10 active players are in league 42
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches with 10 participants each with distinct placements 1-10
    Then the response status is 201 Created
    And 10 rating snapshots are created in season 7

  Scenario: Unauthenticated match recording is rejected
    When an unauthenticated client sends POST /api/seasons/7/matches with valid participants
    Then the response status is 401 Unauthorized

  Scenario: Viewer cannot record a match
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends POST /api/seasons/7/matches with valid participants
    Then the response status is 403 Forbidden

  Scenario: Operator cannot record a match in a league they are not assigned to
    Given a user "bob_op" with role "operator" is authenticated and NOT assigned to league 42
    When "bob_op" sends POST /api/seasons/7/matches with valid participants in league 42
    Then the response status is 403 Forbidden
```
