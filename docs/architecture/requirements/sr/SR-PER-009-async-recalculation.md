# SR-PER-009: Asynchronous Recalculation

**Status:** Draft
**Parent:** UR-ADM-001, UR-PM-002
**Priority:** Must-have

## Description

Full-season rating recalculations triggered by admin match corrections or player alias operations execute asynchronously as background jobs. The system provides status tracking so the initiating user can monitor progress. During recalculation, ratings are eventually consistent -- the system serves stale ratings until the recalculation completes. There is no NFR on recalculation duration; it completes in whatever time is required.

## Rationale

Full-season recalculation can be computationally expensive for seasons with many matches. Synchronous recalculation would block the user and risk HTTP timeouts. Asynchronous execution with status tracking provides a responsive user experience while allowing the system to take the time needed for correctness.

## Acceptance Criteria

- [ ] Admin match corrections trigger a background recalculation job rather than blocking the HTTP response
- [ ] Player alias operations (link and unlink) trigger a background recalculation job rather than blocking the HTTP response
- [ ] The system returns an immediate response to the triggering request indicating that recalculation has been queued (e.g., 202 Accepted with a job ID)
- [ ] Each recalculation job has a trackable status (queued, in_progress, completed, failed)
- [ ] The API exposes an endpoint to query the status of a recalculation job by job ID
- [ ] The UI displays "recalculation in progress" when a pending recalculation job exists for the viewed season
- [ ] During recalculation, the system continues to serve the pre-recalculation ratings (stale but available)
- [ ] Upon successful completion, the recalculated ratings atomically replace the stale ratings
- [ ] If a recalculation job fails, it is retried automatically up to a configurable maximum retry count (default: 3); each retry transitions the job back to `in_progress`
- [ ] After all retries are exhausted, the job transitions to `permanently_failed`; stale pre-recalculation ratings are retained and served
- [ ] The leaderboard response includes a `ratings_stale: true` field when a `permanently_failed` job exists for the season, allowing the UI to surface a staleness warning
- [ ] Each job record exposes `retry_count` and `max_retries` fields queryable via the job status endpoint
- [ ] An admin can re-trigger recalculation after permanent failure by re-submitting the triggering action (e.g., re-correcting the match); SR-PER-010 allows a new job since the prior job is `permanently_failed`
- [ ] Multiple recalculation requests for the same season are serialized (not run concurrently)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Asynchronous Recalculation

  Background:
    Given the platform is running with the background job poller active
    And a user "admin" with role "admin" exists and is authenticated
    And league 1 has an open Elo season (id 7) with 50 matches
    And match 100 exists in season 7

  Scenario: Admin match correction triggers a background recalculation job returning 202
    Given "admin" is authenticated
    When "admin" sends PATCH /api/matches/100 with corrected participants
    Then the response status is 202 Accepted
    And the response body contains job_id (a positive integer)
    And the response body contains status "queued"
    And the response body contains a human-readable message about recalculation being queued
    And the HTTP response is returned immediately (not after recalculation finishes)

  Scenario: Player alias link operation returns 202 with job_id
    Given "admin" is authenticated
    When "admin" sends POST /api/leagues/1/players/10/aliases with alias_player_id 11
    Then the response status is 202 Accepted
    And the response body contains job_id

  Scenario: Each recalculation job has a trackable status with four states
    Given a recalculation job with job_id 42 was created
    And "admin" is authenticated
    When "admin" sends GET /api/jobs/42 immediately after job creation
    Then the response body contains status "queued"
    When the job poller claims the job
    And "admin" sends GET /api/jobs/42
    Then the response body contains status "in_progress"
    When the job completes successfully
    And "admin" sends GET /api/jobs/42
    Then the response body contains status "completed"

  Scenario: API exposes job status endpoint queryable by job_id
    Given recalculation job id 42 exists
    And "admin" is authenticated
    When "admin" sends GET /api/jobs/42
    Then the response status is 200 OK
    And the response body contains job_id 42, status, triggered_at, and season_id

  Scenario: UI displays "recalculation in progress" when a pending job exists for the season
    Given a recalculation job for season 7 has status "in_progress"
    And "admin" is authenticated
    When "admin" sends GET /api/seasons/7/leaderboard
    Then the response body contains a field indicating recalculation_pending: true (or equivalent)

  Scenario: During recalculation, system continues serving pre-recalculation ratings
    Given a recalculation job for season 7 has status "queued"
    And player "Alice" has pre-correction rating 1200 in season 7
    And "admin" is authenticated
    When "admin" sends GET /api/seasons/7/leaderboard while the job is in status "queued"
    Then the response status is 200 OK
    And player "Alice"'s rating in the response is 1200 (the pre-recalculation value)

  Scenario: Upon successful completion, recalculated ratings atomically replace stale ratings
    Given a recalculation job for season 7 transitions from "in_progress" to "completed"
    And the correction reversed a match result changing player "Alice"'s expected final rating to 1050
    And "admin" is authenticated
    When "admin" sends GET /api/seasons/7/leaderboard immediately after the job status is "completed"
    Then player "Alice"'s rating in the response is 1050 (the recalculated value)
    And there is no intermediate state where some players have old ratings and others have new ratings

  Scenario: Failed recalculation job is retried automatically
    Given a recalculation job J1 for season 7 encounters a failure on first attempt
    When the background poller picks up J1 for a retry (retry_count < max_retries)
    Then J1 status transitions back to "in_progress"
    And stale pre-recalculation ratings continue to be served during the retry

  Scenario: Job exhausts retries and transitions to permanently_failed
    Given a recalculation job J1 for season 7 has failed max_retries times
    When the background poller evaluates J1 after the final failure
    Then J1 status transitions to "permanently_failed"
    And GET /api/jobs/J1 returns status "permanently_failed" with a non-null error_message
    And GET /api/jobs/J1 returns retry_count equal to max_retries

  Scenario: Leaderboard indicates stale ratings when a permanently_failed job exists for the season
    Given recalculation job J1 for season 7 has status "permanently_failed"
    And player "Alice" had pre-correction rating 1200
    And "admin" is authenticated
    When "admin" sends GET /api/seasons/7/leaderboard
    Then the response status is 200 OK
    And the response body contains "ratings_stale": true
    And player "Alice"'s rating in the response is 1200 (stale, pre-correction)

  Scenario: Admin re-triggers recalculation after permanent failure by re-correcting the match
    Given recalculation job J1 for season 7 has status "permanently_failed"
    And "admin" is authenticated
    When "admin" sends PATCH /api/matches/100 with the same corrected participants
    Then the HTTP response status is 202
    And a new recalculation job J2 is inserted with status "queued" (J1 is permanently_failed, so SR-PER-010 allows new insertion)
    And the response body contains job_id J2

  Scenario: If a recalculation job fails all retries, pre-recalculation ratings are retained
    Given a recalculation job for season 7 has status "permanently_failed"
    And player "Alice" had pre-recalculation rating 1200
    And "admin" is authenticated
    When "admin" sends GET /api/seasons/7/leaderboard
    Then player "Alice"'s rating is 1200 (unchanged from pre-recalculation)
    When "admin" sends GET /api/jobs/{failed_job_id}
    Then the response body contains status "permanently_failed" and a non-null error_message

  Scenario: Multiple recalculation requests for the same season are serialized
    Given "admin" triggers correction on match 100 (job_id 42 created, status "in_progress")
    And "admin" is authenticated
    When "admin" triggers another correction on match 101 while job 42 is still "in_progress"
    Then job_id 43 is created with status "queued"
    And job 43 does not start processing until job 42 status is "completed" or "failed"
    And both corrections are ultimately applied in the correct order

  Scenario: Job query for a non-existent job_id returns 404
    Given "admin" is authenticated
    When "admin" sends GET /api/jobs/9999
    Then the response status is 404 Not Found
```
