# SR-ALG-002: Parameter Guardrails

**Status:** Draft
**Parent:** UR-LM-001
**Priority:** Must-have

## Description

Each algorithm parameter has defined minimum and maximum allowed values (guardrails). The system rejects parameter values outside these ranges at both the API and UI levels. Guardrails prevent operators from configuring nonsensical or mathematically unstable parameter combinations.

## Rationale

Non-technical operators may not understand the mathematical implications of extreme parameter values (e.g., a negative K-factor, or a sigma of zero). Guardrails protect data integrity and system stability by constraining parameters to valid ranges.

## Acceptance Criteria

- [ ] Every configurable algorithm parameter has a defined minimum and maximum value
- [ ] Parameter values below the minimum or above the maximum are rejected with a clear error message identifying the parameter and its valid range
- [ ] Guardrail definitions are maintained in the library crate alongside the algorithm implementations
- [ ] The API returns structured validation errors with field-level detail when guardrails are violated
- [ ] The UI prevents submission of out-of-range values (client-side validation using the same guardrail definitions)
- [ ] Guardrail ranges are wide enough to accommodate legitimate use cases while preventing mathematically degenerate configurations

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Algorithm Parameter Guardrails

  Background:
    Given the system is running
    And a League Operator is authenticated and assigned to league "Alpha League"
    And "Alpha League" uses the Elo algorithm

  Scenario: Elo K-factor within valid range is accepted
    Given the Elo K-factor valid range is 1 to 100
    When the League Operator submits a configuration update with K-factor = 32
    Then the request succeeds with HTTP 200
    And the league configuration reflects K-factor = 32

  Scenario: Elo K-factor below minimum is rejected with field-level error
    Given the Elo K-factor valid range is 1 to 100
    When the League Operator submits a configuration update with K-factor = 0
    Then the request fails with HTTP 400
    And the response body contains an "error_code" field
    And the response body contains a "details" array
    And the details array includes an entry with field "k_factor", rejected value "0", and the valid range

  Scenario: Elo K-factor above maximum is rejected with field-level error
    Given the Elo K-factor valid range is 1 to 100
    When the League Operator submits a configuration update with K-factor = 101
    Then the request fails with HTTP 400
    And the response body contains a "details" array
    And the details array includes an entry with field "k_factor", rejected value "101", and the valid range

  Scenario: Negative Elo K-factor is rejected
    When the League Operator submits a configuration update with K-factor = -5
    Then the request fails with HTTP 400
    And the error message identifies "k_factor" as the invalid parameter
    And the error message states the valid range

  Scenario: Glicko-2 RD below minimum is rejected
    Given "Beta League" uses the Glicko-2 algorithm
    And the Glicko-2 RD valid range is 1 to 500
    When the League Operator submits a configuration update for "Beta League" with initial_rd = 0
    Then the request fails with HTTP 400
    And the details array identifies "initial_rd" with the valid range

  Scenario: Glicko-2 RD at maximum boundary is accepted
    Given "Beta League" uses the Glicko-2 algorithm
    And the Glicko-2 RD valid range is 1 to 500
    When the League Operator submits a configuration update for "Beta League" with initial_rd = 500
    Then the request succeeds with HTTP 200

  Scenario: TrueSkill sigma of zero is rejected
    Given "Gamma League" uses the TrueSkill algorithm
    When the League Operator submits a configuration update for "Gamma League" with initial_sigma = 0.0
    Then the request fails with HTTP 400
    And the error message identifies "initial_sigma" as the invalid parameter
    And the error message indicates the parameter must be greater than zero

  Scenario: TrueSkill draw_probability outside 0.0–1.0 range is rejected
    Given "Gamma League" uses the TrueSkill algorithm
    When the League Operator submits a configuration update for "Gamma League" with draw_probability = 1.5
    Then the request fails with HTTP 400
    And the details array identifies "draw_probability" with the valid range 0.0 to 1.0

  Scenario: Multiple invalid parameters produce multiple field-level errors in one response
    When the League Operator submits a configuration update with K-factor = -5 and an unrecognized field
    Then the request fails with HTTP 400
    And the "details" array contains at least one entry per violated constraint

  Scenario: Boundary value at minimum is accepted
    Given the Elo K-factor valid range is 1 to 100
    When the League Operator submits a configuration update with K-factor = 1
    Then the request succeeds with HTTP 200

  Scenario: Guardrail error response never exposes internal implementation details
    When the League Operator submits a configuration update with K-factor = -999
    Then the request fails with HTTP 400
    And the response body does not contain stack traces
    And the response body does not contain SQL query text
    And the response body does not contain file system paths
```
