//! League Repository comprehensive unit tests for ladder-rs-persistence
//!
//! Tests cover the full LeagueRepository public API:
//! - create_league (valid, duplicate name, empty name, edge cases)
//! - get_league (existing, nonexistent, empty id)
//! - list_leagues (pagination, active filter, archived filter, visibility)
//! - update_league (field changes, nonexistent id, empty patch)
//! - archive/unarchive_league (set flags, idempotent, nonexistent)
//! - assign_operator / remove_operator / get_operators / is_operator
//!
//! These are TDD-style tests: they exercise the repository stubs and will
//! FAIL at runtime until the stubs in league_repository.rs are implemented.
//! Once implemented, these tests validate the full behavioral contract.
//!
//! Task: ladder-rs-907.4.2

use chrono::Utc;
use ladder_rs_persistence::pool::create_pool;
use ladder_rs_persistence::{LeagueFilter, LeaguePatch, LeagueRepository, PersistenceError};
use sqlx::SqlitePool;

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Runs the full migration suite and returns an in-memory pool.
/// Uses real migrations to prevent schema drift.
async fn setup_test_db() -> SqlitePool {
    use sqlx::migrate::Migrator;
    use std::path::Path;

    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Failed to create pool");

    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path)
            .await
            .expect("Failed to create migrator");
        migrator.run(&pool).await.expect("Failed to run migrations");
    }

    pool
}

/// Creates a test league by raw SQL insert and returns its ID.
/// Used for operator tests and update/archive tests that need an existing league.
async fn seed_league(pool: &SqlitePool, name: &str, algorithm: &str, visibility: &str) -> String {
    let league_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO leagues (id, name, algorithm, visibility) VALUES (?, ?, ?, ?)")
        .bind(&league_id)
        .bind(name)
        .bind(algorithm)
        .bind(visibility)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to insert test league {}: {}", name, e));
    league_id
}

/// Creates a test user by raw SQL insert and returns its ID.
/// Used for operator assignment tests.
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
    .unwrap_or_else(|e| panic!("Failed to insert test user {}: {}", username, e));
    user_id
}

/// Inserts a league with specific id, name, and archived status for filter testing.
async fn seed_league_with_status(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    is_active: bool,
    is_archived: bool,
    visibility: &str,
) {
    sqlx::query(
        "INSERT INTO leagues (id, name, algorithm, visibility, is_active, is_archived) VALUES (?, ?, 'elo', ?, ?, ?)",
    )
    .bind(id)
    .bind(name)
    .bind(visibility)
    .bind(is_active as i32)
    .bind(is_archived as i32)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert league {}: {}", id, e));
}

// ============================================================================
// CREATE LEAGUE TESTS
// ============================================================================

#[tokio::test]
async fn test_create_league_with_valid_params_returns_ok_league() {
    let pool = setup_test_db().await;

    let result = LeagueRepository::create_league(
        &pool,
        "Test League Alpha",
        "A test league for unit testing",
        "glicko",
        "public",
        "system",
    )
    .await;

    assert!(
        result.is_ok(),
        "create_league should succeed for valid inputs: {:?}",
        result
    );

    let league = result.unwrap();
    assert_eq!(league.name, "Test League Alpha");
    assert_eq!(
        league.description.as_deref(),
        Some("A test league for unit testing")
    );
    assert_eq!(league.algorithm, "glicko");
    assert_eq!(league.visibility, "public");
    assert!(league.is_active, "newly created league should be active");
    assert!(
        !league.is_archived,
        "newly created league should not be archived"
    );
    assert!(!league.id.is_empty(), "league should have a non-empty id");
}

#[tokio::test]
async fn test_create_league_generates_unique_ids() {
    let pool = setup_test_db().await;

    let league1 = LeagueRepository::create_league(
        &pool,
        "Unique League 1",
        "First",
        "elo",
        "public",
        "system",
    )
    .await
    .expect("Failed to create league 1");

    let league2 = LeagueRepository::create_league(
        &pool,
        "Unique League 2",
        "Second",
        "elo",
        "public",
        "system",
    )
    .await
    .expect("Failed to create league 2");

    assert_ne!(league1.id, league2.id, "leagues should have unique IDs");
}

#[tokio::test]
async fn test_create_league_has_timestamps() {
    let pool = setup_test_db().await;

    let before = Utc::now();
    let league = LeagueRepository::create_league(
        &pool,
        "Timestamped League",
        "Testing timestamps",
        "elo",
        "private",
        "system",
    )
    .await
    .expect("Failed to create league");

    let after = Utc::now();

    assert!(
        league.created_at >= before,
        "created_at should be >= start time"
    );
    assert!(
        league.created_at <= after,
        "created_at should be <= end time"
    );
    assert!(
        league.updated_at >= before,
        "updated_at should be >= start time"
    );
    assert!(
        league.updated_at <= after,
        "updated_at should be <= end time"
    );
}

#[tokio::test]
async fn test_create_league_defaults_is_active_true_and_is_archived_false() {
    let pool = setup_test_db().await;

    let league = LeagueRepository::create_league(
        &pool,
        "Default Flags League",
        "",
        "elo",
        "public",
        "system",
    )
    .await
    .expect("Failed to create league");

    assert!(league.is_active, "new league should be active by default");
    assert!(
        !league.is_archived,
        "new league should not be archived by default"
    );
}

#[tokio::test]
async fn test_create_league_with_private_visibility() {
    let pool = setup_test_db().await;

    let league = LeagueRepository::create_league(
        &pool,
        "Private League",
        "",
        "trueskill",
        "private",
        "system",
    )
    .await
    .expect("Failed to create private league");

    assert_eq!(league.visibility, "private");
    assert_eq!(league.algorithm, "trueskill");
}

#[tokio::test]
async fn test_create_league_with_different_algorithms() {
    let pool = setup_test_db().await;

    let elo = LeagueRepository::create_league(&pool, "Elo League", "", "elo", "public", "system")
        .await
        .expect("Failed to create elo league");
    assert_eq!(elo.algorithm, "elo");

    let glicko =
        LeagueRepository::create_league(&pool, "Glicko League", "", "glicko", "public", "system")
            .await
            .expect("Failed to create glicko league");
    assert_eq!(glicko.algorithm, "glicko");

    let trueskill = LeagueRepository::create_league(
        &pool,
        "TrueSkill League",
        "",
        "trueskill",
        "public",
        "system",
    )
    .await
    .expect("Failed to create trueskill league");
    assert_eq!(trueskill.algorithm, "trueskill");
}

// ============================================================================
// CREATE LEAGUE - ERROR CASES
// ============================================================================

