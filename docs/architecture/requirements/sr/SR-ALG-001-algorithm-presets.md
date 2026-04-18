# SR-ALG-001: Algorithm Parameter Presets

**Status:** Draft
**Parent:** UR-LM-001
**Priority:** Must-have

## Description

Each rating algorithm (Elo, Glicko-2, TrueSkill) has a defined set of sensible default parameter values that are applied when an operator selects the algorithm during league creation. These presets reduce configuration burden for non-technical operators while still allowing customization.

## Rationale

League operators are non-technical users who may not understand the meaning of parameters like K-factor, tau, or draw probability. Sensible defaults let them create a functional league immediately. Presets are derived from widely accepted values in competitive gaming contexts.

## Acceptance Criteria

- [ ] Selecting Elo pre-fills default parameters (at minimum: K-factor, initial rating)
- [ ] Selecting Glicko-2 pre-fills default parameters (at minimum: initial rating, initial RD, initial volatility, tau, rating period)
- [ ] Selecting TrueSkill pre-fills default parameters (at minimum: initial mu, initial sigma, beta, tau, draw probability)
- [ ] Preset values are defined in the library crate and are not hardcoded in the frontend or backend
- [ ] The preset values result in a functional and reasonable rating system without any operator modification
- [ ] Preset values are documented in the codebase with rationale for each default

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Algorithm Parameter Presets

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And a user "alice_op" with role "operator" exists and is authenticated

  Scenario: Creating an Elo league pre-fills default Elo parameters
    Given "alice_op" is authenticated
    When "alice_op" sends POST /api/leagues with name "Elo League" and algorithm "elo" (no explicit params)
    Then the response status is 201 Created
    And the created season's algorithm_params contains k_factor (a positive number)
    And the created season's algorithm_params contains initial_rating (a positive number)
    And the values are the library-defined presets (not zeros or nulls)

  Scenario: Creating a Glicko-2 league pre-fills default Glicko-2 parameters
    Given "alice_op" is authenticated
    When "alice_op" sends POST /api/leagues with name "Glicko League" and algorithm "glicko2" (no explicit params)
    Then the response status is 201 Created
    And the created season's algorithm_params contains initial_mu
    And the created season's algorithm_params contains initial_rd
    And the created season's algorithm_params contains initial_volatility
    And the created season's algorithm_params contains tau
    And all values are the library-defined Glicko-2 presets

  Scenario: Creating a TrueSkill league pre-fills default TrueSkill parameters
    Given "alice_op" is authenticated
    When "alice_op" sends POST /api/leagues with name "TS League" and algorithm "trueskill" (no explicit params)
    Then the response status is 201 Created
    And the created season's algorithm_params contains initial_mu
    And the created season's algorithm_params contains initial_sigma
    And the created season's algorithm_params contains beta
    And the created season's algorithm_params contains tau
    And the created season's algorithm_params contains draw_probability
    And all values are the library-defined TrueSkill presets

  Scenario: Preset values produce a functional rating system without modification
    Given "alice_op" creates an Elo league with default parameters
    And two players are added to the league
    When a match is recorded between the two players
    Then the match is recorded successfully with non-zero rating change
    And both players' new ratings differ from their starting ratings

  Scenario: Preset values are sourced from the library crate, not hardcoded in frontend or backend
    Given the server API is queried for the Elo algorithm presets
    When "alice_op" sends GET /api/algorithms/elo/presets
    Then the response status is 200 OK
    And the returned preset values match the constants defined in the ladder-rs library crate
    And the frontend uses the API-returned presets to pre-fill the league creation form

  Scenario: Preset values are documented in the codebase
    Given the ladder-rs library source code is inspected
    Then each algorithm preset constant has a doc comment explaining the rationale for the default value

  Scenario: Elo preset results in initial_rating = 1000 and k_factor = 32
    When the server returns Elo algorithm presets
    Then the response contains initial_rating 1000 and k_factor 32

  Scenario: TrueSkill preset results in initial_mu = 25.0, initial_sigma = 25/3 (approximately 8.333), draw_probability = 0.1
    When the server returns TrueSkill algorithm presets
    Then the response contains initial_mu 25.0 and initial_sigma approximately 8.333 and draw_probability 0.1
```
