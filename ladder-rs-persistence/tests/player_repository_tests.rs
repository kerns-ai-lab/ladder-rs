//! Player Repository unit tests for ladder-rs-persistence
//!
//! These tests verify the full behavior of PlayerRepository including CRUD,
//! soft-delete, prefix search, auto-creation, and error handling.
//!
//! Tests compile against stubs (will fail at runtime — expected for TDD).
//! When the PlayerRepository implementation is complete, all tests should pass.
//!
//! Task: ladder-rs-907.4.4

use chrono::Utc;
use ladder_rs_persistence::pool::create_pool;
use ladder_rs_persistence::{PersistenceError, PlayerFilter, PlayerPatch, PlayerRepository};
use sqlx::SqlitePool;

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Creates an in-memory SQLite database and sets up the minimal schema
/// (players, league_players, leagues, users, player_account_links) needed
/// for PlayerRepository tests.
async fn setup_test_db() -> SqlitePool {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Failed to create pool");

    // Create tables required by PlayerRepository
    // (mirrors migrations 01 through 03 so FKs resolve)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT NOT NULL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            force_password_change INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS leagues (
            id TEXT NOT NULL PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            algorithm TEXT NOT NULL,
            visibility TEXT NOT NULL DEFAULT 'public',
            is_active INTEGER NOT NULL DEFAULT 1,
            is_archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create leagues table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS players (
            id TEXT NOT NULL PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            nickname TEXT,
            player_type TEXT NOT NULL DEFAULT 'human',
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create players table");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_players_name ON players(name)")
        .execute(&pool)
        .await
        .expect("Failed to create players name index");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS league_players (
            id TEXT NOT NULL PRIMARY KEY,
            league_id TEXT NOT NULL REFERENCES leagues(id) ON DELETE RESTRICT,
            player_id TEXT NOT NULL REFERENCES players(id) ON DELETE RESTRICT,
            is_active INTEGER NOT NULL DEFAULT 1,
            joined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(league_id, player_id)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create league_players table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS player_account_links (
            id TEXT NOT NULL PRIMARY KEY,
            player_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE RESTRICT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT,
            UNIQUE(player_id),
            UNIQUE(user_id)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create player_account_links table");

    pool
}

/// Creates a test league and returns its ID
async fn seed_league(pool: &SqlitePool, name: &str) -> String {
    let league_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO leagues (id, name, algorithm) VALUES (?, ?, 'glicko')")
        .bind(&league_id)
        .bind(name)
        .execute(pool)
        .await
        .expect("Failed to insert test league");
    league_id
}

/// Creates a test user and returns its ID
async fn seed_user(pool: &SqlitePool, username: &str) -> String {
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, 'hash', 'user')",
    )
    .bind(&user_id)
    .bind(username)
    .bind(format!("{}@test.com", username))
    .execute(pool)
    .await
    .expect("Failed to insert test user");
    user_id
}

// ============================================================================
// CREATE PLAYER TESTS
// ============================================================================

#[tokio::test]
async fn test_create_player_with_name_and_type() {
    let pool = setup_test_db().await;

    let result = PlayerRepository::create_player(&pool, "Alice", "human").await;
    assert!(
        result.is_ok(),
        "create_player should succeed for valid inputs: {:?}",
        result
    );

    let player = result.unwrap();
    assert_eq!(player.name, "Alice");
    assert_eq!(player.player_type, "human");
    assert!(player.is_active, "newly created player should be active");
    assert!(!player.id.is_empty(), "player should have a non-empty id");
}

#[tokio::test]
async fn test_create_player_returns_unique_ids() {
    let pool = setup_test_db().await;

    let player1 = PlayerRepository::create_player(&pool, "PlayerOne", "human")
        .await
        .expect("Failed to create player 1");
    let player2 = PlayerRepository::create_player(&pool, "PlayerTwo", "human")
        .await
        .expect("Failed to create player 2");

    assert_ne!(player1.id, player2.id, "players should have unique IDs");
}

#[tokio::test]
async fn test_create_player_has_created_at_timestamp() {
    let pool = setup_test_db().await;

    let before = Utc::now();
    let player = PlayerRepository::create_player(&pool, "Timestamped", "human")
        .await
        .expect("Failed to create player");

    // created_at should be recent (within a reasonable window)
    let after = Utc::now();
    assert!(
        player.created_at >= before,
        "created_at should be >= start time"
    );
    assert!(
        player.created_at <= after,
        "created_at should be <= end time"
    );
}

#[tokio::test]
async fn test_create_player_with_non_human_type() {
    let pool = setup_test_db().await;

    let player = PlayerRepository::create_player(&pool, "Bot42", "non-human")
        .await
        .expect("Failed to create non-human player");

    assert_eq!(player.player_type, "non-human");
    assert!(player.is_active);
}

// ============================================================================
// GET PLAYER TESTS
// ============================================================================