#[tokio::test]
async fn test_create_league_duplicate_name_returns_error() {
    let pool = setup_test_db().await;

    LeagueRepository::create_league(&pool, "Duplicate Name", "", "elo", "public", "system")
        .await
        .expect("first create should succeed");

    let result =
        LeagueRepository::create_league(&pool, "Duplicate Name", "", "elo", "public", "system")
            .await;

    assert!(
        result.is_err(),
        "create_league with duplicate name should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_league_empty_name_returns_error() {
    let pool = setup_test_db().await;

    let result =
        LeagueRepository::create_league(&pool, "", "some description", "elo", "public", "system")
            .await;

    assert!(
        result.is_err(),
        "create_league with empty name should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_league_whitespace_only_name_returns_error() {
    let pool = setup_test_db().await;

    let result =
        LeagueRepository::create_league(&pool, "   ", "description", "elo", "public", "system")
            .await;

    assert!(
        result.is_err(),
        "create_league with whitespace-only name should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_league_empty_algorithm_returns_error() {
    let pool = setup_test_db().await;

    let result =
        LeagueRepository::create_league(&pool, "Valid Name", "", "", "public", "system").await;

    assert!(
        result.is_err(),
        "create_league with empty algorithm should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_league_empty_visibility_returns_error() {
    let pool = setup_test_db().await;

    let result =
        LeagueRepository::create_league(&pool, "Valid Name", "", "elo", "", "system").await;

    assert!(
        result.is_err(),
        "create_league with empty visibility should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_league_with_very_long_name() {
    let pool = setup_test_db().await;

    let long_name = "L".repeat(10_000);
    let result = LeagueRepository::create_league(
        &pool,
        &long_name,
        "long name test",
        "elo",
        "public",
        "system",
    )
    .await;

    // Very long names should either succeed or return a validation error
    match result {
        Ok(league) => {
            assert_eq!(league.name, long_name);
        }
        Err(_) => {
            // Error for excessively long names is also acceptable
        }
    }
}

// ============================================================================
// GET LEAGUE TESTS
// ============================================================================

#[tokio::test]
async fn test_get_league_returns_some_for_existing_league() {
    let pool = setup_test_db().await;

    let created = LeagueRepository::create_league(
        &pool,
        "Get Me",
        "a league to retrieve",
        "elo",
        "public",
        "system",
    )
    .await
    .expect("Failed to create league");

    let result = LeagueRepository::get_league(&pool, &created.id).await;
    assert!(result.is_ok(), "get_league should succeed: {:?}", result);

    let found = result.unwrap();
    assert!(found.is_some(), "league should be found by ID");
    let found = found.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Get Me");
    assert_eq!(found.algorithm, "elo");
    assert_eq!(found.visibility, "public");
}

#[tokio::test]
async fn test_get_league_returns_ok_none_for_nonexistent_id() {
    let pool = setup_test_db().await;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let result = LeagueRepository::get_league(&pool, &nonexistent_id)
        .await
        .expect("get_league should not error for unknown ID");

    assert!(
        result.is_none(),
        "get_league with nonexistent ID should return None, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_get_league_empty_id_behavior() {
    let pool = setup_test_db().await;

    let result = LeagueRepository::get_league(&pool, "").await;

    // Empty ID should either return Ok(None) or an error
    match result {
        Ok(league_opt) => assert!(league_opt.is_none(), "empty ID should return None"),
        Err(_) => { /* error is also acceptable for empty id */ }
    }
}

// ============================================================================
// LIST LEAGUES TESTS
// ============================================================================

#[tokio::test]
async fn test_list_leagues_returns_all_matching_public_leagues() {
    let pool = setup_test_db().await;

    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Public Alpha",
        true,
        false,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Public Beta",
        true,
        false,
        "public",
    )
    .await;

    let filter = LeagueFilter {
        is_active: None,
        is_archived: None,
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(leagues.len(), 2, "should list 2 public leagues");
    for league in &leagues {
        assert_eq!(league.visibility, "public");
    }
}

#[tokio::test]
async fn test_list_leagues_filters_by_is_active_true() {
    let pool = setup_test_db().await;

    let active_id = uuid::Uuid::new_v4().to_string();
    let inactive_id = uuid::Uuid::new_v4().to_string();

    seed_league_with_status(&pool, &active_id, "Active League", true, false, "public").await;
    seed_league_with_status(
        &pool,
        &inactive_id,
        "Inactive League",
        false,
        false,
        "public",
    )
    .await;

    let filter = LeagueFilter {
        is_active: Some(true),
        is_archived: None,
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(leagues.len(), 1, "should list only active leagues");
    assert_eq!(leagues[0].name, "Active League");
    assert!(leagues[0].is_active);
}

#[tokio::test]
async fn test_list_leagues_filters_by_is_active_false() {
    let pool = setup_test_db().await;

    let active_id = uuid::Uuid::new_v4().to_string();
    let inactive_id = uuid::Uuid::new_v4().to_string();

    seed_league_with_status(&pool, &active_id, "Active League", true, false, "public").await;
    seed_league_with_status(
        &pool,
        &inactive_id,
        "Inactive League",
        false,
        false,
        "public",
    )
    .await;

    let filter = LeagueFilter {
        is_active: Some(false),
        is_archived: None,
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(leagues.len(), 1, "should list only inactive leagues");
    assert_eq!(leagues[0].name, "Inactive League");
    assert!(!leagues[0].is_active);
}

#[tokio::test]
async fn test_list_leagues_filters_by_is_archived_true() {
    let pool = setup_test_db().await;

    let normal_id = uuid::Uuid::new_v4().to_string();
    let archived_id = uuid::Uuid::new_v4().to_string();

    seed_league_with_status(&pool, &normal_id, "Normal League", true, false, "public").await;
    seed_league_with_status(&pool, &archived_id, "Archived League", true, true, "public").await;

    let filter = LeagueFilter {
        is_active: None,
        is_archived: Some(true),
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(leagues.len(), 1, "should list only archived leagues");
    assert_eq!(leagues[0].name, "Archived League");
    assert!(leagues[0].is_archived);
}

#[tokio::test]
async fn test_list_leagues_filters_by_is_archived_false() {
    let pool = setup_test_db().await;

    let normal_id = uuid::Uuid::new_v4().to_string();
    let archived_id = uuid::Uuid::new_v4().to_string();

    seed_league_with_status(&pool, &normal_id, "Normal League", true, false, "public").await;
    seed_league_with_status(&pool, &archived_id, "Archived League", true, true, "public").await;

    let filter = LeagueFilter {
        is_active: None,
        is_archived: Some(false),
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(leagues.len(), 1, "should list only non-archived leagues");
    assert_eq!(leagues[0].name, "Normal League");
    assert!(!leagues[0].is_archived);
}

#[tokio::test]
async fn test_list_leagues_combined_active_and_archived_filters() {
    let pool = setup_test_db().await;

    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Active+NotArchived",
        true,
        false,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Active+Archived",
        true,
        true,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Inactive+NotArchived",
        false,
        false,
        "public",
    )
    .await;

    let filter = LeagueFilter {
        is_active: Some(true),
        is_archived: Some(false),
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(
        leagues.len(),
        1,
        "should match active AND not archived only"
    );
    assert_eq!(leagues[0].name, "Active+NotArchived");
    assert!(leagues[0].is_active);
    assert!(!leagues[0].is_archived);
}

#[tokio::test]
async fn test_list_leagues_respects_limit() {
    let pool = setup_test_db().await;

    for i in 0..10 {
        seed_league_with_status(
            &pool,
            &uuid::Uuid::new_v4().to_string(),
            &format!("Limit League {}", i),
            true,
            false,
            "public",
        )
        .await;
    }

    let filter = LeagueFilter {
        is_active: None,
        is_archived: None,
        limit: Some(3),
        offset: Some(0),
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(leagues.len(), 3, "limit should cap results to 3");
}

#[tokio::test]
async fn test_list_leagues_respects_offset() {
    let pool = setup_test_db().await;

    for i in 0..5 {
        seed_league_with_status(
            &pool,
            &uuid::Uuid::new_v4().to_string(),
            &format!("Offset League {}", i),
            true,
            false,
            "public",
        )
        .await;
    }

    // Get page 1: offset 0, limit 2
    let page1 = LeagueRepository::list_leagues(
        &pool,
        &LeagueFilter {
            is_active: None,
            is_archived: None,
            limit: Some(2),
            offset: Some(0),
        },
    )
    .await
    .expect("list_leagues page 1 should succeed");

    // Get page 2: offset 2, limit 2
    let page2 = LeagueRepository::list_leagues(
        &pool,
        &LeagueFilter {
            is_active: None,
            is_archived: None,
            limit: Some(2),
            offset: Some(2),
        },
    )
    .await
    .expect("list_leagues page 2 should succeed");

    assert_eq!(page1.len(), 2, "page 1 should have 2 results");
    assert_eq!(page2.len(), 2, "page 2 should have 2 results");

    // Verify pages are disjoint
    let page1_ids: Vec<&str> = page1.iter().map(|l| l.id.as_str()).collect();
    let page2_ids: Vec<&str> = page2.iter().map(|l| l.id.as_str()).collect();
    for id in &page2_ids {
        assert!(
            !page1_ids.contains(id),
            "page 2 ids should not overlap page 1"
        );
    }
}

#[tokio::test]
async fn test_list_leagues_default_filter() {
    let pool = setup_test_db().await;

    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Active Not Archived",
        true,
        false,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Inactive Not Archived",
        false,
        false,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Active Archived",
        true,
        true,
        "public",
    )
    .await;

    let filter = LeagueFilter::default();
    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues with default filter should succeed");

    // Default: is_active=Some(true), is_archived=Some(false)
    assert_eq!(
        leagues.len(),
        1,
        "default filter should show only active, non-archived"
    );
    assert_eq!(leagues[0].name, "Active Not Archived");
    assert!(leagues[0].is_active);
    assert!(!leagues[0].is_archived);
}

#[tokio::test]
async fn test_list_leagues_empty_result() {
    let pool = setup_test_db().await;

    // No data seeded, filter for active non-archived
    let filter = LeagueFilter::default();
    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed on empty database");

    assert!(leagues.is_empty(), "empty database should return empty vec");
}

#[tokio::test]
async fn test_list_leagues_limit_zero() {
    let pool = setup_test_db().await;

    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Zero Limit Test",
        true,
        false,
        "public",
    )
    .await;

    let filter = LeagueFilter {
        is_active: None,
        is_archived: None,
        limit: Some(0),
        offset: Some(0),
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues with limit 0 should succeed");

    assert!(leagues.is_empty(), "limit 0 should return empty vec");
}

// ============================================================================
// LIST LEAGUES - VISIBILITY FILTERING
// ============================================================================

#[tokio::test]
async fn test_list_leagues_includes_public_leagues() {
    let pool = setup_test_db().await;

    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Public 1",
        true,
        false,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Public 2",
        true,
        false,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Private 1",
        true,
        false,
        "private",
    )
    .await;

    let filter = LeagueFilter {
        is_active: Some(true),
        is_archived: Some(false),
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    // All three should be listed (public + private); visibility filtering
    // depends on caller context — repository returns what matches active/archived.
    // The visibility field is stored and returned; access control is a higher-level concern.
    let names: Vec<&str> = leagues.iter().map(|l| l.name.as_str()).collect();
    assert!(names.contains(&"Public 1"), "should include Public 1");
    assert!(names.contains(&"Public 2"), "should include Public 2");
    assert!(names.contains(&"Private 1"), "should include Private 1");
}

#[tokio::test]
async fn test_list_leagues_returns_visibility_field() {
    let pool = setup_test_db().await;

    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Vis Private",
        true,
        false,
        "private",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "Vis Public",
        true,
        false,
        "public",
    )
    .await;

    let filter = LeagueFilter {
        is_active: Some(true),
        is_archived: Some(false),
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    for league in &leagues {
        assert!(
            league.visibility == "public" || league.visibility == "private",
            "visibility should be 'public' or 'private', got: '{}'",
            league.visibility
        );
    }
}

// ============================================================================
// UPDATE LEAGUE TESTS
// ============================================================================

#[tokio::test]
async fn test_update_league_changes_name() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Original Name", "elo", "public").await;

    let patch = LeaguePatch {
        name: Some("Updated Name".to_string()),
        description: None,
        visibility: None,
        is_active: None,
    };

    let updated = LeagueRepository::update_league(&pool, &league_id, &patch)
        .await
        .expect("update_league should succeed");

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.id, league_id, "ID should not change on update");
}

#[tokio::test]
async fn test_update_league_changes_description() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Desc Test", "elo", "public").await;

    let patch = LeaguePatch {
        name: None,
        description: Some("A new description".to_string()),
        visibility: None,
        is_active: None,
    };

    let updated = LeagueRepository::update_league(&pool, &league_id, &patch)
        .await
        .expect("update_league should succeed");

    assert_eq!(updated.description.as_deref(), Some("A new description"));
}

#[tokio::test]
async fn test_update_league_clears_description() {
    let pool = setup_test_db().await;

    let created = LeagueRepository::create_league(
        &pool,
        "Clear Desc",
        "will be cleared",
        "elo",
        "public",
        "system",
    )
    .await
    .expect("create should succeed");
    assert!(created.description.is_some());

    let patch = LeaguePatch {
        name: None,
        description: Some("".to_string()), // Empty description
        visibility: None,
        is_active: None,
    };

    let updated = LeagueRepository::update_league(&pool, &created.id, &patch)
        .await
        .expect("update_league should succeed");

    // Empty string description could be stored as empty string or as None
    // Either behavior is acceptable
    match updated.description {
        Some(ref desc) if desc.is_empty() => { /* stored as empty string */ }
        None => { /* converted to None */ }
        other => panic!("unexpected description value: {:?}", other),
    }
}

#[tokio::test]
async fn test_update_league_changes_visibility() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Vis Changer", "elo", "public").await;

    let patch = LeaguePatch {
        name: None,
        description: None,
        visibility: Some("private".to_string()),
        is_active: None,
    };

    let updated = LeagueRepository::update_league(&pool, &league_id, &patch)
        .await
        .expect("update_league should succeed");

    assert_eq!(updated.visibility, "private");
}

#[tokio::test]
async fn test_update_league_changes_is_active() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Active Toggle", "elo", "public").await;

    let patch = LeaguePatch {
        name: None,
        description: None,
        visibility: None,
        is_active: Some(false),
    };

    let updated = LeagueRepository::update_league(&pool, &league_id, &patch)
        .await
        .expect("update_league should succeed");

    assert!(!updated.is_active, "league should be deactivated");
}

#[tokio::test]
async fn test_update_league_changes_multiple_fields() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Multi Change", "elo", "public").await;

    let patch = LeaguePatch {
        name: Some("Multi Renamed".to_string()),
        description: Some("New desc".to_string()),
        visibility: Some("private".to_string()),
        is_active: Some(false),
    };

    let updated = LeagueRepository::update_league(&pool, &league_id, &patch)
        .await
        .expect("update_league should succeed");

    assert_eq!(updated.name, "Multi Renamed");
    assert_eq!(updated.description.as_deref(), Some("New desc"));
    assert_eq!(updated.visibility, "private");
    assert!(!updated.is_active);
}

#[tokio::test]
async fn test_update_league_empty_patch_returns_existing_values() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Noop Patch", "elo", "public").await;

    let patch = LeaguePatch {
        name: None,
        description: None,
        visibility: None,
        is_active: None,
    };

    let updated = LeagueRepository::update_league(&pool, &league_id, &patch)
        .await
        .expect("update_league with empty patch should succeed");

    assert_eq!(updated.name, "Noop Patch");
    assert_eq!(updated.visibility, "public");
    assert!(updated.is_active);
}

#[tokio::test]
async fn test_update_league_updates_updated_at_timestamp() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Timestamp Update", "elo", "public").await;

    // Small delay to ensure timestamps differ
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let before_update = Utc::now();
    let patch = LeaguePatch {
        name: Some("Timestamp Updated".to_string()),
        description: None,
        visibility: None,
        is_active: None,
    };

    let updated = LeagueRepository::update_league(&pool, &league_id, &patch)
        .await
        .expect("update_league should succeed");

    assert!(
        updated.updated_at >= before_update,
        "updated_at should reflect the update time"
    );
}

#[tokio::test]
async fn test_update_league_nonexistent_id_returns_error() {
    let pool = setup_test_db().await;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let patch = LeaguePatch {
        name: Some("Ghost League".to_string()),
        description: None,
        visibility: None,
        is_active: None,
    };

    let result = LeagueRepository::update_league(&pool, &nonexistent_id, &patch).await;

    assert!(
        result.is_err(),
        "update_league with nonexistent ID should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_update_league_empty_id_returns_error() {
    let pool = setup_test_db().await;

    let patch = LeaguePatch {
        name: Some("Never Saved".to_string()),
        description: None,
        visibility: None,
        is_active: None,
    };

    let result = LeagueRepository::update_league(&pool, "", &patch).await;

    assert!(
        result.is_err(),
        "update_league with empty ID should return error"
    );
}

// ============================================================================
// ARCHIVE LEAGUE TESTS
// ============================================================================

#[tokio::test]
async fn test_archive_league_sets_is_archived_true() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Archive Me", "elo", "public").await;

    let result = LeagueRepository::archive_league(&pool, &league_id).await;
    assert!(
        result.is_ok(),
        "archive_league should succeed: {:?}",
        result
    );

    // Verify via get_league
    let found = LeagueRepository::get_league(&pool, &league_id)
        .await
        .expect("get_league should succeed")
        .expect("league should still exist after archive");

    assert!(
        found.is_archived,
        "is_archived should be true after archive"
    );
    assert_eq!(found.id, league_id);
}

#[tokio::test]
async fn test_archive_already_archived_league_is_idempotent() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Already Archived", "elo", "public").await;

    // First archive
    LeagueRepository::archive_league(&pool, &league_id)
        .await
        .expect("first archive should succeed");

    // Second archive on already-archived league
    let result = LeagueRepository::archive_league(&pool, &league_id).await;

    assert!(
        result.is_ok(),
        "archive on already-archived league should be idempotent: {:?}",
        result
    );
}

#[tokio::test]
async fn test_archive_league_nonexistent_id_returns_error() {
    let pool = setup_test_db().await;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let result = LeagueRepository::archive_league(&pool, &nonexistent_id).await;

    assert!(
        result.is_err(),
        "archive_league with nonexistent ID should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_archive_league_empty_id_returns_error() {
    let pool = setup_test_db().await;

    let result = LeagueRepository::archive_league(&pool, "").await;

    assert!(
        result.is_err(),
        "archive_league with empty ID should return error"
    );
}

// ============================================================================
// UNARCHIVE LEAGUE TESTS
// ============================================================================

#[tokio::test]
async fn test_unarchive_league_sets_is_archived_false() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Unarchive Me", "elo", "public").await;

    // First archive it
    LeagueRepository::archive_league(&pool, &league_id)
        .await
        .expect("archive should succeed");

    // Then unarchive
    let result = LeagueRepository::unarchive_league(&pool, &league_id).await;
    assert!(
        result.is_ok(),
        "unarchive_league should succeed: {:?}",
        result
    );

    // Verify via get_league
    let found = LeagueRepository::get_league(&pool, &league_id)
        .await
        .expect("get_league should succeed")
        .expect("league should still exist after unarchive");

    assert!(
        !found.is_archived,
        "is_archived should be false after unarchive"
    );
}

#[tokio::test]
async fn test_unarchive_already_unarchived_league_is_idempotent() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Already Unarchived", "elo", "public").await;

    // League starts unarchived; unarchive should be idempotent
    let result = LeagueRepository::unarchive_league(&pool, &league_id).await;

    assert!(
        result.is_ok(),
        "unarchive on already-unarchived league should be idempotent: {:?}",
        result
    );
}

