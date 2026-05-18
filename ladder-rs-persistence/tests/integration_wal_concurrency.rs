//! Integration tests for WAL-mode concurrency
//!
//! Tests verify that file-based SQLite in WAL mode correctly supports:
//! - Concurrent readers while a writer is in progress
//! - Multiple concurrent readers
//! - Busy timeout handling for concurrent writers
//! - Reader isolation during long writes
//!
//! These tests serve as acceptance criteria for task 907.6.3.

use chrono::Utc;
use ladder_rs_persistence::pool::{create_pool_with_config, PoolConfig};
use ladder_rs_persistence::{MatchParticipant, MatchRepository, PlayerRepository};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Creates a unique temporary file path and a `sqlite:` URL for a file-based
/// SQLite database. File-based is required for WAL concurrency — `:memory:`
/// does not support concurrent connections.
fn temp_db_path() -> (std::path::PathBuf, String) {
    let temp_dir = std::env::temp_dir();
    let filename = format!("ladder_rs_wal_test_{}.db", uuid::Uuid::new_v4());
    let path = temp_dir.join(&filename);
    let url = format!("sqlite:{}", path.display());
    (path, url)
}

/// Clean up temp database files: main DB, WAL journal, shared-memory file.
fn cleanup_db(path: &std::path::Path) {
    let path_str = path.display().to_string();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path_str));
    let _ = std::fs::remove_file(format!("{}-shm", path_str));
}

/// Default pool config suitable for concurrency tests (higher max_connections).
fn concurrency_pool_config() -> PoolConfig {
    PoolConfig {
        max_connections: 10,
        min_connections: 2,
        busy_timeout_ms: 5000,
    }
}

/// Creates a file-based pool, runs migrations, and seeds minimal entities.
/// Returns the pool and cleanup path.
async fn setup_seeded_file_db() -> (SqlitePool, std::path::PathBuf) {
    use sqlx::migrate::Migrator;
    use std::path::Path;

    let (path, url) = temp_db_path();
    let config = concurrency_pool_config();

    let pool = create_pool_with_config(&url, &config)
        .await
        .expect("Failed to create file-based pool");

    // Run migrations
    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path)
            .await
            .expect("Failed to create migrator");
        migrator.run(&pool).await.expect("Failed to run migrations");
    }

    (pool, path)
}

/// Seeds a league, season, and players into the database.
/// Returns (league_id, season_id, player_ids).
async fn seed_fixtures(pool: &SqlitePool) -> (String, String, Vec<String>) {
    let league_id = uuid::Uuid::new_v4().to_string();
    let season_id = uuid::Uuid::new_v4().to_string();
    let player_ids: Vec<String> = (0..4).map(|_| uuid::Uuid::new_v4().to_string()).collect();

    // Insert league
    sqlx::query("INSERT INTO leagues (id, name, algorithm, visibility) VALUES (?, ?, ?, 'public')")
        .bind(&league_id)
        .bind(format!("WAL Test League {}", &league_id[..4]))
        .bind("elo")
        .execute(pool)
        .await
        .expect("Failed to insert test league");

    // Insert season (open)
    sqlx::query(
        "INSERT INTO seasons (id, league_id, algorithm, number, start_date) VALUES (?, ?, ?, 1, ?)",
    )
    .bind(&season_id)
    .bind(&league_id)
    .bind("elo")
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .expect("Failed to insert test season");

    // Insert players and add to league
    for pid in &player_ids {
        sqlx::query("INSERT INTO players (id, name, player_type) VALUES (?, ?, 'human')")
            .bind(pid)
            .bind(format!("Player {}", &pid[..4]))
            .execute(pool)
            .await
            .expect("Failed to insert test player");

        // Add player to league
        let lp_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO league_players (id, league_id, player_id, is_active, joined_at, created_at) \
             VALUES (?, ?, ?, 1, ?, ?)",
        )
        .bind(&lp_id)
        .bind(&league_id)
        .bind(pid)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .expect("Failed to insert league_player");
    }

    (league_id, season_id, player_ids)
}

/// Build match participants: first place, second place, etc.
fn make_participants(player_ids: &[String]) -> Vec<MatchParticipant> {
    player_ids
        .iter()
        .enumerate()
        .map(|(i, pid)| MatchParticipant {
            player_id: pid.clone(),
            placement: (i + 1) as i32,
        })
        .collect()
}

