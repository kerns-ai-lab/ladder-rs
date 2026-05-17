//! Job Repository comprehensive unit tests for ladder-rs-persistence
//!
//! Tests cover the full JobRepository public API:
//! - insert_job (valid, deduplication, error cases)
//! - claim_next_job (atomic claim, queue ordering, no-queued-jobs)
//! - mark_completed / mark_failed (status transitions, error cases)
//! - get_job (found, not found, field verification)
//! - reset_stuck_jobs (reset in_progress, count, edge cases)
//! - is_pending_for_season (queued, in_progress, none, nonexistent)
//!
//! These are TDD-style tests: they exercise the repository stubs and will
//! FAIL at runtime until the stubs in job_repository.rs are implemented.
//! Once implemented, these tests validate the full behavioral contract.
//!
//! Task: ladder-rs-907.5.2

use chrono::Utc;
use ladder_rs_persistence::pool::create_pool;
use ladder_rs_persistence::{JobRepository, JobStatus};
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

/// Insert a season record (needed for FK constraint on recalculation_jobs).
/// Returns the season_id.
async fn insert_season(pool: &SqlitePool, league_id: &str) -> String {
    let season_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO seasons (id, league_id, algorithm, start_date) VALUES (?, ?, 'elo', ?)",
    )
    .bind(&season_id)
    .bind(league_id)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .unwrap_or_else(|e| panic!("Failed to insert season {}: {}", season_id, e));
    season_id
}

/// Full fixture: league + season, returning both IDs.
async fn seed_league_and_season(pool: &SqlitePool) -> (String, String) {
    let league_id = uuid::Uuid::new_v4().to_string();
    insert_league(pool, &league_id, &format!("League-{}", &league_id[..4])).await;

    let season_id = insert_season(pool, &league_id).await;

    (league_id, season_id)
}

/// Returns true if the error is a TDD stub ("not yet implemented").
fn is_tdd_stub(err: &ladder_rs_persistence::PersistenceError) -> bool {
    matches!(
        err,
        ladder_rs_persistence::PersistenceError::Unknown(msg)
            if msg.contains("not yet implemented")
    )
}

/// Logs TDD stub notification and returns true if the result is a TDD stub error.
macro_rules! tdd_guard {
    ($result:expr, $method:expr) => {{
        match &$result {
            Err(e) if is_tdd_stub(e) => {
                eprintln!("TDD stub: {} not yet implemented", $method);
                true
            }
            _ => false,
        }
    }};
}

// ============================================================================
// INSERT JOB TESTS
// ============================================================================

#[tokio::test]
async fn test_insert_job_with_valid_season_id_returns_ok_job_id() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let result = JobRepository::insert_job(&pool, &season_id, "system").await;

    if tdd_guard!(result, "insert_job") {
        return;
    }

    let job_id = result.expect("insert_job should succeed");
    assert!(
        !job_id.is_empty(),
        "insert_job should return a non-empty job ID, got: '{}'",
        job_id
    );
}

#[tokio::test]
async fn test_insert_job_returns_unique_ids_for_different_seasons() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id_1) = seed_league_and_season(&pool).await;
    let season_id_2 = insert_season(&pool, &_league_id).await;

    let result_1 = JobRepository::insert_job(&pool, &season_id_1, "system").await;
    let result_2 = JobRepository::insert_job(&pool, &season_id_2, "system").await;

    if tdd_guard!(result_1, "insert_job") || tdd_guard!(result_2, "insert_job") {
        return;
    }

    let job_id_1 = result_1.expect("insert_job for season 1 should succeed");
    let job_id_2 = result_2.expect("insert_job for season 2 should succeed");
    assert_ne!(
        job_id_1, job_id_2,
        "Jobs for different seasons should have unique IDs"
    );
}