#[tokio::test]
async fn test_archive_unarchive_roundtrip() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Roundtrip League", "elo", "public").await;

    // Verify starts unarchived
    let initial = LeagueRepository::get_league(&pool, &league_id)
        .await
        .expect("get should succeed")
        .expect("league should exist");
    assert!(!initial.is_archived, "should start unarchived");

    // Archive
    LeagueRepository::archive_league(&pool, &league_id)
        .await
        .expect("archive should succeed");

    let archived = LeagueRepository::get_league(&pool, &league_id)
        .await
        .expect("get should succeed")
        .expect("league should exist");
    assert!(archived.is_archived, "should be archived");

    // Unarchive
    LeagueRepository::unarchive_league(&pool, &league_id)
        .await
        .expect("unarchive should succeed");

    let unarchived = LeagueRepository::get_league(&pool, &league_id)
        .await
        .expect("get should succeed")
        .expect("league should exist");
    assert!(!unarchived.is_archived, "should be unarchived again");
}

#[tokio::test]
async fn test_unarchive_league_nonexistent_id_returns_error() {
    let pool = setup_test_db().await;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let result = LeagueRepository::unarchive_league(&pool, &nonexistent_id).await;

    assert!(
        result.is_err(),
        "unarchive_league with nonexistent ID should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_unarchive_league_empty_id_returns_error() {
    let pool = setup_test_db().await;

    let result = LeagueRepository::unarchive_league(&pool, "").await;

    assert!(
        result.is_err(),
        "unarchive_league with empty ID should return error"
    );
}

// ============================================================================
// ASSIGN OPERATOR TESTS
// ============================================================================

#[tokio::test]
async fn test_assign_operator_adds_operator() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Operator League", "elo", "public").await;
    let user_id = seed_user(&pool, "operator_user").await;
    let granted_by = seed_user(&pool, "admin_user").await;

    let result = LeagueRepository::assign_operator(&pool, &league_id, &user_id, &granted_by).await;

    assert!(
        result.is_ok(),
        "assign_operator should succeed: {:?}",
        result
    );

    // Verify via get_operators
    let operators = LeagueRepository::get_operators(&pool, &league_id)
        .await
        .expect("get_operators should succeed");

    assert_eq!(operators.len(), 1, "should have one operator");
    assert_eq!(operators[0].league_id, league_id);
    assert_eq!(operators[0].user_id, user_id);
    assert_eq!(operators[0].granted_by, granted_by);
}

