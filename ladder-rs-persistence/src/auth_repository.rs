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

// ── Helper: parse RFC 3339 text column to DateTime<Utc> ──
fn parse_dt(s: &str) -> Result<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| PersistenceError::DatabaseError(format!("Invalid datetime '{}': {}", s, e)))
}

// ── Helper: read a user row tuple ──
type UserRow = (String, String, String, String, i64, String, String);

fn row_to_user(row: UserRow) -> Result<User> {
    let (id, username, email, role, force_change, created_at, updated_at) = row;
    Ok(User {
        id,
        username,
        email,
        role,
        force_password_change: force_change != 0,
        created_at: parse_dt(&created_at)?,
        updated_at: parse_dt(&updated_at)?,
    })
}

// ── Helper: invite token row tuple ──
type InviteTokenRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
);

fn row_to_invite_token(row: InviteTokenRow) -> Result<InviteToken> {
    let (id, player_id, token_hash, created_by, claimed_by, claimed_at, expires_at, created_at) =
        row;
    Ok(InviteToken {
        id,
        token_hash,
        player_id,
        created_by,
        claimed_by,
        claimed_at: claimed_at.map(|s| parse_dt(&s)).transpose()?,
        expires_at: parse_dt(&expires_at)?,
        created_at: parse_dt(&created_at)?,
    })
}

impl AuthRepository {
    // ── User CRUD ──