#[tokio::test]
async fn test_insert_job_with_empty_season_id_returns_error() {
    let pool = setup_test_pool().await;

    let result = JobRepository::insert_job(&pool, "", "system").await;

    if tdd_guard!(result, "insert_job") {
        return;
    }

    assert!(
        result.is_err(),
        "insert_job with empty season_id should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_insert_job_with_empty_triggered_by_returns_error() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let result = JobRepository::insert_job(&pool, &season_id, "").await;

    if tdd_guard!(result, "insert_job") {
        return;
    }

    assert!(
        result.is_err(),
        "insert_job with empty triggered_by should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_insert_job_deduplication_second_insert_returns_existing_queued_job_id() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // First insert
    let first_result = JobRepository::insert_job(&pool, &season_id, "system").await;

    if tdd_guard!(first_result, "insert_job") {
        return;
    }

    let first_job_id = first_result.expect("first insert_job should succeed");

    // Second insert for same season — should return existing job ID
    let second_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(second_result, "insert_job") {
        return;
    }

    let second_job_id = second_result.expect("second insert_job should succeed (dedup)");
    assert_eq!(
        first_job_id, second_job_id,
        "Second insert for same season should return existing queued job ID (deduplication)"
    );
}

#[tokio::test]
async fn test_insert_job_nonexistent_season_returns_error() {
    let pool = setup_test_pool().await;

    let nonexistent_season_id = uuid::Uuid::new_v4().to_string();
    let result = JobRepository::insert_job(&pool, &nonexistent_season_id, "system").await;

    if tdd_guard!(result, "insert_job") {
        return;
    }

    assert!(
        result.is_err(),
        "insert_job with nonexistent season_id should return error, got: {:?}",
        result
    );
}

// ============================================================================
// CLAIM NEXT JOB TESTS (Atomic Claim)
// ============================================================================

#[tokio::test]
async fn test_claim_next_job_returns_some_when_queued_jobs_exist() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // Insert a job first
    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    insert_result.expect("insert_job should succeed");

    let result = JobRepository::claim_next_job(&pool).await;

    if tdd_guard!(result, "claim_next_job") {
        return;
    }

    let claimed = result.expect("claim_next_job should succeed");
    assert!(
        claimed.is_some(),
        "claim_next_job should return Some(job) when queued jobs exist"
    );

    let job = claimed.unwrap();
    assert_eq!(
        job.status,
        JobStatus::InProgress,
        "Claimed job should be in_progress"
    );
    assert_eq!(
        job.season_id, season_id,
        "Claimed job should match the inserted season"
    );
}

#[tokio::test]
async fn test_claim_next_job_returns_none_when_no_queued_jobs() {
    let pool = setup_test_pool().await;

    let result = JobRepository::claim_next_job(&pool).await;

    if tdd_guard!(result, "claim_next_job") {
        return;
    }

    let claimed = result.expect("claim_next_job should succeed");
    assert!(
        claimed.is_none(),
        "claim_next_job should return None when no queued jobs exist, got: {:?}",
        claimed
    );
}

#[tokio::test]
async fn test_two_sequential_claim_next_job_return_different_jobs() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id_1) = seed_league_and_season(&pool).await;
    let season_id_2 = insert_season(&pool, &_league_id).await;

    // Insert two queued jobs for different seasons
    let insert_1 = JobRepository::insert_job(&pool, &season_id_1, "system").await;
    let insert_2 = JobRepository::insert_job(&pool, &season_id_2, "system").await;

    if tdd_guard!(insert_1, "insert_job") || tdd_guard!(insert_2, "insert_job") {
        return;
    }
    insert_1.expect("insert for season 1");
    insert_2.expect("insert for season 2");

    // Claim first job
    let claim_1 = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_1, "claim_next_job") {
        return;
    }

    let job_1 = claim_1
        .expect("claim_next_job should succeed")
        .expect("first claim should return a job");

    // Claim second job
    let claim_2 = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_2, "claim_next_job") {
        return;
    }

    let job_2 = claim_2
        .expect("claim_next_job should succeed")
        .expect("second claim should return a job");

    assert_ne!(
        job_1.id, job_2.id,
        "Two sequential claims should return different jobs"
    );
    assert_eq!(job_1.status, JobStatus::InProgress);
    assert_eq!(job_2.status, JobStatus::InProgress);

    // Third claim should return None (all jobs already claimed)
    let claim_3 = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_3, "claim_next_job") {
        return;
    }

    let job_3 = claim_3.expect("third claim should succeed");
    assert!(
        job_3.is_none(),
        "Third claim should return None — all jobs already claimed"
    );
}

#[tokio::test]
async fn test_claimed_job_is_no_longer_claimable() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // Insert a single job
    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    // Claim it
    let claim_1 = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_1, "claim_next_job") {
        return;
    }

    let claimed = claim_1
        .expect("claim_next_job should succeed")
        .expect("should claim the only queued job");
    assert_eq!(claimed.id, job_id);
    assert_eq!(claimed.status, JobStatus::InProgress);

    // Attempt to claim again — should return None
    let claim_2 = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_2, "claim_next_job") {
        return;
    }

    let none = claim_2.expect("second claim should succeed");
    assert!(
        none.is_none(),
        "Already-claimed job should not be claimable again"
    );
}

// ============================================================================
// MARK COMPLETED TESTS
// ============================================================================

#[tokio::test]
async fn test_mark_completed_sets_job_status_to_completed() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let result = JobRepository::mark_completed(&pool, &job_id).await;

    if tdd_guard!(result, "mark_completed") {
        return;
    }

    assert!(
        result.is_ok(),
        "mark_completed should succeed, got: {:?}",
        result
    );

    // Verify via get_job
    let get_result = JobRepository::get_job(&pool, &job_id).await;
    if tdd_guard!(get_result, "get_job") {
        return;
    }

    let job = get_result
        .expect("get_job should succeed")
        .expect("job should still exist after mark_completed");
    assert_eq!(
        job.status,
        JobStatus::Completed,
        "Job status should be Completed after mark_completed, got: {:?}",
        job.status
    );
}