#[tokio::test]
async fn test_assign_operator_multiple_operators() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Multi Op League", "elo", "public").await;
    let user1 = seed_user(&pool, "op_user_1").await;
    let user2 = seed_user(&pool, "op_user_2").await;
    let user3 = seed_user(&pool, "op_user_3").await;
    let granted_by = seed_user(&pool, "granting_admin").await;

    LeagueRepository::assign_operator(&pool, &league_id, &user1, &granted_by)
        .await
        .expect("assign user1 should succeed");

    LeagueRepository::assign_operator(&pool, &league_id, &user2, &granted_by)
        .await
        .expect("assign user2 should succeed");

    LeagueRepository::assign_operator(&pool, &league_id, &user3, &granted_by)
        .await
        .expect("assign user3 should succeed");

    let operators = LeagueRepository::get_operators(&pool, &league_id)
        .await
        .expect("get_operators should succeed");

    assert_eq!(operators.len(), 3, "should have three operators");

    let user_ids: Vec<&str> = operators.iter().map(|o| o.user_id.as_str()).collect();
    assert!(user_ids.contains(&user1.as_str()));
    assert!(user_ids.contains(&user2.as_str()));
    assert!(user_ids.contains(&user3.as_str()));
}

#[tokio::test]
async fn test_assign_operator_duplicate_behavior() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Dup Op League", "elo", "public").await;
    let user_id = seed_user(&pool, "dup_op_user").await;
    let granted_by = seed_user(&pool, "dup_granter").await;

    // First assignment
    let first = LeagueRepository::assign_operator(&pool, &league_id, &user_id, &granted_by).await;
    assert!(first.is_ok(), "first assign should succeed");

    // Second assignment — should either be idempotent (Ok) or return Conflict error
    let second = LeagueRepository::assign_operator(&pool, &league_id, &user_id, &granted_by).await;

    match second {
        Ok(()) => { /* idempotent — acceptable */ }
        Err(PersistenceError::Conflict(_)) => { /* explicit conflict — also acceptable */ }
        Err(e) => {
            panic!("unexpected error for duplicate assign_operator: {:?}", e)
        }
    }
}

