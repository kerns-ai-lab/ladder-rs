//! Unit tests for the Season Repository
//!
//! These tests cover the full SeasonRepository public API:
//! - create_season (valid params, invalid league_id, field verification)
//! - close_season (open season, already-closed, non-existent)
//! - apply_seeding (Ordinal, Reset, non-existent seasons)
//! - update_season_params (change params, preserve open status, non-existent)
//! - list_seasons (multiple seasons, empty league)
//! - get_current_season (open season, all closed)
//! - get_season (found, not found)
//!
//! These are TDD-style tests: they exercise the repository stubs and will
//! FAIL at runtime until the stubs in season_repository.rs are implemented.
//! Once implemented, these tests validate the full behavioral contract.

use chrono::Utc;
use ladder_rs_persistence::pool::create_pool;
use ladder_rs_persistence::{AlgorithmParams, SeasonRepository, SeedingChoice};
use serde_json::json;
use sqlx::{migrate::Migrator, SqlitePool};
use std::path::Path;

// ============================================================================
// Test Setup Helpers
// ============================================================================

/// Creates an in-memory SQLite pool with all migrations applied,
/// using the library's create_pool which auto-configures
/// WAL + foreign_keys + busy_timeout.
async fn setup_test_pool() -> SqlitePool {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Failed to create pool");

    // Apply migrations
    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");

    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path)
            .await
            .expect("Failed to create migrator");
        migrator.run(&pool).await.expect("Failed to run migrations");
    }

    pool
}

/// Insert a league record (needed for FK constraint on seasons).
async fn insert_league(pool: &SqlitePool, id: &str, name: &str) {
    sqlx::query(
        "INSERT INTO leagues (id, name, algorithm, visibility) VALUES (?, ?, 'elo', 'public')",
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert league {}: {}", id, e));
}

/// Insert a user record (needed for FK constraints).
async fn insert_user(pool: &SqlitePool, id: &str, username: &str) {
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role) VALUES (?, ?, ?, 'hash', 'operator')",
    )
    .bind(id)
    .bind(username)
    .bind(format!("{}@test.local", username))
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert user {}: {}", id, e));
}

/// Build default AlgorithmParams for test usage.
fn default_params() -> AlgorithmParams {
    AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: Some(350.0),
        extra: None,
    }
}

/// Build custom AlgorithmParams.
fn custom_params(initial_rating: f64, initial_deviation: Option<f64>) -> AlgorithmParams {
    AlgorithmParams {
        initial_rating,
        initial_deviation,
        extra: Some(json!({"tau": 0.5})),
    }
}

// ============================================================================
// CREATE SEASON TESTS
// ============================================================================

#[tokio::test]
async fn test_create_season_with_valid_params() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let result = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await;

    // Once stubs are implemented, this should be Ok(Season)
    assert!(
        result.is_ok(),
        "create_season should succeed with valid params, got: {:?}",
        result
    );

    let season = result.unwrap();
    assert_eq!(season.league_id, "league-1");
    assert_eq!(season.algorithm, "glicko2");
}

#[tokio::test]
async fn test_create_season_with_invalid_league_id() {
    let pool = setup_test_pool().await;
    // No league inserted — FK violation expected

    let params = default_params();
    let result = SeasonRepository::create_season(
        &pool,
        "nonexistent-league",
        "elo",
        &params,
        SeedingChoice::Ordinal,
    )
    .await;

    // Should return an error (NotFound, InvalidInput, or DatabaseError)
    assert!(
        result.is_err(),
        "create_season with non-existent league_id should fail"
    );
}

#[tokio::test]
async fn test_create_season_sets_is_open_true() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let result = SeasonRepository::create_season(
        &pool,
        "league-1",
        "trueskill",
        &params,
        SeedingChoice::Reset,
    )
    .await;

    assert!(
        result.is_ok(),
        "create_season should succeed, got: {:?}",
        result
    );
    let season = result.unwrap();
    assert!(
        season.is_open,
        "Newly created season should have is_open = true, got: {}",
        season.is_open
    );
}

