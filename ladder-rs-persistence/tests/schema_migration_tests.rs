//! Schema migration tests for ladder-rs-persistence
//!
//! These tests verify the database schema AFTER all migrations are applied.
//! They serve as acceptance criteria for migration tasks 907.2.2-907.2.5.
//!
//! Tests will FAIL until all migration files are implemented.
//! Tests should PASS after all four migration tasks are completed.

use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Expected tables in the schema after all migrations
const EXPECTED_TABLES: &[&str] = &[
    // Auth tables (907.2.2)
    "users",
    "login_attempts",
    "sessions",
    "invite_tokens",
    "player_account_links",
    // League/season tables (907.2.3)
    "leagues",
    "league_operators",
    "seasons",
    // Player tables (907.2.4)
    "players",
    "league_players",
    "player_aliases",
    // Match/rating tables (907.2.5)
    "matches",
    "match_participants",
    "match_audit_log",
    "rating_snapshots",
    "recalculation_jobs",
];

/// Expected indexes with their columns
/// Format: (table_name, index_name, columns)
const EXPECTED_INDEXES: &[(&str, &str, &[&str])] = &[
    (
        "rating_snapshots",
        "idx_rating_snapshots_season_rating",
        &["season_id", "conservative_rating"],
    ),
    (
        "rating_snapshots",
        "idx_rating_snapshots_player_season_ts",
        &["player_id", "season_id", "timestamp"],
    ),
    (
        "match_participants",
        "idx_match_participants_player_season",
        &["player_id", "season_id"],
    ),
    (
        "match_participants",
        "idx_match_participants_match_id",
        &["match_id"],
    ),
    (
        "matches",
        "idx_matches_season_recorded",
        &["season_id", "recorded_at"],
    ),
    (
        "recalculation_jobs",
        "idx_recalc_jobs_status_created",
        &["status", "created_at"],
    ),
    ("players", "idx_players_name", &["name"]),
    (
        "league_operators",
        "idx_league_operators_league_user",
        &["league_id", "user_id"],
    ),
    (
        "leagues",
        "idx_leagues_visibility_archived",
        &["visibility", "is_archived"],
    ),
    (
        "login_attempts",
        "idx_login_attempts_user_attempted",
        &["user_id", "attempted_at"],
    ),
];

/// Expected FK constraints
/// Format: (table, column, references_table, references_column, on_delete)
const EXPECTED_FK_CONSTRAINTS: &[(&str, &str, &str, &str, &str)] = &[
    // Auth FKs
    ("login_attempts", "user_id", "users", "id", "CASCADE"),
    ("sessions", "user_id", "users", "id", "CASCADE"),
    ("invite_tokens", "user_id", "users", "id", "CASCADE"),
    ("player_account_links", "user_id", "users", "id", "RESTRICT"),
    (
        "player_account_links",
        "player_id",
        "players",
        "id",
        "RESTRICT",
    ),
    // League FKs
    ("league_operators", "league_id", "leagues", "id", "RESTRICT"),
    ("league_operators", "user_id", "users", "id", "RESTRICT"),
    ("seasons", "league_id", "leagues", "id", "RESTRICT"),
    // Player FKs
    ("league_players", "league_id", "leagues", "id", "RESTRICT"),
    ("league_players", "player_id", "players", "id", "RESTRICT"),
    (
        "player_aliases",
        "primary_player_id",
        "players",
        "id",
        "RESTRICT",
    ),
    (
        "player_aliases",
        "alias_player_id",
        "players",
        "id",
        "RESTRICT",
    ),
    ("player_aliases", "created_by", "users", "id", "RESTRICT"),
    // Match FKs
    ("matches", "season_id", "seasons", "id", "RESTRICT"),
    ("match_participants", "match_id", "matches", "id", "CASCADE"),
    (
        "match_participants",
        "player_id",
        "players",
        "id",
        "RESTRICT",
    ),
    ("match_audit_log", "match_id", "matches", "id", "RESTRICT"),
    (
        "match_audit_log",
        "actor_user_id",
        "users",
        "id",
        "RESTRICT",
    ),
    ("rating_snapshots", "season_id", "seasons", "id", "RESTRICT"),
    ("rating_snapshots", "player_id", "players", "id", "RESTRICT"),
    ("rating_snapshots", "match_id", "matches", "id", "RESTRICT"),
    (
        "recalculation_jobs",
        "season_id",
        "seasons",
        "id",
        "RESTRICT",
    ),
];