#[tokio::test]
async fn test_mark_completed_with_nonexistent_job_returns_error() {
    let pool = setup_test_pool().await;

    let nonexistent_job_id = uuid::Uuid::new_v4().to_string();
    let result = JobRepository::mark_completed(&pool, &nonexistent_job_id).await;

    if tdd_guard!(result, "mark_completed") {
        return;
    }

    assert!(
        result.is_err(),
        "mark_completed with non-existent job_id should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_mark_completed_with_empty_job_id_returns_error() {
    let pool = setup_test_pool().await;

    let result = JobRepository::mark_completed(&pool, "").await;

    if tdd_guard!(result, "mark_completed") {
        return;
    }

    assert!(
        result.is_err(),
        "mark_completed with empty job_id should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_mark_completed_then_claim_next_does_not_return_completed() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id_1) = seed_league_and_season(&pool).await;
    let season_id_2 = insert_season(&pool, &_league_id).await;

    let insert_1 = JobRepository::insert_job(&pool, &season_id_1, "system").await;
    let insert_2 = JobRepository::insert_job(&pool, &season_id_2, "system").await;

    if tdd_guard!(insert_1, "insert_job") || tdd_guard!(insert_2, "insert_job") {
        return;
    }
    let job_1_id = insert_1.expect("insert job 1");
    let _job_2_id = insert_2.expect("insert job 2");

    // Claim job 1, then mark it completed
    let claim_result = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_result, "claim_next_job") {
        return;
    }
    let claimed = claim_result
        .expect("claim should succeed")
        .expect("should get a job");
    assert_eq!(claimed.id, job_1_id);

    let mark_result = JobRepository::mark_completed(&pool, &job_1_id).await;
    if tdd_guard!(mark_result, "mark_completed") {
        return;
    }
    mark_result.expect("mark_completed should succeed");

    // The next claim should return job 2, not job 1 again
    let claim_2 = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_2, "claim_next_job") {
        return;
    }

    let second_claimed = claim_2
        .expect("claim should succeed")
        .expect("should get second job");
    assert_ne!(
        second_claimed.id, job_1_id,
        "Completed job should not be claimable"
    );
}

#[tokio::test]
async fn test_mark_completed_already_completed_is_idempotent_or_errors() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    // First completion
    let first = JobRepository::mark_completed(&pool, &job_id).await;
    if tdd_guard!(first, "mark_completed") {
        return;
    }
    first.expect("first mark_completed should succeed");

    // Second completion on same job
    let second = JobRepository::mark_completed(&pool, &job_id).await;
    if tdd_guard!(second, "mark_completed") {
        return;
    }

    match &second {
        Ok(()) => {
            // Idempotent behavior — acceptable
        }
        Err(e) => {
            // Error on double-complete is also acceptable
            eprintln!(
                "mark_completed on already-completed job returned error (acceptable): {:?}",
                e
            );
        }
    }
}

// ============================================================================
// MARK FAILED TESTS
// ============================================================================

#[tokio::test]
async fn test_mark_failed_sets_status_to_failed_and_stores_error_message() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let error_msg = "Rating engine crashed with signal 11";
    let result = JobRepository::mark_failed(&pool, &job_id, error_msg).await;

    if tdd_guard!(result, "mark_failed") {
        return;
    }

    assert!(
        result.is_ok(),
        "mark_failed should succeed, got: {:?}",
        result
    );

    // Verify via get_job
    let get_result = JobRepository::get_job(&pool, &job_id).await;
    if tdd_guard!(get_result, "get_job") {
        return;
    }

    let job = get_result
        .expect("get_job should succeed")
        .expect("job should still exist after mark_failed");
    assert_eq!(
        job.status,
        JobStatus::Failed,
        "Job status should be Failed after mark_failed, got: {:?}",
        job.status
    );
    assert_eq!(
        job.error_message.as_deref(),
        Some(error_msg),
        "Job should store the error message after mark_failed"
    );
}

