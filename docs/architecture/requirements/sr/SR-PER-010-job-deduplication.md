# SR-PER-010: Recalculation Job Deduplication

**Status:** Draft
**Parent:** UR-ADM-001, UR-PM-002
**Priority:** Should-have

## Description

When a recalculation job is requested for a season (triggered by a match correction or an alias change), the system checks whether a `queued` job already exists for that season. If one exists, the existing job ID is returned and no new job is inserted. If the existing job for that season is `in_progress` (actively running), a new job is inserted, because the in-progress job will not incorporate the triggering change.

## Rationale

Match corrections are admin-only, infrequent operations. However, an admin may make multiple rapid corrections to matches in the same season before the background poller picks up any of them. Without deduplication, each correction inserts a new job, resulting in multiple sequential full-season replays that each immediately overwrite the previous. Only the final replay is meaningful. Deduplication collapses these into a single replay at the cost of a single indexed lookup per job insertion. See ADR-0005.

## Acceptance Criteria

- [ ] When `Job Repository.insert_job(season_id, triggered_by)` is called and a job with `status = 'queued'` already exists for the same `season_id`, the function returns the existing job ID without inserting a new row
- [ ] When `insert_job` is called and the only existing job for the season has `status = 'in_progress'`, a new job is inserted (the in-progress job will not include the triggering change)
- [ ] When `insert_job` is called and no job exists for the season (or all existing jobs are `completed` or `failed`), a new job is inserted normally
- [ ] The API response for a match correction or alias change includes the job ID returned by `insert_job`; if deduplication fired, this is the ID of the pre-existing queued job
- [ ] The deduplication check is implemented as a single indexed SQL query on `(season_id, status)` with no race conditions under SQLite's serialized write model

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Recalculation Job Deduplication

  Background:
    Given the system is running
    And an open season "Season A" exists
    And no recalculation jobs exist for "Season A"

  Scenario: First correction inserts a new job
    When an Admin corrects a match in "Season A"
    Then a new recalculation job is inserted with status "queued"
    And the response includes the new job_id

  Scenario: Second correction before poller fires returns existing job ID
    Given a recalculation job J1 with status "queued" exists for "Season A"
    When an Admin corrects a second match in "Season A"
    Then no new job row is inserted for "Season A"
    And the response includes job_id J1 (the existing queued job)
    And only one queued job exists for "Season A"

  Scenario: Correction while job is in-progress inserts a new job
    Given a recalculation job J1 with status "in_progress" exists for "Season A"
    When an Admin corrects a match in "Season A"
    Then a new job J2 is inserted with status "queued" for "Season A"
    And both J1 (in_progress) and J2 (queued) exist for "Season A"

  Scenario: Correction after prior job completes inserts a new job
    Given a recalculation job J1 with status "completed" exists for "Season A"
    When an Admin corrects a match in "Season A"
    Then a new job J2 is inserted with status "queued"

  Scenario: Alias change deduplicates the same way as corrections
    Given a recalculation job J1 with status "queued" exists for "Season A"
    When an Admin creates an alias between two players who both played in "Season A"
    Then no new job is inserted for "Season A"
    And the response includes job_id J1
```