#[tokio::test]
async fn test_assign_operator_same_user_to_multiple_leagues() {
    let pool = setup_test_db().await;

    let league_a = seed_league(&pool, "Multi League A", "elo", "public").await;
    let league_b = seed_league(&pool, "Multi League B", "elo", "public").await;
    let user_id = seed_user(&pool, "cross_league_op").await;
    let granted_by = seed_user(&pool, "cross_granter").await;

    LeagueRepository::assign_operator(&pool, &league_a, &user_id, &granted_by)
        .await
        .expect("assign to league A should succeed");

    LeagueRepository::assign_operator(&pool, &league_b, &user_id, &granted_by)
        .await
        .expect("assign to league B should succeed");

    // Verify same user is operator of both leagues
    let ops_a = LeagueRepository::get_operators(&pool, &league_a)
        .await
        .expect("get_operators A should succeed");
    let ops_b = LeagueRepository::get_operators(&pool, &league_b)
        .await
        .expect("get_operators B should succeed");

    assert_eq!(ops_a.len(), 1);
    assert_eq!(ops_b.len(), 1);
    assert_eq!(ops_a[0].user_id, user_id);
    assert_eq!(ops_b[0].user_id, user_id);
}

#[tokio::test]
async fn test_assign_operator_nonexistent_league_returns_error() {
    let pool = setup_test_db().await;

    let user_id = seed_user(&pool, "orphan_op").await;
    let granted_by = seed_user(&pool, "orphan_granter").await;
    let nonexistent_league = uuid::Uuid::new_v4().to_string();

    let result =
        LeagueRepository::assign_operator(&pool, &nonexistent_league, &user_id, &granted_by).await;

    assert!(
        result.is_err(),
        "assign_operator with nonexistent league should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_assign_operator_nonexistent_user_returns_error() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Bad User League", "elo", "public").await;
    let granted_by = seed_user(&pool, "bad_user_granter").await;
    let nonexistent_user = uuid::Uuid::new_v4().to_string();

    let result =
        LeagueRepository::assign_operator(&pool, &league_id, &nonexistent_user, &granted_by).await;

    assert!(
        result.is_err(),
        "assign_operator with nonexistent user should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_assign_operator_nonexistent_granted_by_returns_error() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Bad Granter League", "elo", "public").await;
    let user_id = seed_user(&pool, "bad_granter_user").await;
    let nonexistent_granter = uuid::Uuid::new_v4().to_string();

    let result =
        LeagueRepository::assign_operator(&pool, &league_id, &user_id, &nonexistent_granter).await;

    assert!(
        result.is_err(),
        "assign_operator with nonexistent granted_by should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_assign_operator_empty_league_id_returns_error() {
    let pool = setup_test_db().await;

    let user_id = seed_user(&pool, "empty_league_op").await;
    let granted_by = seed_user(&pool, "empty_league_granter").await;

    let result = LeagueRepository::assign_operator(&pool, "", &user_id, &granted_by).await;

    assert!(
        result.is_err(),
        "assign_operator with empty league_id should return error"
    );
}

#[tokio::test]
async fn test_assign_operator_empty_user_id_returns_error() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Empty User League", "elo", "public").await;
    let granted_by = seed_user(&pool, "empty_user_granter").await;

    let result = LeagueRepository::assign_operator(&pool, &league_id, "", &granted_by).await;

    assert!(
        result.is_err(),
        "assign_operator with empty user_id should return error"
    );
}

// ============================================================================
// REMOVE OPERATOR TESTS
// ============================================================================

#[tokio::test]
async fn test_remove_operator_removes_operator() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Remove Op League", "elo", "public").await;
    let user_id = seed_user(&pool, "remove_op_user").await;
    let granted_by = seed_user(&pool, "remove_granter").await;

    // Assign first
    LeagueRepository::assign_operator(&pool, &league_id, &user_id, &granted_by)
        .await
        .expect("assign should succeed");

    // Verify assigned
    let before = LeagueRepository::get_operators(&pool, &league_id)
        .await
        .expect("get_operators should succeed");
    assert_eq!(before.len(), 1);

    // Remove
    let result = LeagueRepository::remove_operator(&pool, &league_id, &user_id).await;
    assert!(
        result.is_ok(),
        "remove_operator should succeed: {:?}",
        result
    );

    // Verify removed
    let after = LeagueRepository::get_operators(&pool, &league_id)
        .await
        .expect("get_operators should succeed");
    assert!(
        after.is_empty(),
        "operator list should be empty after removal"
    );
}

#[tokio::test]
async fn test_remove_operator_not_assigned_is_idempotent() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "No Op League", "elo", "public").await;
    let user_id = seed_user(&pool, "never_assigned").await;

    let result = LeagueRepository::remove_operator(&pool, &league_id, &user_id).await;

    match result {
        Ok(()) => { /* idempotent — acceptable */ }
        Err(PersistenceError::NotFound { .. }) => { /* explicit not-found — also acceptable */ }
        Err(e) => {
            panic!(
                "unexpected error for remove_operator on non-operator: {:?}",
                e
            )
        }
    }
}