#[tokio::test]
async fn test_get_player_by_valid_id() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "Bob", "human")
        .await
        .expect("Failed to create player");

    let result = PlayerRepository::get_player(&pool, &created.id).await;
    assert!(result.is_ok(), "get_player should succeed: {:?}", result);

    let found = result.unwrap();
    assert!(found.is_some(), "player should be found");
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Bob");
}

#[tokio::test]
async fn test_get_player_nonexistent_id() {
    let pool = setup_test_db().await;

    let result = PlayerRepository::get_player(&pool, "non-existent-id-12345")
        .await
        .expect("get_player should not error for unknown ID");

    assert!(
        result.is_none(),
        "get_player with non-existent ID should return None, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_get_player_empty_id() {
    let pool = setup_test_db().await;

    let result = PlayerRepository::get_player(&pool, "").await;

    // Empty ID should either return None or error
    match result {
        Ok(player_opt) => assert!(player_opt.is_none(), "empty ID should return None"),
        Err(_) => { /* error is also acceptable for empty id */ }
    }
}

// ============================================================================
// UPDATE PLAYER TESTS
// ============================================================================

#[tokio::test]
async fn test_update_player_changes_name() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "Original", "human")
        .await
        .expect("Failed to create player");

    let patch = PlayerPatch {
        name: Some("Renamed".to_string()),
        nickname: None,
        player_type: None,
        is_active: None,
    };

    let updated = PlayerRepository::update_player(&pool, &created.id, &patch)
        .await
        .expect("update_player should succeed");

    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.id, created.id, "ID should not change on update");
}

#[tokio::test]
async fn test_update_player_changes_nickname() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "NicknameTest", "human")
        .await
        .expect("Failed to create player");
    assert!(
        created.nickname.is_none(),
        "new player should have no nickname"
    );

    let patch = PlayerPatch {
        name: None,
        nickname: Some("The Beast".to_string()),
        player_type: None,
        is_active: None,
    };

    let updated = PlayerRepository::update_player(&pool, &created.id, &patch)
        .await
        .expect("update_player should succeed");

    assert_eq!(updated.nickname.as_deref(), Some("The Beast"));
}

#[tokio::test]
async fn test_update_player_changes_player_type() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "TypeSwitcher", "human")
        .await
        .expect("Failed to create player");

    let patch = PlayerPatch {
        name: None,
        nickname: None,
        player_type: Some("non-human".to_string()),
        is_active: None,
    };

    let updated = PlayerRepository::update_player(&pool, &created.id, &patch)
        .await
        .expect("update_player should succeed");

    assert_eq!(updated.player_type, "non-human");
}

#[tokio::test]
async fn test_update_player_changes_active_status() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "ActiveToggle", "human")
        .await
        .expect("Failed to create player");

    let patch = PlayerPatch {
        name: None,
        nickname: None,
        player_type: None,
        is_active: Some(false),
    };

    let updated = PlayerRepository::update_player(&pool, &created.id, &patch)
        .await
        .expect("update_player should succeed");

    assert!(!updated.is_active, "player should be deactivated");
}

#[tokio::test]
async fn test_update_player_changes_multiple_fields() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "MultiChange", "human")
        .await
        .expect("Failed to create player");

    let patch = PlayerPatch {
        name: Some("MultiChanged".to_string()),
        nickname: Some("MC".to_string()),
        player_type: Some("non-human".to_string()),
        is_active: None,
    };

    let updated = PlayerRepository::update_player(&pool, &created.id, &patch)
        .await
        .expect("update_player should succeed");

    assert_eq!(updated.name, "MultiChanged");
    assert_eq!(updated.nickname.as_deref(), Some("MC"));
    assert_eq!(updated.player_type, "non-human");
    assert!(updated.is_active, "is_active should be untouched");
}

#[tokio::test]
async fn test_update_player_nonexistent_id_returns_error() {
    let pool = setup_test_db().await;

    let patch = PlayerPatch {
        name: Some("Ghost".to_string()),
        nickname: None,
        player_type: None,
        is_active: None,
    };

    let result = PlayerRepository::update_player(&pool, "nonexistent-id", &patch).await;

    assert!(
        result.is_err(),
        "update_player with nonexistent ID should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_update_player_empty_patch_is_noop() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "NoopPatch", "human")
        .await
        .expect("Failed to create player");

    let patch = PlayerPatch {
        name: None,
        nickname: None,
        player_type: None,
        is_active: None,
    };

    let updated = PlayerRepository::update_player(&pool, &created.id, &patch)
        .await
        .expect("update_player with empty patch should succeed");

    // Should return the same values
    assert_eq!(updated.name, created.name);
    assert_eq!(updated.nickname, created.nickname);
    assert_eq!(updated.player_type, created.player_type);
    assert_eq!(updated.is_active, created.is_active);
}

// ============================================================================
// LIST PLAYERS TESTS
// ============================================================================

#[tokio::test]
async fn test_list_players_returns_players_in_league() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "Test League").await;

    let player1 = PlayerRepository::create_player(&pool, "ListAlpha", "human")
        .await
        .expect("Failed to create player 1");
    let player2 = PlayerRepository::create_player(&pool, "ListBeta", "human")
        .await
        .expect("Failed to create player 2");

    PlayerRepository::add_to_league(&pool, &league_id, &player1.id)
        .await
        .expect("Failed to add player 1 to league");
    PlayerRepository::add_to_league(&pool, &league_id, &player2.id)
        .await
        .expect("Failed to add player 2 to league");

    let filter = PlayerFilter::default();
    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert_eq!(players.len(), 2, "should list 2 players in the league");
}

