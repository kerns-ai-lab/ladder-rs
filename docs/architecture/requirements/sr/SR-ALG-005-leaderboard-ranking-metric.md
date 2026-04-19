# SR-ALG-005: Leaderboard Ranking Metric

**Status:** Draft
**Parent:** UR-LB-001
**Priority:** Must-have

## Description

The leaderboard ranking is determined by a conservative estimate that varies per algorithm. This rewards certainty: new players with high uncertainty start low on the leaderboard until their ratings stabilize.

- **Elo:** Raw rating value (Elo has no uncertainty component)
- **Glicko-2:** mu - 2 * RD (conservative estimate using rating deviation)
- **TrueSkill:** mu - 3 * sigma (conservative estimate using standard deviation)

## Rationale

Using conservative estimates for ranking prevents new or infrequently active players from dominating leaderboards due to high uncertainty. Players must demonstrate consistent performance to climb the rankings. The specific multipliers (2 for Glicko-2, 3 for TrueSkill) align with standard practice for each algorithm.

## Acceptance Criteria

- [ ] For Elo leagues, leaderboard ranking uses the raw rating value as the sort key
- [ ] For Glicko-2 leagues, leaderboard ranking uses (mu - 2 * RD) as the sort key
- [ ] For TrueSkill leagues, leaderboard ranking uses (mu - 3 * sigma) as the sort key
- [ ] The leaderboard displays the conservative estimate as the primary ranking value alongside the raw rating components
- [ ] New players with default (high) uncertainty values are ranked lower than established players with similar mean ratings
- [ ] The ranking metric is recalculated whenever a player's rating is updated
- [ ] Sorting by "rank" on the leaderboard uses the conservative estimate, not the raw mean rating

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Leaderboard Ranking Metric

  Background:
    Given the system is running
    And a Player/Viewer is authenticated

  Scenario: Elo leaderboard ranks players by raw rating value
    Given league "Elo League" uses the Elo algorithm
    And player "Alice" has Elo rating 1200
    And player "Bob" has Elo rating 1150
    And player "Carol" has Elo rating 1100
    When a client requests GET /leagues/elo-league/leaderboard
    Then the response lists players in order: Alice (1), Bob (2), Carol (3)
    And the sort key for Alice is 1200
    And the sort key for Bob is 1150

  Scenario: Glicko-2 leaderboard ranks players by mu minus 2 times RD
    Given league "Glicko League" uses the Glicko-2 algorithm
    And player "Alice" has mu = 1600 and RD = 50 (conservative estimate = 1500.0)
    And player "Bob" has mu = 1550 and RD = 20 (conservative estimate = 1510.0)
    When a client requests GET /leagues/glicko-league/leaderboard
    Then Bob (conservative estimate 1510.0) is ranked above Alice (conservative estimate 1500.0)
    And the sort key displayed for Alice is 1500.0
    And the sort key displayed for Bob is 1510.0

  Scenario: Glicko-2 conservative estimate calculation uses formula mu - 2 * RD
    Given league "Glicko League" uses the Glicko-2 algorithm
    And player "Dave" has mu = 1800 and RD = 150
    When a client requests the leaderboard for "Glicko League"
    Then Dave's displayed conservative estimate is 1500.0 (1800 - 2 * 150)

  Scenario: TrueSkill leaderboard ranks players by mu minus 3 times sigma
    Given league "TrueSkill League" uses the TrueSkill algorithm
    And player "Alice" has mu = 30.0 and sigma = 2.0 (conservative estimate = 24.0)
    And player "Bob" has mu = 28.0 and sigma = 1.0 (conservative estimate = 25.0)
    When a client requests GET /leagues/trueskill-league/leaderboard
    Then Bob (conservative estimate 25.0) is ranked above Alice (conservative estimate 24.0)
    And the sort key displayed for Alice is 24.0
    And the sort key displayed for Bob is 25.0

  Scenario: TrueSkill conservative estimate calculation uses formula mu - 3 * sigma
    Given league "TrueSkill League" uses the TrueSkill algorithm
    And player "Eve" has mu = 35.0 and sigma = 3.0
    When a client requests the leaderboard for "TrueSkill League"
    Then Eve's displayed conservative estimate is 26.0 (35.0 - 3 * 3.0)

  Scenario: New TrueSkill player with default uncertainty ranks below established player with similar mean
    Given league "TrueSkill League" uses the TrueSkill algorithm
    And player "Veteran" has mu = 26.0 and sigma = 1.0 (conservative estimate = 23.0)
    And player "Newcomer" has mu = 25.0 and sigma = 8.333 (conservative estimate = 0.001, i.e., 25.0 - 3 * 8.333 = -0.0)
    When a client requests the leaderboard for "TrueSkill League"
    Then "Veteran" is ranked above "Newcomer" despite Newcomer having a similar mean rating

  Scenario: New Glicko-2 player with default RD=350 ranks below established player with similar mu
    Given league "Glicko League" uses the Glicko-2 algorithm
    And player "Veteran" has mu = 1510 and RD = 30 (conservative estimate = 1450.0)
    And player "Newcomer" has mu = 1500 and RD = 350 (conservative estimate = 800.0)
    When a client requests the leaderboard for "Glicko League"
    Then "Veteran" is ranked above "Newcomer"

  Scenario: Leaderboard response includes both conservative estimate and raw rating components
    Given league "TrueSkill League" uses the TrueSkill algorithm
    And player "Alice" has mu = 30.0 and sigma = 2.0
    When a client requests the leaderboard for "TrueSkill League"
    Then Alice's leaderboard row includes:
      | field                | value |
      | mu                   | 30.0  |
      | sigma                | 2.0   |
      | conservative_estimate| 24.0  |

  Scenario: Leaderboard sort order uses conservative estimate, not raw mean
    Given league "Glicko League" uses the Glicko-2 algorithm
    And player "HighMu" has mu = 2000 and RD = 400 (conservative estimate = 1200.0)
    And player "LowMu" has mu = 1600 and RD = 30 (conservative estimate = 1540.0)
    When a client requests the leaderboard for "Glicko League" sorted by rank ascending
    Then "LowMu" appears above "HighMu" in the response

  Scenario: Conservative estimate is recalculated after each rating update
    Given league "Elo League" uses the Elo algorithm
    And player "Alice" has Elo rating 1200
    When a match is recorded where Alice defeats Bob
    And Alice's new Elo rating is 1216
    Then the leaderboard immediately reflects Alice's new sort key of 1216

  Scenario: Sorting by rank on leaderboard endpoint uses conservative estimate
    Given league "TrueSkill League" uses the TrueSkill algorithm
    When a client requests GET /leagues/trueskill-league/leaderboard?sort=rank&direction=asc
    Then the response is ordered by conservative estimate (mu - 3 * sigma) ascending
    And the ordering is not based on raw mu values alone
```
