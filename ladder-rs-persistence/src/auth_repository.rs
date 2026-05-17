//! Auth repository and admin bootstrap for first-run setup.
//!
//! Provides `is_users_table_empty` detection and admin credential generation.
//! Full authentication features (argon2id, rate limiting, invite tokens)
//! are implemented in the Auth Repository task (907.4.7).

use crate::{PersistenceError, Result};
use sqlx::SqlitePool;

/// Repository for authentication and authorization operations.
pub struct AuthRepository;

impl AuthRepository {
    /// Check if the users table is empty (for first-run bootstrap detection).
    pub async fn is_users_table_empty(pool: &SqlitePool) -> Result<bool> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(row.0 == 0)
    }

    /// Create the initial admin user during first-run bootstrap.
    /// Returns the user ID.
    pub async fn create_admin_user(
        pool: &SqlitePool,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<String> {
        if username.is_empty() || email.is_empty() || password_hash.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "username, email, and password_hash must not be empty".into(),
            ));
        }

        let user_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, role, force_password_change, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'admin', 1, ?, ?)",
        )
        .bind(&user_id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(user_id)
    }
}

/// Credentials generated during first-run admin bootstrap.
#[derive(Debug, Clone)]
pub struct BootstrapCredentials {
    pub user_id: String,
    pub username: String,
    /// Plaintext password — display to operator once, then discard.
    pub password: String,
}

/// Admin bootstrap: detect first-run and generate initial credentials.
///
/// Called at server startup. If the users table is empty, generates an
/// admin credential pair, creates the admin user in the database, and
/// returns the credentials for stdout display.
pub struct AdminBootstrap;

impl AdminBootstrap {
    /// Run bootstrap detection. Returns credentials if a new admin was created,
    /// or None if users already exist.
    pub async fn run(pool: &SqlitePool) -> Result<Option<BootstrapCredentials>> {
        let is_empty = AuthRepository::is_users_table_empty(pool).await?;

        if !is_empty {
            return Ok(None);
        }

        // Generate a random password from UUIDs (128 bits of entropy, no deps needed)
        let password = format!(
            "{}-{}",
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string()
        )
        .replace('-', ""); // 64 hex chars = 256 bits

        // Placeholder hash for bootstrap — the real Auth Repository (907.4.7)
        // will replace this with argon2id hashing.
        let password_hash = format!("bootstrap:{}:{}", "admin@local", password);

        let user_id =
            AuthRepository::create_admin_user(pool, "admin", "admin@local", &password_hash).await?;

        Ok(Some(BootstrapCredentials {
            user_id,
            username: "admin".to_string(),
            password,
        }))
    }
}