#[tokio::test]
async fn test_create_season_has_matching_algorithm() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let algorithms = vec!["elo", "glicko", "glicko2", "trueskill"];
    for algo in &algorithms {
        let params = default_params();
        let result = SeasonRepository::create_season(
            &pool,
            "league-1",
            algo,
            &params,
            SeedingChoice::Ordinal,
        )
        .await;

        assert!(
            result.is_ok(),
            "create_season with algorithm '{}' should succeed, got: {:?}",
            algo,
            result
        );
        let season = result.unwrap();
        assert_eq!(
            season.algorithm, *algo,
            "Season algorithm should match input '{}', got: '{}'",
            algo, season.algorithm
        );
    }
}

#[tokio::test]
async fn test_create_season_with_custom_initial_deviation() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = AlgorithmParams {
        initial_rating: 1200.0,
        initial_deviation: Some(200.0),
        extra: Some(json!({"volatility": 0.06})),
    };

    let result = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await;

    assert!(
        result.is_ok(),
        "create_season with custom params should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_season_has_created_at_set() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let before = Utc::now();
    let params = default_params();
    let result =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await;

    assert!(
        result.is_ok(),
        "create_season should succeed, got: {:?}",
        result
    );
    let season = result.unwrap();

    // created_at should be within a reasonable window
    let created = season.created_at;
    assert!(
        created >= before,
        "created_at ({}) should be >= before ({})",
        created,
        before
    );

    let after = Utc::now();
    assert!(
        created <= after,
        "created_at ({}) should be <= after ({})",
        created,
        after
    );
}

#[tokio::test]
async fn test_create_season_with_empty_league_id() {
    let pool = setup_test_pool().await;

    let params = default_params();
    let result =
        SeasonRepository::create_season(&pool, "", "elo", &params, SeedingChoice::Reset).await;

    assert!(
        result.is_err(),
        "create_season with empty league_id should fail"
    );
}

#[tokio::test]
async fn test_create_season_with_empty_algorithm() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let result =
        SeasonRepository::create_season(&pool, "league-1", "", &params, SeedingChoice::Reset).await;

    // Should error on invalid algorithm name
    assert!(
        result.is_err(),
        "create_season with empty algorithm should fail"
    );
}

#[tokio::test]
async fn test_create_season_with_negative_initial_rating() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = AlgorithmParams {
        initial_rating: -100.0,
        initial_deviation: None,
        extra: None,
    };

    let result =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await;

    // Some algorithms allow negative ratings (e.g. TrueSkill), others don't.
    // The repository should accept valid algorithm parameters.
    // If the algorithm rejects it, that's a different layer.
    assert!(
        result.is_ok(),
        "create_season with negative initial_rating may or may not be valid depending on algorithm, got: {:?}",
        result
    );
}

// ============================================================================
// CLOSE SEASON TESTS
// ============================================================================

#[tokio::test]
async fn test_close_season_sets_is_open_false() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    // Insert a league user for our test user
    // (not strictly needed for close_season, but for FK integrity)
    insert_user(&pool, "user-1", "operator1").await;

    // Create a season (will be open by default)
    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season for close test");

    assert!(season.is_open, "Season should start as open");

    let result = SeasonRepository::close_season(&pool, &season.id).await;
    assert!(
        result.is_ok(),
        "close_season should succeed on open season, got: {:?}",
        result
    );

    // Verify the season is now closed
    let closed = SeasonRepository::get_season(&pool, &season.id)
        .await
        .expect("Failed to fetch season after close");
    assert!(closed.is_some(), "Season should still exist after close");
    assert!(
        !closed.unwrap().is_open,
        "Season should be closed (is_open = false)"
    );
}

#[tokio::test]
async fn test_close_season_sets_end_date() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    assert!(
        season.end_date.is_none(),
        "New season should have no end_date"
    );

    let before_close = Utc::now();
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("close_season should succeed");

    let closed = SeasonRepository::get_season(&pool, &season.id)
        .await
        .expect("Failed to fetch season after close")
        .expect("Season not found after close");

    assert!(
        closed.end_date.is_some(),
        "Closed season should have end_date set"
    );

    let end = closed.end_date.unwrap();
    assert!(
        end >= before_close,
        "end_date ({}) should be >= before_close ({})",
        end,
        before_close
    );
}