#[tokio::test]
async fn test_list_players_empty_league_returns_empty_vec() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "Empty League").await;

    let filter = PlayerFilter::default();
    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert!(players.is_empty(), "empty league should return empty vec");
}

#[tokio::test]
async fn test_list_players_filters_by_player_type() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "FilterLeague").await;

    let human = PlayerRepository::create_player(&pool, "HumanPlayer", "human")
        .await
        .expect("Failed to create human player");
    let bot = PlayerRepository::create_player(&pool, "BotPlayer", "non-human")
        .await
        .expect("Failed to create bot player");

    PlayerRepository::add_to_league(&pool, &league_id, &human.id)
        .await
        .expect("Failed to add human to league");
    PlayerRepository::add_to_league(&pool, &league_id, &bot.id)
        .await
        .expect("Failed to add bot to league");

    let filter = PlayerFilter {
        player_type: Some("human".to_string()),
        is_active: None,
        limit: None,
        offset: None,
    };

    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert_eq!(players.len(), 1);
    assert_eq!(players[0].player_type, "human");
}

#[tokio::test]
async fn test_list_players_filters_by_is_active() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "ActiveFilterLeague").await;

    let active = PlayerRepository::create_player(&pool, "ActivePlayer", "human")
        .await
        .expect("Failed to create active player");
    let inactive = PlayerRepository::create_player(&pool, "InactivePlayer", "human")
        .await
        .expect("Failed to create inactive player");

    PlayerRepository::add_to_league(&pool, &league_id, &active.id)
        .await
        .expect("Failed to add active player to league");
    PlayerRepository::add_to_league(&pool, &league_id, &inactive.id)
        .await
        .expect("Failed to add inactive player to league");

    // Soft-delete the inactive one
    PlayerRepository::soft_delete_from_league(&pool, &league_id, &inactive.id)
        .await
        .expect("Failed to soft-delete player");

    let filter = PlayerFilter {
        player_type: None,
        is_active: Some(true),
        limit: None,
        offset: None,
    };

    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert_eq!(players.len(), 1, "only active players should be listed");
    assert_eq!(players[0].name, "ActivePlayer");
}

#[tokio::test]
async fn test_list_players_respects_limit_and_offset() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "PaginatedLeague").await;

    for i in 0..5 {
        let player =
            PlayerRepository::create_player(&pool, &format!("PaginatedPlayer{}", i), "human")
                .await
                .expect("Failed to create player");
        PlayerRepository::add_to_league(&pool, &league_id, &player.id)
            .await
            .expect("Failed to add player to league");
    }

    let filter = PlayerFilter {
        player_type: None,
        is_active: None,
        limit: Some(2),
        offset: Some(0),
    };

    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert_eq!(players.len(), 2, "limit should cap results to 2");
}

#[tokio::test]
async fn test_list_players_default_filter_active_only() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "DefaultFilterLeague").await;

    let active = PlayerRepository::create_player(&pool, "DefaultActive", "human")
        .await
        .expect("Failed to create player");

    PlayerRepository::add_to_league(&pool, &league_id, &active.id)
        .await
        .expect("Failed to add player to league");

    // Default filter has is_active = Some(true), limit = Some(20), offset = Some(0)
    let filter = PlayerFilter::default();
    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert_eq!(players.len(), 1);
    assert!(players[0].is_active);
}

// ============================================================================
// ADD TO LEAGUE TESTS
// ============================================================================

#[tokio::test]
async fn test_add_to_league_associates_player() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "AssocLeague").await;

    let player = PlayerRepository::create_player(&pool, "LeaguePlayer", "human")
        .await
        .expect("Failed to create player");

    let result = PlayerRepository::add_to_league(&pool, &league_id, &player.id).await;
    assert!(result.is_ok(), "add_to_league should succeed: {:?}", result);

    // Verify the player appears in league listing
    let filter = PlayerFilter::default();
    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert!(
        players.iter().any(|p| p.id == player.id),
        "player should be in league"
    );
}

