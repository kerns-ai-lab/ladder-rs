# SR-ALG-004: Season Transition Seeding

**Status:** Draft
**Parent:** UR-LM-002
**Priority:** Must-have

## Description

When an algorithm type change triggers a new season, the operator chooses one of two seeding strategies: (A) reset all players to the new algorithm's default ratings, or (B) seed from the prior season's ordinal rankings mapped to initial ratings with a spread. Mid-season joiners (players added after the season starts) always receive the new algorithm's default rating regardless of the seeding choice.

## Rationale

Resetting is the simplest approach and appropriate when the operator wants a fresh start. Ordinal seeding preserves relative competitive standing across algorithm changes, which is valuable when continuity matters. The spread ensures meaningful differentiation rather than clustering all seeded players at the same initial rating. Mid-season joiners get defaults because they have no prior ranking to seed from.

## Acceptance Criteria

- [ ] When a new season is created, the system presents the operator with options: (A) Reset to defaults, or (B) Seed from prior rankings
- [ ] Option A initializes all players at the new algorithm's default rating values
- [ ] Option B ranks players by their final rating in the prior season (ordinal ranking) and maps those ranks to initial ratings in the new algorithm's scale with a defined spread between ranks
- [ ] The seeding spread is sufficient to produce meaningful initial rating differentiation (not all players at the same value)
- [ ] Players added to the league after the season has started receive the algorithm's default rating regardless of the seeding choice used at season creation
- [ ] The seeding choice is recorded on the season record for auditability
- [ ] Seeding is applied atomically as part of the season creation transaction

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Season Transition Seeding

  Background:
    Given the system is running
    And an Admin is authenticated
    And league "Omega League" used TrueSkill in Season 1
    And Season 1 has ended with 4 ranked players:
      | Rank | Player  | TrueSkill Conservative Rating |
      | 1    | Alice   | 31.0                          |
      | 2    | Bob     | 27.5                          |
      | 3    | Carol   | 22.0                          |
      | 4    | Dave    | 18.0                          |

  Scenario: Seeding choice presented to operator on algorithm type change
    When the Admin initiates an algorithm type change from TrueSkill to Glicko-2 for "Omega League"
    Then the system presents a seeding choice before creating the new season
    And the choices include "Reset to defaults" and "Seed from prior rankings"

  Scenario: Option A resets all players to Glicko-2 default ratings
    When the Admin selects "Reset to defaults" and confirms the algorithm change to Glicko-2
    Then Season 2 is created for "Omega League" using Glicko-2
    And Alice's Season 2 initial rating is the Glicko-2 default mu = 1500 and RD = 350
    And Bob's Season 2 initial rating is the Glicko-2 default mu = 1500 and RD = 350
    And Carol's Season 2 initial rating is the Glicko-2 default mu = 1500 and RD = 350
    And Dave's Season 2 initial rating is the Glicko-2 default mu = 1500 and RD = 350
    And the season record stores seeding_strategy = "reset"

  Scenario: Option B seeds players from prior season ordinal ranking with spread
    When the Admin selects "Seed from prior rankings" and confirms the algorithm change to Glicko-2
    Then Season 2 is created for "Omega League" using Glicko-2
    And Alice (rank 1) receives a higher initial Glicko-2 mu than Bob (rank 2)
    And Bob (rank 2) receives a higher initial Glicko-2 mu than Carol (rank 3)
    And Carol (rank 3) receives a higher initial Glicko-2 mu than Dave (rank 4)
    And no two players share the same initial mu value
    And the season record stores seeding_strategy = "ordinal"

  Scenario: Ordinal seeding produces meaningful rating differentiation (not all same value)
    When the Admin selects "Seed from prior rankings" and confirms the algorithm change to Glicko-2
    Then the difference in initial mu between Alice (rank 1) and Dave (rank 4) is at least 50 rating points
    And the difference between adjacent ranks is at least 10 rating points

  Scenario: Mid-season joiner always receives algorithm default regardless of seeding choice
    Given "Omega League" is in Season 2 with Glicko-2 and seeding_strategy = "ordinal"
    When a new player "Eve" is added to "Omega League" during Season 2
    Then Eve's initial rating is the Glicko-2 default mu = 1500 and RD = 350
    And Eve is not seeded from any prior season ranking

  Scenario: Mid-season joiner receives default even when seeding_strategy was reset
    Given "Omega League" is in Season 2 with Glicko-2 and seeding_strategy = "reset"
    When a new player "Frank" is added to "Omega League" during Season 2
    Then Frank's initial rating is the Glicko-2 default mu = 1500 and RD = 350

  Scenario: Seeding choice is recorded on the season record for auditability
    When the Admin selects "Seed from prior rankings" and confirms the algorithm change to Glicko-2
    Then the Season 2 record's "seeding_strategy" field is set to "ordinal"
    And the Season 2 record is queryable via the API

  Scenario: Seeding is applied atomically as part of season creation transaction
    Given the system is configured to fail after writing Season 2 but before writing player seeds
    When the Admin initiates a season transition with seeding
    Then the transaction is rolled back entirely
    And no Season 2 record exists
    And no partial player seeds exist
    And Season 1 remains open

  Scenario: Cannot create a new season without selecting a seeding strategy
    When the Admin initiates an algorithm type change from TrueSkill to Elo but does not select a seeding strategy
    Then the request is rejected with HTTP 400
    And no new season is created

  Scenario: Seeding from prior rankings uses final rating (not intermediate) for ordinal rank
    Given Alice played 10 matches in Season 1 and her final conservative rating was 31.0
    And Alice's rating peaked at 35.0 mid-season but ended at 31.0
    When the Admin seeds Season 2 from prior rankings
    Then Alice's seeding rank is based on her final Season 1 conservative rating of 31.0
    And not on her peak mid-season rating of 35.0
```
