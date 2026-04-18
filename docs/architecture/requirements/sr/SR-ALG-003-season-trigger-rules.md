# SR-ALG-003: Season Trigger Rules

**Status:** Draft
**Parent:** UR-LM-002
**Priority:** Must-have

## Description

The system creates a new season ONLY when the algorithm TYPE changes on a league (e.g., Elo to Glicko-2). Changing algorithm parameters within the same type (e.g., adjusting Elo K-factor from 32 to 24) applies the new parameters to the current season without creating a new one. This revises the original spec which stated that algorithm OR parameter changes trigger a new season.

## Rationale

Parameter adjustments within the same algorithm keep the rating scale comparable, so ratings remain meaningful across the change. Algorithm type changes produce fundamentally different rating scales, requiring a clean break. Avoiding unnecessary season breaks reduces fragmentation and preserves rating continuity.

## Acceptance Criteria

- [ ] Changing the algorithm type on a league (e.g., Elo to Glicko-2) closes the current season and opens a new one
- [ ] Changing parameters within the same algorithm type (e.g., Elo K-factor 32 to 24) updates the current season's parameters in place
- [ ] The parameter update is persisted to the season record without changing the season's start date or creating a new season
- [ ] After a parameter-only change, subsequent matches use the new parameters while prior match ratings remain unchanged
- [ ] The system correctly distinguishes between algorithm type changes and parameter-only changes in all code paths (API, UI, library)
- [ ] The season transition logic is implemented in the library crate, not in the server or frontend

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Season Trigger Rules

  Background:
    Given the system is running
    And an Admin is authenticated
    And league "Delta League" exists using the Elo algorithm with K-factor = 32
    And "Delta League" has an open season "Season 1" with 3 recorded matches

  Scenario: Changing algorithm TYPE closes current season and opens a new one
    When the Admin changes "Delta League" algorithm from Elo to Glicko-2
    Then "Season 1" is closed with an end timestamp
    And a new season "Season 2" is created with status "open"
    And "Season 2" uses the Glicko-2 algorithm
    And the total number of seasons for "Delta League" is 2

  Scenario: Changing a parameter within the same algorithm type does NOT create a new season
    When the Admin changes "Delta League" Elo K-factor from 32 to 24
    Then the number of seasons for "Delta League" remains 1
    And "Season 1" retains its original start date
    And "Season 1" status is still "open"
    And "Season 1" K-factor is now 24

  Scenario: Parameter-only change persists to the season record
    When the Admin changes "Delta League" Elo K-factor from 32 to 48
    Then "Season 1" record's parameter field "k_factor" equals 48
    And the season start date has not changed

  Scenario: Matches recorded after a parameter-only change use the new parameter value
    Given "Delta League" Elo K-factor is 32
    And match 4 has not yet been recorded
    When the Admin changes "Delta League" Elo K-factor from 32 to 48
    And the League Operator records match 4 in "Delta League"
    Then match 4's rating delta is calculated using K-factor = 48
    And matches 1 through 3's stored rating deltas are unchanged

  Scenario: Prior match ratings are unaffected by a parameter-only change
    Given player "Alice" has a stored Elo rating snapshot of 1050 after match 3
    When the Admin changes "Delta League" Elo K-factor from 32 to 64
    Then player "Alice"'s rating snapshot for match 3 remains 1050

  Scenario: Elo to TrueSkill change triggers a new season
    When the Admin changes "Delta League" algorithm from Elo to TrueSkill
    Then "Season 1" is closed
    And a new season "Season 2" is opened using TrueSkill

  Scenario: Glicko-2 to TrueSkill change (type change) triggers a new season
    Given league "Epsilon League" uses Glicko-2 with an open season
    When the Admin changes "Epsilon League" algorithm from Glicko-2 to TrueSkill
    Then the Glicko-2 season is closed
    And a new TrueSkill season is opened

  Scenario: TrueSkill sigma parameter change within same algorithm type does not create a season
    Given league "Zeta League" uses TrueSkill with initial_sigma = 8.333
    When the Admin changes "Zeta League" initial_sigma to 6.0
    Then the number of seasons for "Zeta League" remains 1
    And the season record's "initial_sigma" field equals 6.0

  Scenario: Algorithm type change is distinguished from parameter change via API
    When the Admin submits a PATCH request to "Delta League" with only the "k_factor" field changed
    Then the system treats the request as a parameter-only change
    And no new season is created

  Scenario: Season transition logic is enforced even when called from the library crate directly
    Given a swarm operator invokes the library crate's league update function with algorithm type changed from Elo to Glicko-2
    Then the library crate closes the current season and opens a new one
    And the new season is persisted to the database
```