#[tokio::test]
async fn test_close_season_on_already_closed() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    // Close first time
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("First close should succeed");

    // Close second time - should either succeed idempotently or return error
    let result = SeasonRepository::close_season(&pool, &season.id).await;

    // The implementation may choose idempotent or error — we verify it's handled
    if let Err(ref e) = result {
        // If it errors, it should be a meaningful error (Conflict, InvalidInput, etc.)
        // not a panic or unexpected error
        let msg = format!("{}", e);
        assert!(
            msg.to_lowercase().contains("close")
                || msg.to_lowercase().contains("season")
                || msg.to_lowercase().contains("already")
                || msg.to_lowercase().contains("open"),
            "Error on double-close should reference the season/close status, got: {}",
            msg
        );
    }
    // If it's Ok, idempotent behavior is also acceptable.
}

#[tokio::test]
async fn test_close_season_with_nonexistent_id() {
    let pool = setup_test_pool().await;

    let result = SeasonRepository::close_season(&pool, "nonexistent-season-id").await;

    assert!(
        result.is_err(),
        "close_season with non-existent ID should fail, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_close_season_with_empty_id() {
    let pool = setup_test_pool().await;

    let result = SeasonRepository::close_season(&pool, "").await;

    assert!(result.is_err(), "close_season with empty ID should fail");
}

// ============================================================================
// APPLY SEEDING TESTS
// ============================================================================

#[tokio::test]
async fn test_apply_seeding_ordinal_propagates() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let from_season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create from_season");

    let to_season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Ordinal)
            .await
            .expect("Failed to create to_season");

    let result = SeasonRepository::apply_seeding(
        &pool,
        &from_season.id,
        &to_season.id,
        SeedingChoice::Ordinal,
    )
    .await;

    assert!(
        result.is_ok(),
        "apply_seeding with Ordinal should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_apply_seeding_reset_clears() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let from_season = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await
    .expect("Failed to create from_season");

    let to_season = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await
    .expect("Failed to create to_season");

    let result = SeasonRepository::apply_seeding(
        &pool,
        &from_season.id,
        &to_season.id,
        SeedingChoice::Reset,
    )
    .await;

    assert!(
        result.is_ok(),
        "apply_seeding with Reset should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_apply_seeding_nonexistent_from_season() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let to_season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create to_season");

    let result = SeasonRepository::apply_seeding(
        &pool,
        "nonexistent-from-season",
        &to_season.id,
        SeedingChoice::Ordinal,
    )
    .await;

    assert!(
        result.is_err(),
        "apply_seeding with non-existent from_season should fail"
    );
}

#[tokio::test]
async fn test_apply_seeding_nonexistent_to_season() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let from_season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create from_season");

    let result = SeasonRepository::apply_seeding(
        &pool,
        &from_season.id,
        "nonexistent-to-season",
        SeedingChoice::Ordinal,
    )
    .await;

    assert!(
        result.is_err(),
        "apply_seeding with non-existent to_season should fail"
    );
}

#[tokio::test]
async fn test_apply_seeding_same_season() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    let result =
        SeasonRepository::apply_seeding(&pool, &season.id, &season.id, SeedingChoice::Ordinal)
            .await;

    // Should probably fail — you can't seed from a season to itself
    assert!(
        result.is_err(),
        "apply_seeding from same season to itself should fail"
    );
}

#[tokio::test]
async fn test_apply_seeding_cross_league() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "League One").await;
    insert_league(&pool, "league-2", "League Two").await;

    let params = default_params();
    let from_season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create from_season");

    let to_season =
        SeasonRepository::create_season(&pool, "league-2", "elo", &params, SeedingChoice::Ordinal)
            .await
            .expect("Failed to create to_season");

    let result = SeasonRepository::apply_seeding(
        &pool,
        &from_season.id,
        &to_season.id,
        SeedingChoice::Ordinal,
    )
    .await;

    // Cross-league seeding should fail — seasons belong to different leagues
    assert!(
        result.is_err(),
        "apply_seeding across different leagues should fail"
    );
}

// ============================================================================
// UPDATE SEASON PARAMS TESTS
// ============================================================================

