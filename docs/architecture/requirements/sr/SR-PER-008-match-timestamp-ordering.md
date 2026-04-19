# SR-PER-008: Match Timestamp Ordering

**Status:** Draft
**Parent:** UR-ME-001, UR-ME-002
**Priority:** Must-have

## Description

Matches are timestamped at the moment of submission. The timestamp determines the processing order for rating calculations. When rating recalculations occur (alias changes, match corrections), matches are replayed in timestamp order.

## Rationale

Rating calculations are order-dependent: the outcome of processing match A before match B produces different ratings than the reverse order. A deterministic ordering based on submission timestamp ensures reproducible results and makes recalculations consistent.

## Acceptance Criteria

- [ ] Each match record includes a timestamp set at the moment the match is submitted to the persistence layer
- [ ] Rating calculations process matches in ascending timestamp order
- [ ] Batch-submitted matches receive distinct timestamps or a deterministic tie-breaking order within the batch
- [ ] Recalculations (alias, correction) replay matches in the same timestamp order as the original calculation
- [ ] The timestamp is stored with sufficient precision to distinguish matches submitted in rapid succession (millisecond precision minimum)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Match Timestamp Ordering

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And league 1 has an open Elo season (id 7)
    And players "Alice" (id 1) and "Bob" (id 2) are in league 1

  Scenario: Each match record has a timestamp set at submission time
    When record_match(pool, season_id: 7, ...) is called at time T
    Then the match record's recorded_at timestamp is within 1 second of T

  Scenario: Timestamp has millisecond precision minimum
    When two matches are submitted 100 milliseconds apart
    Then their recorded_at timestamps differ by at least 1 millisecond
    And they are distinguishable by timestamp for ordering purposes

  Scenario: Rating calculations process matches in ascending timestamp order during recalculation
    Given match M1 at timestamp T1 and match M2 at timestamp T2 where T1 < T2
    And both matches involve player 1
    And a recalculation job is triggered for season 7
    When the recalculation worker processes the job
    Then M1 is processed before M2
    And the rating snapshot after M1 uses the state computed from before M1
    And the rating snapshot after M2 uses the state computed from after M1

  Scenario: Recalculations replay matches in the same timestamp order as the original calculation
    Given season 7 has 10 matches with timestamps T1 through T10 in ascending order
    And a correction to match M5 triggers a recalculation
    When the recalculation processes all 10 matches
    Then matches are processed in the order T1, T2, T3, T4, T5_corrected, T6, T7, T8, T9, T10

  Scenario: Batch-submitted matches receive distinct timestamps or deterministic tie-breaking
    When a batch of 5 matches is submitted simultaneously (same wall-clock second)
    Then each match record has either a unique timestamp or an explicit ordering mechanism (e.g., sequence within batch)
    And get_rating_history for a player who appeared in multiple batch matches returns entries in a consistent deterministic order

  Scenario: Alias recalculation uses timestamp order from the combined match set
    Given player 10 has matches at T1, T3, T5 and player 11 has matches at T2, T4, T6
    And players 10 and 11 are linked as aliases
    When the recalculation worker processes the alias recalculation job
    Then matches are processed in the interleaved order: T1, T2, T3, T4, T5, T6
    And no match from T3 is processed before T2

  Scenario: Timestamp ordering is stable for correction replays
    Given match M3 is corrected and a recalculation is triggered
    And season 7 has matches at T1, T2, T3(corrected), T4, T5
    When recalculation completes
    Then the rating history for all players reflects ratings computed in strictly T1 < T2 < T3 < T4 < T5 order

  Scenario: Schema does not default recorded_at to CURRENT_TIMESTAMP
    Given the matches table schema is inspected
    When the column definition for recorded_at is examined
    Then recorded_at has no DEFAULT constraint
    And recorded_at is NOT NULL
    And the application must explicitly supply the timestamp on every INSERT
```
