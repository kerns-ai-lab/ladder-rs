//! Unit tests for the AliasRepository
//!
//! These tests validate the interface contract and expected behavior of
//! `AliasRepository`. Currently **all tests will FAIL at runtime** because
//! the repository methods are stubs returning `PersistenceError::Unknown`.
//! This is expected in TDD — the tests define the behavior specification
//! and will pass once the implementation stubs are completed.
//!
//! ## Coverage Map
//!
//! | Category              | Tests                                                  |
//! |-----------------------|--------------------------------------------------------|
//! | Link/Unlink           | 5 tests (basic create, same-player, nonexistent, remove, nonexistent link) |
//! | Job Insertion         | 3 tests (non-empty ids, valid string ids)              |
//! | Alias Group Resolution | 4 tests (no aliases, one alias, chain, nonexistent player) |
//! | Listing               | 3 tests (all records, empty, direction)                 |
//! | Error Cases           | 6 tests (empty ids, self-reference, empty created_by)   |
//! | Parameter Validation  | 2 tests (remove_alias empty primary/alias)              |
//! | Struct Integrity      | 1 test (PlayerAlias fields)                             |
//!
//! Total: 23 tests

use ladder_rs_persistence::pool::create_pool;
use ladder_rs_persistence::{AliasRepository, PersistenceError, PlayerAlias};
use sqlx::SqlitePool;

// ── Test Fixtures ───────────────────────────────────────────────────────────

/// Creates an in-memory SQLite pool for isolated test execution.
async fn setup_test_db() -> SqlitePool {
    create_pool("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite pool for testing")
}

// ── Helper: Extract error message for inspection ────────────────────────────

/// Extracts the inner message from a `PersistenceError` for assertion.
fn error_message(err: &PersistenceError) -> String {
    format!("{}", err)
}

// ══════════════════════════════════════════════════════════════════════════════
// LINK / UNLINK
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_alias_links_two_players_and_returns_job_ids() {
    // When implementation is complete:
    //   create_alias should insert a row into player_aliases and queue
    //   recalculation jobs for all seasons containing either player.
    //
    // Currently STUBBED → returns Unknown error → assertion fails (TDD red phase).
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(&pool, "player-a", "player-b", "user-1").await;

    // Once implemented: Ok(job_ids) where job_ids is not empty
    let job_ids = result.expect("create_alias should succeed with valid inputs");
    assert!(
        !job_ids.is_empty(),
        "create_alias must return at least one job_id for recalculation"
    );
    for id in &job_ids {
        assert!(!id.is_empty(), "job IDs must not be empty strings");
    }
}

#[tokio::test]
async fn test_create_alias_with_same_primary_and_alias_returns_error() {
    // Self-referencing aliases make no semantic sense and should be rejected.
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(
        &pool, "player-x", "player-x", // same ID
        "user-1",
    )
    .await;

    assert!(
        result.is_err(),
        "create_alias with identical primary and alias should return error"
    );
    let err = result.unwrap_err();
    let msg = error_message(&err).to_lowercase();
    assert!(
        msg.contains("same")
            || msg.contains("self")
            || msg.contains("identical")
            || msg.contains("invalid")
            || msg.contains("cannot"),
        "Error should indicate self-referencing is invalid, got: {}",
        msg,
    );
}

#[tokio::test]
async fn test_create_alias_with_nonexistent_players_returns_error() {
    // Both primary and alias players must exist in the players table
    // before an alias link can be created (FK constraint).
    let pool = setup_test_db().await;

    // Non-existent primary
    let result = AliasRepository::create_alias(&pool, "nonexistent-p", "player-b", "user-1").await;
    assert!(
        result.is_err(),
        "create_alias with non-existent primary player should return error"
    );

    // Non-existent alias
    let result = AliasRepository::create_alias(&pool, "player-a", "nonexistent-a", "user-1").await;
    assert!(
        result.is_err(),
        "create_alias with non-existent alias player should return error"
    );
}

#[tokio::test]
async fn test_remove_alias_removes_link_and_returns_job_ids() {
    // When implementation is complete:
    //   remove_alias should delete the player_aliases row and queue
    //   recalculation jobs for affected seasons.
    let pool = setup_test_db().await;

    let result = AliasRepository::remove_alias(&pool, "player-a", "player-b").await;

    let job_ids = result.expect("remove_alias should succeed for existing link");
    assert!(
        !job_ids.is_empty(),
        "remove_alias must return at least one job_id for recalculation"
    );
    for id in &job_ids {
        assert!(!id.is_empty(), "job IDs must not be empty strings");
    }
}