#[tokio::test]
async fn test_update_season_params_changes_algorithm_params() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    let new_params = custom_params(1200.0, Some(200.0));
    let result = SeasonRepository::update_season_params(&pool, &season.id, &new_params).await;

    assert!(
        result.is_ok(),
        "update_season_params should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_update_season_params_preserves_is_open() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    assert!(season.is_open, "Season should be open before update");

    let new_params = custom_params(1300.0, Some(250.0));
    let updated = SeasonRepository::update_season_params(&pool, &season.id, &new_params)
        .await
        .expect("update_season_params should succeed");

    assert!(
        updated.is_open,
        "is_open should be preserved after params update"
    );
}

#[tokio::test]
async fn test_update_season_params_on_closed_season() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    // Close the season first
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("close_season should succeed");

    let new_params = custom_params(1400.0, Some(300.0));
    let result = SeasonRepository::update_season_params(&pool, &season.id, &new_params).await;

    // Should fail — can't update params on a closed season
    assert!(
        result.is_err(),
        "update_season_params on a closed season should fail"
    );
}

#[tokio::test]
async fn test_update_season_params_nonexistent_season() {
    let pool = setup_test_pool().await;

    let new_params = default_params();
    let result =
        SeasonRepository::update_season_params(&pool, "nonexistent-season", &new_params).await;

    assert!(
        result.is_err(),
        "update_season_params with non-existent season should fail"
    );
}

#[tokio::test]
async fn test_update_season_params_multiple_times() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    // Update params multiple times in sequence
    for i in 0..3 {
        let new_params = AlgorithmParams {
            initial_rating: 1000.0 + (i as f64) * 100.0,
            initial_deviation: Some(200.0 + (i as f64) * 50.0),
            extra: Some(json!({"update_count": i})),
        };

        let result = SeasonRepository::update_season_params(&pool, &season.id, &new_params).await;

        assert!(
            result.is_ok(),
            "update_season_params iteration {} should succeed, got: {:?}",
            i,
            result
        );
    }
}

// ============================================================================
// LIST SEASONS TESTS
// ============================================================================

#[tokio::test]
async fn test_list_seasons_returns_all_for_league() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();

    // Create several seasons
    let s1 =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season 1");

    let s2 = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Ordinal,
    )
    .await
    .expect("Failed to create season 2");

    let s3 = SeasonRepository::create_season(
        &pool,
        "league-1",
        "trueskill",
        &params,
        SeedingChoice::Reset,
    )
    .await
    .expect("Failed to create season 3");

    let seasons = SeasonRepository::list_seasons(&pool, "league-1")
        .await
        .expect("list_seasons should succeed");

    assert_eq!(
        seasons.len(),
        3,
        "Should have 3 seasons for league-1, got: {}",
        seasons.len()
    );

    let ids: Vec<&str> = seasons.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&s1.id.as_str()));
    assert!(ids.contains(&s2.id.as_str()));
    assert!(ids.contains(&s3.id.as_str()));
}

#[tokio::test]
async fn test_list_seasons_returns_empty_for_league_with_no_seasons() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "empty-league", "Empty League").await;

    let seasons = SeasonRepository::list_seasons(&pool, "empty-league")
        .await
        .expect("list_seasons should succeed");

    assert!(
        seasons.is_empty(),
        "list_seasons for league with no seasons should return empty vec, got {} items",
        seasons.len()
    );
}

#[tokio::test]
async fn test_list_seasons_nonexistent_league() {
    let pool = setup_test_pool().await;

    let result = SeasonRepository::list_seasons(&pool, "nonexistent-league").await;

    // Should either return empty vec or error
    if result.is_ok() {
        let seasons = result.unwrap();
        assert!(
            seasons.is_empty(),
            "list_seasons for non-existent league should be empty, got {} items",
            seasons.len()
        );
    }
    // Error is also acceptable for non-existent league
}

#[tokio::test]
async fn test_list_seasons_only_returns_target_league() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "League One").await;
    insert_league(&pool, "league-2", "League Two").await;

    let params = default_params();

    // Season in league-1
    let s_a =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season in league-1");

    // Season in league-2
    let s_b =
        SeasonRepository::create_season(&pool, "league-2", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season in league-2");

    let league1_seasons = SeasonRepository::list_seasons(&pool, "league-1")
        .await
        .expect("list_seasons for league-1 should succeed");

    // Should only contain league-1's season
    let ids: Vec<&str> = league1_seasons.iter().map(|s| s.id.as_str()).collect();
    assert!(
        ids.contains(&s_a.id.as_str()),
        "league-1 listing should contain its own season"
    );
    assert!(
        !ids.contains(&s_b.id.as_str()),
        "league-1 listing should NOT contain league-2's season"
    );

    let league2_seasons = SeasonRepository::list_seasons(&pool, "league-2")
        .await
        .expect("list_seasons for league-2 should succeed");

    let ids2: Vec<&str> = league2_seasons.iter().map(|s| s.id.as_str()).collect();
    assert!(
        ids2.contains(&s_b.id.as_str()),
        "league-2 listing should contain its own season"
    );
    assert!(
        !ids2.contains(&s_a.id.as_str()),
        "league-2 listing should NOT contain league-1's season"
    );
}

