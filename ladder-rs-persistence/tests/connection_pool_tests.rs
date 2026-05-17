//! Connection pool initialization tests for ladder-rs-persistence
//!
//! These tests verify that SqlitePool is configured correctly for the ladder-rs
//! persistence layer, including:
//! - WAL journal mode for crash recovery
//! - Foreign key enforcement
//! - Busy timeout for concurrent access
//! - Pool sizing configuration
//! - Connection health
//! - Error handling for invalid URLs
//!
//! These tests serve as acceptance criteria for tasks 907.3.1 and 907.3.2.

use ladder_rs_persistence::pool::{
    acquire_connection, create_pool, create_pool_with_config, PoolConfig,
};

/// Creates a temporary file path for a test database.
/// Returns the path and the sqlite:// URL.
fn temp_db_path() -> (std::path::PathBuf, String) {
    let temp_dir = std::env::temp_dir();
    let filename = format!(
        "ladder_rs_test_{}_{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = temp_dir.join(&filename);
    // Use sqlite: prefix for absolute paths (not sqlite:// which expects host)
    let url = format!("sqlite:{}", path.display());
    (path, url)
}

/// Clean up temp database files (main, -wal, -shm).
fn cleanup_db(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

// ============================================================================
// TEST: Pool Creation
// ============================================================================

#[tokio::test]
async fn test_create_pool_success() {
    let pool = create_pool("sqlite::memory:").await;
    assert!(
        pool.is_ok(),
        "create_pool should succeed with valid in-memory URL"
    );
}

#[tokio::test]
async fn test_create_pool_returns_usable_pool() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    // Verify we can execute a simple query
    let result: Result<(i64,), sqlx::Error> = sqlx::query_as("SELECT 1").fetch_one(&pool).await;
    assert!(
        result.is_ok(),
        "Should be able to execute query on created pool"
    );
    assert_eq!(result.unwrap().0, 1);
}

// ============================================================================
// TEST: WAL Mode
// ============================================================================

#[tokio::test]
async fn test_pool_uses_wal_journal_mode() {
    // WAL mode requires a file-based database (in-memory always uses "memory" journal mode)
    let (path, url) = temp_db_path();
    let _guard = scopeguard::guard(&path, |p| cleanup_db(p));

    let pool = create_pool(&url).await.expect("Pool creation failed");

    let row: (String,) = sqlx::query_as("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .expect("Failed to query journal_mode");

    assert_eq!(
        row.0.to_lowercase(),
        "wal",
        "journal_mode should be 'wal', got: '{}'",
        row.0
    );
}

// ============================================================================
// TEST: Foreign Keys
// ============================================================================

#[tokio::test]
async fn test_pool_has_foreign_keys_enabled() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    let row: (i64,) = sqlx::query_as("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .expect("Failed to query foreign_keys");

    assert_eq!(
        row.0, 1,
        "foreign_keys should be enabled (1), got: {}",
        row.0
    );
}

#[tokio::test]
async fn test_foreign_keys_actually_enforced() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    // Create tables with a foreign key relationship
    sqlx::query(
        "CREATE TABLE parent (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create parent table");

    sqlx::query(
        "CREATE TABLE child (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER NOT NULL REFERENCES parent(id),
            name TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create child table");

    // Insert a valid parent
    sqlx::query("INSERT INTO parent (id, name) VALUES (1, 'test')")
        .execute(&pool)
        .await
        .expect("Failed to insert parent");

    // Try to insert a child with a non-existent parent - should fail if FKs are enforced
    let result = sqlx::query("INSERT INTO child (id, parent_id, name) VALUES (1, 999, 'orphan')")
        .execute(&pool)
        .await;

    assert!(
        result.is_err(),
        "Foreign key constraint should prevent inserting child with non-existent parent"
    );
}

// ============================================================================
// TEST: Busy Timeout
// ============================================================================

#[tokio::test]
async fn test_pool_has_busy_timeout_set() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    let row: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .expect("Failed to query busy_timeout");

    assert!(row.0 > 0, "busy_timeout should be non-zero, got: {}", row.0);
}

#[tokio::test]
async fn test_pool_busy_timeout_matches_config() {
    let config = PoolConfig {
        max_connections: 1,
        min_connections: 1,
        busy_timeout_ms: 3000,
    };
    let pool = create_pool_with_config("sqlite::memory:", &config)
        .await
        .expect("Pool creation failed");

    let row: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .expect("Failed to query busy_timeout");

    assert_eq!(
        row.0, 3000,
        "busy_timeout should be 3000ms as configured, got: {}",
        row.0
    );
}

#[tokio::test]
async fn test_pool_default_busy_timeout() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    let row: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .expect("Failed to query busy_timeout");

    // Default is 5000ms
    assert_eq!(
        row.0, 5000,
        "Default busy_timeout should be 5000ms, got: {}",
        row.0
    );
}

// ============================================================================
// TEST: Pool Sizing
// ============================================================================

#[tokio::test]
async fn test_pool_config_defaults() {
    let config = PoolConfig::default();

    assert_eq!(
        config.max_connections, 10,
        "Default max_connections should be 10"
    );
    assert_eq!(
        config.min_connections, 1,
        "Default min_connections should be 1"
    );
    assert_eq!(
        config.busy_timeout_ms, 5000,
        "Default busy_timeout_ms should be 5000"
    );
}