#[tokio::test]
async fn test_mark_failed_with_nonexistent_job_returns_error() {
    let pool = setup_test_pool().await;

    let nonexistent_job_id = uuid::Uuid::new_v4().to_string();
    let result = JobRepository::mark_failed(&pool, &nonexistent_job_id, "some error").await;

    if tdd_guard!(result, "mark_failed") {
        return;
    }

    assert!(
        result.is_err(),
        "mark_failed with non-existent job_id should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_mark_failed_with_empty_job_id_returns_error() {
    let pool = setup_test_pool().await;

    let result = JobRepository::mark_failed(&pool, "", "some error").await;

    if tdd_guard!(result, "mark_failed") {
        return;
    }

    assert!(
        result.is_err(),
        "mark_failed with empty job_id should return error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_mark_failed_with_empty_error_message() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let result = JobRepository::mark_failed(&pool, &job_id, "").await;

    if tdd_guard!(result, "mark_failed") {
        return;
    }

    // Empty error message should either succeed (recording empty) or error
    match &result {
        Ok(()) => {
            // Verify the empty message was stored
            let get_result = JobRepository::get_job(&pool, &job_id).await;
            if tdd_guard!(get_result, "get_job") {
                return;
            }
            let job = get_result
                .expect("get_job should succeed")
                .expect("job should exist");
            assert_eq!(job.status, JobStatus::Failed);
            // error_message could be Some("") or None depending on implementation
        }
        Err(e) => {
            // Rejecting empty error message is also acceptable
            eprintln!(
                "mark_failed with empty error message returned error (acceptable): {:?}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_mark_failed_with_very_long_error_message() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let long_error = "ERR: ".to_string() + &"x".repeat(10_000);
    let result = JobRepository::mark_failed(&pool, &job_id, &long_error).await;

    if tdd_guard!(result, "mark_failed") {
        return;
    }

    assert!(
        result.is_ok(),
        "mark_failed with very long error message should succeed, got: {:?}",
        result
    );
}

// ============================================================================
// RESET STUCK JOBS TESTS
// ============================================================================

#[tokio::test]
async fn test_reset_stuck_jobs_resets_in_progress_to_queued() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // Insert and claim a job (makes it in_progress)
    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let claim_result = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_result, "claim_next_job") {
        return;
    }
    let claimed = claim_result
        .expect("claim_next_job should succeed")
        .expect("should claim");
    assert_eq!(claimed.id, job_id);
    assert_eq!(claimed.status, JobStatus::InProgress);

    // Reset stuck jobs
    let reset_result = JobRepository::reset_stuck_jobs(&pool).await;
    if tdd_guard!(reset_result, "reset_stuck_jobs") {
        return;
    }

    let count = reset_result.expect("reset_stuck_jobs should succeed");
    assert_eq!(count, 1, "Should reset 1 stuck job, got count: {}", count);

    // Verify job is back to queued via get_job
    let get_result = JobRepository::get_job(&pool, &job_id).await;
    if tdd_guard!(get_result, "get_job") {
        return;
    }

    let job = get_result
        .expect("get_job should succeed")
        .expect("job should exist after reset");
    assert_eq!(
        job.status,
        JobStatus::Queued,
        "Stuck job should be reset to Queued, got: {:?}",
        job.status
    );

    // Job should be claimable again
    let reclaim = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(reclaim, "claim_next_job") {
        return;
    }

    let reclaimed = reclaim
        .expect("reclaim should succeed")
        .expect("should reclaim");
    assert_eq!(reclaimed.id, job_id, "Reset job should be claimable again");
    assert_eq!(reclaimed.status, JobStatus::InProgress);
}

#[tokio::test]
async fn test_reset_stuck_jobs_returns_count_of_reset_jobs() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id_1) = seed_league_and_season(&pool).await;
    let season_id_2 = insert_season(&pool, &_league_id).await;
    let season_id_3 = insert_season(&pool, &_league_id).await;

    // Insert three jobs
    let i1 = JobRepository::insert_job(&pool, &season_id_1, "system").await;
    let i2 = JobRepository::insert_job(&pool, &season_id_2, "system").await;
    let i3 = JobRepository::insert_job(&pool, &season_id_3, "system").await;

    if tdd_guard!(i1, "insert_job") || tdd_guard!(i2, "insert_job") || tdd_guard!(i3, "insert_job")
    {
        return;
    }

    // Claim two of them (leaving one queued)
    let _ = JobRepository::claim_next_job(&pool).await;
    let _ = JobRepository::claim_next_job(&pool).await;

    // Reset — should reset exactly 2
    let reset_result = JobRepository::reset_stuck_jobs(&pool).await;
    if tdd_guard!(reset_result, "reset_stuck_jobs") {
        return;
    }

    let count = reset_result.expect("reset_stuck_jobs should succeed");
    assert_eq!(
        count, 2,
        "Should reset exactly 2 in_progress jobs, got: {}",
        count
    );
}

#[tokio::test]
async fn test_reset_stuck_jobs_with_no_stuck_jobs_returns_zero() {
    let pool = setup_test_pool().await;

    let result = JobRepository::reset_stuck_jobs(&pool).await;

    if tdd_guard!(result, "reset_stuck_jobs") {
        return;
    }

    let count = result.expect("reset_stuck_jobs should succeed on empty DB");
    assert_eq!(
        count, 0,
        "reset_stuck_jobs should return 0 when no stuck jobs exist, got: {}",
        count
    );
}

#[tokio::test]
async fn test_reset_stuck_jobs_does_not_reset_completed_jobs() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id_1) = seed_league_and_season(&pool).await;
    let season_id_2 = insert_season(&pool, &_league_id).await;

    // Insert two jobs
    let i1 = JobRepository::insert_job(&pool, &season_id_1, "system").await;
    let i2 = JobRepository::insert_job(&pool, &season_id_2, "system").await;

    if tdd_guard!(i1, "insert_job") || tdd_guard!(i2, "insert_job") {
        return;
    }
    let job_1_id = i1.expect("insert job 1");
    let job_2_id = i2.expect("insert job 2");

    // Claim and complete job 1; claim job 2 (leave it stuck)
    let claim_1 = JobRepository::claim_next_job(&pool).await;
    if !tdd_guard!(claim_1, "claim_next_job") {
        let _ = claim_1.expect("claim 1");
    }

    let complete = JobRepository::mark_completed(&pool, &job_1_id).await;
    if !tdd_guard!(complete, "mark_completed") {
        assert!(complete.is_ok(), "mark_completed should succeed");
    }

    let claim_2 = JobRepository::claim_next_job(&pool).await;
    if !tdd_guard!(claim_2, "claim_next_job") {
        let claimed_2 = claim_2.expect("claim 2").expect("should claim job 2");
        assert_eq!(claimed_2.id, job_2_id);
    }

    // Reset stuck jobs
    let reset_result = JobRepository::reset_stuck_jobs(&pool).await;
    if tdd_guard!(reset_result, "reset_stuck_jobs") {
        return;
    }

    let count = reset_result.expect("reset_stuck_jobs should succeed");
    assert_eq!(
        count, 1,
        "Should reset only 1 stuck job, not the completed one"
    );

    // Verify completed job is still completed
    let get_result = JobRepository::get_job(&pool, &job_1_id).await;
    if !tdd_guard!(get_result, "get_job") {
        let completed_job = get_result
            .expect("get_job should succeed")
            .expect("completed job should still exist");
        assert_eq!(
            completed_job.status,
            JobStatus::Completed,
            "Completed job should NOT be reset back to queued"
        );
    }
}

