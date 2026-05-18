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

use chrono::Utc;
use ladder_rs_persistence::pool::create_pool;
use ladder_rs_persistence::{AliasRepository, PersistenceError, PlayerAlias};
use sqlx::SqlitePool;
use uuid::Uuid;

// ── Test Fixtures ───────────────────────────────────────────────────────────

/// Creates an in-memory SQLite pool with full migrations applied.
async fn setup_test_db() -> SqlitePool {
    use sqlx::migrate::Migrator;
    use std::path::Path;

    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite pool for testing");

    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path)
            .await
            .expect("Failed to create migrator");
        migrator.run(&pool).await.expect("Failed to run migrations");
    }

    pool
}

// ── Data Seeding Helpers ────────────────────────────────────────────────────

async fn seed_user(pool: &SqlitePool, id: &str, username: &str) {
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, 'hash', 'user')",
    )
    .bind(id)
    .bind(username)
    .bind(format!("{}@test.local", username))
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert user {}: {}", id, e));
}

async fn seed_player(pool: &SqlitePool, id: &str, name: &str) {
    sqlx::query("INSERT INTO players (id, name, player_type) VALUES (?, ?, 'human')")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to insert player {}: {}", id, e));
}

async fn seed_league(pool: &SqlitePool, id: &str, name: &str) {
    sqlx::query(
        "INSERT INTO leagues (id, name, algorithm, visibility) VALUES (?, ?, 'elo', 'public')",
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert league {}: {}", id, e));
}

async fn seed_season(pool: &SqlitePool, id: &str, league_id: &str) {
    sqlx::query(
        "INSERT INTO seasons (id, league_id, algorithm, start_date) VALUES (?, ?, 'elo', ?)",
    )
    .bind(id)
    .bind(league_id)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert season {}: {}", id, e));
}

async fn seed_match(pool: &SqlitePool, id: &str, season_id: &str) {
    sqlx::query("INSERT INTO matches (id, season_id, recorded_at) VALUES (?, ?, ?)")
        .bind(id)
        .bind(season_id)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to insert match {}: {}", id, e));
}

async fn seed_match_participant(pool: &SqlitePool, id: &str, match_id: &str, player_id: &str) {
    sqlx::query(
        "INSERT INTO match_participants (id, match_id, player_id, placement) VALUES (?, ?, ?, 1)",
    )
    .bind(id)
    .bind(match_id)
    .bind(player_id)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert match_participant {}: {}", id, e));
}

async fn seed_alias(pool: &SqlitePool, primary_id: &str, alias_id: &str, created_by: &str) {
    sqlx::query(
        "INSERT INTO player_aliases (id, primary_player_id, alias_player_id, created_by) VALUES (?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(primary_id)
    .bind(alias_id)
    .bind(created_by)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert alias {}-{}: {}", primary_id, alias_id, e));
}

/// Full seeding for tests that need job creation (players + user + season + match).
async fn seed_for_job_tests(pool: &SqlitePool) {
    seed_user(pool, "user-1", "user1").await;
    seed_player(pool, "player-a", "Player A").await;
    seed_player(pool, "player-b", "Player B").await;
    seed_player(pool, "player-x", "Player X").await;
    seed_player(pool, "player-y", "Player Y").await;
    seed_league(pool, "league-1", "Test League").await;
    seed_season(pool, "season-1", "league-1").await;
    seed_match(pool, "match-1", "season-1").await;
    seed_match_participant(pool, "mp-1", "match-1", "player-a").await;
}

/// Seeding for alias group resolution tests (players + aliases).
async fn seed_for_resolve_tests(pool: &SqlitePool) {
    seed_user(pool, "user-1", "user1").await;
    seed_player(pool, "player-a", "Player A").await;
    seed_player(pool, "player-b", "Player B").await;
    seed_player(pool, "player-c", "Player C").await;
    seed_player(pool, "player-no-aliases", "Player No Aliases").await;
    seed_player(pool, "player-no-links", "Player No Links").await;
    seed_player(pool, "player-solo", "Player Solo").await;
    // Alias chain: A -> B -> C
    seed_alias(pool, "player-a", "player-b", "user-1").await;
    seed_alias(pool, "player-b", "player-c", "user-1").await;
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
    let pool = setup_test_db().await;
    seed_for_job_tests(&pool).await;

    let result = AliasRepository::create_alias(&pool, "player-a", "player-b", "user-1").await;

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
    let pool = setup_test_db().await;
    // No players exist — FK constraint will reject the INSERT.

    let result = AliasRepository::create_alias(&pool, "nonexistent-p", "player-b", "user-1").await;
    assert!(
        result.is_err(),
        "create_alias with non-existent primary player should return error"
    );

    let result = AliasRepository::create_alias(&pool, "player-a", "nonexistent-a", "user-1").await;
    assert!(
        result.is_err(),
        "create_alias with non-existent alias player should return error"
    );
}

#[tokio::test]
async fn test_remove_alias_removes_link_and_returns_job_ids() {
    let pool = setup_test_db().await;
    seed_for_job_tests(&pool).await;
    // Pre-seed the alias so we can remove it
    seed_alias(&pool, "player-a", "player-b", "user-1").await;

    let result = AliasRepository::remove_alias(&pool, "player-a", "player-b").await;

    let job_ids = result.expect("remove_alias should succeed for existing link");
    assert!(
        !job_ids.is_empty(),
        "remove_alias must return at least one job_id for recalculation"
    );
    for id in &job_ids {
        assert!(!id.is_empty(), "job IDs must not be empty strings");
    }

    // Verify the alias was actually removed
    let aliases = AliasRepository::get_aliases(&pool, "player-a")
        .await
        .expect("get_aliases should succeed");
    let still_linked = aliases.iter().any(|a| {
        (a.primary_player_id == "player-a" && a.alias_player_id == "player-b")
            || (a.primary_player_id == "player-b" && a.alias_player_id == "player-a")
    });
    assert!(!still_linked, "Alias should be removed after remove_alias");
}

#[tokio::test]
async fn test_remove_alias_with_nonexistent_link_is_idempotent_or_error() {
    let pool = setup_test_db().await;
    // Players must exist to avoid FK issues on the lookup
    seed_user(&pool, "user-1", "user1").await;
    seed_player(&pool, "player-no-link", "Player No Link").await;
    seed_player(&pool, "player-other", "Player Other").await;

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
    let pool = setup_test_db().await;
    seed_for_job_tests(&pool).await;

    let result = AliasRepository::create_alias(&pool, "player-a", "player-b", "user-1").await;

    let job_ids = result.expect("create_alias should succeed");
    assert!(
        !job_ids.is_empty(),
        "create_alias must return at least one job_id; seasons need recalculation"
    );
}

#[tokio::test]
async fn test_remove_alias_returns_non_empty_vec_of_job_ids() {
    let pool = setup_test_db().await;
    seed_for_job_tests(&pool).await;
    seed_alias(&pool, "player-a", "player-b", "user-1").await;

    let result = AliasRepository::remove_alias(&pool, "player-a", "player-b").await;

    let job_ids = result.expect("remove_alias should succeed");
    assert!(
        !job_ids.is_empty(),
        "remove_alias must return at least one job_id; seasons need recalculation"
    );
}

#[tokio::test]
async fn test_job_ids_are_valid_strings() {
    let pool = setup_test_db().await;
    seed_for_job_tests(&pool).await;

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
    let pool = setup_test_db().await;
    seed_user(&pool, "user-1", "user1").await;
    seed_player(&pool, "player-no-aliases", "Player No Aliases").await;

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
    let pool = setup_test_db().await;
    seed_user(&pool, "user-1", "user1").await;
    seed_player(&pool, "player-a", "Player A").await;
    seed_player(&pool, "player-b", "Player B").await;
    seed_alias(&pool, "player-a", "player-b", "user-1").await;

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
    let pool = setup_test_db().await;
    seed_for_resolve_tests(&pool).await;

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
    let pool = setup_test_db().await;
    seed_user(&pool, "user-1", "user1").await;
    seed_player(&pool, "player-a", "Player A").await;
    seed_player(&pool, "player-b", "Player B").await;
    seed_player(&pool, "player-c", "Player C").await;
    seed_player(&pool, "player-x", "Player X").await;
    // player-a is primary in two aliases, alias in one
    seed_alias(&pool, "player-a", "player-b", "user-1").await;
    seed_alias(&pool, "player-a", "player-c", "user-1").await;
    seed_alias(&pool, "player-x", "player-a", "user-1").await;

    let result = AliasRepository::get_aliases(&pool, "player-a").await;

    let aliases = result.expect("get_aliases should succeed");
    assert!(
        !aliases.is_empty(),
        "get_aliases should return alias records for a player with links"
    );

    for alias in &aliases {
        let matches = alias.primary_player_id == "player-a" || alias.alias_player_id == "player-a";
        assert!(
            matches,
            "Every returned alias must involve the queried player. Got primary={}, alias={}",
            alias.primary_player_id, alias.alias_player_id,
        );

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
    let pool = setup_test_db().await;
    seed_user(&pool, "user-1", "user1").await;
    seed_player(&pool, "player-no-links", "Player No Links").await;

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
    let pool = setup_test_db().await;
    seed_user(&pool, "user-1", "user1").await;
    seed_player(&pool, "player-a", "Player A").await;
    seed_player(&pool, "player-b", "Player B").await;
    seed_player(&pool, "player-c", "Player C").await;
    seed_player(&pool, "player-x", "Player X").await;
    // player-a as primary
    seed_alias(&pool, "player-a", "player-b", "user-1").await;
    seed_alias(&pool, "player-a", "player-c", "user-1").await;
    // player-a as alias
    seed_alias(&pool, "player-x", "player-a", "user-1").await;

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

    // Sum of both categories should equal total returned records
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