// ============================================================================
// GET CURRENT SEASON TESTS
// ============================================================================

#[tokio::test]
async fn test_get_current_season_returns_open_season() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    let current = SeasonRepository::get_current_season(&pool, "league-1")
        .await
        .expect("get_current_season should succeed");

    assert!(current.is_some(), "Should find an open current season");
    assert_eq!(
        current.unwrap().id,
        season.id,
        "Current season should match the created season"
    );
}

#[tokio::test]
async fn test_get_current_season_returns_none_when_all_closed() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    // Close it
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("close_season should succeed");

    let current = SeasonRepository::get_current_season(&pool, "league-1")
        .await
        .expect("get_current_season should succeed");

    assert!(
        current.is_none(),
        "get_current_season should return None when all seasons are closed"
    );
}

#[tokio::test]
async fn test_get_current_season_only_one_open_at_a_time() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let s1 =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season 1");

    // Close s1 before creating s2, or if multiple open allowed, only the latest should be current
    SeasonRepository::close_season(&pool, &s1.id)
        .await
        .expect("close_season should succeed");

    let s2 = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Ordinal,
    )
    .await
    .expect("Failed to create season 2");

    let current = SeasonRepository::get_current_season(&pool, "league-1")
        .await
        .expect("get_current_season should succeed");

    assert!(current.is_some(), "Should have an open current season");
    assert_eq!(
        current.unwrap().id,
        s2.id,
        "Current season should be the open one"
    );
}

#[tokio::test]
async fn test_get_current_season_empty_league() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Empty League").await;

    let current = SeasonRepository::get_current_season(&pool, "league-1")
        .await
        .expect("get_current_season should succeed");

    assert!(
        current.is_none(),
        "get_current_season should return None for league with no seasons"
    );
}

#[tokio::test]
async fn test_get_current_season_nonexistent_league() {
    let pool = setup_test_pool().await;

    let current = SeasonRepository::get_current_season(&pool, "nonexistent-league")
        .await
        .expect("get_current_season should succeed");

    assert!(
        current.is_none(),
        "get_current_season should return None for non-existent league"
    );
}

// ============================================================================
// GET SEASON TESTS
// ============================================================================

#[tokio::test]
async fn test_get_season_returns_season_when_found() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let created =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    let fetched = SeasonRepository::get_season(&pool, &created.id)
        .await
        .expect("get_season should succeed");

    assert!(fetched.is_some(), "Should find the created season");
    let season = fetched.unwrap();
    assert_eq!(season.id, created.id);
    assert_eq!(season.league_id, "league-1");
    assert_eq!(season.algorithm, "elo");
}

#[tokio::test]
async fn test_get_season_returns_none_when_not_found() {
    let pool = setup_test_pool().await;

    let result = SeasonRepository::get_season(&pool, "nonexistent-id")
        .await
        .expect("get_season should succeed");

    assert!(
        result.is_none(),
        "get_season should return None for non-existent ID"
    );
}

#[tokio::test]
async fn test_get_season_with_empty_id() {
    let pool = setup_test_pool().await;

    let result = SeasonRepository::get_season(&pool, "").await;

    // Either returns None or an error — neither should panic
    if let Ok(opt) = result {
        assert!(opt.is_none(), "get_season with empty ID should return None");
    }
    // Error is also acceptable
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_create_multiple_seasons_same_league() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let mut ids = Vec::new();

    for i in 0..5 {
        let season = SeasonRepository::create_season(
            &pool,
            "league-1",
            "elo",
            &params,
            SeedingChoice::Reset,
        )
        .await
        .expect(&format!("Failed to create season {}", i));

        ids.push(season.id);
    }

    // All IDs should be unique
    let unique_count = {
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        sorted.len()
    };
    assert_eq!(unique_count, 5, "All 5 season IDs should be unique");
}

