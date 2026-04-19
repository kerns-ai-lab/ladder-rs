# UR-SW-001: Swarm Dashboard

**Status:** Draft
**Parent:** Spec Section 4.7 (Swarm Dashboard)
**Priority:** Must-have

## Description

A Swarm Operator can view aggregate performance metrics for their AI agents through a read-only dashboard in the web UI. The dashboard surfaces raw data: rating distribution, rating velocity, match volume over time, top/bottom agents, agent lifecycle information, and win rate by rating bucket. Swarm operators write data via the library crate, not via this UI. "Active agent" status is derived from a configurable per-league match recency threshold set by the League Operator or Admin; an agent is active if it has at least one match within the threshold window. The threshold has a server-defined default and is not fixed. Anomaly detection is deferred to post-v1 (RQ-R2-8).

## Rationale

Swarm operators manage large populations of autonomous agents and need aggregate views to understand population-level dynamics. The dashboard provides operational visibility without requiring the swarm operator to interact with individual player records. Read-only access reflects the swarm operator's workflow: they write data programmatically and observe results through the dashboard.

## Acceptance Criteria

- [ ] Dashboard displays a rating distribution histogram across all agents in a league
- [ ] Dashboard displays rating velocity (rate of rating change over time) per agent
- [ ] Dashboard displays match volume over time with selectable time periods (`hour`, `day`, `week`, `monthly`)
- [ ] Dashboard displays top N and bottom N agents by current rating
- [ ] Dashboard displays agent lifecycle information: start date, total matches played, current status
- [ ] Dashboard displays win rate grouped by rating buckets
- [ ] "Active agent" is derived from a configurable per-league match recency threshold (set by the operator); agents with at least one match within the threshold window are classified as active, and this classification is surfaced as a UI filter on the dashboard
- [ ] The dashboard is entirely read-only; no write operations are available through this view
- [ ] Dashboard scopes data to a selected league

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Swarm Dashboard

  Background:
    Given the platform is running and the database is initialized
    And league 50 "Swarm League" is active with a TrueSkill season (id 10)
    And league 50 has active_agent_threshold_days set to 30
    And 20 non-human agent players are in league 50
    And agents with ids 1-15 have matches within the last 30 days (active)
    And agents with ids 16-20 have no matches within the last 30 days (inactive)
    And a user "swarm_op" with role "operator" is authenticated and assigned to league 50

  Scenario: Dashboard displays rating distribution histogram
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/rating-distribution
    Then the response status is 200 OK
    And the response body contains a histogram of current ratings across agents in league 50
    And each histogram bucket contains a rating range and an agent count

  Scenario: Dashboard displays rating velocity per agent
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/rating-velocity
    Then the response status is 200 OK
    And the response body contains rate-of-change values per agent in league 50

  Scenario: Dashboard displays match volume over time with selectable period
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/match-volume?period=day
    Then the response status is 200 OK
    And the response body contains match counts grouped by day
    When "swarm_op" sends GET /api/leagues/50/dashboard/match-volume?period=hour
    Then the response body contains match counts grouped by hour
    When "swarm_op" sends GET /api/leagues/50/dashboard/match-volume?period=week
    Then the response body contains match counts grouped by week
    When "swarm_op" sends GET /api/leagues/50/dashboard/match-volume?period=monthly
    Then the response body contains match counts grouped by calendar month

  Scenario: Invalid period parameter returns 400 with valid values listed
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/match-volume?period=quarterly
    Then the HTTP response status is 400
    And the response body contains error_code "VALIDATION_ERROR"
    And the details array contains an entry with field "period" and rejected_value "quarterly"
    And the constraint lists the valid values: hour, day, week, monthly

  Scenario: Dashboard returns structured empty response when league has zero agents
    Given league 50 has zero agents enrolled in any season
    And "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/rating-distribution
    Then the HTTP response status is 200 OK
    And the response body contains {"buckets": [], "total_agents": 0}

  Scenario: Dashboard displays top N and bottom N agents by current rating
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/top-bottom?n=5
    Then the response status is 200 OK
    And the response body contains 5 agents with the highest conservative_rating
    And the response body contains 5 agents with the lowest conservative_rating

  Scenario: Dashboard displays agent lifecycle information
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/agent-lifecycle
    Then the response status is 200 OK
    And each agent entry contains start_date (first match), total_matches, and current_status (active or inactive)

  Scenario: Dashboard displays win rate grouped by rating buckets
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/win-rate-by-bucket
    Then the response status is 200 OK
    And the response body contains win_percentage values grouped by rating ranges

  Scenario: Active agent filter returns only agents with match within threshold window
    Given league 50 has active_agent_threshold_days 30
    And agents 1-15 have matches within the last 30 days
    And agents 16-20 have no matches within the last 30 days
    And "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/agents?active_only=true
    Then the response body contains exactly agents 1-15
    And the response body does not contain agents 16-20

  Scenario: Dashboard displays current active_agent_threshold_days value
    Given "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard
    Then the response body contains active_agent_threshold_days 30

  Scenario: Dashboard is scoped to the selected league
    Given league 51 "Other League" exists with agents 100-110
    And "swarm_op" is also assigned to league 51
    And "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/rating-distribution
    Then the response body contains only agents from league 50
    And the response body does not contain agents from league 51

  Scenario: Dashboard is entirely read-only - write operations are rejected
    Given "swarm_op" is authenticated
    When "swarm_op" sends POST /api/leagues/50/dashboard/rating-distribution with any body
    Then the response status is 404 Not Found or 405 Method Not Allowed
    And no data is modified

  Scenario: Agent with zero matches is never classified as active
    Given agent "InactiveBot" (id 20) has 0 matches in league 50
    And "swarm_op" is authenticated
    When "swarm_op" sends GET /api/leagues/50/dashboard/agents?active_only=true
    Then the response body does not contain agent "InactiveBot"

  Scenario: Unauthenticated request to swarm dashboard is rejected
    When an unauthenticated client sends GET /api/leagues/50/dashboard
    Then the response status is 401 Unauthorized
```
