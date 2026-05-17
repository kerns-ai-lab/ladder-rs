//! Comprehensive unit tests for AuthRepository.
//!
//! These tests operate against an in-memory SQLite database with real migrations.
//! Most methods are currently stubs (TDD RED phase), so tests that directly call
//! stub methods will fail until the implementation is complete.
//!
//! The `is_users_table_empty` method and `AdminBootstrap::run` are already
//! implemented and should pass immediately.

use chrono::{Duration as ChronoDuration, Utc};
use ladder_rs_persistence::{create_pool, AuthRepository, InviteToken};
use sqlx::{migrate::Migrator, query, query_as, SqlitePool};
use std::path::Path;
use std::time::Duration;

// ────────────────────────────────────────────────────────────────────────────────
// Test helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Create an in-memory SQLite pool and run all migrations.
async fn setup_test_db() -> SqlitePool {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("Failed to create in-memory pool");
    let mig = Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
        .await
        .expect("Failed to create migrator");
    mig.run(&pool).await.expect("Failed to run migrations");
    pool
}

/// Insert a user directly via raw SQL (bypasses the stub create_user).
async fn insert_user_raw(
    pool: &SqlitePool,
    id: &str,
    username: &str,
    email: &str,
    password_hash: &str,
    role: &str,
    force_change: bool,
) {
    let now = Utc::now().to_rfc3339();
    query(
        "INSERT INTO users (id, username, email, password_hash, role, force_password_change, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .bind(force_change as i32)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Failed to insert user");
}