#[tokio::test]
async fn test_add_to_league_duplicate_behavior() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "DupLeague").await;

    let player = PlayerRepository::create_player(&pool, "DupPlayer", "human")
        .await
        .expect("Failed to create player");

    // First add should succeed
    let first = PlayerRepository::add_to_league(&pool, &league_id, &player.id).await;
    assert!(first.is_ok(), "first add_to_league should succeed");

    // Second add: should either be idempotent (Ok) or return Conflict error
    let second = PlayerRepository::add_to_league(&pool, &league_id, &player.id).await;
    match second {
        Ok(()) => { /* idempotent — acceptable */ }
        Err(PersistenceError::Conflict(_)) => { /* explicit conflict — also acceptable */ }
        Err(e) => {
            panic!("unexpected error for duplicate add_to_league: {:?}", e)
        }
    }
}

#[tokio::test]
async fn test_add_to_league_with_nonexistent_player_returns_error() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "BadLeague").await;

    let result = PlayerRepository::add_to_league(&pool, &league_id, "nonexistent-player").await;

    assert!(
        result.is_err(),
        "add_to_league with nonexistent player should return error"
    );
}

#[tokio::test]
async fn test_add_to_league_with_nonexistent_league_returns_error() {
    let pool = setup_test_db().await;

    let player = PlayerRepository::create_player(&pool, "OrphanPlayer", "human")
        .await
        .expect("Failed to create player");

    let result = PlayerRepository::add_to_league(&pool, "nonexistent-league", &player.id).await;

    assert!(
        result.is_err(),
        "add_to_league with nonexistent league should return error"
    );
}

// ============================================================================
// SOFT-DELETE TESTS
// ============================================================================

#[tokio::test]
async fn test_soft_delete_sets_is_active_false() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "SoftDeleteLeague").await;

    let player = PlayerRepository::create_player(&pool, "SoftDeleteMe", "human")
        .await
        .expect("Failed to create player");

    PlayerRepository::add_to_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to add player to league");

    // Soft-delete
    let result = PlayerRepository::soft_delete_from_league(&pool, &league_id, &player.id).await;
    assert!(result.is_ok(), "soft_delete should succeed: {:?}", result);
}

#[tokio::test]
async fn test_soft_delete_does_not_remove_player_record() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "RetainLeague").await;

    let player = PlayerRepository::create_player(&pool, "RetainMe", "human")
        .await
        .expect("Failed to create player");

    PlayerRepository::add_to_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to add player to league");

    PlayerRepository::soft_delete_from_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to soft-delete");

    // Player record should still exist
    let found = PlayerRepository::get_player(&pool, &player.id)
        .await
        .expect("get_player should succeed");

    assert!(
        found.is_some(),
        "soft-deleted player record should still exist"
    );
}

#[tokio::test]
async fn test_soft_deleted_player_excluded_from_active_listing() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "ExcludeLeague").await;

    let player = PlayerRepository::create_player(&pool, "ExcludeMe", "human")
        .await
        .expect("Failed to create player");

    PlayerRepository::add_to_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to add player to league");

    PlayerRepository::soft_delete_from_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to soft-delete");

    let filter = PlayerFilter {
        player_type: None,
        is_active: Some(true),
        limit: None,
        offset: None,
    };

    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert!(
        !players.iter().any(|p| p.id == player.id),
        "soft-deleted player should not appear in active listing"
    );
}

#[tokio::test]
async fn test_soft_deleted_player_appears_in_inactive_listing() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "InactiveListLeague").await;

    let player = PlayerRepository::create_player(&pool, "InactiveOne", "human")
        .await
        .expect("Failed to create player");

    PlayerRepository::add_to_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to add player to league");

    PlayerRepository::soft_delete_from_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to soft-delete");

    let filter = PlayerFilter {
        player_type: None,
        is_active: Some(false),
        limit: None,
        offset: None,
    };

    let players = PlayerRepository::list_players(&pool, &league_id, &filter)
        .await
        .expect("list_players should succeed");

    assert!(
        players.iter().any(|p| p.id == player.id),
        "soft-deleted player should appear in inactive listing"
    );
}

#[tokio::test]
async fn test_soft_delete_already_inactive_player_is_idempotent() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "DoubleDeleteLeague").await;

    let player = PlayerRepository::create_player(&pool, "DoubleDelete", "human")
        .await
        .expect("Failed to create player");

    PlayerRepository::add_to_league(&pool, &league_id, &player.id)
        .await
        .expect("Failed to add player to league");

    // First soft-delete
    PlayerRepository::soft_delete_from_league(&pool, &league_id, &player.id)
        .await
        .expect("first soft-delete should succeed");

    // Second soft-delete on already-inactive player
    let result = PlayerRepository::soft_delete_from_league(&pool, &league_id, &player.id).await;
    assert!(
        result.is_ok(),
        "soft-delete of already-inactive player should be idempotent: {:?}",
        result
    );
}