#[tokio::test]
async fn test_remove_operator_nonexistent_league_returns_error() {
    let pool = setup_test_db().await;

    let user_id = seed_user(&pool, "orphan_remove").await;
    let nonexistent_league = uuid::Uuid::new_v4().to_string();

    let result = LeagueRepository::remove_operator(&pool, &nonexistent_league, &user_id).await;

    assert!(
        result.is_err(),
        "remove_operator with nonexistent league should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_remove_operator_nonexistent_user_returns_error() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Bad Remove League", "elo", "public").await;
    let nonexistent_user = uuid::Uuid::new_v4().to_string();

    let result = LeagueRepository::remove_operator(&pool, &league_id, &nonexistent_user).await;

    assert!(
        result.is_err(),
        "remove_operator with nonexistent user should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_remove_operator_empty_league_id_returns_error() {
    let pool = setup_test_db().await;

    let user_id = seed_user(&pool, "empty_league_rem").await;

    let result = LeagueRepository::remove_operator(&pool, "", &user_id).await;

    assert!(
        result.is_err(),
        "remove_operator with empty league_id should return error"
    );
}

#[tokio::test]
async fn test_remove_operator_empty_user_id_returns_error() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Empty Rem User", "elo", "public").await;

    let result = LeagueRepository::remove_operator(&pool, &league_id, "").await;

    assert!(
        result.is_err(),
        "remove_operator with empty user_id should return error"
    );
}

// ============================================================================
// GET OPERATORS TESTS
// ============================================================================

#[tokio::test]
async fn test_get_operators_returns_all_operators() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "All Ops League", "elo", "public").await;
    let user1 = seed_user(&pool, "all_ops_1").await;
    let user2 = seed_user(&pool, "all_ops_2").await;
    let granted_by = seed_user(&pool, "all_ops_admin").await;

    LeagueRepository::assign_operator(&pool, &league_id, &user1, &granted_by)
        .await
        .expect("assign user1");
    LeagueRepository::assign_operator(&pool, &league_id, &user2, &granted_by)
        .await
        .expect("assign user2");

    let operators = LeagueRepository::get_operators(&pool, &league_id)
        .await
        .expect("get_operators should succeed");

    assert_eq!(operators.len(), 2);
    // Verify struct fields
    for op in &operators {
        assert_eq!(op.league_id, league_id);
        assert!(!op.user_id.is_empty());
        assert_eq!(op.granted_by, granted_by);
        assert!(
            op.granted_at <= Utc::now(),
            "granted_at should be in the past"
        );
    }
}

#[tokio::test]
async fn test_get_operators_empty_league_returns_empty_vec() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Empty Ops", "elo", "public").await;

    // No operators assigned
    let operators = LeagueRepository::get_operators(&pool, &league_id)
        .await
        .expect("get_operators should succeed");

    assert!(
        operators.is_empty(),
        "empty league should return empty operator list"
    );
}

#[tokio::test]
async fn test_get_operators_nonexistent_league_returns_empty_or_error() {
    let pool = setup_test_db().await;

    let nonexistent_league = uuid::Uuid::new_v4().to_string();

    let result = LeagueRepository::get_operators(&pool, &nonexistent_league).await;

    match result {
        Ok(operators) => assert!(
            operators.is_empty(),
            "nonexistent league should return empty operator list"
        ),
        Err(_) => { /* error is also acceptable */ }
    }
}

#[tokio::test]
async fn test_get_operators_empty_league_id_returns_empty_or_error() {
    let pool = setup_test_db().await;

    let result = LeagueRepository::get_operators(&pool, "").await;

    match result {
        Ok(operators) => assert!(
            operators.is_empty(),
            "empty league_id should return empty operator list"
        ),
        Err(_) => { /* error is also acceptable */ }
    }
}

// ============================================================================
// IS OPERATOR TESTS
// ============================================================================

#[tokio::test]
async fn test_is_operator_returns_true_for_assigned_operator() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Is Op League", "elo", "public").await;
    let user_id = seed_user(&pool, "is_op_user").await;
    let granted_by = seed_user(&pool, "is_op_admin").await;

    LeagueRepository::assign_operator(&pool, &league_id, &user_id, &granted_by)
        .await
        .expect("assign should succeed");

    let result = LeagueRepository::is_operator(&pool, &league_id, &user_id).await;

    assert!(result.is_ok(), "is_operator should succeed: {:?}", result);
    assert!(
        result.unwrap(),
        "is_operator should return true for assigned operator"
    );
}

#[tokio::test]
async fn test_is_operator_returns_false_for_not_assigned_user() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Not Op League", "elo", "public").await;
    let assigned_user = seed_user(&pool, "assigned_user").await;
    let not_assigned_user = seed_user(&pool, "not_assigned_user").await;
    let granted_by = seed_user(&pool, "is_not_op_admin").await;

    // Assign only one user
    LeagueRepository::assign_operator(&pool, &league_id, &assigned_user, &granted_by)
        .await
        .expect("assign should succeed");

    // Check the other user
    let result = LeagueRepository::is_operator(&pool, &league_id, &not_assigned_user).await;

    assert!(result.is_ok(), "is_operator should succeed: {:?}", result);
    assert!(
        !result.unwrap(),
        "is_operator should return false for non-operator user"
    );
}

#[tokio::test]
async fn test_is_operator_returns_false_after_removal() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Was Op League", "elo", "public").await;
    let user_id = seed_user(&pool, "was_op_user").await;
    let granted_by = seed_user(&pool, "was_op_admin").await;

    // Assign
    LeagueRepository::assign_operator(&pool, &league_id, &user_id, &granted_by)
        .await
        .expect("assign should succeed");

    // Verify true before removal
    let before = LeagueRepository::is_operator(&pool, &league_id, &user_id)
        .await
        .expect("is_operator before removal should succeed");
    assert!(before, "should be operator before removal");

    // Remove
    LeagueRepository::remove_operator(&pool, &league_id, &user_id)
        .await
        .expect("remove should succeed");

    // Verify false after removal
    let after = LeagueRepository::is_operator(&pool, &league_id, &user_id)
        .await
        .expect("is_operator after removal should succeed");
    assert!(!after, "should not be operator after removal");
}

#[tokio::test]
async fn test_is_operator_returns_false_for_nonexistent_league() {
    let pool = setup_test_db().await;

    let user_id = seed_user(&pool, "ghost_league_user").await;
    let nonexistent_league = uuid::Uuid::new_v4().to_string();

    let result = LeagueRepository::is_operator(&pool, &nonexistent_league, &user_id).await;

    assert!(
        result.is_ok(),
        "is_operator for nonexistent league should succeed: {:?}",
        result
    );
    assert!(
        !result.unwrap(),
        "is_operator should return false for nonexistent league"
    );
}

#[tokio::test]
async fn test_is_operator_returns_false_for_nonexistent_user() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Ghost User League", "elo", "public").await;
    let nonexistent_user = uuid::Uuid::new_v4().to_string();

    let result = LeagueRepository::is_operator(&pool, &league_id, &nonexistent_user).await;

    assert!(
        result.is_ok(),
        "is_operator for nonexistent user should succeed: {:?}",
        result
    );
    assert!(
        !result.unwrap(),
        "is_operator should return false for nonexistent user"
    );
}

