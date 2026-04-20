//! Connection pool creation and configuration for SQLite
//!
//! This module provides the primary entry point for creating database connections.
//! All pools are configured with:
//! - WAL journal mode for crash recovery and concurrent access
//! - Foreign key enforcement enabled
//! - Busy timeout for handling concurrent access between server and swarm operator
//! - Configurable min/max connection pool sizing

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;
use std::time::Duration;

use crate::error::PersistenceError;

/// Default busy timeout in milliseconds
pub const DEFAULT_BUSY_TIMEOUT_MS: u64 = 5000;

/// Default minimum connections
pub const DEFAULT_MIN_CONNECTIONS: u32 = 1;

/// Default maximum connections
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Configuration for the connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Busy timeout in milliseconds for SQLite
    pub busy_timeout_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            busy_timeout_ms: DEFAULT_BUSY_TIMEOUT_MS,
        }
    }
}

/// Creates a configured SQLite connection pool with WAL mode, foreign keys, and busy timeout.
///
/// The pool is configured with:
/// - WAL journal mode for crash recovery and concurrent read/write access
/// - Foreign key constraints enforced
/// - Busy timeout to handle concurrent access gracefully
/// - Configurable min/max connection limits
///
/// # Arguments
///
/// * `database_url` - SQLite connection string (e.g., `sqlite::memory:` or `sqlite:///path/to/db`)
///
/// # Returns
///
/// A configured `SqlitePool` or a `PersistenceError` if the URL is invalid or connection fails.
///
/// # Example
///
/// ```no_run
/// use ladder_rs_persistence::pool::create_pool;
///
/// #[tokio::main]
/// async fn main() {
///     let pool = create_pool("sqlite::memory:").await.expect("Failed to create pool");
/// }
/// ```
pub async fn create_pool(database_url: &str) -> Result<Pool<Sqlite>, PersistenceError> {
    let config = PoolConfig::default();
    create_pool_with_config(database_url, &config).await
}

/// Creates a configured SQLite connection pool with custom configuration.
///
/// See [`create_pool`] for details on the pool configuration.
///
/// # Arguments
///
/// * `database_url` - SQLite connection string
/// * `config` - Custom pool configuration
pub async fn create_pool_with_config(
    database_url: &str,
    config: &PoolConfig,
) -> Result<Pool<Sqlite>, PersistenceError> {
    if database_url.is_empty() {
        return Err(PersistenceError::InvalidInput(
            "Database URL cannot be empty".to_string(),
        ));
    }

    let mut connect_options = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| PersistenceError::InvalidInput(format!("Invalid database URL: {}", e)))?
        .create_if_missing(true);

    // Apply PRAGMA settings via connect options so they apply to every connection
    let busy_timeout_value = config.busy_timeout_ms.to_string();
    connect_options = connect_options
        .pragma("journal_mode", "WAL")
        .pragma("foreign_keys", "ON")
        .pragma("busy_timeout", busy_timeout_value);

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Some(Duration::from_secs(600)))
        .connect_with(connect_options)
        .await
        .map_err(|e| PersistenceError::DatabaseError(format!("Failed to create pool: {}", e)))?;

    Ok(pool)
}

/// Acquires a single connection from the pool for health checking or direct use.
pub async fn acquire_connection(
    pool: &Pool<Sqlite>,
) -> Result<sqlx::pool::PoolConnection<Sqlite>, PersistenceError> {
    pool.acquire().await.map_err(|e| {
        PersistenceError::DatabaseError(format!("Failed to acquire connection: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool_memory() {
        let pool = create_pool("sqlite::memory:").await;
        assert!(pool.is_ok(), "Failed to create in-memory pool: {:?}", pool);
    }

    #[tokio::test]
    async fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(config.min_connections, DEFAULT_MIN_CONNECTIONS);
        assert_eq!(config.busy_timeout_ms, DEFAULT_BUSY_TIMEOUT_MS);
    }

    #[tokio::test]
    async fn test_create_pool_with_custom_config() {
        let config = PoolConfig {
            max_connections: 5,
            min_connections: 1,
            busy_timeout_ms: 3000,
        };
        let pool = create_pool_with_config("sqlite::memory:", &config).await;
        assert!(
            pool.is_ok(),
            "Failed to create pool with custom config: {:?}",
            pool
        );
    }
}
