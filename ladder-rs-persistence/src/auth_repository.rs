//! Auth repository for user authentication and authorization.
//!
//! Manages the `users` table, login attempts, invite tokens,
//! and password hashing (argon2id).

use crate::{PersistenceError, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use std::time::Duration;

/// Repository for authentication and authorization operations.
pub struct AuthRepository;

/// Represents a user record.
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub force_password_change: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// An invite token for player account linking.
#[derive(Debug, Clone)]
pub struct InviteToken {
    pub id: String,
    pub token_hash: String,
    pub player_id: String,
    pub created_by: String,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<chrono::DateTime<Utc>>,
    pub expires_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
}

impl AuthRepository {
    // ── User CRUD ──

    /// Create a new user.
    pub async fn create_user(
        _pool: &SqlitePool,
        _username: &str,
        _email: &str,
        _password_hash: &str,
        _role: &str,
    ) -> Result<User> {
        Err(PersistenceError::Unknown(
            "create_user not yet implemented".into(),
        ))
    }

    /// Get a user by username.
    pub async fn get_user_by_username(_pool: &SqlitePool, _username: &str) -> Result<Option<User>> {
        Err(PersistenceError::Unknown(
            "get_user_by_username not yet implemented".into(),
        ))
    }

    /// Get a user by email.
    pub async fn get_user_by_email(_pool: &SqlitePool, _email: &str) -> Result<Option<User>> {
        Err(PersistenceError::Unknown(
            "get_user_by_email not yet implemented".into(),
        ))
    }

    /// Get a user by ID.
    pub async fn get_user(_pool: &SqlitePool, _id: &str) -> Result<Option<User>> {
        Err(PersistenceError::Unknown(
            "get_user not yet implemented".into(),
        ))
    }

    /// Set a user's password hash and optionally force a change on next login.
    pub async fn set_password(
        _pool: &SqlitePool,
        _user_id: &str,
        _password_hash: &str,
        _force_change: bool,
    ) -> Result<()> {
        Err(PersistenceError::Unknown(
            "set_password not yet implemented".into(),
        ))
    }

    /// Clear the force-password-change flag.
    pub async fn clear_force_change(_pool: &SqlitePool, _user_id: &str) -> Result<()> {
        Err(PersistenceError::Unknown(
            "clear_force_change not yet implemented".into(),
        ))
    }

    /// Check if the users table is empty (for first-run bootstrap).
    pub async fn is_users_table_empty(pool: &SqlitePool) -> Result<bool> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(row.0 == 0)
    }

    // ── Roles ──

    /// Get a user's role.
    pub async fn get_user_role(_pool: &SqlitePool, _user_id: &str) -> Result<String> {
        Err(PersistenceError::Unknown(
            "get_user_role not yet implemented".into(),
        ))
    }

    /// Get a user's league operator assignments.
    pub async fn get_league_assignments(_pool: &SqlitePool, _user_id: &str) -> Result<Vec<String>> {
        Err(PersistenceError::Unknown(
            "get_league_assignments not yet implemented".into(),
        ))
    }

    // ── Login Rate Limiting ──

    /// Record a login attempt (success or failure).
    pub async fn record_attempt(_pool: &SqlitePool, _user_id: &str, _success: bool) -> Result<()> {
        Err(PersistenceError::Unknown(
            "record_attempt not yet implemented".into(),
        ))
    }

    /// Count consecutive login failures within a time window.
    pub async fn consecutive_failures(
        _pool: &SqlitePool,
        _user_id: &str,
        _window: Duration,
    ) -> Result<u32> {
        Err(PersistenceError::Unknown(
            "consecutive_failures not yet implemented".into(),
        ))
    }

    /// Check if a user is locked out (>10 failures in 15 min).
    pub async fn is_locked_out(_pool: &SqlitePool, _user_id: &str) -> Result<bool> {
        Err(PersistenceError::Unknown(
            "is_locked_out not yet implemented".into(),
        ))
    }

    // ── Invite Tokens ──

    /// Create an invite token for player account linking.
    pub async fn create_invite_token(
        _pool: &SqlitePool,
        _player_id: &str,
        _created_by: &str,
    ) -> Result<InviteToken> {
        Err(PersistenceError::Unknown(
            "create_invite_token not yet implemented".into(),
        ))
    }

    /// Claim an invite token (links player to user).
    /// Returns the player_id if successful, None if token expired or already claimed.
    pub async fn claim_invite_token(
        _pool: &SqlitePool,
        _token_hash: &str,
        _claiming_user_id: &str,
    ) -> Result<Option<String>> {
        Err(PersistenceError::Unknown(
            "claim_invite_token not yet implemented".into(),
        ))
    }

    /// Get an invite token by hash.
    pub async fn get_invite_token(
        _pool: &SqlitePool,
        _token_hash: &str,
    ) -> Result<Option<InviteToken>> {
        Err(PersistenceError::Unknown(
            "get_invite_token not yet implemented".into(),
        ))
    }
}

/// Admin bootstrap: detect first-run and generate initial credentials.
pub struct AdminBootstrap;

/// Credentials from bootstrap.
#[derive(Debug, Clone)]
pub struct BootstrapCredentials {
    pub user_id: String,
    pub username: String,
    pub password: String,
}

impl AdminBootstrap {
    /// Run bootstrap detection. Returns credentials if a new admin was created.
    pub async fn run(pool: &SqlitePool) -> Result<Option<BootstrapCredentials>> {
        let is_empty = AuthRepository::is_users_table_empty(pool).await?;

        if !is_empty {
            return Ok(None);
        }

        let password = format!(
            "{}-{}",
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string()
        )
        .replace('-', "");

        let password_hash = format!("bootstrap:admin@local:{}", password);
        let now = Utc::now().to_rfc3339();
        let user_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, role, force_password_change, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'admin', 1, ?, ?)",
        )
        .bind(&user_id)
        .bind("admin")
        .bind("admin@local")
        .bind(&password_hash)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(Some(BootstrapCredentials {
            user_id,
            username: "admin".to_string(),
            password,
        }))
    }
}