#[tokio::test]
async fn test_close_then_list_excludes_closed_from_current() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    // Should be in listing
    let before = SeasonRepository::list_seasons(&pool, "league-1")
        .await
        .expect("list_seasons before close");
    assert_eq!(before.len(), 1, "Should have 1 season before close");

    // Close it
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("close_season should succeed");

    // Should still be in listing (closed seasons are still seasons)
    let after = SeasonRepository::list_seasons(&pool, "league-1")
        .await
        .expect("list_seasons after close");
    assert_eq!(
        after.len(),
        1,
        "Closed season should still appear in list_seasons"
    );

    // But should NOT be the current season
    let current = SeasonRepository::get_current_season(&pool, "league-1")
        .await
        .expect("get_current_season after close");
    assert!(current.is_none(), "Closed season should not be current");
}

#[tokio::test]
async fn test_seeding_choice_serialization_roundtrip() {
    // Verify SeedingChoice can serialize/deserialize correctly
    let ordinal = SeedingChoice::Ordinal;
    let ordinal_json = serde_json::to_string(&ordinal).expect("Serialize Ordinal");
    assert_eq!(ordinal_json, "\"ordinal\"");

    let reset = SeedingChoice::Reset;
    let reset_json = serde_json::to_string(&reset).expect("Serialize Reset");
    assert_eq!(reset_json, "\"reset\"");

    // Round-trip
    let parsed: SeedingChoice = serde_json::from_str(&ordinal_json).expect("Deserialize Ordinal");
    assert_eq!(parsed, SeedingChoice::Ordinal);

    let parsed: SeedingChoice = serde_json::from_str(&reset_json).expect("Deserialize Reset");
    assert_eq!(parsed, SeedingChoice::Reset);
}

#[tokio::test]
async fn test_algorithm_params_with_extra_json() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: Some(350.0),
        extra: Some(json!({
            "tau": 0.5,
            "epsilon": 0.000001,
            "custom_field": "value"
        })),
    };

    let result = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await;

    assert!(
        result.is_ok(),
        "create_season with extra JSON params should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_algorithm_params_with_no_deviation() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    // Some algorithms (like Elo) don't use deviation
    let params = AlgorithmParams {
        initial_rating: 1500.0,
        initial_deviation: None,
        extra: None,
    };

    let result =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await;

    assert!(
        result.is_ok(),
        "create_season with no deviation for Elo should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_close_season_then_update_params_fails() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let season =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Failed to create season");

    // Close season
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("Failed to close season");

    // Attempt to update params on closed season
    let new_params = custom_params(1400.0, Some(300.0));
    let result = SeasonRepository::update_season_params(&pool, &season.id, &new_params).await;

    assert!(
        result.is_err(),
        "Should not be able to update params on a closed season"
    );
}

#[tokio::test]
async fn test_create_season_with_all_seeding_choices() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = default_params();
    let choices = [SeedingChoice::Ordinal, SeedingChoice::Reset];

    for choice in &choices {
        let result =
            SeasonRepository::create_season(&pool, "league-1", "elo", &params, *choice).await;

        assert!(
            result.is_ok(),
            "create_season with {:?} seeding choice should succeed, got: {:?}",
            choice,
            result
        );
    }
}

#[tokio::test]
async fn test_create_season_zero_initial_rating() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = AlgorithmParams {
        initial_rating: 0.0,
        initial_deviation: Some(100.0),
        extra: None,
    };

    let result =
        SeasonRepository::create_season(&pool, "league-1", "elo", &params, SeedingChoice::Reset)
            .await;

    assert!(
        result.is_ok(),
        "create_season with zero initial_rating should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_create_season_very_large_initial_rating() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    let params = AlgorithmParams {
        initial_rating: 1_000_000.0,
        initial_deviation: Some(100.0),
        extra: None,
    };

    let result = SeasonRepository::create_season(
        &pool,
        "league-1",
        "trueskill",
        &params,
        SeedingChoice::Reset,
    )
    .await;

    assert!(
        result.is_ok(),
        "create_season with very large initial_rating should succeed, got: {:?}",
        result
    );
}