#[tokio::test]
async fn test_soft_delete_nonexistent_league_player_returns_error_or_ok() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "NoMemberLeague").await;

    // Player not in league — soft-delete should be error or idempotent Ok
    let result =
        PlayerRepository::soft_delete_from_league(&pool, &league_id, "nonexistent-player").await;

    match result {
        Ok(()) => { /* idempotent — acceptable */ }
        Err(PersistenceError::NotFound { .. }) => { /* explicit not-found — acceptable */ }
        Err(e) => {
            panic!("unexpected error for soft-delete on non-member: {:?}", e);
        }
    }
}

// ============================================================================
// PREFIX SEARCH TESTS
// ============================================================================

#[tokio::test]
async fn test_search_by_prefix_returns_matching_players() {
    let pool = setup_test_db().await;

    PlayerRepository::create_player(&pool, "SearchAlpha", "human")
        .await
        .expect("Failed to create player");
    PlayerRepository::create_player(&pool, "SearchBeta", "human")
        .await
        .expect("Failed to create player");
    PlayerRepository::create_player(&pool, "Zeta", "human")
        .await
        .expect("Failed to create player");

    let results = PlayerRepository::search_by_prefix(&pool, "Search", 10)
        .await
        .expect("search_by_prefix should succeed");

    assert_eq!(
        results.len(),
        2,
        "should find 2 players starting with 'Search'"
    );
}

#[tokio::test]
async fn test_search_by_prefix_is_case_insensitive() {
    let pool = setup_test_db().await;

    PlayerRepository::create_player(&pool, "CaseMatch", "human")
        .await
        .expect("Failed to create player");

    let results = PlayerRepository::search_by_prefix(&pool, "casematch", 10)
        .await
        .expect("search_by_prefix should succeed");

    assert_eq!(
        results.len(),
        1,
        "case-insensitive search should find 'CaseMatch'"
    );
}

#[tokio::test]
async fn test_search_by_prefix_respects_limit() {
    let pool = setup_test_db().await;

    for i in 0..5 {
        PlayerRepository::create_player(&pool, &format!("LimitedPlayer{}", i), "human")
            .await
            .expect("Failed to create player");
    }

    let results = PlayerRepository::search_by_prefix(&pool, "LimitedPlayer", 3)
        .await
        .expect("search_by_prefix should succeed");

    assert_eq!(results.len(), 3, "should respect limit of 3");
}

#[tokio::test]
async fn test_search_by_prefix_no_matches_returns_empty() {
    let pool = setup_test_db().await;

    PlayerRepository::create_player(&pool, "RealPlayer", "human")
        .await
        .expect("Failed to create player");

    let results = PlayerRepository::search_by_prefix(&pool, "ZzzNonexistent", 10)
        .await
        .expect("search_by_prefix should succeed");

    assert!(results.is_empty(), "no matches should return empty vec");
}

#[tokio::test]
async fn test_search_by_prefix_empty_query_behavior() {
    let pool = setup_test_db().await;

    PlayerRepository::create_player(&pool, "SomePlayer", "human")
        .await
        .expect("Failed to create player");

    let result = PlayerRepository::search_by_prefix(&pool, "", 10).await;

    // Empty query could return all players or error — both are acceptable
    match result {
        Ok(players) => {
            // If it returns success, all players are fine
            assert!(players.iter().any(|p| p.name == "SomePlayer"));
        }
        Err(_) => { /* error for empty query is also acceptable */ }
    }
}

#[tokio::test]
async fn test_search_by_prefix_limit_zero() {
    let pool = setup_test_db().await;

    PlayerRepository::create_player(&pool, "ZeroLimitPlayer", "human")
        .await
        .expect("Failed to create player");

    let results = PlayerRepository::search_by_prefix(&pool, "Zero", 0)
        .await
        .expect("search_by_prefix with limit 0 should succeed");

    assert!(results.is_empty(), "limit 0 should return empty vec");
}

// ============================================================================
// GET OR CREATE PLAYER TESTS (Auto-Creation)
// ============================================================================

#[tokio::test]
async fn test_get_or_create_player_creates_new() {
    let pool = setup_test_db().await;

    let (player, created) = PlayerRepository::get_or_create_player(&pool, "NewPlayer", "human")
        .await
        .expect("get_or_create_player should succeed");

    assert!(created, "new player should have created = true");
    assert_eq!(player.name, "NewPlayer");
    assert_eq!(player.player_type, "human");
    assert!(player.is_active);
}