#[tokio::test]
async fn test_is_operator_empty_league_id_returns_false_or_error() {
    let pool = setup_test_db().await;

    let user_id = seed_user(&pool, "empty_league_is_op").await;

    let result = LeagueRepository::is_operator(&pool, "", &user_id).await;

    match result {
        Ok(is_op) => assert!(!is_op, "empty league_id should return false"),
        Err(_) => { /* error is also acceptable */ }
    }
}

#[tokio::test]
async fn test_is_operator_empty_user_id_returns_false_or_error() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Empty User Check", "elo", "public").await;

    let result = LeagueRepository::is_operator(&pool, &league_id, "").await;

    match result {
        Ok(is_op) => assert!(!is_op, "empty user_id should return false"),
        Err(_) => { /* error is also acceptable */ }
    }
}

// ============================================================================
// COMPREHENSIVE LIFECYCLE TESTS
// ============================================================================

#[tokio::test]
async fn test_league_full_lifecycle() {
    let pool = setup_test_db().await;

    // 1. CREATE
    let league = LeagueRepository::create_league(
        &pool,
        "Full Lifecycle League",
        "Testing the full lifecycle",
        "glicko",
        "public",
        "system",
    )
    .await
    .expect("create should succeed");
    assert_eq!(league.name, "Full Lifecycle League");
    assert!(league.is_active);
    assert!(!league.is_archived);

    // 2. READ
    let found = LeagueRepository::get_league(&pool, &league.id)
        .await
        .expect("get should succeed")
        .expect("league should be found");
    assert_eq!(found.id, league.id);
    assert_eq!(found.algorithm, "glicko");

    // 3. UPDATE
    let patch = LeaguePatch {
        name: Some("Lifecycle Renamed".to_string()),
        description: Some("Updated during lifecycle test".to_string()),
        visibility: Some("private".to_string()),
        is_active: None,
    };
    let updated = LeagueRepository::update_league(&pool, &league.id, &patch)
        .await
        .expect("update should succeed");
    assert_eq!(updated.name, "Lifecycle Renamed");
    assert_eq!(
        updated.description.as_deref(),
        Some("Updated during lifecycle test")
    );
    assert_eq!(updated.visibility, "private");

    // 4. ASSIGN OPERATOR
    let user_id = seed_user(&pool, "lifecycle_op").await;
    let admin_id = seed_user(&pool, "lifecycle_admin").await;

    LeagueRepository::assign_operator(&pool, &league.id, &user_id, &admin_id)
        .await
        .expect("assign operator should succeed");

    let is_op = LeagueRepository::is_operator(&pool, &league.id, &user_id)
        .await
        .expect("is_operator should succeed");
    assert!(is_op, "user should be operator");

    // 5. ARCHIVE
    LeagueRepository::archive_league(&pool, &league.id)
        .await
        .expect("archive should succeed");

    let archived = LeagueRepository::get_league(&pool, &league.id)
        .await
        .expect("get should succeed")
        .expect("league should exist");
    assert!(archived.is_archived);
    assert!(archived.is_active, "is_active unchanged by archive");

    // 6. UNARCHIVE
    LeagueRepository::unarchive_league(&pool, &league.id)
        .await
        .expect("unarchive should succeed");

    let unarchived = LeagueRepository::get_league(&pool, &league.id)
        .await
        .expect("get should succeed")
        .expect("league should exist");
    assert!(!unarchived.is_archived);

    // 7. REMOVE OPERATOR
    LeagueRepository::remove_operator(&pool, &league.id, &user_id)
        .await
        .expect("remove operator should succeed");

    let is_op_after = LeagueRepository::is_operator(&pool, &league.id, &user_id)
        .await
        .expect("is_operator should succeed");
    assert!(!is_op_after, "user should no longer be operator");
}

#[tokio::test]
async fn test_create_update_get_verifies_all_fields_roundtrip() {
    let pool = setup_test_db().await;

    // Create with all fields populated
    let created = LeagueRepository::create_league(
        &pool,
        "Roundtrip Field Test",
        "Full description for roundtrip",
        "trueskill",
        "private",
        "admin_user",
    )
    .await
    .expect("create should succeed");

    // Update to change everything
    let patch = LeaguePatch {
        name: Some("Roundtrip Updated".to_string()),
        description: Some("Modified description".to_string()),
        visibility: Some("public".to_string()),
        is_active: Some(false),
    };

    let updated = LeagueRepository::update_league(&pool, &created.id, &patch)
        .await
        .expect("update should succeed");

    // Verify update return value matches expected changes
    assert_eq!(updated.name, "Roundtrip Updated");
    assert_eq!(updated.visibility, "public");
    assert!(!updated.is_active);

    // Read back and verify roundtrip
    let found = LeagueRepository::get_league(&pool, &created.id)
        .await
        .expect("get should succeed")
        .expect("league should exist");

    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Roundtrip Updated");
    assert_eq!(found.description.as_deref(), Some("Modified description"));
    assert_eq!(found.algorithm, "trueskill", "algorithm should not change");
    assert_eq!(found.visibility, "public");
    assert!(!found.is_active);
    assert!(!found.is_archived);

    // Verify timestamps: updated_at should be >= created_at
    assert!(
        found.updated_at >= found.created_at,
        "updated_at should be >= created_at after update"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_create_league_with_special_characters_in_name() {
    let pool = setup_test_db().await;

    let result = LeagueRepository::create_league(
        &pool,
        "League with spéciäl chars! @#$%^&*()",
        "Special chars test",
        "elo",
        "public",
        "system",
    )
    .await;

    assert!(
        result.is_ok(),
        "create_league with special characters should succeed: {:?}",
        result
    );

    let league = result.unwrap();
    assert_eq!(league.name, "League with spéciäl chars! @#$%^&*()");
}

#[tokio::test]
async fn test_create_league_with_sql_injection_attempt_in_name() {
    let pool = setup_test_db().await;

    let result = LeagueRepository::create_league(
        &pool,
        "League'; DROP TABLE leagues; --",
        "SQL injection test",
        "elo",
        "public",
        "system",
    )
    .await;

    // Should succeed with parameterized queries and not drop the table
    assert!(
        result.is_ok(),
        "create_league with SQL injection attempt should succeed (parameterized): {:?}",
        result
    );

    // Verify the table still exists by querying
    let league = result.unwrap();
    let found = LeagueRepository::get_league(&pool, &league.id)
        .await
        .expect("get should succeed");
    assert!(
        found.is_some(),
        "table should still exist and contain the league"
    );
}

#[tokio::test]
async fn test_get_league_with_very_long_id() {
    let pool = setup_test_db().await;

    let long_id = "x".repeat(10_000);

    let result = LeagueRepository::get_league(&pool, &long_id).await;

    match result {
        Ok(league_opt) => assert!(league_opt.is_none(), "very long ID should return None"),
        Err(_) => { /* error is also acceptable for excessively long IDs */ }
    }
}

#[tokio::test]
async fn test_archive_league_does_not_affect_other_fields() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Preserve Fields", "glicko", "private").await;

    // Get initial state
    let before = LeagueRepository::get_league(&pool, &league_id)
        .await
        .expect("get should succeed")
        .expect("league should exist");

    // Archive
    LeagueRepository::archive_league(&pool, &league_id)
        .await
        .expect("archive should succeed");

    // Get after archive
    let after = LeagueRepository::get_league(&pool, &league_id)
        .await
        .expect("get should succeed")
        .expect("league should exist");

    // Only is_archived should change
    assert_eq!(after.name, before.name);
    assert_eq!(after.description, before.description);
    assert_eq!(after.algorithm, before.algorithm);
    assert_eq!(after.visibility, before.visibility);
    assert_eq!(after.is_active, before.is_active);
    assert!(after.is_archived, "is_archived should be true");
}

#[tokio::test]
async fn test_update_league_to_same_name_is_noop_or_error() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Same Name", "elo", "public").await;

    let patch = LeaguePatch {
        name: Some("Same Name".to_string()),
        description: None,
        visibility: None,
        is_active: None,
    };

    let result = LeagueRepository::update_league(&pool, &league_id, &patch).await;

    // Setting the same name should either succeed (noop) or return a validation error
    match result {
        Ok(updated) => {
            assert_eq!(updated.name, "Same Name");
        }
        Err(_) => {
            // Error for no-change update is also acceptable
        }
    }
}