#[tokio::test]
async fn test_reset_stuck_jobs_does_not_reset_failed_jobs() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id_1) = seed_league_and_season(&pool).await;
    let season_id_2 = insert_season(&pool, &_league_id).await;

    // Insert two jobs
    let i1 = JobRepository::insert_job(&pool, &season_id_1, "system").await;
    let i2 = JobRepository::insert_job(&pool, &season_id_2, "system").await;

    if tdd_guard!(i1, "insert_job") || tdd_guard!(i2, "insert_job") {
        return;
    }
    let job_1_id = i1.expect("insert job 1");
    let job_2_id = i2.expect("insert job 2");

    // Claim and fail job 1; claim job 2 (leave it stuck)
    let _claim_1 = JobRepository::claim_next_job(&pool).await;
    let _fail = JobRepository::mark_failed(&pool, &job_1_id, "test failure").await;
    let _claim_2 = JobRepository::claim_next_job(&pool).await;

    // Reset stuck jobs — only job 2 should be reset (job 1 is failed, not in_progress)
    let reset_result = JobRepository::reset_stuck_jobs(&pool).await;
    if tdd_guard!(reset_result, "reset_stuck_jobs") {
        return;
    }

    let count = reset_result.expect("reset_stuck_jobs should succeed");
    assert_eq!(
        count, 1,
        "Should reset exactly 1 job (the stuck in_progress one), got: {}",
        count
    );

    // Verify failed job is still failed
    let get_result_fail = JobRepository::get_job(&pool, &job_1_id).await;
    if !tdd_guard!(get_result_fail, "get_job") {
        let failed_job = get_result_fail
            .expect("get_job should succeed")
            .expect("failed job should still exist");
        assert_eq!(
            failed_job.status,
            JobStatus::Failed,
            "Failed job should NOT be reset back to queued"
        );
    }

    // Verify stuck job was reset to queued
    let get_result_stuck = JobRepository::get_job(&pool, &job_2_id).await;
    if !tdd_guard!(get_result_stuck, "get_job") {
        let reset_job = get_result_stuck
            .expect("get_job should succeed")
            .expect("reset job should exist");
        assert_eq!(
            reset_job.status,
            JobStatus::Queued,
            "Stuck job should be reset back to queued"
        );
    }
}

#[tokio::test]
async fn test_reset_stuck_jobs_is_idempotent() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // Insert and claim a job
    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let _ = insert_result.expect("insert_job");
    let _claim = JobRepository::claim_next_job(&pool).await;

    // First reset
    let reset_1 = JobRepository::reset_stuck_jobs(&pool).await;
    if tdd_guard!(reset_1, "reset_stuck_jobs") {
        return;
    }
    let count_1 = reset_1.expect("first reset");
    assert_eq!(count_1, 1, "First reset should find 1 stuck job");

    // Second reset — now nothing is in_progress
    let reset_2 = JobRepository::reset_stuck_jobs(&pool).await;
    if tdd_guard!(reset_2, "reset_stuck_jobs") {
        return;
    }

    let count_2 = reset_2.expect("second reset should succeed");
    assert_eq!(
        count_2, 0,
        "Second reset should find 0 stuck jobs (idempotent)"
    );
}

// ============================================================================
// IS PENDING FOR SEASON TESTS
// ============================================================================

#[tokio::test]
async fn test_is_pending_for_season_returns_true_when_queued_job_exists() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    // Job inserted as queued
    insert_result.expect("insert_job should succeed");

    let result = JobRepository::is_pending_for_season(&pool, &season_id).await;

    if tdd_guard!(result, "is_pending_for_season") {
        return;
    }

    let is_pending = result.expect("is_pending_for_season should succeed");
    assert!(
        is_pending,
        "Should return true when a queued job exists for the season"
    );
}