    /// Create a new user.
    pub async fn create_user(
        pool: &SqlitePool,
        username: &str,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> Result<User> {
        if username.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Username cannot be empty".into(),
            ));
        }
        if email.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Email cannot be empty".into(),
            ));
        }
        if password_hash.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Password hash cannot be empty".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, role, force_password_change, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 0, ?, ?)",
        )
        .bind(&id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(&now_str)
        .bind(&now_str)
        .execute(pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE constraint failed: users.username") {
                PersistenceError::Conflict(format!("Username '{}' already exists", username))
            } else if msg.contains("UNIQUE constraint failed: users.email") {
                PersistenceError::Conflict(format!("Email '{}' already exists", email))
            } else {
                PersistenceError::DatabaseError(msg)
            }
        })?;

        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            force_password_change: false,
            created_at: now,
            updated_at: now,
        })
    }

    /// Get a user by username.
    pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, username, email, role, force_password_change, created_at, updated_at \
             FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        row.map(row_to_user).transpose()
    }

    /// Get a user by email.
    pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, username, email, role, force_password_change, created_at, updated_at \
             FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        row.map(row_to_user).transpose()
    }

    /// Get a user by ID.
    pub async fn get_user(pool: &SqlitePool, id: &str) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, username, email, role, force_password_change, created_at, updated_at \
             FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        row.map(row_to_user).transpose()
    }

    /// Set a user's password hash and optionally force a change on next login.
    pub async fn set_password(
        pool: &SqlitePool,
        user_id: &str,
        password_hash: &str,
        force_change: bool,
    ) -> Result<()> {
        if user_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "User ID cannot be empty".into(),
            ));
        }

        // Check user exists
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if exists.0 == 0 {
            return Err(PersistenceError::NotFound {
                entity: "user".into(),
                id: user_id.to_string(),
            });
        }

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE users SET password_hash = ?, force_password_change = ?, updated_at = ? WHERE id = ?",
        )
        .bind(password_hash)
        .bind(force_change as i64)
        .bind(&now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Clear the force-password-change flag.
    pub async fn clear_force_change(pool: &SqlitePool, user_id: &str) -> Result<()> {
        // Check user exists
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if exists.0 == 0 {
            return Err(PersistenceError::NotFound {
                entity: "user".into(),
                id: user_id.to_string(),
            });
        }

        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE users SET force_password_change = 0, updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(())
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
    pub async fn get_user_role(pool: &SqlitePool, user_id: &str) -> Result<String> {
        let row: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match row {
            Some((role,)) => Ok(role),
            None => Err(PersistenceError::NotFound {
                entity: "user".into(),
                id: user_id.to_string(),
            }),
        }
    }

    /// Get a user's league operator assignments.
    pub async fn get_league_assignments(pool: &SqlitePool, user_id: &str) -> Result<Vec<String>> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT league_id FROM league_operators WHERE user_id = ?")
                .bind(user_id)
                .fetch_all(pool)
                .await
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    // ── Login Rate Limiting ──

    /// Record a login attempt (success or failure).
    pub async fn record_attempt(pool: &SqlitePool, user_id: &str, success: bool) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO login_attempts (id, user_id, attempted_at, success, created_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(&now)
        .bind(success as i64)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Count consecutive login failures within a time window.
    ///
    /// Counts failures starting from the most recent attempt, moving backward
    /// until a success is found or the window boundary is reached.
    pub async fn consecutive_failures(
        pool: &SqlitePool,
        user_id: &str,
        window: Duration,
    ) -> Result<u32> {
        let cutoff = (Utc::now()
            - chrono::Duration::from_std(window)
                .map_err(|e| PersistenceError::InvalidInput(format!("Invalid duration: {}", e)))?)
        .to_rfc3339();

        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT success FROM login_attempts \
             WHERE user_id = ? AND attempted_at >= ? \
             ORDER BY attempted_at DESC",
        )
        .bind(user_id)
        .bind(&cutoff)
        .fetch_all(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut count: u32 = 0;
        for (success,) in rows {
            if success == 0 {
                count += 1;
            } else {
                break; // Success resets the consecutive failure chain
            }
        }
        Ok(count)
    }

    /// Check if a user is locked out (>=10 failures in 15 min).
    pub async fn is_locked_out(pool: &SqlitePool, user_id: &str) -> Result<bool> {
        let failures =
            Self::consecutive_failures(pool, user_id, Duration::from_secs(15 * 60)).await?;
        Ok(failures >= 10)
    }

    // ── Invite Tokens ──

    /// Create an invite token for player account linking.
    pub async fn create_invite_token(
        pool: &SqlitePool,
        player_id: &str,
        created_by: &str,
    ) -> Result<InviteToken> {
        if player_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Player ID cannot be empty".into(),
            ));
        }
        if created_by.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Created-by user ID cannot be empty".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let token_hash: String = uuid::Uuid::new_v4()
            .as_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);

        sqlx::query(
            "INSERT INTO invite_tokens (id, player_id, token_hash, created_by, expires_at, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(player_id)
        .bind(&token_hash)
        .bind(created_by)
        .bind(&expires_at.to_rfc3339())
        .bind(&now.to_rfc3339())
        .execute(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(InviteToken {
            id,
            token_hash,
            player_id: player_id.to_string(),
            created_by: created_by.to_string(),
            claimed_by: None,
            claimed_at: None,
            expires_at,
            created_at: now,
        })
    }

    /// Claim an invite token (links player to user).
    /// Returns the player_id if successful, None if token expired or already claimed.
    pub async fn claim_invite_token(
        pool: &SqlitePool,
        token_hash: &str,
        claiming_user_id: &str,
    ) -> Result<Option<String>> {
        let token = Self::get_invite_token(pool, token_hash).await?;

        match token {
            None => Ok(None),
            Some(t) => {
                let now = Utc::now();

                // Check if expired
                if t.expires_at < now {
                    return Ok(None);
                }

                // Check if already claimed
                if t.claimed_by.is_some() {
                    return Ok(None);
                }

                // Claim it
                sqlx::query("UPDATE invite_tokens SET claimed_by = ?, claimed_at = ? WHERE id = ?")
                    .bind(claiming_user_id)
                    .bind(&now.to_rfc3339())
                    .bind(&t.id)
                    .execute(pool)
                    .await
                    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

                Ok(Some(t.player_id))
            }
        }
    }

    /// Get an invite token by hash.
    pub async fn get_invite_token(
        pool: &SqlitePool,
        token_hash: &str,
    ) -> Result<Option<InviteToken>> {
        let row: Option<InviteTokenRow> = sqlx::query_as(
            "SELECT id, player_id, token_hash, created_by, claimed_by, claimed_at, expires_at, created_at \
             FROM invite_tokens WHERE token_hash = ?",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        row.map(row_to_invite_token).transpose()
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