/// Expected column definitions per table
/// Format: (table_name, column_name, column_type, not_null, default_value)
const EXPECTED_COLUMNS: &[(&str, &str, &str, bool, Option<&str>)] = &[
    // users table
    ("users", "id", "TEXT", true, None),
    ("users", "username", "TEXT", true, None),
    ("users", "email", "TEXT", true, None),
    ("users", "password_hash", "TEXT", true, None),
    ("users", "role", "TEXT", true, None),
    ("users", "force_password_change", "INTEGER", true, Some("0")),
    (
        "users",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    (
        "users",
        "updated_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // login_attempts table
    ("login_attempts", "id", "TEXT", true, None),
    ("login_attempts", "user_id", "TEXT", true, None),
    (
        "login_attempts",
        "attempted_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    ("login_attempts", "success", "INTEGER", true, None),
    // sessions table
    ("sessions", "id", "TEXT", true, None),
    ("sessions", "user_id", "TEXT", true, None),
    ("sessions", "token", "TEXT", true, None),
    ("sessions", "expires_at", "TEXT", true, None),
    (
        "sessions",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // invite_tokens table
    ("invite_tokens", "id", "TEXT", true, None),
    ("invite_tokens", "user_id", "TEXT", true, None),
    ("invite_tokens", "token", "TEXT", true, None),
    ("invite_tokens", "expires_at", "TEXT", true, None),
    ("invite_tokens", "used_at", "TEXT", false, None),
    (
        "invite_tokens",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // player_account_links table
    ("player_account_links", "id", "TEXT", true, None),
    ("player_account_links", "player_id", "TEXT", true, None),
    ("player_account_links", "user_id", "TEXT", true, None),
    (
        "player_account_links",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // leagues table
    ("leagues", "id", "TEXT", true, None),
    ("leagues", "name", "TEXT", true, None),
    ("leagues", "description", "TEXT", false, None),
    ("leagues", "algorithm", "TEXT", true, None),
    ("leagues", "visibility", "TEXT", true, None),
    ("leagues", "is_active", "INTEGER", true, Some("1")),
    ("leagues", "is_archived", "INTEGER", true, Some("0")),
    (
        "leagues",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    (
        "leagues",
        "updated_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // league_operators table
    ("league_operators", "id", "TEXT", true, None),
    ("league_operators", "league_id", "TEXT", true, None),
    ("league_operators", "user_id", "TEXT", true, None),
    (
        "league_operators",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // seasons table
    ("seasons", "id", "TEXT", true, None),
    ("seasons", "league_id", "TEXT", true, None),
    ("seasons", "algorithm", "TEXT", true, None),
    ("seasons", "params_json", "TEXT", false, None),
    ("seasons", "start_date", "TEXT", true, None),
    ("seasons", "end_date", "TEXT", false, None),
    (
        "seasons",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // players table
    ("players", "id", "TEXT", true, None),
    ("players", "name", "TEXT", true, None),
    ("players", "nickname", "TEXT", false, None),
    ("players", "player_type", "TEXT", true, Some("'human'")),
    ("players", "is_active", "INTEGER", true, Some("1")),
    (
        "players",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    (
        "players",
        "updated_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // league_players table
    ("league_players", "id", "TEXT", true, None),
    ("league_players", "league_id", "TEXT", true, None),
    ("league_players", "player_id", "TEXT", true, None),
    ("league_players", "is_active", "INTEGER", true, Some("1")),
    (
        "league_players",
        "joined_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    (
        "league_players",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // player_aliases table
    ("player_aliases", "id", "TEXT", true, None),
    ("player_aliases", "primary_player_id", "TEXT", true, None),
    ("player_aliases", "alias_player_id", "TEXT", true, None),
    ("player_aliases", "created_by", "TEXT", true, None),
    (
        "player_aliases",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // matches table
    ("matches", "id", "TEXT", true, None),
    ("matches", "season_id", "TEXT", true, None),
    ("matches", "recorded_at", "TEXT", true, None), // NO DEFAULT, NOT NULL
    ("matches", "score_metadata_json", "TEXT", false, None),
    ("matches", "is_corrected", "INTEGER", true, Some("0")),
    (
        "matches",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // match_participants table
    ("match_participants", "id", "TEXT", true, None),
    ("match_participants", "match_id", "TEXT", true, None),
    ("match_participants", "player_id", "TEXT", true, None),
    ("match_participants", "placement", "INTEGER", true, None),
    ("match_participants", "rating_before", "TEXT", false, None),
    ("match_participants", "rating_after", "TEXT", false, None),
    (
        "match_participants",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // match_audit_log table
    ("match_audit_log", "id", "TEXT", true, None),
    ("match_audit_log", "match_id", "TEXT", true, None),
    ("match_audit_log", "actor_user_id", "TEXT", true, None),
    ("match_audit_log", "original_data_json", "TEXT", true, None),
    ("match_audit_log", "corrected_data_json", "TEXT", true, None),
    ("match_audit_log", "reason", "TEXT", false, None),
    (
        "match_audit_log",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // rating_snapshots table
    ("rating_snapshots", "id", "TEXT", true, None),
    ("rating_snapshots", "season_id", "TEXT", true, None),
    ("rating_snapshots", "player_id", "TEXT", true, None),
    ("rating_snapshots", "match_id", "TEXT", false, None),
    (
        "rating_snapshots",
        "conservative_rating",
        "REAL",
        true,
        None,
    ),
    ("rating_snapshots", "rating_json", "TEXT", true, None),
    (
        "rating_snapshots",
        "timestamp",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    (
        "rating_snapshots",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    // recalculation_jobs table
    ("recalculation_jobs", "id", "TEXT", true, None),
    ("recalculation_jobs", "season_id", "TEXT", true, None),
    ("recalculation_jobs", "job_type", "TEXT", true, None),
    (
        "recalculation_jobs",
        "status",
        "TEXT",
        true,
        Some("'queued'"),
    ),
    (
        "recalculation_jobs",
        "created_at",
        "TEXT",
        true,
        Some("CURRENT_TIMESTAMP"),
    ),
    ("recalculation_jobs", "completed_at", "TEXT", false, None),
];

/// Creates an in-memory SQLite database and runs migrations
async fn setup_test_database() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite pool");

    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");

    if migrations_path.exists() {
        let migrator = Migrator::new(migrations_path)
            .await
            .expect("Failed to create migrator");

        migrator.run(&pool).await.expect("Failed to run migrations");
    } else {
        panic!(
            "Migrations directory not found at {}. Migration tasks (907.2.2-907.2.5) must create migration files first.",
            migrations_path.display()
        );
    }

    pool
}

/// Get all table names from sqlite_master
async fn get_all_tables(pool: &Pool<Sqlite>) -> HashSet<String> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name"
    )
    .fetch_all(pool)
    .await
    .expect("Failed to query tables");

    rows.into_iter().map(|(name,)| name).collect()
}

/// Get column info for a table using PRAGMA table_info
struct ColumnInfo {
    name: String,
    col_type: String,
    not_null: bool,
    default_value: Option<String>,
}

async fn get_table_columns(pool: &Pool<Sqlite>, table: &str) -> Vec<ColumnInfo> {
    let query = format!("PRAGMA table_info({})", table);
    let rows: Vec<(i64, String, String, bool, Option<String>, bool)> = sqlx::query_as(&query)
        .fetch_all(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to get columns for table '{}': {}", table, e));

    rows.into_iter()
        .map(
            |(_, name, col_type, not_null, default_value, _)| ColumnInfo {
                name,
                col_type,
                not_null,
                default_value,
            },
        )
        .collect()
}

/// Get index info for a table using PRAGMA index_list
struct IndexInfo {
    name: String,
    unique: bool,
}

async fn get_table_indexes(pool: &Pool<Sqlite>, table: &str) -> Vec<IndexInfo> {
    let query = format!("PRAGMA index_list({})", table);
    let rows: Vec<(i64, String, bool, String, bool)> = sqlx::query_as(&query)
        .fetch_all(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to get indexes for table '{}': {}", table, e));

    rows.into_iter()
        .map(|(_, name, unique, _, _)| IndexInfo { name, unique })
        .collect()
}

/// Get columns for a specific index using PRAGMA index_info
async fn get_index_columns(pool: &Pool<Sqlite>, index_name: &str) -> Vec<String> {
    let query = format!("PRAGMA index_info({})", index_name);
    let rows: Vec<(i64, i64, String)> = sqlx::query_as(&query)
        .fetch_all(pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to get columns for index '{}': {}", index_name, e));

    let mut cols: Vec<_> = rows.into_iter().map(|(_, _, name)| name).collect();
    cols.sort();
    cols
}

/// Get FK constraints for a table using PRAGMA foreign_key_list
struct FKInfo {
    from: String,
    table: String,
    to: String,
    on_delete: String,
}

async fn get_table_fks(pool: &Pool<Sqlite>, table: &str) -> Vec<FKInfo> {
    let query = format!("PRAGMA foreign_key_list({})", table);
    let rows: Vec<(i64, i64, String, String, String, String, String, String)> =
        sqlx::query_as(&query)
            .fetch_all(pool)
            .await
            .unwrap_or_else(|e| panic!("Failed to get FKs for table '{}': {}", table, e));

    rows.into_iter()
        .map(|(_, _, table, from, to, _, on_delete, _)| FKInfo {
            from,
            table,
            to,
            on_delete,
        })
        .collect()
}

// ============================================================================
// TESTS: Table Existence
// ============================================================================

#[tokio::test]
async fn test_all_expected_tables_exist() {
    let pool = setup_test_database().await;
    let tables = get_all_tables(&pool).await;

    let expected: HashSet<&str> = EXPECTED_TABLES.iter().copied().collect();
    let actual: HashSet<&str> = tables
        .iter()
        .map(|s| s.as_str())
        .filter(|s| !s.starts_with("sqlite_") && *s != "_sqlx_migrations")
        .collect();

    let missing: Vec<_> = expected.difference(&actual).collect();
    let extra: Vec<_> = actual.difference(&expected).collect();

    if !missing.is_empty() {
        panic!(
            "Missing tables: {:?}\nExpected {} tables, found {}.\nAll expected: {:?}\nAll found: {:?}",
            missing,
            expected.len(),
            actual.len(),
            expected.iter().collect::<Vec<_>>(),
            actual.iter().collect::<Vec<_>>()
        );
    }

    if !extra.is_empty() {
        println!(
            "Note: Extra tables found (not in expected list): {:?}",
            extra
        );
    }

    assert_eq!(
        expected.len(),
        actual.len(),
        "Expected {} tables but found {}. Missing: {:?}",
        expected.len(),
        actual.len(),
        missing
    );
}

#[tokio::test]
async fn test_auth_tables_exist() {
    let pool = setup_test_database().await;
    let tables = get_all_tables(&pool).await;

    let auth_tables = [
        "users",
        "login_attempts",
        "sessions",
        "invite_tokens",
        "player_account_links",
    ];

    for table in &auth_tables {
        assert!(
            tables.contains(*table),
            "Auth table '{}' not found. Migration task 907.2.2 should create this table.",
            table
        );
    }
}

#[tokio::test]
async fn test_league_season_tables_exist() {
    let pool = setup_test_database().await;
    let tables = get_all_tables(&pool).await;

    let league_tables = ["leagues", "league_operators", "seasons"];

    for table in &league_tables {
        assert!(
            tables.contains(*table),
            "League/season table '{}' not found. Migration task 907.2.3 should create this table.",
            table
        );
    }
}

#[tokio::test]
async fn test_player_tables_exist() {
    let pool = setup_test_database().await;
    let tables = get_all_tables(&pool).await;

    let player_tables = ["players", "league_players", "player_aliases"];

    for table in &player_tables {
        assert!(
            tables.contains(*table),
            "Player table '{}' not found. Migration task 907.2.4 should create this table.",
            table
        );
    }
}

#[tokio::test]
async fn test_match_rating_tables_exist() {
    let pool = setup_test_database().await;
    let tables = get_all_tables(&pool).await;

    let match_tables = [
        "matches",
        "match_participants",
        "match_audit_log",
        "rating_snapshots",
        "recalculation_jobs",
    ];

    for table in &match_tables {
        assert!(
            tables.contains(*table),
            "Match/rating table '{}' not found. Migration task 907.2.5 should create this table.",
            table
        );
    }
}

// ============================================================================
// TESTS: Column Verification (per table)
// ============================================================================

#[tokio::test]
async fn test_users_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "users").await;
    verify_columns("users", &columns, "users");
}

#[tokio::test]
async fn test_login_attempts_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "login_attempts").await;
    verify_columns("login_attempts", &columns, "login_attempts");
}

#[tokio::test]
async fn test_sessions_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "sessions").await;
    verify_columns("sessions", &columns, "sessions");
}

#[tokio::test]
async fn test_invite_tokens_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "invite_tokens").await;
    verify_columns("invite_tokens", &columns, "invite_tokens");
}

#[tokio::test]
async fn test_player_account_links_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "player_account_links").await;
    verify_columns("player_account_links", &columns, "player_account_links");
}

#[tokio::test]
async fn test_leagues_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "leagues").await;
    verify_columns("leagues", &columns, "leagues");
}

#[tokio::test]
async fn test_league_operators_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "league_operators").await;
    verify_columns("league_operators", &columns, "league_operators");
}

#[tokio::test]
async fn test_seasons_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "seasons").await;
    verify_columns("seasons", &columns, "seasons");
}

#[tokio::test]
async fn test_players_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "players").await;
    verify_columns("players", &columns, "players");
}

#[tokio::test]
async fn test_league_players_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "league_players").await;
    verify_columns("league_players", &columns, "league_players");
}

