# SR-ADM-001: Audit Log

**Status:** Draft
**Parent:** UR-ADM-001
**Priority:** Should-have

## Description

All admin-level match corrections are recorded in an append-only audit log. Each log entry captures: the identity of the administrator who made the change, the original match data, the corrected match data, and the timestamp of the correction. The audit log is queryable and cannot be modified or deleted.

## Rationale

Audit logging is essential for accountability in systems that allow data modification. Without it, there is no way to investigate disputed results or detect misuse of correction privileges. The append-only constraint ensures the log itself cannot be tampered with.

## Acceptance Criteria

- [ ] Every match correction creates an audit log entry before the correction is applied
- [ ] Each audit log entry contains: verified authenticated user identity (from the session, not self-reported), original match record (players, outcome, timestamp), corrected match record (players, outcome, timestamp), and correction timestamp
- [ ] The audit log is stored in the database as append-only records (no UPDATE or DELETE operations on audit rows)
- [ ] The audit log is queryable via the API (list corrections, filter by league/season/date range)
- [ ] Audit log entries reference the match ID they pertain to
- [ ] The audit log persists independently of the match record (deleting/archiving a league does not remove its audit entries)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Audit Log for Match Corrections

  Background:
    Given the system is running
    And an Admin "admin_user" is authenticated
    And league "Theta League" has a match "match-001" with outcome: Alice defeats Bob
    And the match was recorded at "2026-03-01T14:00:00Z"

  Scenario: Match correction creates an audit log entry before applying the change
    When admin_user corrects match-001 to change the outcome from "Alice defeats Bob" to "Bob defeats Alice"
    Then an audit log entry exists for match-001
    And the audit log entry was created before the corrected match record was written

  Scenario: Audit log entry contains the required fields
    When admin_user corrects match-001 to change the outcome from "Alice defeats Bob" to "Bob defeats Alice"
    Then the audit log entry for match-001 contains:
      | field                  | expected value                     |
      | actor_user_id          | (admin_user's verified session ID) |
      | original_players       | [Alice, Bob]                       |
      | original_outcome       | "Alice defeats Bob"                |
      | original_timestamp     | "2026-03-01T14:00:00Z"             |
      | corrected_players      | [Alice, Bob]                       |
      | corrected_outcome      | "Bob defeats Alice"                |
      | correction_timestamp   | (current server time)              |
      | match_id               | "match-001"                        |

  Scenario: Audit log actor identity comes from the session, not self-reported data
    When admin_user corrects a match while impersonating a different user ID in the request body
    Then the audit log entry records admin_user's verified session identity
    And the self-reported user ID in the request body is not used as the actor

  Scenario: Audit log entries cannot be modified after creation
    Given an audit log entry exists for match-001
    When a direct UPDATE is attempted on the audit log row for match-001
    Then the operation is rejected by the persistence layer
    And the audit log entry remains unchanged

  Scenario: Audit log entries cannot be deleted
    Given an audit log entry exists for match-001
    When a direct DELETE is attempted on the audit log row for match-001
    Then the operation is rejected by the persistence layer
    And the audit log entry remains in the database

  Scenario: Audit log is queryable via the API — list corrections for a league
    Given 3 match corrections have been made in "Theta League"
    When admin_user requests GET /leagues/theta-league/audit-log
    Then the response contains 3 audit log entries
    And each entry includes the match_id, actor identity, and timestamps

  Scenario: Audit log can be filtered by date range
    Given corrections were made on 2026-03-01, 2026-03-15, and 2026-04-01
    When admin_user requests GET /leagues/theta-league/audit-log?from=2026-03-10&to=2026-03-31
    Then the response contains exactly the correction from 2026-03-15
    And corrections from 2026-03-01 and 2026-04-01 are not included

  Scenario: Audit log persists after the league is archived
    Given audit log entries exist for "Theta League"
    When an Admin archives "Theta League"
    Then the audit log entries for "Theta League" still exist
    And they remain queryable via the API

  Scenario: Non-Admin user cannot access the audit log endpoint
    Given a League Operator is authenticated
    When the League Operator requests the audit log for a league they manage
    Then the HTTP response status is 403

  Scenario: Audit log entry references the match ID it pertains to
    When admin_user corrects match-001
    Then the resulting audit log entry contains "match_id" = "match-001"
    And a GET request for match-001 can be cross-referenced with the audit log
```
