# SR-PER-007: Player Alias Recalculation

**Status:** Draft
**Parent:** UR-PM-002
**Priority:** Should-have

## Description

When two player records are linked as aliases, the persistence layer triggers a full rating recalculation for all matches involving either player in the affected season(s). The recalculation treats all matches from both records as belonging to a single player and recomputes ratings in timestamp order. When an alias is removed, another full recalculation is triggered to revert to separate histories.

## Rationale

Aliasing changes the effective match history for the merged player and for all opponents who faced either alias. A full recalculation ensures that all ratings in the season accurately reflect the corrected history. Partial recalculation would leave inconsistencies.

## Acceptance Criteria

- [ ] Linking two players as aliases triggers a recalculation of all ratings in the affected season from the earliest match involving any player in the alias equivalence group
- [ ] The recalculation processes matches in timestamp order using the combined match set of all players in the equivalence group (aliases are transitive: if A→B and A→C, then A, B, and C all contribute to the merged rating)
- [ ] All rating snapshots for the affected season are updated to reflect the merged history
- [ ] Opponents' ratings are also recalculated where their matches involved any member of the alias group
- [ ] Removing an alias link triggers a recalculation that recomputes the equivalence groups without the removed link and recalculates accordingly
- [ ] The recalculation executes asynchronously as a background job (see SR-PER-009) rather than blocking the alias operation
- [ ] The recalculation correctly handles cases where both aliases participated in the same match (self-play is rejected or flagged)
- [ ] During recalculation, the system serves pre-recalculation ratings; updated ratings become available upon job completion

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player Alias Recalculation

  Background:
    Given the ladder-rs-persistence crate is available
    And a SQLite database connection pool is initialized
    And league 1 has an open Elo season (id 7)
    And player "Alice_old" (id 10) has 5 matches in season 7 with timestamps T1-T5
    And player "Alice_new" (id 11) has 3 matches in season 7 with timestamps T6-T8
    And the matches are interleaved by timestamp: T1, T3, T5 for id 10 and T2, T4, T6 for id 11, T7 and T8 for both separately

  Scenario: Linking two players as aliases triggers a recalculation job for the affected season
    When link_aliases(pool, primary_player_id: 10, alias_player_id: 11, created_by: admin_user_id) is called
    Then the function returns Ok(job_id)
    And a recalculation job exists in the recalculation_jobs table with season_id 7 and status "queued"

  Scenario: Recalculation processes the combined match set in timestamp order
    Given players 10 and 11 are linked as aliases
    And the recalculation job for season 7 is processed
    When get_rating_history(pool, player_id: 10, season_id: 7) is called after completion
    Then the rating history reflects matches from both players 10 and 11 combined
    And the entries are in ascending timestamp order (T1, T2, T3, T4, T5, T6, T7, T8)

  Scenario: All rating snapshots for the affected season are updated after recalculation
    Given players 10 and 11 are linked and the recalculation job has status "completed"
    When the rating_snapshots table is queried for season 7
    Then the snapshots for player 10's matches reflect the merged history computation
    And the snapshots for opponents who faced either player also reflect updated ratings

  Scenario: Opponents' ratings are also recalculated when affected by alias matches
    Given player "Opponent" (id 20) faced player 10 in match at T3 and player 11 in match at T6
    And players 10 and 11 are linked and recalculation is completed
    When get_rating_history(pool, player_id: 20, season_id: 7) is queried
    Then player 20's rating snapshots are updated to reflect the merged opponent history

  Scenario: Removing an alias link triggers a recalculation reverting to separate histories
    Given players 10 and 11 are currently linked as aliases
    When unlink_aliases(pool, primary_player_id: 10, alias_player_id: 11, triggered_by: admin_user_id) is called
    Then the function returns Ok(job_id)
    And a new recalculation job exists for season 7 with status "queued"
    And after the job completes, player 10 ratings are based only on their 5 matches
    And player 11 ratings are based only on their 3 matches

  Scenario: Recalculation executes asynchronously rather than blocking the alias operation
    When link_aliases(pool, ...) is called
    Then the function returns Ok(job_id) immediately (before recalculation completes)
    And the recalculation job can be polled via get_job_status(pool, job_id)

  Scenario: During recalculation the system serves pre-recalculation ratings
    Given players 10 and 11 are linked and the recalculation job status is "in_progress"
    When get_leaderboard(pool, season_id: 7) is called
    Then the leaderboard returns the pre-alias ratings (stale but available)
    And no error is returned

  Scenario: Upon completion, recalculated ratings atomically replace stale ratings
    Given the recalculation job for season 7 transitions from "in_progress" to "completed"
    When get_leaderboard(pool, season_id: 7) is called immediately after completion
    Then the leaderboard returns the post-recalculation ratings
    And no intermediate partial state is visible

  Scenario: Adding a third alias expands the equivalence group and triggers unified recalculation
    Given players 10 (A) and 11 (B) are linked as aliases and recalculation has completed for season 7
    And player 12 (C) has 2 matches in season 7
    When link_aliases(pool, primary_player_id: 10, alias_player_id: 12, created_by: admin_user_id) is called
    Then the function returns Ok(job_id)
    And a recalculation job is queued for season 7
    And after the job completes, get_leaderboard(pool, season_id: 7) shows players 10, 11, and 12 with the same unified rating
    And the match_count for the group reflects all matches from players 10, 11, and 12 combined

  Scenario: Self-play is rejected when both aliases participated in the same match
    Given players 10 and 11 are linked as aliases
    And a match exists in season 7 where both player 10 and player 11 are participants
    When the recalculation job processes this match
    Then the match is flagged with a self-play warning
    And the recalculation job does not fail entirely due to self-play (it is flagged, not aborted)
```