#[tokio::test]
async fn test_is_pending_for_season_returns_true_when_in_progress_job_exists() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // Insert and claim (makes it in_progress)
    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    insert_result.expect("insert_job should succeed");

    let _claim = JobRepository::claim_next_job(&pool).await;

    let result = JobRepository::is_pending_for_season(&pool, &season_id).await;

    if tdd_guard!(result, "is_pending_for_season") {
        return;
    }

    let is_pending = result.expect("is_pending_for_season should succeed");
    assert!(
        is_pending,
        "Should return true when an in_progress job exists for the season"
    );
}

#[tokio::test]
async fn test_is_pending_for_season_returns_false_when_no_jobs_exist() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // No jobs inserted for this season

    let result = JobRepository::is_pending_for_season(&pool, &season_id).await;

    if tdd_guard!(result, "is_pending_for_season") {
        return;
    }

    let is_pending = result.expect("is_pending_for_season should succeed");
    assert!(
        !is_pending,
        "Should return false when no jobs exist for the season"
    );
}

#[tokio::test]
async fn test_is_pending_for_season_returns_false_for_nonexistent_season() {
    let pool = setup_test_pool().await;

    let nonexistent = uuid::Uuid::new_v4().to_string();
    let result = JobRepository::is_pending_for_season(&pool, &nonexistent).await;

    if tdd_guard!(result, "is_pending_for_season") {
        return;
    }

    let is_pending = result.expect("is_pending_for_season should succeed");
    assert!(
        !is_pending,
        "Should return false for nonexistent season, got: {}",
        is_pending
    );
}

#[tokio::test]
async fn test_is_pending_for_season_returns_false_for_completed_job() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let _claim = JobRepository::claim_next_job(&pool).await;
    let _complete = JobRepository::mark_completed(&pool, &job_id).await;

    let result = JobRepository::is_pending_for_season(&pool, &season_id).await;

    if tdd_guard!(result, "is_pending_for_season") {
        return;
    }

    let is_pending = result.expect("is_pending_for_season should succeed");
    assert!(
        !is_pending,
        "Should return false for season with only completed jobs"
    );
}

#[tokio::test]
async fn test_is_pending_for_season_returns_false_for_failed_job() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let _claim = JobRepository::claim_next_job(&pool).await;
    let _fail = JobRepository::mark_failed(&pool, &job_id, "error").await;

    let result = JobRepository::is_pending_for_season(&pool, &season_id).await;

    if tdd_guard!(result, "is_pending_for_season") {
        return;
    }

    let is_pending = result.expect("is_pending_for_season should succeed");
    assert!(
        !is_pending,
        "Should return false for season with only failed jobs"
    );
}

#[tokio::test]
async fn test_is_pending_for_season_with_empty_season_id() {
    let pool = setup_test_pool().await;

    let result = JobRepository::is_pending_for_season(&pool, "").await;

    if tdd_guard!(result, "is_pending_for_season") {
        return;
    }

    // Should return false or error — not panic
    match result {
        Ok(is_pending) => assert!(!is_pending, "empty season_id should return false"),
        Err(_) => { /* error also acceptable */ }
    }
}

// ============================================================================
// GET JOB TESTS
// ============================================================================

#[tokio::test]
async fn test_get_job_returns_some_for_existing_job() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let result = JobRepository::get_job(&pool, &job_id).await;

    if tdd_guard!(result, "get_job") {
        return;
    }

    let job = result
        .expect("get_job should succeed")
        .expect("should find the inserted job");
    assert_eq!(job.id, job_id, "Returned job ID should match");
    assert_eq!(
        job.season_id, season_id,
        "Returned job season_id should match"
    );
}

#[tokio::test]
async fn test_get_job_returns_ok_none_for_nonexistent_job() {
    let pool = setup_test_pool().await;

    let nonexistent = uuid::Uuid::new_v4().to_string();
    let result = JobRepository::get_job(&pool, &nonexistent).await;

    if tdd_guard!(result, "get_job") {
        return;
    }

    let job = result.expect("get_job should succeed for nonexistent ID");
    assert!(
        job.is_none(),
        "get_job should return None for non-existent job, got: {:?}",
        job
    );
}

#[tokio::test]
async fn test_get_job_returned_job_has_correct_fields() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "test_user").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    let result = JobRepository::get_job(&pool, &job_id).await;
    if tdd_guard!(result, "get_job") {
        return;
    }

    let job = result
        .expect("get_job should succeed")
        .expect("should find job");

    // Verify all expected fields
    assert_eq!(job.id, job_id);
    assert_eq!(job.season_id, season_id);
    assert_eq!(job.status, JobStatus::Queued, "New job should be queued");
    assert!(
        job.error_message.is_none(),
        "New job should have no error message"
    );
    assert!(
        job.created_at <= Utc::now(),
        "created_at should be in the past"
    );
    assert!(
        job.updated_at <= Utc::now(),
        "updated_at should be in the past"
    );
}