#[tokio::test]
async fn test_pool_with_custom_max_connections() {
    let config = PoolConfig {
        max_connections: 5,
        min_connections: 1,
        busy_timeout_ms: 5000,
    };
    let pool = create_pool_with_config("sqlite::memory:", &config)
        .await
        .expect("Pool creation failed");

    // Verify the pool works with custom max connections
    let result: Result<(i64,), sqlx::Error> = sqlx::query_as("SELECT 1").fetch_one(&pool).await;
    assert!(
        result.is_ok(),
        "Pool with custom max_connections should work"
    );
}

#[tokio::test]
async fn test_pool_with_custom_min_connections() {
    let config = PoolConfig {
        max_connections: 10,
        min_connections: 2,
        busy_timeout_ms: 5000,
    };
    let pool = create_pool_with_config("sqlite::memory:", &config)
        .await
        .expect("Pool creation failed");

    let result: Result<(i64,), sqlx::Error> = sqlx::query_as("SELECT 1").fetch_one(&pool).await;
    assert!(
        result.is_ok(),
        "Pool with custom min_connections should work"
    );
}

// ============================================================================
// TEST: Connection Health
// ============================================================================

#[tokio::test]
async fn test_acquire_connection_from_pool() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    let conn = acquire_connection(&pool).await;
    assert!(
        conn.is_ok(),
        "Should be able to acquire connection from pool"
    );
}

#[tokio::test]
async fn test_connection_can_execute_query() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    let mut conn = acquire_connection(&pool)
        .await
        .expect("Failed to acquire connection");

    let result: (i64,) = sqlx::query_as("SELECT 42")
        .fetch_one(&mut *conn)
        .await
        .expect("Failed to execute query");

    assert_eq!(result.0, 42, "Query should return expected value");
}

#[tokio::test]
async fn test_connection_can_create_and_query_table() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Pool creation failed");

    let mut conn = acquire_connection(&pool)
        .await
        .expect("Failed to acquire connection");

    sqlx::query("CREATE TABLE test_health (id INTEGER PRIMARY KEY, value TEXT)")
        .execute(&mut *conn)
        .await
        .expect("Failed to create table");

    sqlx::query("INSERT INTO test_health (id, value) VALUES (1, 'hello')")
        .execute(&mut *conn)
        .await
        .expect("Failed to insert row");

    let row: (String,) = sqlx::query_as("SELECT value FROM test_health WHERE id = 1")
        .fetch_one(&mut *conn)
        .await
        .expect("Failed to query row");

    assert_eq!(row.0, "hello", "Should retrieve inserted value");
}

// ============================================================================
// TEST: Error Handling
// ============================================================================

#[tokio::test]
async fn test_create_pool_empty_url_returns_error() {
    let result = create_pool("").await;
    assert!(result.is_err(), "create_pool should fail with empty URL");
}

#[tokio::test]
async fn test_create_pool_non_sqlite_scheme_returns_error() {
    let bad_urls = [
        "http://not-sqlite.db",
        "postgresql://not-sqlite.db",
        "mysql://not-sqlite.db",
    ];

    for url in &bad_urls {
        let result = create_pool(url).await;
        assert!(
            result.is_err(),
            "create_pool should fail with non-sqlite URL: {}",
            url
        );
    }
}

#[tokio::test]
async fn test_create_pool_file_in_nonexistent_dir_returns_error() {
    // A file path in a directory that doesn't exist should fail
    let url = "sqlite:///nonexistent_dir_12345/subdir/test.db";
    let result = create_pool(url).await;
    assert!(
        result.is_err(),
        "create_pool should fail when parent directory doesn't exist"
    );
}

// ============================================================================
// TEST: Temp File Database
// ============================================================================

#[tokio::test]
async fn test_create_pool_with_temp_file() {
    let (path, url) = temp_db_path();
    let _guard = scopeguard::guard(&path, |p| cleanup_db(p));

    let pool = create_pool(&url).await;
    assert!(
        pool.is_ok(),
        "create_pool should succeed with temp file URL: {:?}",
        pool
    );
}

#[tokio::test]
async fn test_temp_file_pool_has_wal_mode() {
    let (path, url) = temp_db_path();
    let _guard = scopeguard::guard(&path, |p| cleanup_db(p));

    let pool = create_pool(&url).await.expect("Pool creation failed");

    let row: (String,) = sqlx::query_as("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .expect("Failed to query journal_mode");

    assert_eq!(
        row.0.to_lowercase(),
        "wal",
        "File-based pool should use WAL mode, got: '{}'",
        row.0
    );
}

#[tokio::test]
async fn test_temp_file_pool_has_foreign_keys() {
    let (path, url) = temp_db_path();
    let _guard = scopeguard::guard(&path, |p| cleanup_db(p));

    let pool = create_pool(&url).await.expect("Pool creation failed");

    let row: (i64,) = sqlx::query_as("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .expect("Failed to query foreign_keys");

    assert_eq!(
        row.0, 1,
        "File-based pool should have foreign keys enabled, got: {}",
        row.0
    );
}

#[tokio::test]
async fn test_temp_file_pool_has_busy_timeout() {
    let (path, url) = temp_db_path();
    let _guard = scopeguard::guard(&path, |p| cleanup_db(p));

    let pool = create_pool(&url).await.expect("Pool creation failed");

    let row: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .expect("Failed to query busy_timeout");

    assert_eq!(
        row.0, 5000,
        "File-based pool should have default busy_timeout of 5000ms, got: {}",
        row.0
    );
}