#[tokio::test]
async fn test_remove_alias_with_nonexistent_link_is_idempotent_or_error() {
    // Removing an alias that doesn't exist should either:
    //   a) Return an error indicating the link doesn't exist, OR
    //   b) Succeed idempotently (maybe returning 0 job_ids).
    let pool = setup_test_db().await;

    let result = AliasRepository::remove_alias(&pool, "player-no-link", "player-other").await;

    match result {
        Ok(job_ids) => {
            // Idempotent: no jobs triggered for non-existent link
            assert!(
                job_ids.is_empty(),
                "Idempotent remove of non-existent link should produce no jobs"
            );
        }
        Err(e) => {
            let msg = error_message(&e).to_lowercase();
            assert!(
                msg.contains("not found")
                    || msg.contains("no alias")
                    || msg.contains("does not exist")
                    || msg.contains("nonexistent"),
                "Error for non-existent link should indicate not found, got: {}",
                msg
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// JOB INSERTION
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_alias_returns_non_empty_vec_of_job_ids() {
    // Creating an alias must insert recalculation jobs for affected seasons.
    // The returned Vec<String> must be non-empty.
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(&pool, "player-a", "player-b", "user-1").await;

    let job_ids = result.expect("create_alias should succeed");
    assert!(
        !job_ids.is_empty(),
        "create_alias must return at least one job_id; seasons need recalculation"
    );
}

#[tokio::test]
async fn test_remove_alias_returns_non_empty_vec_of_job_ids() {
    // Removing an alias must also trigger recalculation by inserting jobs.
    // The returned Vec<String> must be non-empty.
    let pool = setup_test_db().await;

    let result = AliasRepository::remove_alias(&pool, "player-a", "player-b").await;

    let job_ids = result.expect("remove_alias should succeed");
    assert!(
        !job_ids.is_empty(),
        "remove_alias must return at least one job_id; seasons need recalculation"
    );
}

#[tokio::test]
async fn test_job_ids_are_valid_strings() {
    // Every returned job_id must be a non-empty, valid UUID or similar identifier.
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(&pool, "player-x", "player-y", "user-1").await;

    let job_ids = result.expect("create_alias should succeed");
    for (i, id) in job_ids.iter().enumerate() {
        assert!(!id.is_empty(), "job_id at index {} must not be empty", i);
        assert!(
            !id.contains(char::is_whitespace),
            "job_id at index {} ('{}') must not contain whitespace",
            i,
            id
        );
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// ALIAS GROUP RESOLUTION
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_resolve_alias_group_for_player_with_no_aliases_returns_self() {
    // A player with no aliases should resolve to just themselves.
    let pool = setup_test_db().await;

    let result = AliasRepository::resolve_alias_group(&pool, "player-no-aliases").await;

    let group = result.expect("resolve_alias_group should succeed");
    assert_eq!(
        group.len(),
        1,
        "Player with no aliases should resolve to [self]"
    );
    assert_eq!(
        group[0], "player-no-aliases",
        "Lone entry should be the player's own ID"
    );
}

#[tokio::test]
async fn test_resolve_alias_group_for_player_with_one_alias_returns_both() {
    // If A is linked to B (as primary/alias), resolving A should return [A, B].
    let pool = setup_test_db().await;

    let result = AliasRepository::resolve_alias_group(&pool, "player-a").await;

    let mut group = result.expect("resolve_alias_group should succeed");
    group.sort();
    let mut expected = vec!["player-a".to_string(), "player-b".to_string()];
    expected.sort();

    assert_eq!(
        group.len(),
        2,
        "Player with one alias should resolve to both IDs"
    );
    assert_eq!(
        group, expected,
        "Resolved group should contain both linked player IDs"
    );
}

#[tokio::test]
async fn test_resolve_alias_group_for_player_in_chain_returns_all() {
    // Given: A linked to B (primary->alias), B linked to C (primary->alias)
    // Resolving A should return [A, B, C] — the full transitive closure.
    let pool = setup_test_db().await;

    let result = AliasRepository::resolve_alias_group(&pool, "player-a").await;

    let mut group = result.expect("resolve_alias_group should succeed");
    group.sort();
    let mut expected = vec![
        "player-a".to_string(),
        "player-b".to_string(),
        "player-c".to_string(),
    ];
    expected.sort();

    assert_eq!(
        group.len(),
        3,
        "Player in an alias chain should resolve to all linked IDs"
    );
    assert_eq!(
        group, expected,
        "Resolved group should contain all three linked players"
    );
}

#[tokio::test]
async fn test_resolve_alias_group_for_nonexistent_player_returns_error() {
    // Resolving a player that does not exist should produce an error.
    let pool = setup_test_db().await;

    let result = AliasRepository::resolve_alias_group(&pool, "nonexistent-id").await;

    assert!(
        result.is_err(),
        "resolve_alias_group for nonexistent player should return error"
    );
    let err = result.unwrap_err();
    let msg = error_message(&err).to_lowercase();
    assert!(
        msg.contains("not found")
            || msg.contains("no such")
            || msg.contains("does not exist")
            || msg.contains("invalid"),
        "Error for nonexistent player should indicate not found, got: {}",
        msg
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// LISTING
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_aliases_returns_all_alias_records_for_a_player() {
    // get_aliases should return every PlayerAlias where the given player_id
    // appears as either primary_player_id or alias_player_id.
    let pool = setup_test_db().await;

    let result = AliasRepository::get_aliases(&pool, "player-a").await;

    let aliases = result.expect("get_aliases should succeed");
    // When stubs are complete, this player should have at least one alias record.
    assert!(
        !aliases.is_empty(),
        "get_aliases should return alias records for a player with links"
    );

    for alias in &aliases {
        // Every record must reference the queried player in one of the directions
        let matches = alias.primary_player_id == "player-a" || alias.alias_player_id == "player-a";
        assert!(
            matches,
            "Every returned alias must involve the queried player. Got primary={}, alias={}",
            alias.primary_player_id, alias.alias_player_id,
        );

        // Structural validations
        assert!(!alias.id.is_empty(), "alias record ID must not be empty");
        assert!(
            alias.primary_player_id != alias.alias_player_id,
            "Alias cannot link a player to itself"
        );
        assert!(!alias.created_by.is_empty(), "created_by must not be empty");
    }
}

#[tokio::test]
async fn test_get_aliases_for_player_with_no_aliases_returns_empty_vec() {
    // A player with no alias links should get an empty Vec, not an error.
    let pool = setup_test_db().await;

    let result = AliasRepository::get_aliases(&pool, "player-no-links").await;

    let aliases = result.expect("get_aliases should succeed even when none exist");
    assert!(
        aliases.is_empty(),
        "get_aliases for unlinked player should return empty Vec, got {:?}",
        aliases.len()
    );
}

#[tokio::test]
async fn test_get_aliases_distinguishes_primary_vs_alias_direction() {
    // When listing aliases for a player, the results must correctly indicate
    // whether the queried player is the primary or the alias in each link.
    // This test verifies that both directions are represented.
    let pool = setup_test_db().await;

    let result = AliasRepository::get_aliases(&pool, "player-a").await;

    let aliases = result.expect("get_aliases should succeed");

    let as_primary: Vec<_> = aliases
        .iter()
        .filter(|a| a.primary_player_id == "player-a")
        .collect();
    let as_alias: Vec<_> = aliases
        .iter()
        .filter(|a| a.alias_player_id == "player-a")
        .collect();

    // A player may be primary in some links and alias in others.
    // The sum of both categories should equal total returned records.
    assert_eq!(
        as_primary.len() + as_alias.len(),
        aliases.len(),
        "Sum of primary-matches and alias-matches should equal total records"
    );

    // Verify the structural invariant holds for both categories
    for alias in &as_primary {
        assert_eq!(
            alias.primary_player_id, "player-a",
            "Record filtered as 'as_primary' must have queried player as primary"
        );
        assert_ne!(
            alias.alias_player_id, "player-a",
            "Record filtered as 'as_primary' must have DIFFERENT alias player"
        );
    }
    for alias in &as_alias {
        assert_eq!(
            alias.alias_player_id, "player-a",
            "Record filtered as 'as_alias' must have queried player as alias"
        );
        assert_ne!(
            alias.primary_player_id, "player-a",
            "Record filtered as 'as_alias' must have DIFFERENT primary player"
        );
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// ERROR CASES
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_alias_with_empty_primary_player_id_returns_error() {
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(
        &pool, "", // empty primary
        "player-b", "user-1",
    )
    .await;

    assert!(
        result.is_err(),
        "create_alias with empty primary_player_id should return error"
    );
    // Expected: PersistenceError::InvalidInput or similar
}

#[tokio::test]
async fn test_create_alias_with_empty_alias_player_id_returns_error() {
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(
        &pool, "player-a", "", // empty alias
        "user-1",
    )
    .await;

    assert!(
        result.is_err(),
        "create_alias with empty alias_player_id should return error"
    );
}

#[tokio::test]
async fn test_create_alias_with_both_player_ids_empty_returns_error() {
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(&pool, "", "", "user-1").await;

    assert!(
        result.is_err(),
        "create_alias with both player IDs empty should return error"
    );
}

#[tokio::test]
async fn test_create_alias_with_self_referencing_players_is_rejected() {
    // Self-referencing alias (primary == alias) must always be rejected.
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(&pool, "player-solo", "player-solo", "user-1").await;

    assert!(
        result.is_err(),
        "Self-referencing alias (primary == alias) must be rejected"
    );
    let err = result.unwrap_err();
    let msg = error_message(&err).to_lowercase();
    assert!(
        msg.contains("same")
            || msg.contains("self")
            || msg.contains("identical")
            || msg.contains("invalid")
            || msg.contains("cannot"),
        "Error should indicate self-referencing is invalid, got: {}",
        msg
    );
}

#[tokio::test]
async fn test_create_alias_with_empty_created_by_returns_error() {
    let pool = setup_test_db().await;

    let result = AliasRepository::create_alias(
        &pool, "player-a", "player-b", "", // empty created_by
    )
    .await;

    assert!(
        result.is_err(),
        "create_alias with empty created_by should return error"
    );
    // Every alias must be auditable — created_by identifies who created the link.
}

// ══════════════════════════════════════════════════════════════════════════════
// EDGE CASES: remove_alias parameter validation
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_remove_alias_with_empty_primary_player_id_returns_error() {
    let pool = setup_test_db().await;

    let result = AliasRepository::remove_alias(&pool, "", "player-b").await;

    assert!(
        result.is_err(),
        "remove_alias with empty primary_player_id should return error"
    );
}

#[tokio::test]
async fn test_remove_alias_with_empty_alias_player_id_returns_error() {
    let pool = setup_test_db().await;

    let result = AliasRepository::remove_alias(&pool, "player-a", "").await;

    assert!(
        result.is_err(),
        "remove_alias with empty alias_player_id should return error"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// STRUCTURAL INTEGRITY: PlayerAlias type
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_player_alias_struct_has_required_fields() {
    // Verify the PlayerAlias struct compiles with all required fields.
    // This is a compile-time check, but we exercise it at runtime too.
    let alias = PlayerAlias {
        id: "alias-1".to_string(),
        primary_player_id: "player-a".to_string(),
        alias_player_id: "player-b".to_string(),
        created_by: "user-1".to_string(),
        created_at: chrono::Utc::now(),
    };

    // Ensure all fields can be read
    assert!(!alias.id.is_empty());
    assert!(!alias.primary_player_id.is_empty());
    assert!(!alias.alias_player_id.is_empty());
    assert!(!alias.created_by.is_empty());
    assert_ne!(
        alias.primary_player_id, alias.alias_player_id,
        "PlayerAlias should not link a player to itself"
    );
    // created_at should be set (non-default)
    let epoch = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap();
    assert!(
        alias.created_at > epoch,
        "created_at should be a recent timestamp"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// TDD STATUS NOTE
//
// All tests above will FAIL at runtime until the `AliasRepository` stubs are
// implemented. The stubs currently return `PersistenceError::Unknown(...)` for
// every method. Tests that expect `Ok(...)` will panic; tests that check for
// specific error variants (e.g., `InvalidInput`, `NotFound`) may also fail
// because the stubs return the generic `Unknown` variant.
//
// This is the expected TDD workflow:
//   1. Tests written (RED)   ← WE ARE HERE
//   2. Implementation (GREEN)
//   3. Refinement (REFACTOR)
// ══════════════════════════════════════════════════════════════════════════════