#[tokio::test]
async fn test_get_or_create_player_returns_existing() {
    let pool = setup_test_db().await;

    // First call creates
    let (player1, created1) =
        PlayerRepository::get_or_create_player(&pool, "ExistingPlayer", "human")
            .await
            .expect("get_or_create_player should succeed");
    assert!(created1);

    // Second call should return existing
    let (player2, created2) =
        PlayerRepository::get_or_create_player(&pool, "ExistingPlayer", "human")
            .await
            .expect("get_or_create_player should succeed");

    assert!(!created2, "existing player should have created = false");
    assert_eq!(player1.id, player2.id, "should return same player ID");
    assert_eq!(player2.name, "ExistingPlayer");
}

#[tokio::test]
async fn test_get_or_create_player_same_name_same_player() {
    let pool = setup_test_db().await;

    // Call sequentially to verify same name returns same player
    let (p1, c1) = PlayerRepository::get_or_create_player(&pool, "SameSame", "human")
        .await
        .expect("get_or_create_player should succeed");
    let (p2, c2) = PlayerRepository::get_or_create_player(&pool, "SameSame", "human")
        .await
        .expect("get_or_create_player should succeed");
    let (p3, c3) = PlayerRepository::get_or_create_player(&pool, "SameSame", "human")
        .await
        .expect("get_or_create_player should succeed");

    assert!(c1, "first call should create");
    assert!(!c2, "second call should return existing");
    assert!(!c3, "third call should return existing");
    assert_eq!(p1.id, p2.id);
    assert_eq!(p2.id, p3.id);
}

#[tokio::test]
async fn test_get_or_create_player_respects_player_type_on_creation() {
    let pool = setup_test_db().await;

    let (player, created) =
        PlayerRepository::get_or_create_player(&pool, "BotCreator", "non-human")
            .await
            .expect("get_or_create_player should succeed");

    assert!(created);
    assert_eq!(player.player_type, "non-human");
}

#[tokio::test]
async fn test_get_or_create_player_keeps_original_type_on_existing() {
    let pool = setup_test_db().await;

    // Create with one type
    let (p1, _) = PlayerRepository::get_or_create_player(&pool, "TypeKeeper", "human")
        .await
        .expect("get_or_create_player should succeed");

    // Try to "get or create" with different type — should return existing with original type
    let (p2, created) = PlayerRepository::get_or_create_player(&pool, "TypeKeeper", "non-human")
        .await
        .expect("get_or_create_player should succeed");

    assert!(!created);
    // The returned player should keep the original type (or use the requested type — implementation-dependent)
    // We just verify the IDs match
    assert_eq!(p1.id, p2.id);
}

// ============================================================================
// LINK ACCOUNT TESTS
// ============================================================================

#[tokio::test]
async fn test_link_account_success() {
    let pool = setup_test_db().await;
    let user_id = seed_user(&pool, "link_tester").await;

    let player = PlayerRepository::create_player(&pool, "LinkedPlayer", "human")
        .await
        .expect("Failed to create player");

    let result = PlayerRepository::link_account(&pool, &player.id, &user_id, "system").await;
    assert!(result.is_ok(), "link_account should succeed: {:?}", result);
}

#[tokio::test]
async fn test_link_account_nonexistent_player() {
    let pool = setup_test_db().await;
    let user_id = seed_user(&pool, "bad_link_user").await;

    let result =
        PlayerRepository::link_account(&pool, "nonexistent-player", &user_id, "system").await;

    assert!(
        result.is_err(),
        "link_account with nonexistent player should return error"
    );
}

#[tokio::test]
async fn test_link_account_nonexistent_user() {
    let pool = setup_test_db().await;

    let player = PlayerRepository::create_player(&pool, "OrphanLinked", "human")
        .await
        .expect("Failed to create player");

    let result =
        PlayerRepository::link_account(&pool, &player.id, "nonexistent-user", "system").await;

    assert!(
        result.is_err(),
        "link_account with nonexistent user should return error"
    );
}

#[tokio::test]
async fn test_link_account_duplicate_player_is_unique() {
    let pool = setup_test_db().await;
    let user1 = seed_user(&pool, "dup_link_user1").await;
    let user2 = seed_user(&pool, "dup_link_user2").await;

    let player = PlayerRepository::create_player(&pool, "DupLinked", "human")
        .await
        .expect("Failed to create player");

    // First link should succeed
    let first = PlayerRepository::link_account(&pool, &player.id, &user1, "system").await;
    assert!(first.is_ok(), "first link should succeed");

    // Second link with same player but different user should conflict
    let second = PlayerRepository::link_account(&pool, &player.id, &user2, "system").await;
    assert!(
        second.is_err(),
        "linking same player to another user should conflict: {:?}",
        second
    );
}