/// Insert a login attempt directly via raw SQL.
async fn insert_login_attempt_raw(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    attempted_at: &str,
    success: bool,
) {
    let now = Utc::now().to_rfc3339();
    query(
        "INSERT INTO login_attempts (id, user_id, attempted_at, success, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(user_id)
    .bind(attempted_at)
    .bind(success as i32)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Failed to insert login attempt");
}

/// Insert a league directly via raw SQL.
async fn insert_league_raw(pool: &SqlitePool, id: &str, name: &str, algorithm: &str) {
    let now = Utc::now().to_rfc3339();
    query(
        "INSERT INTO leagues (id, name, algorithm, visibility, is_active, created_at, updated_at)
         VALUES (?, ?, ?, 'public', 1, ?, ?)",
    )
    .bind(id)
    .bind(name)
    .bind(algorithm)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Failed to insert league");
}

/// Insert a league_operator assignment directly via raw SQL.
async fn insert_league_operator_raw(pool: &SqlitePool, id: &str, league_id: &str, user_id: &str) {
    let now = Utc::now().to_rfc3339();
    query(
        "INSERT INTO league_operators (id, league_id, user_id, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(league_id)
    .bind(user_id)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Failed to insert league operator");
}

/// Insert an invite token directly via raw SQL.
async fn insert_invite_token_raw(
    pool: &SqlitePool,
    id: &str,
    player_id: &str,
    created_by: &str,
    token_hash: &str,
    expires_at: &str,
    claimed_at: Option<&str>,
) {
    let now = Utc::now().to_rfc3339();
    // If claimed_at is set, the token was already claimed; provide a placeholder claimed_by.
    let claimed_by: Option<&str> = claimed_at.map(|_| "previous-claimant");
    query(
        "INSERT INTO invite_tokens (id, player_id, token_hash, created_by, claimed_by, claimed_at, expires_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(player_id)
    .bind(token_hash)
    .bind(created_by)
    .bind(claimed_by)
    .bind(claimed_at)
    .bind(expires_at)
    .bind(&now)
    .execute(pool)
    .await
    .expect("Failed to insert invite token");
}

/// Count rows in the users table (for assertions).
async fn count_users(pool: &SqlitePool) -> i64 {
    let row: (i64,) = query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .expect("Failed to count users");
    row.0
}

/// Count rows in the login_attempts table (for assertions).
async fn count_login_attempts(pool: &SqlitePool) -> i64 {
    let row: (i64,) = query_as("SELECT COUNT(*) FROM login_attempts")
        .fetch_one(pool)
        .await
        .expect("Failed to count login attempts");
    row.0
}

/// Generate a unique ID string using UUID v4.
fn unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ────────────────────────────────────────────────────────────────────────────────
// User CRUD tests
// ────────────────────────────────────────────────────────────────────────────────

mod user_crud {
    use super::*;

    /// create_user with valid inputs should return Ok(User).
    #[tokio::test]
    async fn create_user_valid() {
        let pool = setup_test_db().await;
        let result = AuthRepository::create_user(
            &pool,
            "testuser",
            "test@example.com",
            "hashed_password_123",
            "player",
        )
        .await;
        // RED: currently returns Err because the method is a stub
        assert!(
            result.is_ok(),
            "create_user should succeed with valid inputs, got: {:?}",
            result
        );
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, "player");
        assert!(!user.force_password_change);
    }

    /// create_user with duplicate username should return a Conflict error.
    #[tokio::test]
    async fn create_user_duplicate_username() {
        let pool = setup_test_db().await;
        // Pre-insert a user so the username is taken
        insert_user_raw(
            &pool,
            &unique_id(),
            "dupeuser",
            "first@example.com",
            "hash1",
            "player",
            false,
        )
        .await;

        let result =
            AuthRepository::create_user(&pool, "dupeuser", "second@example.com", "hash2", "player")
                .await;
        // RED: stub returns Unknown, but should be Conflict or similar
        assert!(
            result.is_err(),
            "Duplicate username should error: {:?}",
            result
        );

        // After implementation, check it's a Conflict variant:
        // assert!(matches!(result, Err(PersistenceError::Conflict(_))));
    }

    /// create_user with duplicate email should return a Conflict error.
    #[tokio::test]
    async fn create_user_duplicate_email() {
        let pool = setup_test_db().await;
        insert_user_raw(
            &pool,
            &unique_id(),
            "firstuser",
            "dupe@example.com",
            "hash1",
            "player",
            false,
        )
        .await;

        let result =
            AuthRepository::create_user(&pool, "seconduser", "dupe@example.com", "hash2", "player")
                .await;
        assert!(
            result.is_err(),
            "Duplicate email should error: {:?}",
            result
        );
    }

    /// create_user with empty username should return an InvalidInput error.
    #[tokio::test]
    async fn create_user_empty_username() {
        let pool = setup_test_db().await;
        let result = AuthRepository::create_user(&pool, "", "e@e.com", "hash", "player").await;
        assert!(result.is_err(), "Empty username should error: {:?}", result);
    }

    /// create_user with empty email should return an InvalidInput error.
    #[tokio::test]
    async fn create_user_empty_email() {
        let pool = setup_test_db().await;
        let result = AuthRepository::create_user(&pool, "user", "", "hash", "player").await;
        assert!(result.is_err(), "Empty email should error: {:?}", result);
    }

    /// create_user with empty password_hash should return an InvalidInput error.
    #[tokio::test]
    async fn create_user_empty_password() {
        let pool = setup_test_db().await;
        let result = AuthRepository::create_user(&pool, "user", "e@e.com", "", "player").await;
        assert!(result.is_err(), "Empty password should error: {:?}", result);
    }

    /// get_user_by_username should return Some(User) for an existing user.
    #[tokio::test]
    async fn get_user_by_username_existing() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "alice",
            "alice@example.com",
            "hash_a",
            "admin",
            false,
        )
        .await;

        let result = AuthRepository::get_user_by_username(&pool, "alice").await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "get_user_by_username should succeed: {:?}",
            result
        );
        let user = result.unwrap();
        assert!(user.is_some(), "User should be found");
        let u = user.unwrap();
        assert_eq!(u.username, "alice");
        assert_eq!(u.email, "alice@example.com");
        assert_eq!(u.role, "admin");
        assert!(!u.force_password_change);
    }

    /// get_user_by_username should return Ok(None) for a nonexistent username.
    #[tokio::test]
    async fn get_user_by_username_nonexistent() {
        let pool = setup_test_db().await;
        let result = AuthRepository::get_user_by_username(&pool, "nobody").await;
        // RED: stub returns Err(Unknown), but should be Ok(None)
        assert!(
            result.is_ok(),
            "get_user_by_username should succeed: {:?}",
            result
        );
        assert!(result.unwrap().is_none(), "Nonexistent user should be None");
    }

    /// get_user_by_email should return Some(User) for an existing email.
    #[tokio::test]
    async fn get_user_by_email_existing() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "bob",
            "bob@example.com",
            "hash_b",
            "player",
            true,
        )
        .await;

        let result = AuthRepository::get_user_by_email(&pool, "bob@example.com").await;
        assert!(
            result.is_ok(),
            "get_user_by_email should succeed: {:?}",
            result
        );
        let user = result.unwrap();
        assert!(user.is_some(), "User should be found by email");
        let u = user.unwrap();
        assert_eq!(u.username, "bob");
        assert_eq!(u.email, "bob@example.com");
        assert!(
            u.force_password_change,
            "force_password_change should be true"
        );
    }

    /// get_user should return Some(User) for an existing ID.
    #[tokio::test]
    async fn get_user_existing() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "charlie",
            "charlie@example.com",
            "hash_c",
            "operator",
            false,
        )
        .await;

        let result = AuthRepository::get_user(&pool, &user_id).await;
        assert!(result.is_ok(), "get_user should succeed: {:?}", result);
        let user = result.unwrap();
        assert!(user.is_some(), "User should be found by ID");
        let u = user.unwrap();
        assert_eq!(u.id, user_id);
        assert_eq!(u.username, "charlie");
    }

    /// get_user should return Ok(None) for a nonexistent ID.
    #[tokio::test]
    async fn get_user_nonexistent() {
        let pool = setup_test_db().await;
        let result = AuthRepository::get_user(&pool, "nonexistent-id").await;
        assert!(result.is_ok(), "get_user should succeed: {:?}", result);
        assert!(result.unwrap().is_none(), "Nonexistent user should be None");
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Password tests
// ────────────────────────────────────────────────────────────────────────────────

mod password {
    use super::*;

    /// set_password should update the password hash.
    #[tokio::test]
    async fn set_password_changes_hash() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "pw_user",
            "pw@example.com",
            "old_hash",
            "player",
            false,
        )
        .await;

        let result = AuthRepository::set_password(&pool, &user_id, "new_hash_value", false).await;
        // RED: stub returns Err(Unknown)
        assert!(result.is_ok(), "set_password should succeed: {:?}", result);

        // After implementation, verify the hash changed via get_user:
        // let user = AuthRepository::get_user(&pool, &user_id).await.unwrap().unwrap();
        // (would need a get_password_hash method or check indirectly)

        // For now, verify via raw SQL that the table was updated
        let row: (String,) = query_as("SELECT password_hash FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to query password_hash");
        assert_eq!(row.0, "new_hash_value", "Password hash should be updated");
    }

    /// set_password with force_change=true should set the flag.
    #[tokio::test]
    async fn set_password_force_change_flag() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "force_user",
            "force@example.com",
            "old_hash",
            "player",
            false,
        )
        .await;

        let result = AuthRepository::set_password(&pool, &user_id, "forced_hash", true).await;
        assert!(
            result.is_ok(),
            "set_password with force_change should succeed: {:?}",
            result
        );

        // Verify force_password_change is now 1 (true) via raw SQL
        let row: (i64,) = query_as("SELECT force_password_change FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to query force_password_change");
        assert_eq!(row.0, 1, "force_password_change should be 1 (true)");
    }

    /// clear_force_change should clear the flag to false.
    #[tokio::test]
    async fn clear_force_change_flag() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "clear_user",
            "clear@example.com",
            "hash",
            "player",
            true, // force_change = true
        )
        .await;

        let result = AuthRepository::clear_force_change(&pool, &user_id).await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "clear_force_change should succeed: {:?}",
            result
        );

        // Verify force_password_change is now 0 (false) via raw SQL
        let row: (i64,) = query_as("SELECT force_password_change FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to query force_password_change");
        assert_eq!(row.0, 0, "force_password_change should be 0 (false)");
    }

    /// set_password with empty user_id should return an error.
    #[tokio::test]
    async fn set_password_empty_user_id() {
        let pool = setup_test_db().await;
        let result = AuthRepository::set_password(&pool, "", "some_hash", false).await;
        assert!(result.is_err(), "Empty user_id should error: {:?}", result);
    }

    /// set_password for nonexistent user should return an error.
    #[tokio::test]
    async fn set_password_nonexistent_user() {
        let pool = setup_test_db().await;
        let result = AuthRepository::set_password(&pool, "nonexistent-user", "hash", false).await;
        // Should return NotFound or similar error
        assert!(
            result.is_err(),
            "Nonexistent user should error: {:?}",
            result
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// is_users_table_empty tests
// ────────────────────────────────────────────────────────────────────────────────

mod table_empty {
    use super::*;

    /// Returns true when the users table has no rows.
    #[tokio::test]
    async fn is_empty_when_no_users() {
        let pool = setup_test_db().await;
        let result = AuthRepository::is_users_table_empty(&pool).await;
        // This method IS already implemented
        assert!(
            result.is_ok(),
            "is_users_table_empty should succeed: {:?}",
            result
        );
        assert!(result.unwrap(), "Empty table should return true");
    }

    /// Returns false after inserting a user.
    #[tokio::test]
    async fn is_not_empty_after_creating_user() {
        let pool = setup_test_db().await;
        // First, it should be empty
        let empty = AuthRepository::is_users_table_empty(&pool)
            .await
            .expect("is_users_table_empty should succeed");
        assert!(empty, "Table should be empty initially");

        // Insert a user via raw SQL (since create_user is a stub)
        insert_user_raw(
            &pool,
            &unique_id(),
            "someone",
            "someone@example.com",
            "hash",
            "admin",
            false,
        )
        .await;

        let not_empty = AuthRepository::is_users_table_empty(&pool)
            .await
            .expect("is_users_table_empty should succeed");
        assert!(!not_empty, "Table should not be empty after insert");
    }

    /// Returns false when multiple users exist.
    #[tokio::test]
    async fn is_not_empty_with_multiple_users() {
        let pool = setup_test_db().await;
        insert_user_raw(&pool, &unique_id(), "u1", "u1@e.com", "h1", "player", false).await;
        insert_user_raw(&pool, &unique_id(), "u2", "u2@e.com", "h2", "admin", false).await;
        insert_user_raw(
            &pool,
            &unique_id(),
            "u3",
            "u3@e.com",
            "h3",
            "operator",
            false,
        )
        .await;

        let result = AuthRepository::is_users_table_empty(&pool)
            .await
            .expect("is_users_table_empty should succeed");
        assert!(!result, "Table with 3 users should not be empty");
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Role tests
// ────────────────────────────────────────────────────────────────────────────────

mod roles {
    use super::*;

    /// get_user_role should return the role string for an existing user.
    #[tokio::test]
    async fn get_user_role_returns_correct_role() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "role_user",
            "role@example.com",
            "hash",
            "operator",
            false,
        )
        .await;

        let result = AuthRepository::get_user_role(&pool, &user_id).await;
        // RED: stub returns Err(Unknown)
        assert!(result.is_ok(), "get_user_role should succeed: {:?}", result);
        assert_eq!(result.unwrap(), "operator");
    }

    /// get_user_role should return the admin role.
    #[tokio::test]
    async fn get_user_role_admin() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "admin_user",
            "admin@example.com",
            "hash",
            "admin",
            false,
        )
        .await;

        let result = AuthRepository::get_user_role(&pool, &user_id).await;
        assert!(result.is_ok(), "get_user_role should succeed: {:?}", result);
        assert_eq!(result.unwrap(), "admin");
    }

    /// get_user_role for a nonexistent user should return an error.
    #[tokio::test]
    async fn get_user_role_nonexistent() {
        let pool = setup_test_db().await;
        let result = AuthRepository::get_user_role(&pool, "nonexistent-user-id").await;
        assert!(
            result.is_err(),
            "Nonexistent user should error: {:?}",
            result
        );
    }

    /// get_league_assignments should return league IDs for an operator.
    #[tokio::test]
    async fn get_league_assignments_returns_leagues() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        let league_id_1 = unique_id();
        let league_id_2 = unique_id();

        insert_user_raw(
            &pool,
            &user_id,
            "op_user",
            "op@example.com",
            "hash",
            "operator",
            false,
        )
        .await;
        insert_league_raw(&pool, &league_id_1, "League Alpha", "elo").await;
        insert_league_raw(&pool, &league_id_2, "League Beta", "glicko").await;
        insert_league_operator_raw(&pool, &unique_id(), &league_id_1, &user_id).await;
        insert_league_operator_raw(&pool, &unique_id(), &league_id_2, &user_id).await;

        let result = AuthRepository::get_league_assignments(&pool, &user_id).await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "get_league_assignments should succeed: {:?}",
            result
        );
        let leagues = result.unwrap();
        assert_eq!(leagues.len(), 2, "Operator should have 2 leagues");
        assert!(
            leagues.contains(&league_id_1),
            "Should contain League Alpha"
        );
        assert!(leagues.contains(&league_id_2), "Should contain League Beta");
    }

    /// get_league_assignments should return empty vec for user with no league assignments.
    #[tokio::test]
    async fn get_league_assignments_empty() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "no_op",
            "noop@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let result = AuthRepository::get_league_assignments(&pool, &user_id).await;
        assert!(
            result.is_ok(),
            "get_league_assignments should succeed: {:?}",
            result
        );
        assert!(
            result.unwrap().is_empty(),
            "Should return empty vec for player with no leagues"
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Login rate limiting tests
// ────────────────────────────────────────────────────────────────────────────────

mod rate_limiting {
    use super::*;

    /// record_attempt should store a successful login attempt.
    #[tokio::test]
    async fn record_attempt_success() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "login_user",
            "login@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let before_count = count_login_attempts(&pool).await;

        let result = AuthRepository::record_attempt(&pool, &user_id, true).await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "record_attempt should succeed: {:?}",
            result
        );

        let after_count = count_login_attempts(&pool).await;
        assert_eq!(
            after_count,
            before_count + 1,
            "Login attempts count should increase by 1"
        );

        // Verify the attempt was recorded as success
        let row: (i64,) =
            query_as("SELECT success FROM login_attempts WHERE user_id = ? ORDER BY attempted_at DESC LIMIT 1")
                .bind(&user_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to query login attempt");
        assert_eq!(row.0, 1, "Login attempt should be recorded as success (1)");
    }

    /// record_attempt should store a failed login attempt.
    #[tokio::test]
    async fn record_attempt_failure() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "fail_user",
            "fail@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let result = AuthRepository::record_attempt(&pool, &user_id, false).await;
        assert!(
            result.is_ok(),
            "record_attempt should succeed: {:?}",
            result
        );

        let row: (i64,) =
            query_as("SELECT success FROM login_attempts WHERE user_id = ? ORDER BY attempted_at DESC LIMIT 1")
                .bind(&user_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to query login attempt");
        assert_eq!(row.0, 0, "Login attempt should be recorded as failure (0)");
    }

    /// consecutive_failures should count failures within the given window.
    #[tokio::test]
    async fn consecutive_failures_counts_within_window() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "cf_user",
            "cf@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let now = Utc::now();
        // Insert 5 failed attempts within the last 5 minutes
        for i in 0..5 {
            let ts = (now - ChronoDuration::minutes(i)).to_rfc3339();
            insert_login_attempt_raw(&pool, &format!("attempt_{}", i), &user_id, &ts, false).await;
        }

        let result = AuthRepository::consecutive_failures(
            &pool,
            &user_id,
            Duration::from_secs(900), // 15-minute window
        )
        .await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "consecutive_failures should succeed: {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            5,
            "Should count 5 consecutive failures within 15-min window"
        );
    }

    /// consecutive_failures should return 0 when there are no failures.
    #[tokio::test]
    async fn consecutive_failures_zero_when_no_attempts() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "clean_user",
            "clean@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let result =
            AuthRepository::consecutive_failures(&pool, &user_id, Duration::from_secs(900)).await;
        assert!(
            result.is_ok(),
            "consecutive_failures should succeed: {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            0,
            "Should return 0 for user with no attempts"
        );
    }

    /// consecutive_failures should only count failures, not successes.
    #[tokio::test]
    async fn consecutive_failures_ignores_successes() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "mixed_user",
            "mixed@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let now = Utc::now();
        // Insert: fail, fail, success, fail, fail
        let attempts = vec![
            (0, false),
            (1, false),
            (2, true), // success should break the consecutive chain
            (3, false),
            (4, false),
        ];
        for (offset, success) in &attempts {
            let ts = (now - ChronoDuration::minutes(*offset)).to_rfc3339();
            insert_login_attempt_raw(
                &pool,
                &format!("mixed_attempt_{}", offset),
                &user_id,
                &ts,
                *success,
            )
            .await;
        }

        let result =
            AuthRepository::consecutive_failures(&pool, &user_id, Duration::from_secs(900)).await;
        assert!(
            result.is_ok(),
            "consecutive_failures should succeed: {:?}",
            result
        );
        // After a success, consecutive failures should reset. The last 2 are failures.
        assert_eq!(
            result.unwrap(),
            2,
            "Consecutive failures after a success should only count trailing failures"
        );
    }

    /// consecutive_failures should respect the time window.
    #[tokio::test]
    async fn consecutive_failures_respects_window() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "window_user",
            "window@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let now = Utc::now();
        // Insert 3 failures within the last 2 minutes, 3 older than 30 minutes
        for i in 0..3 {
            let ts = (now - ChronoDuration::minutes(i)).to_rfc3339();
            insert_login_attempt_raw(&pool, &format!("recent_fail_{}", i), &user_id, &ts, false)
                .await;
        }
        for i in 0..3 {
            let ts = (now - ChronoDuration::minutes(30 + i)).to_rfc3339();
            insert_login_attempt_raw(&pool, &format!("old_fail_{}", i), &user_id, &ts, false).await;
        }

        // With a 10-minute window, only the 3 recent failures should count
        let result = AuthRepository::consecutive_failures(
            &pool,
            &user_id,
            Duration::from_secs(600), // 10-minute window
        )
        .await;
        assert!(
            result.is_ok(),
            "consecutive_failures should succeed: {:?}",
            result
        );
        assert_eq!(
            result.unwrap(),
            3,
            "Only recent failures within window should be counted"
        );
    }

    /// is_locked_out should return true after >=10 failures within 15 minutes.
    #[tokio::test]
    async fn is_locked_out_after_10_failures() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "locked_user",
            "locked@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let now = Utc::now();
        // Insert 10 failed attempts within the last 5 minutes
        for i in 0..10 {
            let ts = (now - ChronoDuration::minutes(i / 2)).to_rfc3339();
            insert_login_attempt_raw(&pool, &format!("lock_fail_{}", i), &user_id, &ts, false)
                .await;
        }

        let result = AuthRepository::is_locked_out(&pool, &user_id).await;
        // RED: stub returns Err(Unknown)
        assert!(result.is_ok(), "is_locked_out should succeed: {:?}", result);
        assert!(
            result.unwrap(),
            "User should be locked out after 10 failures"
        );
    }

    /// is_locked_out should return false when under the threshold.
    #[tokio::test]
    async fn is_locked_out_false_under_threshold() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "safe_user",
            "safe@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let now = Utc::now();
        // Insert only 5 failed attempts
        for i in 0..5 {
            let ts = (now - ChronoDuration::minutes(i)).to_rfc3339();
            insert_login_attempt_raw(&pool, &format!("safe_fail_{}", i), &user_id, &ts, false)
                .await;
        }

        let result = AuthRepository::is_locked_out(&pool, &user_id).await;
        assert!(result.is_ok(), "is_locked_out should succeed: {:?}", result);
        assert!(
            !result.unwrap(),
            "User should NOT be locked out with only 5 failures"
        );
    }

    /// is_locked_out should return false for a user with no attempts.
    #[tokio::test]
    async fn is_locked_out_false_no_attempts() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "fresh_user",
            "fresh@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let result = AuthRepository::is_locked_out(&pool, &user_id).await;
        assert!(result.is_ok(), "is_locked_out should succeed: {:?}", result);
        assert!(!result.unwrap(), "Fresh user should not be locked out");
    }

    /// is_locked_out should return false when failures are outside the window.
    #[tokio::test]
    async fn is_locked_out_false_old_failures() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        insert_user_raw(
            &pool,
            &user_id,
            "old_fail_user",
            "oldfail@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        let now = Utc::now();
        // Insert 10+ failed attempts, all older than 20 minutes
        for i in 0..12 {
            let ts = (now - ChronoDuration::minutes(20 + i)).to_rfc3339();
            insert_login_attempt_raw(&pool, &format!("old_fail_{}", i), &user_id, &ts, false).await;
        }

        let result = AuthRepository::is_locked_out(&pool, &user_id).await;
        assert!(result.is_ok(), "is_locked_out should succeed: {:?}", result);
        assert!(
            !result.unwrap(),
            "Old failures outside the window should not trigger lockout"
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Invite token tests
// ────────────────────────────────────────────────────────────────────────────────

mod invite_tokens {
    use super::*;

    /// create_invite_token should return a token with the player_id.
    #[tokio::test]
    async fn create_invite_token_populates_player_id() {
        let pool = setup_test_db().await;
        let created_by = unique_id();
        insert_user_raw(
            &pool,
            &created_by,
            "creator",
            "creator@example.com",
            "hash",
            "admin",
            false,
        )
        .await;

        let result = AuthRepository::create_invite_token(&pool, "player-123", &created_by).await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "create_invite_token should succeed: {:?}",
            result
        );
        let token: InviteToken = result.unwrap();
        assert_eq!(
            token.player_id, "player-123",
            "Token should reference the correct player"
        );
        assert_eq!(token.created_by, created_by, "Token should record creator");
        assert!(!token.id.is_empty(), "Token should have an ID");
        assert!(!token.token_hash.is_empty(), "Token should have a hash");
        assert!(token.claimed_by.is_none(), "Token should not be claimed");
        assert!(
            token.claimed_at.is_none(),
            "Token should have no claimed_at"
        );
    }

    /// create_invite_token should set expiry in the future.
    #[tokio::test]
    async fn create_invite_token_expiry_in_future() {
        let pool = setup_test_db().await;
        let created_by = unique_id();
        insert_user_raw(
            &pool,
            &created_by,
            "expiry_creator",
            "expiry@example.com",
            "hash",
            "admin",
            false,
        )
        .await;

        let before = Utc::now();
        let result = AuthRepository::create_invite_token(&pool, "player-456", &created_by).await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "create_invite_token should succeed: {:?}",
            result
        );
        let token = result.unwrap();
        assert!(
            token.expires_at > before,
            "expires_at ({:?}) should be after creation time ({:?})",
            token.expires_at,
            before
        );
    }

    /// claim_invite_token should return Some(player_id) when token is valid.
    #[tokio::test]
    async fn claim_invite_token_valid() {
        let pool = setup_test_db().await;
        let claiming_user_id = unique_id();
        let created_by = unique_id();

        insert_user_raw(
            &pool,
            &created_by,
            "token_creator",
            "tokencreator@example.com",
            "hash",
            "admin",
            false,
        )
        .await;
        insert_user_raw(
            &pool,
            &claiming_user_id,
            "claimant",
            "claimant@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        // Create a token
        let token_hash: String;
        let created = AuthRepository::create_invite_token(&pool, "player-789", &created_by).await;
        if let Ok(ref token) = created {
            token_hash = token.token_hash.clone();
        } else {
            // Fallback: while create_invite_token is a stub, insert via raw SQL
            let future = (Utc::now() + ChronoDuration::days(7)).to_rfc3339();
            let th = "valid_token_hash_abc".to_string();
            insert_invite_token_raw(
                &pool,
                &unique_id(),
                "player-789",
                &created_by,
                &th,
                &future,
                None,
            )
            .await;
            token_hash = th;
        }

        let result =
            AuthRepository::claim_invite_token(&pool, &token_hash, &claiming_user_id).await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "claim_invite_token should succeed: {:?}",
            result
        );
        let claimed_player = result.unwrap();
        assert!(
            claimed_player.is_some(),
            "Should return Some(player_id) for valid token"
        );
    }

    /// claim_invite_token should return error/None when token is expired.
    #[tokio::test]
    async fn claim_invite_token_expired() {
        let pool = setup_test_db().await;
        let claiming_user_id = unique_id();
        insert_user_raw(
            &pool,
            &claiming_user_id,
            "late_claimant",
            "late@example.com",
            "hash",
            "player",
            false,
        )
        .await;

        // Insert an already-expired token
        let past = (Utc::now() - ChronoDuration::days(1)).to_rfc3339();
        insert_invite_token_raw(
            &pool,
            &unique_id(),
            &claiming_user_id,
            &claiming_user_id,
            "expired_token_hash",
            &past,
            None,
        )
        .await;

        let result =
            AuthRepository::claim_invite_token(&pool, "expired_token_hash", &claiming_user_id)
                .await;
        assert!(
            result.is_ok(),
            "claim_invite_token should succeed: {:?}",
            result
        );
        let claimed = result.unwrap();
        assert!(
            claimed.is_none(),
            "Expired token should return None, got: {:?}",
            claimed
        );
    }

    /// claim_invite_token should return error/None when token is already claimed.
    #[tokio::test]
    async fn claim_invite_token_already_claimed() {
        let pool = setup_test_db().await;
        let original_claimant = unique_id();
        let second_claimant = unique_id();
        let future = (Utc::now() + ChronoDuration::days(7)).to_rfc3339();
        let claimed_at = Utc::now().to_rfc3339();

        insert_user_raw(
            &pool,
            &original_claimant,
            "first_claimant",
            "first@example.com",
            "h1",
            "player",
            false,
        )
        .await;
        insert_user_raw(
            &pool,
            &second_claimant,
            "second_claimant",
            "second@example.com",
            "h2",
            "player",
            false,
        )
        .await;

        // Insert a token that's already been used
        insert_invite_token_raw(
            &pool,
            &unique_id(),
            &original_claimant,
            &original_claimant,
            "claimed_token_hash",
            &future,
            Some(&claimed_at),
        )
        .await;

        let result =
            AuthRepository::claim_invite_token(&pool, "claimed_token_hash", &second_claimant).await;
        assert!(
            result.is_ok(),
            "claim_invite_token should succeed: {:?}",
            result
        );
        let claimed = result.unwrap();
        assert!(
            claimed.is_none(),
            "Already-claimed token should return None, got: {:?}",
            claimed
        );
    }

    /// get_invite_token should return token details for a valid hash.
    #[tokio::test]
    async fn get_invite_token_returns_details() {
        let pool = setup_test_db().await;
        let user_id = unique_id();
        let future = (Utc::now() + ChronoDuration::days(7)).to_rfc3339();

        insert_user_raw(
            &pool,
            &user_id,
            "token_owner",
            "tokenowner@example.com",
            "hash",
            "player",
            false,
        )
        .await;
        insert_invite_token_raw(
            &pool,
            &unique_id(),
            &user_id,
            &user_id,
            "findable_token_hash",
            &future,
            None,
        )
        .await;

        let result = AuthRepository::get_invite_token(&pool, "findable_token_hash").await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "get_invite_token should succeed: {:?}",
            result
        );
        let token = result.unwrap();
        assert!(token.is_some(), "Should find the token");
        let t = token.unwrap();
        assert_eq!(t.token_hash, "findable_token_hash");
        assert!(!t.id.is_empty());
    }

    /// get_invite_token should return Ok(None) for a nonexistent hash.
    #[tokio::test]
    async fn get_invite_token_nonexistent() {
        let pool = setup_test_db().await;
        let result = AuthRepository::get_invite_token(&pool, "nonexistent_token_hash").await;
        assert!(
            result.is_ok(),
            "get_invite_token should succeed: {:?}",
            result
        );
        assert!(
            result.unwrap().is_none(),
            "Nonexistent token should be None"
        );
    }

    /// create_invite_token with empty player_id should return an error.
    #[tokio::test]
    async fn create_invite_token_empty_player_id() {
        let pool = setup_test_db().await;
        let created_by = unique_id();
        insert_user_raw(
            &pool,
            &created_by,
            "empty_creator",
            "emptycreator@example.com",
            "hash",
            "admin",
            false,
        )
        .await;

        let result = AuthRepository::create_invite_token(&pool, "", &created_by).await;
        assert!(
            result.is_err(),
            "Empty player_id should error: {:?}",
            result
        );
    }

    /// create_invite_token with empty created_by should return an error.
    #[tokio::test]
    async fn create_invite_token_empty_created_by() {
        let pool = setup_test_db().await;
        let result = AuthRepository::create_invite_token(&pool, "player-999", "").await;
        assert!(
            result.is_err(),
            "Empty created_by should error: {:?}",
            result
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Edge case tests
// ────────────────────────────────────────────────────────────────────────────────

mod edge_cases {
    use super::*;

    /// create_user with a very long username (within reasonable bounds) should succeed.
    #[tokio::test]
    async fn very_long_username() {
        let pool = setup_test_db().await;
        let long_username = "a".repeat(200);
        let result = AuthRepository::create_user(
            &pool,
            &long_username,
            "longuser@example.com",
            "hash",
            "player",
        )
        .await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "Very long username should be accepted: {:?}",
            result
        );
    }

    /// create_user with a very long email should succeed.
    #[tokio::test]
    async fn very_long_email() {
        let pool = setup_test_db().await;
        let long_email = format!("{}@example.com", "a".repeat(200));
        let result =
            AuthRepository::create_user(&pool, "longemailuser", &long_email, "hash", "player")
                .await;
        assert!(
            result.is_ok(),
            "Very long email should be accepted: {:?}",
            result
        );
    }

    /// Various operations with a nonexistent user ID should fail gracefully.
    #[tokio::test]
    async fn nonexistent_user_id_operations() {
        let pool = setup_test_db().await;
        let fake_id = "definitely-not-a-real-user-id-12345";

        // get_user should return Ok(None)
        let get = AuthRepository::get_user(&pool, fake_id).await;
        assert!(
            get.is_ok(),
            "get_user for nonexistent should be Ok: {:?}",
            get
        );
        assert!(get.unwrap().is_none(), "Should be None");

        // get_user_role should error
        let role = AuthRepository::get_user_role(&pool, fake_id).await;
        assert!(
            role.is_err(),
            "get_user_role for nonexistent should error: {:?}",
            role
        );

        // get_user_by_username should return Ok(None)
        let by_name = AuthRepository::get_user_by_username(&pool, fake_id).await;
        assert!(
            by_name.is_ok(),
            "get_user_by_username should be Ok: {:?}",
            by_name
        );
        assert!(by_name.unwrap().is_none(), "Should be None");

        // get_user_by_email should return Ok(None)
        let by_email = AuthRepository::get_user_by_email(&pool, fake_id).await;
        assert!(
            by_email.is_ok(),
            "get_user_by_email should be Ok: {:?}",
            by_email
        );
        assert!(by_email.unwrap().is_none(), "Should be None");

        // set_password should error
        let set_pw = AuthRepository::set_password(&pool, fake_id, "h", false).await;
        assert!(
            set_pw.is_err(),
            "set_password for nonexistent should error: {:?}",
            set_pw
        );

        // clear_force_change should error
        let clear = AuthRepository::clear_force_change(&pool, fake_id).await;
        assert!(
            clear.is_err(),
            "clear_force_change for nonexistent should error: {:?}",
            clear
        );

        // record_attempt with nonexistent user should error (FK constraint)
        let record = AuthRepository::record_attempt(&pool, fake_id, false).await;
        assert!(
            record.is_err(),
            "record_attempt for nonexistent user should error: {:?}",
            record
        );

        // consecutive_failures should return 0 for nonexistent user
        let failures =
            AuthRepository::consecutive_failures(&pool, fake_id, Duration::from_secs(900)).await;
        assert!(
            failures.is_ok(),
            "consecutive_failures should be Ok: {:?}",
            failures
        );
        assert_eq!(failures.unwrap(), 0, "Should return 0 for nonexistent user");

        // is_locked_out should be false for nonexistent user
        let locked = AuthRepository::is_locked_out(&pool, fake_id).await;
        assert!(locked.is_ok(), "is_locked_out should be Ok: {:?}", locked);
        assert!(
            !locked.unwrap(),
            "Nonexistent user should not be locked out"
        );
    }

    /// Calling create_user with a role that is not standard should still succeed
    /// (the repository should not validate role names - that's a service layer concern).
    #[tokio::test]
    async fn create_user_with_unusual_role() {
        let pool = setup_test_db().await;
        let result = AuthRepository::create_user(
            &pool,
            "unusual_role_user",
            "unusual@example.com",
            "hash",
            "super-duper-admin-extra",
        )
        .await;
        // RED: stub returns Err(Unknown)
        assert!(
            result.is_ok(),
            "Unusual role should be accepted by repository layer: {:?}",
            result
        );
    }

    /// create_user with unicode characters in username should succeed.
    #[tokio::test]
    async fn create_user_unicode_username() {
        let pool = setup_test_db().await;
        let result = AuthRepository::create_user(
            &pool,
            "ユーザー名",
            "unicode@example.com",
            "hash",
            "player",
        )
        .await;
        assert!(
            result.is_ok(),
            "Unicode username should be accepted: {:?}",
            result
        );
    }

    /// Ensure that created_at and updated_at are set on a newly created user.
    #[tokio::test]
    async fn create_user_sets_timestamps() {
        let pool = setup_test_db().await;
        let before = Utc::now();
        let result =
            AuthRepository::create_user(&pool, "ts_user", "ts@example.com", "hash", "player").await;
        // RED: stub returns Err(Unknown)
        assert!(result.is_ok(), "create_user should succeed: {:?}", result);
        let user = result.unwrap();
        assert!(
            user.created_at >= before,
            "created_at should be >= creation start time"
        );
        assert!(
            user.updated_at >= before,
            "updated_at should be >= creation start time"
        );
        assert!(
            user.created_at <= Utc::now(),
            "created_at should not be in the future"
        );
        assert!(
            user.updated_at <= Utc::now(),
            "updated_at should not be in the future"
        );
    }

    /// Multiple invocations of is_users_table_empty should be consistent.
    #[tokio::test]
    async fn is_users_table_empty_idempotent() {
        let pool = setup_test_db().await;

        let first = AuthRepository::is_users_table_empty(&pool)
            .await
            .expect("first call");
        let second = AuthRepository::is_users_table_empty(&pool)
            .await
            .expect("second call");

        assert_eq!(first, second, "Multiple calls should return same result");
        assert!(first, "Fresh DB should be empty");
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// AdminBootstrap smoke tests
// ────────────────────────────────────────────────────────────────────────────────

mod admin_bootstrap {
    use super::*;
    use ladder_rs_persistence::AdminBootstrap;

    /// AdminBootstrap should create an admin user when the users table is empty.
    #[tokio::test]
    async fn bootstrap_creates_admin_when_empty() {
        let pool = setup_test_db().await;

        let result = AdminBootstrap::run(&pool).await;
        // This method IS already implemented
        assert!(result.is_ok(), "Bootstrap should succeed: {:?}", result);
        let creds = result.unwrap();
        assert!(creds.is_some(), "Should create bootstrap credentials");
        let c = creds.unwrap();
        assert_eq!(c.username, "admin");
        assert!(!c.password.is_empty(), "Password should not be empty");
        assert!(!c.user_id.is_empty(), "User ID should not be empty");

        // Verify user exists in the database
        let count = count_users(&pool).await;
        assert_eq!(count, 1, "Should have exactly 1 user after bootstrap");
    }

    /// AdminBootstrap should return Ok(None) when users already exist.
    #[tokio::test]
    async fn bootstrap_skips_when_users_exist() {
        let pool = setup_test_db().await;

        // Insert a user first
        insert_user_raw(
            &pool,
            &unique_id(),
            "existing_user",
            "existing@example.com",
            "hash",
            "admin",
            false,
        )
        .await;

        let result = AdminBootstrap::run(&pool).await;
        assert!(result.is_ok(), "Bootstrap should succeed: {:?}", result);
        assert!(result.unwrap().is_none(), "Should skip when users exist");

        // Should still only have 1 user
        let count = count_users(&pool).await;
        assert_eq!(count, 1, "Should still have exactly 1 user");
    }
}