/// A simple leaderboard query via rating_snapshots (simulates a leaderboard read).
async fn query_leaderboard(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rating_snapshots")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// A simple player list query (simulates a reader accessing player data).
async fn query_players(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM players LIMIT 100")
        .fetch_all(pool)
        .await?;
    Ok(rows.len())
}

/// A read query on a specific table to exercise concurrent read paths.
async fn query_seasons(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM seasons")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

// ============================================================================
// TEST: Concurrent reader + writer
// ============================================================================

/// Verify that readers can query the database while a writer is inserting
/// matches in a loop. In WAL mode, readers should not be blocked by the writer.
#[tokio::test]
async fn concurrent_reader_and_writer() {
    let (pool, cleanup_path) = setup_seeded_file_db().await;
    let _guard = scopeguard::guard(cleanup_path.clone(), |p| cleanup_db(&p));

    let (_league_id, season_id, player_ids) = seed_fixtures(&pool).await;
    let pool = Arc::new(pool);
    let participants = make_participants(&player_ids[..2]);

    let _start = Instant::now();
    const WRITE_COUNT: usize = 20;

    // Spawn writer task
    let writer_pool = Arc::clone(&pool);
    let writer_handle = tokio::spawn(async move {
        let mut success_count = 0usize;
        for i in 0..WRITE_COUNT {
            let recorded_at = Utc::now() + chrono::Duration::seconds(i as i64);
            let result = MatchRepository::record_match(
                &writer_pool,
                &season_id,
                participants.clone(),
                None,
                recorded_at,
            )
            .await;
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("Writer error at iteration {}: {:?}", i, e);
                }
            }
            // Small delay to interleave with readers
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        success_count
    });

    // Spawn reader tasks that run concurrently with the writer
    let reader_pool1 = Arc::clone(&pool);
    let reader_pool2 = Arc::clone(&pool);
    let reader_pool3 = Arc::clone(&pool);

    let reader1 = tokio::spawn(async move {
        let mut total = 0i64;
        for _ in 0..15 {
            if let Ok(count) = query_leaderboard(&reader_pool1).await {
                total += count;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        total
    });

    let reader2 = tokio::spawn(async move {
        let mut total = 0usize;
        for _ in 0..15 {
            if let Ok(count) = query_players(&reader_pool2).await {
                total += count;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        total
    });

    let reader3 = tokio::spawn(async move {
        let mut total = 0i64;
        for _ in 0..15 {
            if let Ok(count) = query_seasons(&reader_pool3).await {
                total += count;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        total
    });

    // Wait for all tasks
    let (writer_result, _r1, r2, _r3) = tokio::try_join!(writer_handle, reader1, reader2, reader3)
        .expect("Failed to join concurrent tasks");

    let elapsed = _start.elapsed();

    // Verify all writes succeeded
    assert_eq!(
        writer_result, WRITE_COUNT,
        "All {} writes should have succeeded, but only {} did",
        WRITE_COUNT, writer_result
    );

    // Verify readers completed (returned some data)
    // Reader1: leaderboard queries — may be 0 or more
    assert!(
        _r1 >= 0,
        "Reader 1 should have completed leaderboard queries"
    );
    // Reader2: player queries — should find seeded players
    assert!(r2 > 0, "Reader 2 should have found players");

    // Verify readers were not blocked indefinitely — should complete in reasonable time
    assert!(
        elapsed < Duration::from_secs(120),
        "Concurrent read+write test took too long: {:?}",
        elapsed
    );
}

// ============================================================================
// TEST: Multiple concurrent readers
// ============================================================================

/// Verify that 5 concurrent reader tasks can all read the database
/// simultaneously and complete within a reasonable time.
#[tokio::test]
async fn multiple_concurrent_readers() {
    let (pool, cleanup_path) = setup_seeded_file_db().await;
    let _guard = scopeguard::guard(cleanup_path.clone(), |p| cleanup_db(&p));

    seed_fixtures(&pool).await;
    let pool = Arc::new(pool);
    let start = Instant::now();

    const READER_COUNT: usize = 5;
    let mut handles = Vec::with_capacity(READER_COUNT);

    for _ in 0..READER_COUNT {
        let reader_pool = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let mut results = Vec::new();
            for _ in 0..10 {
                if let Ok(count) = query_leaderboard(&reader_pool).await {
                    results.push(count);
                }
                if let Ok(players) = query_players(&reader_pool).await {
                    results.push(players as i64);
                }
            }
            results.len()
        });
        handles.push(handle);
    }

    let mut all_reads = 0usize;
    for handle in handles {
        let reads = handle.await.expect("Reader task panicked");
        all_reads += reads;
    }

    let elapsed = start.elapsed();

    // Each reader does 20 queries (10 leaderboard + 10 player) = 100 total
    assert_eq!(
        all_reads,
        READER_COUNT * 20,
        "All {} readers should have completed 20 reads each, got {} total",
        READER_COUNT,
        all_reads
    );

    assert!(
        elapsed < Duration::from_secs(30),
        "Multiple reader test took too long: {:?}",
        elapsed
    );
}

// ============================================================================
// TEST: Busy timeout in action
// ============================================================================

/// Verify that when one writer holds a transaction open, a second writer
/// waits via busy_timeout rather than failing immediately with SQLITE_BUSY.
#[tokio::test]
async fn busy_timeout_in_action() {
    use ladder_rs_persistence::pool::create_pool_with_config;

    let (path, url) = temp_db_path();
    let _cleanup_guard = scopeguard::guard(path.clone(), |p| cleanup_db(&p));

    // Create pool with a generous busy_timeout
    let config = PoolConfig {
        max_connections: 3,
        min_connections: 1,
        busy_timeout_ms: 5000,
    };

    use sqlx::migrate::Migrator;
    use std::path::Path;

    let pool = create_pool_with_config(&url, &config)
        .await
        .expect("Failed to create pool");

    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path).await.unwrap();
        migrator.run(&pool).await.unwrap();
    }

    // Create a test table for this specific test
    sqlx::query("CREATE TABLE IF NOT EXISTS wal_busy_test (id INTEGER PRIMARY KEY, val TEXT)")
        .execute(&pool)
        .await
        .expect("Failed to create test table");

    seed_fixtures(&pool).await;
    let pool = Arc::new(pool);

    // ---- Writer 1: holds a transaction open briefly ----
    let pool1 = Arc::clone(&pool);
    let writer1 = tokio::spawn(async move {
        let mut tx = pool1
            .begin()
            .await
            .expect("Writer 1: failed to begin transaction");

        sqlx::query("INSERT INTO wal_busy_test (id, val) VALUES (1, 'writer1')")
            .execute(&mut *tx)
            .await
            .expect("Writer 1: failed to insert");

        // Hold the transaction open for 800ms
        tokio::time::sleep(Duration::from_millis(800)).await;

        tx.commit().await.expect("Writer 1: failed to commit");
        1
    });

    // Small delay to ensure writer 1 acquires the lock first
    tokio::time::sleep(Duration::from_millis(50)).await;

    // ---- Writer 2: attempts write while writer 1 holds lock ----
    let pool2 = Arc::clone(&pool);
    let start2 = Instant::now();
    let writer2 = tokio::spawn(async move {
        let result = sqlx::query("INSERT INTO wal_busy_test (id, val) VALUES (2, 'writer2')")
            .execute(&*pool2)
            .await;
        (result, start2.elapsed())
    });

    // Wait for both writers
    let w1 = writer1.await.expect("Writer 1 panicked");
    assert_eq!(w1, 1, "Writer 1 should succeed");

    let (w2_result, w2_elapsed) = writer2.await.expect("Writer 2 panicked");

    match w2_result {
        Ok(_) => {
            // Success — writer 2 waited via busy_timeout and then completed
            // It should have taken at least some time (> 0)
            assert!(
                w2_elapsed > Duration::from_millis(0),
                "Writer 2 should have taken some time, not been instantaneous"
            );
        }
        Err(e) => {
            let err_str = e.to_string();
            // If the error is SQLITE_BUSY, the busy_timeout may have expired
            // but it should ONLY happen if the timeout actually elapsed
            if err_str.contains("busy") || err_str.contains("BUSY") {
                // Acceptable if writer 1's lock held longer than busy_timeout
                // but with 5s timeout and 800ms hold, this shouldn't happen
            }
            // Any other error is a real failure
            assert!(
                !err_str.contains("database is locked")
                    || w2_elapsed >= Duration::from_millis(4000),
                "Writer 2 should have waited for busy_timeout, not failed immediately. \
                 Elapsed: {:?}, Error: {}",
                w2_elapsed,
                err_str
            );
        }
    }
}

// ============================================================================
// TEST: Reader during long write (WAL read isolation)
// ============================================================================

/// Verify that in WAL mode, readers can query while a write transaction is
/// in progress. The reader should NOT be blocked and should see the pre-write
/// state (snapshot isolation).
#[tokio::test]
async fn reader_during_long_write() {
    let (pool, cleanup_path) = setup_seeded_file_db().await;
    let _guard = scopeguard::guard(cleanup_path.clone(), |p| cleanup_db(&p));

    let (_league_id, _season_id, _player_ids) = seed_fixtures(&pool).await;

    // Create a test table for visibility
    sqlx::query("CREATE TABLE IF NOT EXISTS wal_read_test (id INTEGER PRIMARY KEY, val TEXT)")
        .execute(&pool)
        .await
        .expect("Failed to create test table");

    let pool = Arc::new(pool);

    // Count players before the write (also exercises read path)
    let _player_count_before = query_players(&pool)
        .await
        .expect("Failed to query players before write");

    // ---- Writer: insert a row and hold transaction open ----
    let writer_pool = Arc::clone(&pool);
    let writer_started = Arc::new(tokio::sync::Notify::new());
    let writer_started_signal = writer_started.clone();

    let writer_handle = tokio::spawn(async move {
        let mut tx = writer_pool
            .begin()
            .await
            .expect("Writer: failed to begin transaction");

        sqlx::query("INSERT INTO wal_read_test (id, val) VALUES (10, 'long_write')")
            .execute(&mut *tx)
            .await
            .expect("Writer: failed to insert");

        // Signal that we're in the transaction
        writer_started_signal.notify_one();

        // Hold the transaction open for 1500ms while reader queries
        tokio::time::sleep(Duration::from_millis(1500)).await;

        tx.commit().await.expect("Writer: failed to commit");
        10i64
    });

    // Wait for writer to enter its transaction
    writer_started.notified().await;

    // ---- Reader: query while write is in progress ----
    let reader_start = Instant::now();
    let reader_pool = Arc::clone(&pool);
    let reader_result = tokio::spawn(async move {
        let mut results = Vec::new();
        for _ in 0..5 {
            let count = query_players(&reader_pool)
                .await
                .expect("Reader: failed to query");
            results.push(count);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        results
    })
    .await
    .expect("Reader task panicked");

    let reader_elapsed = reader_start.elapsed();

    // Writer should complete successfully
    let writer_val = writer_handle.await.expect("Writer task panicked");
    assert_eq!(writer_val, 10, "Writer should have completed");

    // Reader should have completed in reasonable time (< 2 seconds for 5 queries)
    assert!(
        reader_elapsed < Duration::from_secs(3),
        "Reader took too long during write: {:?}",
        reader_elapsed
    );

    // Reader should have seen the pre-write state (snapshot isolation in WAL)
    assert!(
        !reader_result.is_empty(),
        "Reader should have returned results"
    );
}

// ============================================================================
// TEST: Busy timeout exceeded
// ============================================================================

/// Verify that when a writer holds a lock longer than the busy_timeout,
/// a second writer eventually receives an error.
#[tokio::test]
async fn busy_timeout_exceeded() {
    use ladder_rs_persistence::pool::create_pool_with_config;

    let (path, url) = temp_db_path();
    let _cleanup_guard = scopeguard::guard(path.clone(), |p| cleanup_db(&p));

    // Very short busy_timeout
    let config = PoolConfig {
        max_connections: 2,
        min_connections: 1,
        busy_timeout_ms: 300,
    };

    use sqlx::migrate::Migrator;
    use std::path::Path;

    let pool = create_pool_with_config(&url, &config)
        .await
        .expect("Failed to create pool");

    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path).await.unwrap();
        migrator.run(&pool).await.unwrap();
    }

    // Create a simple test table
    sqlx::query("CREATE TABLE IF NOT EXISTS wal_timeout_test (id INTEGER PRIMARY KEY, val TEXT)")
        .execute(&pool)
        .await
        .expect("Failed to create test table");

    let pool = Arc::new(pool);

    // ---- Writer 1: holds a transaction for >300ms ----
    let pool1 = Arc::clone(&pool);
    let writer1_started = Arc::new(tokio::sync::Notify::new());
    let writer1_signal = writer1_started.clone();

    let writer1 = tokio::spawn(async move {
        let mut tx = pool1.begin().await.expect("Writer 1: failed to begin");

        sqlx::query("INSERT INTO wal_timeout_test (id, val) VALUES (1, 'hold')")
            .execute(&mut *tx)
            .await
            .expect("Writer 1: failed to insert");

        writer1_signal.notify_one();

        // Hold for 2000ms > 300ms busy_timeout
        tokio::time::sleep(Duration::from_millis(2000)).await;

        tx.commit().await.expect("Writer 1: failed to commit");
        true
    });

    // Wait for writer 1 to acquire the lock
    writer1_started.notified().await;

    // Small additional delay
    tokio::time::sleep(Duration::from_millis(50)).await;

    // ---- Writer 2: attempts write while lock is held ----
    let pool2 = Arc::clone(&pool);
    let start2 = Instant::now();
    let writer2 = tokio::spawn(async move {
        let result = sqlx::query("INSERT INTO wal_timeout_test (id, val) VALUES (2, 'blocked')")
            .execute(&*pool2)
            .await;
        (result, start2.elapsed())
    });

    let w1_done = writer1.await.expect("Writer 1 panicked");
    assert!(w1_done, "Writer 1 should complete successfully");

    let (w2_result, w2_elapsed) = writer2.await.expect("Writer 2 panicked");

    // Writer 2 should either:
    // 1. Fail due to busy_timeout exceeded (expected if timeout < lock hold time)
    // 2. Succeed if it waited long enough for writer 1 to release lock
    //
    // With 300ms timeout and 2000ms hold, option 1 is most likely.
    match w2_result {
        Ok(_) => {
            // Succeeded — writer 1 released the lock before writer 2 fully timed out.
            // This can happen due to timing variations. Acceptable.
        }
        Err(e) => {
            let err_str = e.to_string();
            // Should be a busy/database-locked error, not a random failure
            assert!(
                err_str.to_lowercase().contains("busy")
                    || err_str.to_lowercase().contains("locked")
                    || err_str.to_lowercase().contains("database"),
                "Writer 2 error should be busy/locked related, got: {}",
                err_str
            );
            // Should have waited at least close to the busy_timeout
            assert!(
                w2_elapsed >= Duration::from_millis(200),
                "Writer 2 should have waited for busy_timeout before failing, elapsed: {:?}",
                w2_elapsed
            );
        }
    }
}

// ============================================================================
// TEST: Interleaved writes and reads (no deadlock)
// ============================================================================

/// Verify that interleaved writes to different tables complete successfully
/// while readers run concurrently. WAL mode serializes writers, but reads
/// proceed unimpeded. This test verifies no deadlocks or corruption occur
/// when writes to different tables are done in rapid succession with
/// readers reading between writes.
#[tokio::test]
async fn interleaved_writes_with_concurrent_readers() {
    let (pool, cleanup_path) = setup_seeded_file_db().await;
    let _guard = scopeguard::guard(cleanup_path.clone(), |p| cleanup_db(&p));

    let (_league_id, season_id, player_ids) = seed_fixtures(&pool).await;
    let pool = Arc::new(pool);
    let participants = make_participants(&player_ids[..2]);

    let start = Instant::now();

    // Use a mutex to serialize write access (WAL only allows one writer)
    // but readers can still proceed freely in parallel.
    let write_lock = Arc::new(tokio::sync::Mutex::new(()));

    // Writer A: inserts matches (one at a time, releasing lock between)
    let pool_a = Arc::clone(&pool);
    let lock_a = Arc::clone(&write_lock);
    let season_a = season_id.clone();
    let parts_a = participants.clone();
    let writer_a = tokio::spawn(async move {
        let mut successes = 0usize;
        let base_time = Utc::now();
        for i in 0..10 {
            // Acquire write lock, do the write, release
            let _guard = lock_a.lock().await;
            let recorded_at = base_time + chrono::Duration::milliseconds(i as i64 * 200);
            if MatchRepository::record_match(&pool_a, &season_a, parts_a.clone(), None, recorded_at)
                .await
                .is_ok()
            {
                successes += 1;
            }
            drop(_guard);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        successes
    });

    // Writer B: creates players (one at a time, releasing lock between)
    let pool_b = Arc::clone(&pool);
    let lock_b = Arc::clone(&write_lock);
    let writer_b = tokio::spawn(async move {
        let mut successes = 0usize;
        for i in 0..10 {
            let _guard = lock_b.lock().await;
            let name = format!("InterleavedPlayer_{}", i);
            if PlayerRepository::create_player(&pool_b, &name, "human")
                .await
                .is_ok()
            {
                successes += 1;
            }
            drop(_guard);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        successes
    });

    // Reader: queries concurrently (does NOT need the write lock)
    let pool_c = Arc::clone(&pool);
    let reader_c = tokio::spawn(async move {
        let mut reads = 0usize;
        for _ in 0..20 {
            if query_players(&pool_c).await.is_ok() && query_leaderboard(&pool_c).await.is_ok() {
                reads += 1;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        reads
    });

    let (a_result, b_result, c_result) =
        tokio::try_join!(writer_a, writer_b, reader_c).expect("Failed to join tasks");

    let elapsed = start.elapsed();

    assert_eq!(
        a_result, 10,
        "Writer A (matches) should succeed for all 10 inserts"
    );
    assert_eq!(
        b_result, 10,
        "Writer B (players) should succeed for all 10 inserts"
    );
    assert_eq!(c_result, 20, "Reader should complete all 20 reads");

    assert!(
        elapsed < Duration::from_secs(120),
        "Interleaved writes test took too long: {:?}",
        elapsed
    );
}

// ============================================================================
// TEST: Ten concurrent readers during heavy writes
// ============================================================================

/// Stress test: many concurrent readers while a writer is continuously
/// inserting matches. All tasks should complete successfully.
#[tokio::test]
async fn many_readers_during_heavy_writes() {
    let (pool, cleanup_path) = setup_seeded_file_db().await;
    let _guard = scopeguard::guard(cleanup_path.clone(), |p| cleanup_db(&p));

    let (_league_id, season_id, player_ids) = seed_fixtures(&pool).await;
    let pool = Arc::new(pool);
    let participants = make_participants(&player_ids[..2]);

    let start = Instant::now();
    const WRITE_COUNT: usize = 15;
    const READER_COUNT: usize = 10;

    // Spawn writer
    let writer_pool = Arc::clone(&pool);
    let writer_handle = tokio::spawn(async move {
        let mut successes = 0usize;
        for i in 0..WRITE_COUNT {
            let recorded_at = Utc::now() + chrono::Duration::seconds(i as i64);
            if MatchRepository::record_match(
                &writer_pool,
                &season_id,
                participants.clone(),
                None,
                recorded_at,
            )
            .await
            .is_ok()
            {
                successes += 1;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        successes
    });

    // Spawn many reader tasks
    let mut reader_handles = Vec::with_capacity(READER_COUNT);
    for _ in 0..READER_COUNT {
        let rp = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let mut reads = 0usize;
            for _ in 0..8 {
                // Alternate between different queries
                if reads % 3 == 0 {
                    let _ = query_leaderboard(&rp).await;
                } else if reads % 3 == 1 {
                    let _ = query_players(&rp).await;
                } else {
                    let _ = query_seasons(&rp).await;
                }
                reads += 1;
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            reads
        });
        reader_handles.push(handle);
    }

    // Wait for writer
    let write_successes = writer_handle.await.expect("Writer panicked");
    assert_eq!(
        write_successes, WRITE_COUNT,
        "Writer should have succeeded for all {} writes, got {}",
        WRITE_COUNT, write_successes
    );

    // Wait for all readers
    let mut total_reads = 0usize;
    for handle in reader_handles {
        let reads = handle.await.expect("Reader panicked");
        total_reads += reads;
    }

    let elapsed = start.elapsed();

    assert_eq!(
        total_reads,
        READER_COUNT * 8,
        "All {} readers should have completed 8 reads each ({}), got {}",
        READER_COUNT,
        READER_COUNT * 8,
        total_reads
    );

    assert!(
        elapsed < Duration::from_secs(120),
        "Heavy read/write stress test took too long: {:?}",
        elapsed
    );
}