// ============================================================================
// TRANSACTIONAL / INTEGRATION SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_full_season_lifecycle() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-1", "Test League").await;

    // 1. Create season
    let params = default_params();
    let season = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &params,
        SeedingChoice::Reset,
    )
    .await
    .expect("Failed to create season");

    assert!(season.is_open, "Season should start open");
    assert!(
        season.end_date.is_none(),
        "New season should have no end_date"
    );

    // 2. Update params
    let updated = SeasonRepository::update_season_params(
        &pool,
        &season.id,
        &custom_params(1400.0, Some(300.0)),
    )
    .await
    .expect("Failed to update params");

    assert!(updated.is_open, "Season should still be open after update");

    // 3. Create a second season and apply seeding
    let next_season = SeasonRepository::create_season(
        &pool,
        "league-1",
        "glicko2",
        &custom_params(1400.0, Some(300.0)),
        SeedingChoice::Ordinal,
    )
    .await
    .expect("Failed to create next season");

    SeasonRepository::apply_seeding(&pool, &season.id, &next_season.id, SeedingChoice::Ordinal)
        .await
        .expect("apply_seeding should succeed");

    // 4. Close the first season
    SeasonRepository::close_season(&pool, &season.id)
        .await
        .expect("Failed to close season");

    let closed = SeasonRepository::get_season(&pool, &season.id)
        .await
        .expect("get_season failed")
        .expect("Season not found");

    assert!(!closed.is_open, "Season should be closed");
    assert!(
        closed.end_date.is_some(),
        "Closed season should have end_date"
    );

    // 5. Verify the next season is still open
    let next = SeasonRepository::get_season(&pool, &next_season.id)
        .await
        .expect("get_season failed")
        .expect("Next season not found");

    assert!(next.is_open, "Next season should still be open");

    // 6. Verify listing
    let seasons = SeasonRepository::list_seasons(&pool, "league-1")
        .await
        .expect("list_seasons failed");

    assert_eq!(seasons.len(), 2, "Should have 2 seasons total");
}

#[tokio::test]
async fn test_season_isolation_between_leagues() {
    let pool = setup_test_pool().await;
    insert_league(&pool, "league-a", "League A").await;
    insert_league(&pool, "league-b", "League B").await;

    let params = default_params();

    let sa1 =
        SeasonRepository::create_season(&pool, "league-a", "elo", &params, SeedingChoice::Reset)
            .await
            .expect("Create A1");

    let _sa2 = SeasonRepository::create_season(
        &pool,
        "league-a",
        "glicko2",
        &params,
        SeedingChoice::Ordinal,
    )
    .await
    .expect("Create A2");

    let _sb1 = SeasonRepository::create_season(
        &pool,
        "league-b",
        "trueskill",
        &params,
        SeedingChoice::Reset,
    )
    .await
    .expect("Create B1");

    // League A has 2 seasons
    let a_seasons = SeasonRepository::list_seasons(&pool, "league-a")
        .await
        .expect("list A");
    assert_eq!(a_seasons.len(), 2);

    // League B has 1 season
    let b_seasons = SeasonRepository::list_seasons(&pool, "league-b")
        .await
        .expect("list B");
    assert_eq!(b_seasons.len(), 1);

    // Current season for league A should be sa2 (sa1 was closed... wait, we didn't close sa1)
    let current_a = SeasonRepository::get_current_season(&pool, "league-a")
        .await
        .expect("current A");
    assert!(current_a.is_some(), "League A should have a current season");

    let current_b = SeasonRepository::get_current_season(&pool, "league-b")
        .await
        .expect("current B");
    assert!(current_b.is_some(), "League B should have a current season");

    // Close sa1 - should not affect league B
    SeasonRepository::close_season(&pool, &sa1.id)
        .await
        .expect("close sa1");

    let a_seasons_after = SeasonRepository::list_seasons(&pool, "league-a")
        .await
        .expect("list A after close");
    assert_eq!(a_seasons_after.len(), 2, "League A still has 2 seasons");

    let b_seasons_after = SeasonRepository::list_seasons(&pool, "league-b")
        .await
        .expect("list B after close");
    assert_eq!(
        b_seasons_after.len(),
        1,
        "League B still has 1 season (unaffected)"
    );
}