#[tokio::test]
async fn test_list_leagues_with_no_filters_returns_all() {
    let pool = setup_test_db().await;

    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "All Filter A",
        true,
        false,
        "public",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "All Filter B",
        false,
        true,
        "private",
    )
    .await;
    seed_league_with_status(
        &pool,
        &uuid::Uuid::new_v4().to_string(),
        "All Filter C",
        true,
        true,
        "public",
    )
    .await;

    let filter = LeagueFilter {
        is_active: None,
        is_archived: None,
        limit: None,
        offset: None,
    };

    let leagues = LeagueRepository::list_leagues(&pool, &filter)
        .await
        .expect("list_leagues should succeed");

    assert_eq!(leagues.len(), 3, "no filters should return all leagues");
}

#[tokio::test]
async fn test_many_leagues_pagination_boundaries() {
    let pool = setup_test_db().await;

    // Create 25 leagues
    for i in 0..25 {
        seed_league_with_status(
            &pool,
            &uuid::Uuid::new_v4().to_string(),
            &format!("Page League {:02}", i),
            true,
            false,
            "public",
        )
        .await;
    }

    // Page 1: limit=10, offset=0 → 10 results
    let page1 = LeagueRepository::list_leagues(
        &pool,
        &LeagueFilter {
            is_active: None,
            is_archived: None,
            limit: Some(10),
            offset: Some(0),
        },
    )
    .await
    .expect("page 1 should succeed");
    assert_eq!(page1.len(), 10);

    // Page 2: limit=10, offset=10 → 10 results
    let page2 = LeagueRepository::list_leagues(
        &pool,
        &LeagueFilter {
            is_active: None,
            is_archived: None,
            limit: Some(10),
            offset: Some(10),
        },
    )
    .await
    .expect("page 2 should succeed");
    assert_eq!(page2.len(), 10);

    // Page 3: limit=10, offset=20 → 5 results (last page)
    let page3 = LeagueRepository::list_leagues(
        &pool,
        &LeagueFilter {
            is_active: None,
            is_archived: None,
            limit: Some(10),
            offset: Some(20),
        },
    )
    .await
    .expect("page 3 should succeed");
    assert_eq!(page3.len(), 5, "last page should have remaining 5 items");

    // Page past end: limit=10, offset=30 → 0 results
    let page4 = LeagueRepository::list_leagues(
        &pool,
        &LeagueFilter {
            is_active: None,
            is_archived: None,
            limit: Some(10),
            offset: Some(30),
        },
    )
    .await
    .expect("page past end should succeed");
    assert!(page4.is_empty(), "page past end should return empty");
}

#[tokio::test]
async fn test_operators_are_league_specific() {
    let pool = setup_test_db().await;

    let league_a = seed_league(&pool, "League A", "elo", "public").await;
    let league_b = seed_league(&pool, "League B", "elo", "public").await;
    let user = seed_user(&pool, "shared_user").await;
    let admin = seed_user(&pool, "shared_admin").await;

    // Assign user to league A only
    LeagueRepository::assign_operator(&pool, &league_a, &user, &admin)
        .await
        .expect("assign to A should succeed");

    // User should be operator of A but not B
    let is_op_a = LeagueRepository::is_operator(&pool, &league_a, &user)
        .await
        .expect("is_operator A should succeed");
    let is_op_b = LeagueRepository::is_operator(&pool, &league_b, &user)
        .await
        .expect("is_operator B should succeed");

    assert!(is_op_a, "user should be operator of league A");
    assert!(!is_op_b, "user should NOT be operator of league B");

    // get_operators should return user only for league A
    let ops_a = LeagueRepository::get_operators(&pool, &league_a)
        .await
        .expect("get_operators A should succeed");
    let ops_b = LeagueRepository::get_operators(&pool, &league_b)
        .await
        .expect("get_operators B should succeed");

    assert_eq!(ops_a.len(), 1);
    assert_eq!(ops_a[0].user_id, user);
    assert!(ops_b.is_empty());
}

#[tokio::test]
async fn test_get_operators_has_granted_at_timestamp() {
    let pool = setup_test_db().await;

    let league_id = seed_league(&pool, "Granted At League", "elo", "public").await;
    let user_id = seed_user(&pool, "granted_at_user").await;
    let granted_by = seed_user(&pool, "granted_at_admin").await;

    let before_assign = Utc::now();

    LeagueRepository::assign_operator(&pool, &league_id, &user_id, &granted_by)
        .await
        .expect("assign should succeed");

    let operators = LeagueRepository::get_operators(&pool, &league_id)
        .await
        .expect("get_operators should succeed");

    assert_eq!(operators.len(), 1);

    let after_assign = Utc::now();

    assert!(
        operators[0].granted_at >= before_assign,
        "granted_at should be >= assignment start time"
    );
    assert!(
        operators[0].granted_at <= after_assign,
        "granted_at should be <= assignment end time"
    );
}

// ============================================================================
// STRUCT DEFAULTS AND SERIALIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_league_filter_default_values() {
    let filter = LeagueFilter::default();

    assert_eq!(filter.is_active, Some(true));
    assert_eq!(filter.is_archived, Some(false));
    assert_eq!(filter.limit, Some(20));
    assert_eq!(filter.offset, Some(0));
}

#[test]
fn test_league_patch_all_none_is_default() {
    let patch = LeaguePatch {
        name: None,
        description: None,
        visibility: None,
        is_active: None,
    };

    assert!(patch.name.is_none());
    assert!(patch.description.is_none());
    assert!(patch.visibility.is_none());
    assert!(patch.is_active.is_none());
}

#[test]
fn test_league_filter_can_be_fully_specified() {
    let filter = LeagueFilter {
        is_active: Some(false),
        is_archived: Some(true),
        limit: Some(50),
        offset: Some(10),
    };

    assert_eq!(filter.is_active, Some(false));
    assert_eq!(filter.is_archived, Some(true));
    assert_eq!(filter.limit, Some(50));
    assert_eq!(filter.offset, Some(10));
}