#[tokio::test]
async fn test_player_aliases_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "player_aliases").await;
    verify_columns("player_aliases", &columns, "player_aliases");
}

#[tokio::test]
async fn test_matches_table_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "matches").await;
    verify_columns("matches", &columns, "matches");
}

#[tokio::test]
async fn test_match_participants_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "match_participants").await;
    verify_columns("match_participants", &columns, "match_participants");
}

#[tokio::test]
async fn test_match_audit_log_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "match_audit_log").await;
    verify_columns("match_audit_log", &columns, "match_audit_log");
}

#[tokio::test]
async fn test_rating_snapshots_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "rating_snapshots").await;
    verify_columns("rating_snapshots", &columns, "rating_snapshots");
}

#[tokio::test]
async fn test_recalculation_jobs_columns() {
    let pool = setup_test_database().await;
    let columns = get_table_columns(&pool, "recalculation_jobs").await;
    verify_columns("recalculation_jobs", &columns, "recalculation_jobs");
}

/// Helper to verify columns for a specific table
fn verify_columns(table_name: &str, actual_columns: &[ColumnInfo], expected_table: &str) {
    let expected: Vec<_> = EXPECTED_COLUMNS
        .iter()
        .filter(|(t, _, _, _, _)| *t == expected_table)
        .collect();

    let actual_map: HashMap<&str, &ColumnInfo> = actual_columns
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut errors = Vec::new();

    for (_exp_table, exp_name, exp_type, exp_not_null, exp_default) in &expected {
        let col = actual_map.get(*exp_name);

        match col {
            None => {
                errors.push(format!("  Missing column: '{}'", exp_name));
            }
            Some(actual) => {
                let actual_type_upper = actual.col_type.to_uppercase();
                let exp_type_upper = exp_type.to_uppercase();
                if actual_type_upper != exp_type_upper {
                    errors.push(format!(
                        "  Column '{}': expected type '{}', got '{}'",
                        exp_name, exp_type, actual.col_type
                    ));
                }

                if *exp_not_null && !actual.not_null {
                    errors.push(format!(
                        "  Column '{}': expected NOT NULL, but allows NULL",
                        exp_name
                    ));
                }

                if let Some(exp_default) = exp_default {
                    let actual_default = actual.default_value.as_deref().unwrap_or("(no default)");

                    let normalized_actual = actual_default
                        .to_uppercase()
                        .replace('"', "'")
                        .trim()
                        .to_string();
                    let normalized_expected = exp_default.to_uppercase().trim().to_string();

                    if normalized_actual != normalized_expected {
                        errors.push(format!(
                            "  Column '{}': expected default '{}', got '{}'",
                            exp_name, exp_default, actual_default
                        ));
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "Column verification failed for table '{}':\n{}",
            table_name,
            errors.join("\n")
        );
    }
}

// ============================================================================
// TESTS: Index Verification
// ============================================================================

#[tokio::test]
async fn test_rating_snapshots_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "rating_snapshots").await;
}

#[tokio::test]
async fn test_match_participants_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "match_participants").await;
}

