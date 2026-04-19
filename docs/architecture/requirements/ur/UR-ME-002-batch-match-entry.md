# UR-ME-002: Batch Match Entry

**Status:** Draft
**Parent:** Spec Section 4.5 (Bulk Match Import), RQ6 decision
**Priority:** Should-have

## Description

A League Operator can enter multiple matches in a batch through a UI-driven workflow. This is NOT a raw file upload. The UI provides an interactive batch entry interface that handles player resolution, validation, and confirmation. Matches are processed sequentially in entry order because order matters for rating calculation.

## Rationale

Operators frequently need to record results from an entire tournament round or session at once. A UI-driven approach (rather than raw CSV/JSON upload) provides better validation feedback, player resolution assistance, and error handling. Sequential processing ensures rating calculations are deterministic.

## Acceptance Criteria

- [ ] Operator can enter multiple match results in a single batch workflow through the UI
- [ ] The UI validates each match entry interactively (player resolution, outcome validity)
- [ ] The operator can review and confirm the full batch before submission
- [ ] Errors on individual matches are reported per-entry without aborting the entire batch
- [ ] Matches are processed sequentially in the order they appear in the batch
- [ ] Non-convergence on individual TrueSkill matches does not abort the batch; affected matches are flagged and processing continues
- [ ] Each match in the batch is atomically recorded with its rating update
- [ ] The batch entry mechanism is entirely UI-driven (no raw CSV/JSON file upload)
- [ ] Batch match entry supports 1v1 matches only for v1; N-player ranked events must be entered individually through the standard match entry form

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Batch Match Entry

  Background:
    Given the platform is running and the database is initialized
    And a user "alice" with role "operator" exists and is authenticated
    And league 42 "Alpha League" is active
    And league 42 has an open Elo season (id 7, k_factor 32, initial_rating 1000)
    And "alice" is assigned as operator of league 42
    And players "Alice" (id 1), "Bob" (id 2), "Carol" (id 3), and "Dave" (id 4) are in league 42

  Scenario: Operator submits a valid batch of three 1v1 matches
    Given "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches/batch with matches:
      | match_order | winner_id | loser_id |
      | 1           | 1         | 2        |
      | 2           | 3         | 1        |
      | 3           | 2         | 4        |
    Then the response status is 201 Created
    And 3 match records exist in season 7 in order
    And rating snapshots are created for all involved players after each match
    And the ratings reflect sequential processing (match 1 result affects match 2 starting ratings)

  Scenario: Batch matches are processed in entry order affecting subsequent ratings
    Given "alice" is authenticated
    And player "Alice" (id 1) initial rating is 1000 and player "Bob" (id 2) initial rating is 1000
    When "alice" sends POST /api/seasons/7/matches/batch with matches in order:
      | 1: Alice (winner) vs Bob (loser) |
      | 2: Alice (winner) vs Bob (loser) |
    Then after match 1, Alice's rating increases and Bob's decreases
    And match 2 is computed using the post-match-1 ratings (not the original 1000/1000)

  Scenario: Error on one batch entry is reported per-entry without aborting valid entries
    Given "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches/batch with matches:
      | match_order | winner_id | loser_id | note                         |
      | 1           | 1         | 2        | valid                        |
      | 2           | 9999      | 2        | invalid: player 9999 unknown |
      | 3           | 3         | 4        | valid                        |
    Then the response body contains per-entry results
    And entry 1 has status "success"
    And entry 2 has status "error" with a message indicating player 9999 does not exist
    And entry 3 has status "success"
    And 2 match records are created (entries 1 and 3)

  Scenario: TrueSkill non-convergence on a batch entry does not abort the batch
    Given league 43 "TS League" has an open TrueSkill season (id 8)
    And "alice" is assigned to league 43 and authenticated
    And match 2 in the batch causes TrueSkill to return BestApproximation (non-converged)
    When "alice" sends POST /api/seasons/8/matches/batch with 3 valid 1v1 entries
    Then the response status is 201 Created
    And entry 2 has convergence_quality "degraded" and status "success"
    And all 3 matches are recorded

  Scenario: Each match in the batch is atomically recorded with its rating update
    Given "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches/batch with 2 valid matches
    Then for each match, its match record and rating snapshots are either both present or both absent (no partial state)

  Scenario: Batch entry does not support N-player ranked events
    Given "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches/batch with an entry containing 3 participants and ranked placements
    Then the response status is 400 Bad Request
    And the response body indicates batch entry supports only 1v1 matches

  Scenario: Batch entry workflow requires confirmation before submission
    Given "alice" uses the UI to enter a batch of 5 matches
    When "alice" clicks the confirmation step and reviews the batch
    And "alice" confirms the submission
    Then the batch is submitted to the server and processed in order
    And the UI shows per-entry results

  Scenario: Viewer cannot submit a batch of matches
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends POST /api/seasons/7/matches/batch with valid matches
    Then the response status is 403 Forbidden

  Scenario: Batch entry is rejected for a closed season
    Given season 6 in league 42 has end_date set (season is closed)
    And "alice" is authenticated
    When "alice" sends POST /api/seasons/6/matches/batch with valid matches
    Then the response status is 409 Conflict
    And the response body contains error_code "SEASON_CLOSED"

  Scenario: Empty batch is accepted and returns an empty results list
    Given "alice" is authenticated
    When "alice" sends POST /api/seasons/7/matches/batch with an empty matches array
    Then the response status is 200 OK
    And the response body contains an empty results array
```
