# SR-SW-001: Configurable Active Agent Threshold

**Status:** Draft
**Parent:** UR-SW-001, RQ-R3-8
**Priority:** Must-have

## Description

Each league has a configurable recency threshold (a duration in days) that defines what it means for an agent to be "active." An agent is active if it has at least one recorded match within the threshold window relative to the current time. The threshold is set and updated by an Admin or assigned League Operator. The swarm dashboard "active agent" filter applies this threshold when filtering the agent list.

## Rationale

Different swarm leagues have different match frequencies. A threshold that is appropriate for a high-frequency swarm (where 24 hours without a match is unusual) would misclassify all agents as inactive in a slower swarm. Making the threshold operator-configurable per league ensures the "active agent" concept reflects each league's operational reality. Heartbeat-based connectivity tracking is deferred to post-v1.

## Acceptance Criteria

- [ ] The `leagues` table (or an associated configuration table) stores an `active_agent_threshold_days` integer field; the field has a server-defined default value (architecture to determine the exact default, within the range 1–365)
- [ ] The league creation endpoint accepts an optional `active_agent_threshold_days` parameter; when omitted, the server default is applied
- [ ] The league update endpoint allows an Admin or assigned League Operator to update `active_agent_threshold_days` for their league
- [ ] The swarm dashboard "active agent" filter, when applied, returns only agents whose most recent match timestamp is within `active_agent_threshold_days` days of the current server time
- [ ] The dashboard displays the current `active_agent_threshold_days` value so operators can see which threshold is in effect
- [ ] Setting `active_agent_threshold_days` to a value less than 1 or greater than 365 is rejected with a 400 validation error
- [ ] Changing the threshold takes effect immediately for all subsequent dashboard queries; no recalculation of historical data is required
- [ ] An agent with zero recorded matches in the league is never classified as active, regardless of the threshold

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Configurable Active Agent Threshold

  Background:
    Given the system is running
    And an Admin "admin1" is authenticated
    And league "Swarm League" exists with active_agent_threshold_days = 7
    And the current server time is 2026-04-15T12:00:00Z
    And the following agents exist in "Swarm League":
      | Agent     | Last match timestamp       |
      | AgentA    | 2026-04-14T12:00:00Z (1 day ago)  |
      | AgentB    | 2026-04-09T12:00:00Z (6 days ago) |
      | AgentC    | 2026-04-07T12:00:00Z (8 days ago) |
      | AgentD    | (no matches recorded)              |

  Scenario: Active agent filter returns only agents within the threshold window
    Given a League Operator is authenticated and assigned to "Swarm League"
    When the operator requests the swarm dashboard with the active agent filter enabled
    Then the response includes "AgentA" (1 day ago, within 7-day threshold)
    And the response includes "AgentB" (6 days ago, within 7-day threshold)
    And the response does not include "AgentC" (8 days ago, outside 7-day threshold)
    And the response does not include "AgentD" (no matches, never active)

  Scenario: Agent with zero matches is never classified as active
    Given the threshold is set to 365 days
    When the operator requests the active agent filter
    Then "AgentD" does not appear in the active agent results
    And "AgentD" appears in the inactive agent results

  Scenario: League creation without specifying threshold uses server default
    When admin1 creates a new league without specifying active_agent_threshold_days
    Then the new league's active_agent_threshold_days equals the server-defined default
    And the server default is between 1 and 365 inclusive

  Scenario: League creation with explicit threshold stores the specified value
    When admin1 creates a new league with active_agent_threshold_days = 30
    Then the league record stores active_agent_threshold_days = 30

  Scenario: Admin can update the active agent threshold for a league
    Given admin1 is authenticated
    When admin1 PATCHes /leagues/swarm-league with active_agent_threshold_days = 14
    Then the HTTP response status is 200
    And the league record reflects active_agent_threshold_days = 14

  Scenario: League Operator can update the threshold for their assigned league
    Given a League Operator "op1" is authenticated and assigned to "Swarm League"
    When op1 PATCHes /leagues/swarm-league with active_agent_threshold_days = 3
    Then the HTTP response status is 200
    And the league's threshold is now 3 days

  Scenario: Changing threshold takes immediate effect on next dashboard query
    Given "Swarm League" has active_agent_threshold_days = 7
    And "AgentC" last played 8 days ago (outside the 7-day window)
    When admin1 updates the threshold to 14 days
    And the operator queries the active agent filter
    Then "AgentC" now appears in the active agent results

  Scenario: Dashboard displays the current threshold value
    When the operator views the swarm dashboard for "Swarm League"
    Then the dashboard response includes "active_agent_threshold_days": 7

  Scenario: Setting threshold below 1 is rejected with 400
    When admin1 PATCHes /leagues/swarm-league with active_agent_threshold_days = 0
    Then the HTTP response status is 400
    And the error message identifies "active_agent_threshold_days" with the valid range 1–365

  Scenario: Setting threshold above 365 is rejected with 400
    When admin1 PATCHes /leagues/swarm-league with active_agent_threshold_days = 366
    Then the HTTP response status is 400
    And the error message identifies "active_agent_threshold_days" with the valid range 1–365

  Scenario: Threshold boundary value 1 is accepted
    When admin1 PATCHes /leagues/swarm-league with active_agent_threshold_days = 1
    Then the HTTP response status is 200
    And the league's threshold is 1 day

  Scenario: Threshold boundary value 365 is accepted
    When admin1 PATCHes /leagues/swarm-league with active_agent_threshold_days = 365
    Then the HTTP response status is 200
    And the league's threshold is 365 days

  Scenario: Threshold change does not require recalculation of historical match data
    Given "AgentA" has 1000 historical match records
    When admin1 updates the threshold from 7 to 3 days
    Then the historical match records for "AgentA" are unchanged
    And the updated threshold is reflected immediately without a background job

  Scenario: Match volume endpoint accepts all valid period values including monthly
    Given a League Operator is authenticated and assigned to "Swarm League"
    When the operator requests GET /api/leagues/swarm-league/dashboard/match-volume?period=monthly
    Then the HTTP response status is 200 OK
    And the response body contains match counts grouped by calendar month

  Scenario: Invalid period parameter returns 400 with field-level error
    Given a League Operator is authenticated
    When the operator requests GET /api/leagues/swarm-league/dashboard/match-volume?period=quarterly
    Then the HTTP response status is 400
    And the response body contains error_code "VALIDATION_ERROR"
    And the details array entry for field "period" lists the valid values: hour, day, week, monthly

  Scenario: Rating distribution returns structured empty response for a league with zero agents
    Given "Swarm League" has zero agents enrolled
    And a League Operator is authenticated
    When the operator requests GET /api/leagues/swarm-league/dashboard/rating-distribution
    Then the HTTP response status is 200 OK
    And the response body contains {"buckets": [], "total_agents": 0}
```