#[tokio::test]
async fn test_matches_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "matches").await;
}

#[tokio::test]
async fn test_recalculation_jobs_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "recalculation_jobs").await;
}

#[tokio::test]
async fn test_players_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "players").await;
}

#[tokio::test]
async fn test_league_operators_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "league_operators").await;
}

#[tokio::test]
async fn test_leagues_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "leagues").await;
}

#[tokio::test]
async fn test_login_attempts_indexes() {
    let pool = setup_test_database().await;
    verify_table_indexes(&pool, "login_attempts").await;
}

/// Helper to verify indexes for a specific table
async fn verify_table_indexes(pool: &Pool<Sqlite>, table: &str) {
    let expected_for_table: Vec<_> = EXPECTED_INDEXES
        .iter()
        .filter(|(t, _, _)| *t == table)
        .collect();

    if expected_for_table.is_empty() {
        return;
    }

    let indexes = get_table_indexes(pool, table).await;

    let mut errors = Vec::new();

    for (_, exp_name, _exp_cols) in &expected_for_table {
        let found = indexes.iter().find(|idx| idx.name == *exp_name);

        if found.is_none() {
            errors.push(format!(
                "  Missing index '{}' on table '{}' with columns {:?}",
                exp_name, table, _exp_cols
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "Index verification failed for table '{}':\n{}\nExisting indexes: {:?}",
            table,
            errors.join("\n"),
            indexes.iter().map(|i| &i.name).collect::<Vec<_>>()
        );
    }
}

// ============================================================================
// TESTS: Foreign Key Verification
// ============================================================================

#[tokio::test]
async fn test_auth_fk_constraints() {
    let pool = setup_test_database().await;
    verify_fk_for_table(&pool, "login_attempts").await;
    verify_fk_for_table(&pool, "sessions").await;
    verify_fk_for_table(&pool, "invite_tokens").await;
    verify_fk_for_table(&pool, "player_account_links").await;
}

#[tokio::test]
async fn test_league_fk_constraints() {
    let pool = setup_test_database().await;
    verify_fk_for_table(&pool, "league_operators").await;
    verify_fk_for_table(&pool, "seasons").await;
}

#[tokio::test]
async fn test_player_fk_constraints() {
    let pool = setup_test_database().await;
    verify_fk_for_table(&pool, "league_players").await;
    verify_fk_for_table(&pool, "player_aliases").await;
}

#[tokio::test]
async fn test_match_fk_constraints() {
    let pool = setup_test_database().await;
    verify_fk_for_table(&pool, "matches").await;
    verify_fk_for_table(&pool, "match_participants").await;
    verify_fk_for_table(&pool, "match_audit_log").await;
    verify_fk_for_table(&pool, "rating_snapshots").await;
    verify_fk_for_table(&pool, "recalculation_jobs").await;
}

/// Helper to verify FK constraints for a specific table
async fn verify_fk_for_table(pool: &Pool<Sqlite>, table: &str) {
    let expected_fks: Vec<_> = EXPECTED_FK_CONSTRAINTS
        .iter()
        .filter(|(t, _, _, _, _)| *t == table)
        .collect();

    if expected_fks.is_empty() {
        return;
    }

    let actual_fks = get_table_fks(pool, table).await;

    let mut errors = Vec::new();

    for (exp_table, exp_from, exp_ref_table, exp_ref_col, exp_on_delete) in &expected_fks {
        let found = actual_fks.iter().find(|fk| {
            fk.from == *exp_from
                && fk.table == *exp_ref_table
                && fk.to == *exp_ref_col
                && fk.on_delete.to_uppercase() == exp_on_delete.to_uppercase()
        });

        if found.is_none() {
            errors.push(format!(
                "  Missing FK: {}.{} -> {}.{} (ON DELETE {})",
                exp_table, exp_from, exp_ref_table, exp_ref_col, exp_on_delete
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "FK verification failed for table '{}':\n{}\nExisting FKs: {:?}",
            table,
            errors.join("\n"),
            actual_fks
                .iter()
                .map(|fk| format!(
                    "{}.{} -> {}.{} (ON DELETE {})",
                    table, fk.from, fk.table, fk.to, fk.on_delete
                ))
                .collect::<Vec<_>>()
        );
    }
}

// ============================================================================
// TESTS: Column Defaults
// ============================================================================

#[tokio::test]
async fn test_created_at_defaults() {
    let pool = setup_test_database().await;

    for table in EXPECTED_TABLES {
        let columns = get_table_columns(&pool, table).await;
        let created_at = columns.iter().find(|c| c.name == "created_at");

        match created_at {
            None => {
                panic!("Table '{}' missing 'created_at' column", table);
            }
            Some(col) => {
                let default = col.default_value.as_deref().unwrap_or("(no default)");
                let normalized = default.to_uppercase();
                assert!(
                    normalized.contains("CURRENT_TIMESTAMP"),
                    "Table '{}': created_at should have DEFAULT CURRENT_TIMESTAMP, got: '{}'",
                    table,
                    default
                );
            }
        }
    }
}

#[tokio::test]
async fn test_is_active_defaults() {
    let pool = setup_test_database().await;

    let columns = get_table_columns(&pool, "leagues").await;
    let is_active = columns
        .iter()
        .find(|c| c.name == "is_active")
        .expect("leagues.is_active not found");
    assert!(
        is_active
            .default_value
            .as_deref()
            .map(|d| d == "1")
            .unwrap_or(false),
        "leagues.is_active should have DEFAULT 1, got: {:?}",
        is_active.default_value
    );

    let columns = get_table_columns(&pool, "league_players").await;
    let is_active = columns
        .iter()
        .find(|c| c.name == "is_active")
        .expect("league_players.is_active not found");
    assert!(
        is_active
            .default_value
            .as_deref()
            .map(|d| d == "1")
            .unwrap_or(false),
        "league_players.is_active should have DEFAULT 1, got: {:?}",
        is_active.default_value
    );
}

#[tokio::test]
async fn test_is_archived_default() {
    let pool = setup_test_database().await;

    let columns = get_table_columns(&pool, "leagues").await;
    let is_archived = columns
        .iter()
        .find(|c| c.name == "is_archived")
        .expect("leagues.is_archived not found");
    assert!(
        is_archived
            .default_value
            .as_deref()
            .map(|d| d == "0")
            .unwrap_or(false),
        "leagues.is_archived should have DEFAULT 0, got: {:?}",
        is_archived.default_value
    );
}

#[tokio::test]
async fn test_force_password_change_default() {
    let pool = setup_test_database().await;

    let columns = get_table_columns(&pool, "users").await;
    let fpc = columns
        .iter()
        .find(|c| c.name == "force_password_change")
        .expect("users.force_password_change not found");
    assert!(
        fpc.default_value
            .as_deref()
            .map(|d| d == "0")
            .unwrap_or(false),
        "users.force_password_change should have DEFAULT 0, got: {:?}",
        fpc.default_value
    );
}

#[tokio::test]
async fn test_status_default() {
    let pool = setup_test_database().await;

    let columns = get_table_columns(&pool, "recalculation_jobs").await;
    let status = columns
        .iter()
        .find(|c| c.name == "status")
        .expect("recalculation_jobs.status not found");
    let default = status.default_value.as_deref().unwrap_or("(no default)");
    assert!(
        default.to_lowercase().contains("queued"),
        "recalculation_jobs.status should have DEFAULT 'queued', got: '{}'",
        default
    );
}

#[tokio::test]
async fn test_player_type_default() {
    let pool = setup_test_database().await;

    let columns = get_table_columns(&pool, "players").await;
    let player_type = columns
        .iter()
        .find(|c| c.name == "player_type")
        .expect("players.player_type not found");
    let default = player_type
        .default_value
        .as_deref()
        .unwrap_or("(no default)");
    assert!(
        default.to_lowercase().contains("human"),
        "players.player_type should have DEFAULT 'human', got: '{}'",
        default
    );
}

#[tokio::test]
async fn test_matches_recorded_at_no_default() {
    let pool = setup_test_database().await;

    let columns = get_table_columns(&pool, "matches").await;
    let recorded_at = columns
        .iter()
        .find(|c| c.name == "recorded_at")
        .expect("matches.recorded_at not found");

    assert!(
        recorded_at.not_null,
        "matches.recorded_at should be NOT NULL"
    );
    assert!(
        recorded_at.default_value.is_none(),
        "matches.recorded_at should have NO DEFAULT, got: {:?}",
        recorded_at.default_value
    );
}

// ============================================================================
// TESTS: UNIQUE Constraints
// ============================================================================

#[tokio::test]
async fn test_player_account_links_unique_constraints() {
    let pool = setup_test_database().await;
    let indexes = get_table_indexes(&pool, "player_account_links").await;

    let unique_indexes: Vec<_> = indexes.iter().filter(|i| i.unique).collect();

    let player_id_unique = unique_indexes.iter().any(|idx| {
        let cols = futures::executor::block_on(get_index_columns(&pool, &idx.name));
        cols.iter().any(|c| c == "player_id")
    });

    assert!(
        player_id_unique,
        "player_account_links should have a UNIQUE constraint on player_id. Unique indexes: {:?}",
        unique_indexes.iter().map(|i| &i.name).collect::<Vec<_>>()
    );

    let user_id_unique = unique_indexes.iter().any(|idx| {
        let cols = futures::executor::block_on(get_index_columns(&pool, &idx.name));
        cols.iter().any(|c| c == "user_id")
    });

    assert!(
        user_id_unique,
        "player_account_links should have a UNIQUE constraint on user_id. Unique indexes: {:?}",
        unique_indexes.iter().map(|i| &i.name).collect::<Vec<_>>()
    );
}

// ============================================================================
// TESTS: Comprehensive Schema Verification
// ============================================================================

#[tokio::test]
async fn test_complete_schema_verification() {
    let pool = setup_test_database().await;

    // Verify all tables exist
    let tables = get_all_tables(&pool).await;
    assert_eq!(
        tables.len(),
        EXPECTED_TABLES.len(),
        "Expected {} tables, found {}. Tables: {:?}",
        EXPECTED_TABLES.len(),
        tables.len(),
        tables
    );

    for expected_table in EXPECTED_TABLES {
        assert!(
            tables.contains(&expected_table.to_string()),
            "Table '{}' not found",
            expected_table
        );
    }

    // Verify each table has at least the expected columns
    for table in EXPECTED_TABLES {
        let columns = get_table_columns(&pool, table).await;
        assert!(!columns.is_empty(), "Table '{}' has no columns", table);

        let has_id = columns.iter().any(|c| c.name == "id");
        assert!(has_id, "Table '{}' missing 'id' column", table);
    }

    // Verify all expected FKs exist
    for (table, from, ref_table, ref_col, on_delete) in EXPECTED_FK_CONSTRAINTS {
        let fks = get_table_fks(&pool, table).await;
        let found = fks.iter().any(|fk| {
            fk.from == *from
                && fk.table == *ref_table
                && fk.to == *ref_col
                && fk.on_delete.to_uppercase() == on_delete.to_uppercase()
        });
        assert!(
            found,
            "Missing FK: {}.{} -> {}.{} (ON DELETE {})",
            table, from, ref_table, ref_col, on_delete
        );
    }
}
