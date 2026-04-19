# UR-ADM-001: Admin Match Correction

**Status:** Draft
**Parent:** RQ7 decision (Data Integrity - match corrections)
**Priority:** Should-have

## Description

An administrator can correct previously recorded match results. Matches are not strictly immutable; an admin-level audited override exists for corrections. All corrections are logged with who made the change, what was changed, and when. This provides a safety valve for data entry errors while maintaining an audit trail.

## Rationale

Despite best efforts at validation, data entry errors will occur. Strict immutability would force operators to live with incorrect data permanently. An audited correction mechanism balances data integrity with operational reality. The audit log ensures accountability and traceability.

## Acceptance Criteria

- [ ] An authenticated user with the Admin role can modify the outcome of a previously recorded match
- [ ] Modifying a match triggers an asynchronous full rating recalculation for all subsequent matches in the affected season (see SR-PER-009)
- [ ] The admin sees a "recalculation in progress" indicator after submitting a correction; ratings become eventually consistent
- [ ] Every match correction is logged with: the authenticated identity of the admin who made the change, what was changed (before and after), and when the correction was made
- [ ] The audit log is append-only and cannot be modified or deleted
- [ ] Match corrections are not available through the standard match entry UI; they require an authenticated Admin-role action (see UR-AUTH-002)
- [ ] Correcting a match in a closed season is not permitted

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Admin Match Correction

  Background:
    Given the platform is running and the database is initialized
    And a user "admin" with role "admin" exists and is authenticated
    And league 42 "Alpha League" is active with an open Elo season (id 7)
    And match 100 exists in season 7 with player 1 (winner, placement 1) and player 2 (loser, placement 2)
    And match 100 has convergence_quality "converged" and is_corrected false

  Scenario: Admin corrects the outcome of an existing match
    Given "admin" is authenticated
    When "admin" sends PATCH /api/matches/100 with participants: player 2 placement 1, player 1 placement 2, and reason "entered wrong winner"
    Then the response status is 202 Accepted
    And the response body contains a job_id
    And the response body contains status "queued"
    And the response body contains a message indicating recalculation has been queued
    And match 100 is_corrected flag is set to 1 in the database

  Scenario: Match correction triggers asynchronous recalculation for the affected season
    Given "admin" is authenticated
    When "admin" sends PATCH /api/matches/100 with corrected participants
    Then the response status is 202 Accepted
    And a recalculation job is created with status "queued" for season 7
    And the HTTP response returns immediately without waiting for recalculation

  Scenario: Pre-correction ratings remain available during recalculation
    Given "admin" has submitted a correction for match 100
    And the recalculation job for season 7 is still in status "queued"
    And a user "alice" with role "operator" is authenticated
    When "alice" sends GET /api/seasons/7/leaderboard
    Then the response status is 200 OK
    And the leaderboard returns the pre-correction ratings (not yet updated)

  Scenario: Ratings atomically reflect correction after recalculation completes
    Given "admin" submitted a correction for match 100 that reversed the outcome
    And the recalculation job for season 7 has status "completed"
    And "admin" is authenticated
    When "admin" sends GET /api/seasons/7/leaderboard
    Then the leaderboard reflects ratings recomputed from the corrected match data

  Scenario: Every match correction is logged in the audit log
    Given "admin" is authenticated
    When "admin" sends PATCH /api/matches/100 with corrected participants and reason "typo in result"
    Then the response status is 202 Accepted
    And an audit log entry exists for match 100 containing:
      | field        | value                             |
      | changed_by   | admin user id                     |
      | before_state | JSON snapshot of original match   |
      | after_state  | JSON snapshot of corrected match  |
      | changed_at   | current timestamp                 |

  Scenario: Audit log is append-only and cannot be deleted
    Given an audit log entry exists for match 100
    When "admin" sends DELETE /api/matches/100/audit
    Then the response status is 404 Not Found or 405 Method Not Allowed
    And the audit log entry for match 100 still exists

  Scenario: Admin sees "recalculation in progress" indicator while job is running
    Given "admin" submitted a correction and a recalculation job id 17 is in status "in_progress"
    And "admin" is authenticated
    When "admin" sends GET /api/jobs/17
    Then the response status is 200 OK
    And the response body contains status "in_progress"

  Scenario: Match correction in a closed season is rejected
    Given season 6 in league 42 has end_date set (closed)
    And match 200 exists in season 6
    And "admin" is authenticated
    When "admin" sends PATCH /api/matches/200 with corrected participants
    Then the response status is 409 Conflict
    And the response body indicates correction is not permitted on a closed season

  Scenario: League Operator cannot correct a match (Admin only)
    Given a user "alice" with role "operator" is authenticated
    When "alice" sends PATCH /api/matches/100 with corrected participants
    Then the response status is 403 Forbidden

  Scenario: Player/Viewer cannot correct a match
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends PATCH /api/matches/100 with corrected participants
    Then the response status is 403 Forbidden

  Scenario: Unauthenticated correction attempt is rejected
    When an unauthenticated client sends PATCH /api/matches/100 with corrected participants
    Then the response status is 401 Unauthorized

  Scenario: Correction of a non-existent match returns 404
    Given "admin" is authenticated
    When "admin" sends PATCH /api/matches/9999 with corrected participants
    Then the response status is 404 Not Found

  Scenario: Second correction before first job fires returns the existing queued job_id
    Given "admin" corrected match-001 and recalculation job J1 is status "queued" for season 7
    When "admin" sends PATCH /api/matches/100 with a second correction before J1 runs
    Then the HTTP response status is 202
    And the response body contains job_id J1 (the pre-existing queued job)
    And no new recalculation job row is inserted for season 7
    And a second audit log entry exists for match-001 recording the second correction
```
