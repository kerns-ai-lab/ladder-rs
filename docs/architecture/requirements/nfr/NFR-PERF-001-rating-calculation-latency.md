# NFR-PERF-001: Rating Calculation Latency

**Status:** Draft
**Parent:** Spec Section 6 (Performance)
**Priority:** Must-have

## Description

Single match rating updates must complete within strict latency bounds at the library crate level: less than 1ms for Elo, less than 5ms for Glicko-2, and less than 10ms for TrueSkill. These targets apply to the rating calculation itself, measured on commodity hardware, and do not include database I/O.

## Rationale

Rating calculation is on the critical path for every match recording operation. Low latency ensures that match entry feels instantaneous to league operators and that swarm operators can achieve high-throughput programmatic match recording. The per-algorithm targets reflect the inherent computational complexity differences (Elo is arithmetic, TrueSkill involves iterative approximation).

## Acceptance Criteria

- [ ] Elo single-match rating update completes in less than 1ms (p99) on commodity hardware
- [ ] Glicko-2 single-match rating update completes in less than 5ms (p99) on commodity hardware
- [ ] TrueSkill single-match rating update completes in less than 10ms (p99) on commodity hardware
- [ ] Latency is measured at the library crate boundary, excluding database I/O
- [ ] Benchmark tests exist for each algorithm that validate these latency bounds
- [ ] "Commodity hardware" is defined as: x86_64 processor, 2+ GHz, released within the last 5 years

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Rating Calculation Latency

  Background:
    Given the rating calculation benchmark is measured at the library crate boundary
    And database I/O is excluded from the measurement
    And the benchmark runs on commodity hardware (x86_64, 2+ GHz, released within 5 years)

  Scenario: Elo single-match rating update completes within 1ms at p99
    Given two players with Elo ratings 1200 and 1000 and K-factor = 32
    When the Elo rating update function is called for a single match outcome
    Then the 99th-percentile execution time across 10,000 invocations is less than 1 millisecond
    And the updated ratings for both players are returned

  Scenario: Glicko-2 single-match rating update completes within 5ms at p99
    Given two players with Glicko-2 ratings mu = 1500, RD = 350 each
    When the Glicko-2 rating update function is called for a single match outcome
    Then the 99th-percentile execution time across 10,000 invocations is less than 5 milliseconds
    And the updated mu and RD for both players are returned

  Scenario: TrueSkill single-match rating update completes within 10ms at p99
    Given two players with TrueSkill ratings mu = 25.0, sigma = 8.333 each
    When the TrueSkill rating update function is called for a single match outcome
    Then the 99th-percentile execution time across 10,000 invocations is less than 10 milliseconds
    And the updated mu and sigma for both players are returned

  Scenario: Elo calculation does not regress to exceed 1ms p99 after code changes
    Given a benchmark test exists in the library crate for Elo rating updates
    When the benchmark test is executed
    Then the result is below the 1ms p99 threshold
    And the benchmark result is recorded to detect future regressions

  Scenario: Glicko-2 calculation does not regress to exceed 5ms p99 after code changes
    Given a benchmark test exists in the library crate for Glicko-2 rating updates
    When the benchmark test is executed
    Then the result is below the 5ms p99 threshold

  Scenario: TrueSkill calculation does not regress to exceed 10ms p99 after code changes
    Given a benchmark test exists in the library crate for TrueSkill rating updates
    When the benchmark test is executed
    Then the result is below the 10ms p99 threshold

  Scenario: Rating calculation latency is measured at the library crate boundary, not end-to-end
    Given a single match recording request is made via the HTTP API
    When the server processes the request
    Then only the library-level calculation time is attributed to the NFR-PERF-001 budget
    And database write time and HTTP handling time are not included in the latency measurement
```