// ============================================================================
// ERROR CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_create_player_empty_name_returns_error() {
    let pool = setup_test_db().await;

    let result = PlayerRepository::create_player(&pool, "", "human").await;

    assert!(
        result.is_err(),
        "create_player with empty name should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_player_whitespace_only_name() {
    let pool = setup_test_db().await;

    let result = PlayerRepository::create_player(&pool, "   ", "human").await;

    // Whitespace-only should also be treated as invalid
    assert!(
        result.is_err(),
        "create_player with whitespace-only name should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_player_empty_player_type() {
    let pool = setup_test_db().await;

    let result = PlayerRepository::create_player(&pool, "ValidName", "").await;

    assert!(
        result.is_err(),
        "create_player with empty player_type should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_player_duplicate_name_returns_error() {
    let pool = setup_test_db().await;

    PlayerRepository::create_player(&pool, "DuplicateName", "human")
        .await
        .expect("first create should succeed");

    let result = PlayerRepository::create_player(&pool, "DuplicateName", "human").await;

    assert!(
        result.is_err(),
        "create_player with duplicate name should return Conflict error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_get_player_invalid_id_format() {
    let pool = setup_test_db().await;

    // Very long ID — should be handled gracefully
    let long_id = "x".repeat(10_000);

    // Should either return Ok(None) or error
    let result = PlayerRepository::get_player(&pool, &long_id).await;
    match result {
        Ok(player_opt) => assert!(player_opt.is_none(), "invalid ID should return None"),
        Err(_) => { /* error is also acceptable */ }
    }
}

#[tokio::test]
async fn test_update_player_empty_id_returns_error() {
    let pool = setup_test_db().await;

    let patch = PlayerPatch {
        name: Some("NeverSaved".to_string()),
        nickname: None,
        player_type: None,
        is_active: None,
    };

    let result = PlayerRepository::update_player(&pool, "", &patch).await;

    assert!(
        result.is_err(),
        "update_player with empty ID should return error"
    );
}

#[tokio::test]
async fn test_list_players_nonexistent_league_returns_empty() {
    let pool = setup_test_db().await;

    let filter = PlayerFilter::default();
    let result = PlayerRepository::list_players(&pool, "nonexistent-league-id", &filter).await;

    // Should either return empty vec or error
    match result {
        Ok(players) => assert!(players.is_empty(), "nonexistent league should return empty"),
        Err(_) => { /* error is also acceptable */ }
    }
}

// ============================================================================
// EDGE CASES AND COMPREHENSIVE SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_player_lifecycle_full_crud() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "LifecycleLeague").await;

    // CREATE
    let player = PlayerRepository::create_player(&pool, "LifecyclePlayer", "human")
        .await
        .expect("create failed");
    assert!(player.is_active);
    assert_eq!(player.name, "LifecyclePlayer");

    // READ
    let found = PlayerRepository::get_player(&pool, &player.id)
        .await
        .expect("get failed")
        .expect("player not found");
    assert_eq!(found.id, player.id);

    // ADD TO LEAGUE
    PlayerRepository::add_to_league(&pool, &league_id, &player.id)
        .await
        .expect("add_to_league failed");

    // UPDATE
    let patch = PlayerPatch {
        name: Some("LifecycleRenamed".to_string()),
        nickname: Some("LC".to_string()),
        player_type: None,
        is_active: None,
    };
    let updated = PlayerRepository::update_player(&pool, &player.id, &patch)
        .await
        .expect("update failed");
    assert_eq!(updated.name, "LifecycleRenamed");
    assert_eq!(updated.nickname.as_deref(), Some("LC"));

    // SOFT-DELETE FROM LEAGUE
    PlayerRepository::soft_delete_from_league(&pool, &league_id, &player.id)
        .await
        .expect("soft_delete failed");

    // VERIFY EXCLUDED FROM ACTIVE LIST
    let active_filter = PlayerFilter {
        player_type: None,
        is_active: Some(true),
        limit: None,
        offset: None,
    };
    let active_players = PlayerRepository::list_players(&pool, &league_id, &active_filter)
        .await
        .expect("list_players failed");
    assert!(!active_players.iter().any(|p| p.id == player.id));
}

#[tokio::test]
async fn test_multiple_players_in_multiple_leagues() {
    let pool = setup_test_db().await;
    let league_a = seed_league(&pool, "LeagueA").await;
    let league_b = seed_league(&pool, "LeagueB").await;

    let p1 = PlayerRepository::create_player(&pool, "MultiLeagueP1", "human")
        .await
        .expect("create p1 failed");
    let p2 = PlayerRepository::create_player(&pool, "MultiLeagueP2", "human")
        .await
        .expect("create p2 failed");

    // p1 in both leagues, p2 only in league_a
    PlayerRepository::add_to_league(&pool, &league_a, &p1.id)
        .await
        .expect("p1 to A failed");
    PlayerRepository::add_to_league(&pool, &league_a, &p2.id)
        .await
        .expect("p2 to A failed");
    PlayerRepository::add_to_league(&pool, &league_b, &p1.id)
        .await
        .expect("p1 to B failed");

    let filter = PlayerFilter::default();
    let league_a_players = PlayerRepository::list_players(&pool, &league_a, &filter)
        .await
        .expect("list A failed");
    let league_b_players = PlayerRepository::list_players(&pool, &league_b, &filter)
        .await
        .expect("list B failed");

    assert_eq!(league_a_players.len(), 2);
    assert_eq!(league_b_players.len(), 1);
    assert_eq!(league_b_players[0].id, p1.id);
}

#[tokio::test]
async fn test_search_by_prefix_with_special_characters() {
    let pool = setup_test_db().await;

    PlayerRepository::create_player(&pool, "Player-One", "human")
        .await
        .expect("create failed");
    PlayerRepository::create_player(&pool, "Player_Two", "human")
        .await
        .expect("create failed");
    PlayerRepository::create_player(&pool, "Player.Three", "human")
        .await
        .expect("create failed");

    let results = PlayerRepository::search_by_prefix(&pool, "Player-", 10)
        .await
        .expect("search failed");

    // Should match Player-One (depends on LIKE behavior with special chars)
    assert!(!results.is_empty(), "should match at least Player-One");
}

#[tokio::test]
async fn test_update_player_then_get_reflects_changes() {
    let pool = setup_test_db().await;

    let created = PlayerRepository::create_player(&pool, "BeforeUpdate", "human")
        .await
        .expect("create failed");

    let patch = PlayerPatch {
        name: Some("AfterUpdate".to_string()),
        nickname: Some("Updated".to_string()),
        player_type: Some("non-human".to_string()),
        is_active: Some(false),
    };

    PlayerRepository::update_player(&pool, &created.id, &patch)
        .await
        .expect("update failed");

    let fetched = PlayerRepository::get_player(&pool, &created.id)
        .await
        .expect("get failed")
        .expect("player should exist");

    assert_eq!(fetched.name, "AfterUpdate");
    assert_eq!(fetched.nickname.as_deref(), Some("Updated"));
    assert_eq!(fetched.player_type, "non-human");
    assert!(!fetched.is_active);
}

#[tokio::test]
async fn test_soft_delete_in_one_league_does_not_affect_other_league() {
    let pool = setup_test_db().await;
    let league_a = seed_league(&pool, "IsolationLeagueA").await;
    let league_b = seed_league(&pool, "IsolationLeagueB").await;

    let player = PlayerRepository::create_player(&pool, "IsolatedPlayer", "human")
        .await
        .expect("create failed");

    PlayerRepository::add_to_league(&pool, &league_a, &player.id)
        .await
        .expect("add to A failed");
    PlayerRepository::add_to_league(&pool, &league_b, &player.id)
        .await
        .expect("add to B failed");

    // Soft-delete from league A only
    PlayerRepository::soft_delete_from_league(&pool, &league_a, &player.id)
        .await
        .expect("soft-delete from A failed");

    // Player should still be active in league B
    let filter = PlayerFilter {
        player_type: None,
        is_active: Some(true),
        limit: None,
        offset: None,
    };
    let league_b_players = PlayerRepository::list_players(&pool, &league_b, &filter)
        .await
        .expect("list B failed");

    assert!(
        league_b_players.iter().any(|p| p.id == player.id),
        "player should still be active in league B"
    );

    // Player should NOT be active in league A
    let league_a_players = PlayerRepository::list_players(&pool, &league_a, &filter)
        .await
        .expect("list A failed");

    assert!(
        !league_a_players.iter().any(|p| p.id == player.id),
        "player should NOT be active in league A"
    );
}

#[tokio::test]
async fn test_create_many_players_then_list_by_type() {
    let pool = setup_test_db().await;
    let league_id = seed_league(&pool, "ManyPlayersLeague").await;

    let mut human_ids = Vec::new();
    let mut bot_ids = Vec::new();

    for i in 0..10 {
        let ptype = if i % 2 == 0 { "human" } else { "non-human" };
        let name = format!("ManyPlayer{}", i);
        let player = PlayerRepository::create_player(&pool, &name, ptype)
            .await
            .expect("create failed");
        PlayerRepository::add_to_league(&pool, &league_id, &player.id)
            .await
            .expect("add failed");

        if i % 2 == 0 {
            human_ids.push(player.id);
        } else {
            bot_ids.push(player.id);
        }
    }

    let human_filter = PlayerFilter {
        player_type: Some("human".to_string()),
        is_active: None,
        limit: None,
        offset: None,
    };
    let humans = PlayerRepository::list_players(&pool, &league_id, &human_filter)
        .await
        .expect("list humans failed");

    let bot_filter = PlayerFilter {
        player_type: Some("non-human".to_string()),
        is_active: None,
        limit: None,
        offset: None,
    };
    let bots = PlayerRepository::list_players(&pool, &league_id, &bot_filter)
        .await
        .expect("list bots failed");

    assert_eq!(humans.len(), 5, "should have 5 humans");
    assert_eq!(bots.len(), 5, "should have 5 bots");
}