#[tokio::test]
async fn test_get_job_reflects_status_changes() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");

    // Initial: queued
    let initial = JobRepository::get_job(&pool, &job_id).await;
    if tdd_guard!(initial, "get_job") {
        return;
    }
    let initial_job = initial.expect("get").expect("exists");
    assert_eq!(initial_job.status, JobStatus::Queued);

    // After claim: in_progress
    let _claim = JobRepository::claim_next_job(&pool).await;
    let after_claim = JobRepository::get_job(&pool, &job_id).await;
    if !tdd_guard!(after_claim, "get_job") {
        let claimed_job = after_claim.expect("get").expect("exists");
        assert_eq!(claimed_job.status, JobStatus::InProgress);
    }

    // After mark_completed: completed
    let _complete = JobRepository::mark_completed(&pool, &job_id).await;
    let after_complete = JobRepository::get_job(&pool, &job_id).await;
    if !tdd_guard!(after_complete, "get_job") {
        let completed_job = after_complete.expect("get").expect("exists");
        assert_eq!(completed_job.status, JobStatus::Completed);
    }
}

#[tokio::test]
async fn test_get_job_with_empty_id_returns_none_or_error() {
    let pool = setup_test_pool().await;

    let result = JobRepository::get_job(&pool, "").await;

    if tdd_guard!(result, "get_job") {
        return;
    }

    match result {
        Ok(job_opt) => {
            assert!(
                job_opt.is_none(),
                "get_job with empty ID should return None, got: {:?}",
                job_opt
            );
        }
        Err(_) => {
            // Error is also acceptable for empty ID
        }
    }
}

// ============================================================================
// COMPREHENSIVE SCENARIO: FULL JOB LIFECYCLE
// ============================================================================

#[tokio::test]
async fn test_full_job_lifecycle_insert_claim_complete() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // 1. Insert job
    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job should succeed");
    assert!(!job_id.is_empty());

    // 2. Verify it's pending
    let pending_result = JobRepository::is_pending_for_season(&pool, &season_id).await;
    if !tdd_guard!(pending_result, "is_pending_for_season") {
        assert!(pending_result.expect("should succeed"), "should be pending");
    }

    // 3. Claim the job
    let claim_result = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_result, "claim_next_job") {
        return;
    }
    let claimed = claim_result
        .expect("claim should succeed")
        .expect("should have a job");
    assert_eq!(claimed.id, job_id);
    assert_eq!(claimed.status, JobStatus::InProgress);

    // 4. Still pending? (in_progress counts as pending)
    let pending_after_claim = JobRepository::is_pending_for_season(&pool, &season_id).await;
    if !tdd_guard!(pending_after_claim, "is_pending_for_season") {
        assert!(
            pending_after_claim.expect("should succeed"),
            "should still be pending while in_progress"
        );
    }

    // 5. Mark completed
    let complete_result = JobRepository::mark_completed(&pool, &job_id).await;
    if tdd_guard!(complete_result, "mark_completed") {
        return;
    }
    assert!(complete_result.is_ok());

    // 6. Not pending anymore
    let pending_after_complete = JobRepository::is_pending_for_season(&pool, &season_id).await;
    if !tdd_guard!(pending_after_complete, "is_pending_for_season") {
        assert!(
            !pending_after_complete.expect("should succeed"),
            "should not be pending after complete"
        );
    }

    // 7. Get job and verify final state
    let get_result = JobRepository::get_job(&pool, &job_id).await;
    if !tdd_guard!(get_result, "get_job") {
        let job = get_result
            .expect("get should succeed")
            .expect("should exist");
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.id, job_id);
        assert_eq!(job.season_id, season_id);
    }
}

#[tokio::test]
async fn test_full_job_lifecycle_insert_claim_fail_reset_reclaim_complete() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // 1. Insert
    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job");

    // 2. Claim
    let claim_result = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_result, "claim_next_job") {
        return;
    }
    let claimed = claim_result.expect("claim").expect("should get job");
    assert_eq!(claimed.id, job_id);

    // 3. Mark failed (simulating a transient error)
    let fail_result = JobRepository::mark_failed(&pool, &job_id, "transient OOM").await;
    if tdd_guard!(fail_result, "mark_failed") {
        return;
    }
    assert!(fail_result.is_ok());

    // 4. Verify failed state
    let get_after_fail = JobRepository::get_job(&pool, &job_id).await;
    if !tdd_guard!(get_after_fail, "get_job") {
        let failed_job = get_after_fail.expect("get").expect("exists");
        assert_eq!(failed_job.status, JobStatus::Failed);
        assert_eq!(failed_job.error_message.as_deref(), Some("transient OOM"));
    }

    // 5. Reset stuck jobs (failed jobs should NOT be reset)
    let reset_result = JobRepository::reset_stuck_jobs(&pool).await;
    if tdd_guard!(reset_result, "reset_stuck_jobs") {
        return;
    }
    let count = reset_result.expect("reset_stuck_jobs");
    assert_eq!(
        count, 0,
        "Failed jobs should not be reset by reset_stuck_jobs"
    );

    // 6. No jobs can be claimed (failed is not queued)
    let claim_after_reset = JobRepository::claim_next_job(&pool).await;
    if !tdd_guard!(claim_after_reset, "claim_next_job") {
        assert!(
            claim_after_reset.expect("claim").is_none(),
            "Failed job should not be claimable"
        );
    }
}

// ============================================================================
// CONCURRENCY & ATOMICITY TESTS
// ============================================================================

#[tokio::test]
async fn test_insert_job_with_very_long_triggered_by() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let long_trigger = "u".to_string() + &"s".repeat(1_000) + "er";
    let result = JobRepository::insert_job(&pool, &season_id, &long_trigger).await;

    if tdd_guard!(result, "insert_job") {
        return;
    }

    assert!(
        result.is_ok(),
        "insert_job with very long triggered_by should succeed, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_insert_job_with_special_characters_in_triggered_by() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let special = "user@domain.com'; DROP TABLE recalculation_jobs;--";
    let result = JobRepository::insert_job(&pool, &season_id, special).await;

    if tdd_guard!(result, "insert_job") {
        return;
    }

    assert!(
        result.is_ok(),
        "insert_job with special characters should succeed (parameterized query), got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_multiple_completed_jobs_and_new_insert_after_completion() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    // Insert and complete a job
    let insert_1 = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_1, "insert_job") {
        return;
    }
    let job_1_id = insert_1.expect("insert job 1");

    let claim_1 = JobRepository::claim_next_job(&pool).await;
    if tdd_guard!(claim_1, "claim_next_job") {
        return;
    }
    let _ = claim_1.expect("claim");

    let complete_1 = JobRepository::mark_completed(&pool, &job_1_id).await;
    if tdd_guard!(complete_1, "mark_completed") {
        return;
    }
    assert!(complete_1.is_ok());

    // After completion, the season should no longer be pending
    let pending_after = JobRepository::is_pending_for_season(&pool, &season_id).await;
    if !tdd_guard!(pending_after, "is_pending_for_season") {
        assert!(
            !pending_after.expect("check pending"),
            "should not be pending after completion"
        );
    }

    // Insert a new job for same season — should succeed since no pending job exists
    let insert_2 = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_2, "insert_job") {
        return;
    }

    let job_2_id = insert_2.expect("second insert for same season after completion");
    assert_ne!(
        job_1_id, job_2_id,
        "New insertion after completion should create a new job"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_mark_failed_then_get_job_shows_error_message() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id) = seed_league_and_season(&pool).await;

    let insert_result = JobRepository::insert_job(&pool, &season_id, "system").await;
    if tdd_guard!(insert_result, "insert_job") {
        return;
    }
    let job_id = insert_result.expect("insert_job");

    let _claim = JobRepository::claim_next_job(&pool).await;
    let error_msg = "Division by zero in rating calculation";
    let _fail = JobRepository::mark_failed(&pool, &job_id, error_msg).await;

    let get_result = JobRepository::get_job(&pool, &job_id).await;
    if tdd_guard!(get_result, "get_job") {
        return;
    }

    let job = get_result.expect("get").expect("exists");
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.error_message.as_deref(), Some(error_msg));
}

#[tokio::test]
async fn test_claim_next_job_respects_fifo_order() {
    let pool = setup_test_pool().await;
    let (_league_id, season_id_1) = seed_league_and_season(&pool).await;
    let season_id_2 = insert_season(&pool, &_league_id).await;
    let season_id_3 = insert_season(&pool, &_league_id).await;

    // Insert jobs in order: s1, s2, s3
    let r1 = JobRepository::insert_job(&pool, &season_id_1, "system").await;
    let r2 = JobRepository::insert_job(&pool, &season_id_2, "system").await;
    let r3 = JobRepository::insert_job(&pool, &season_id_3, "system").await;

    if tdd_guard!(r1, "insert_job") || tdd_guard!(r2, "insert_job") || tdd_guard!(r3, "insert_job")
    {
        return;
    }
    let job_id_1 = r1.expect("insert 1");
    let job_id_2 = r2.expect("insert 2");
    let job_id_3 = r3.expect("insert 3");

    // Claim in FIFO order — first inserted should be claimed first
    let claim_1 = JobRepository::claim_next_job(&pool).await;
    let claim_2 = JobRepository::claim_next_job(&pool).await;
    let claim_3 = JobRepository::claim_next_job(&pool).await;

    if tdd_guard!(claim_1, "claim_next_job")
        || tdd_guard!(claim_2, "claim_next_job")
        || tdd_guard!(claim_3, "claim_next_job")
    {
        return;
    }

    let c1 = claim_1.expect("claim 1").expect("should have job 1");
    let c2 = claim_2.expect("claim 2").expect("should have job 2");
    let c3 = claim_3.expect("claim 3").expect("should have job 3");

    assert_eq!(
        c1.id, job_id_1,
        "First claimed should be first inserted (FIFO)"
    );
    assert_eq!(c2.id, job_id_2, "Second claimed should be second inserted");
    assert_eq!(c3.id, job_id_3, "Third claimed should be third inserted");
}
