//! League repository for league CRUD and operator management.
//!
//! Manages the `leagues` table and `league_operators` join table.
//! Visibility filtering enforces SR-AUTH-006 based on viewer context.

use crate::{PersistenceError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

/// A filter for listing leagues.
#[derive(Debug, Clone)]
pub struct LeagueFilter {
    /// Show only active leagues
    pub is_active: Option<bool>,
    /// Show only archived/unarchived
    pub is_archived: Option<bool>,
    /// Limit results
    pub limit: Option<usize>,
    /// Pagination offset
    pub offset: Option<usize>,
}

impl Default for LeagueFilter {
    fn default() -> Self {
        Self {
            is_active: Some(true),
            is_archived: Some(false),
            limit: Some(20),
            offset: Some(0),
        }
    }
}

/// A patch for updating a league.
#[derive(Debug, Clone)]
pub struct LeaguePatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<String>,
    pub is_active: Option<bool>,
}

/// A league model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct League {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub algorithm: String,
    pub visibility: String,
    pub is_active: bool,
    pub is_archived: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A league operator assignment.
#[derive(Debug, Clone)]
pub struct LeagueOperator {
    pub league_id: String,
    pub user_id: String,
    pub granted_by: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse an rfc3339 string from the database into a DateTime<Utc>.
fn parse_rfc3339(s: &str) -> std::result::Result<chrono::DateTime<Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Failed to parse timestamp '{}': {}", s, e))
}

/// Convert an `sqlx::Error` into a `PersistenceError`.
fn map_sqlx_error(e: sqlx::Error) -> PersistenceError {
    if let Some(db_err) = e.as_database_error() {
        if db_err.is_unique_violation() {
            return PersistenceError::Conflict(db_err.message().to_string());
        }
        if db_err.is_foreign_key_violation() {
            return PersistenceError::InvalidInput(format!(
                "Referenced entity does not exist: {}",
                db_err.message()
            ));
        }
    }
    PersistenceError::DatabaseError(e.to_string())
}

/// Build a `League` value from a database row.
fn row_to_league(row: &sqlx::sqlite::SqliteRow) -> std::result::Result<League, sqlx::Error> {
    let created_at_str: String = row.try_get("created_at")?;
    let updated_at_str: String = row.try_get("updated_at")?;
    let is_active_int: i32 = row.try_get("is_active")?;
    let is_archived_int: i32 = row.try_get("is_archived")?;

    Ok(League {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        algorithm: row.try_get("algorithm")?,
        visibility: row.try_get("visibility")?,
        is_active: is_active_int != 0,
        is_archived: is_archived_int != 0,
        created_at: parse_rfc3339(&created_at_str).map_err(|msg| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )))
        })?,
        updated_at: parse_rfc3339(&updated_at_str).map_err(|msg| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )))
        })?,
    })
}

/// Build a `LeagueOperator` value from a database row.
fn row_to_operator(
    row: &sqlx::sqlite::SqliteRow,
) -> std::result::Result<LeagueOperator, sqlx::Error> {
    let granted_at_str: String = row.try_get("granted_at")?;
    Ok(LeagueOperator {
        league_id: row.try_get("league_id")?,
        user_id: row.try_get("user_id")?,
        granted_by: row.try_get("granted_by")?,
        granted_at: parse_rfc3339(&granted_at_str).map_err(|msg| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )))
        })?,
    })
}

/// Repository for league operations.
pub struct LeagueRepository;

impl LeagueRepository {
    /// Create a new league.
    pub async fn create_league(
        pool: &SqlitePool,
        name: &str,
        description: &str,
        algorithm: &str,
        visibility: &str,
        _created_by: &str,
    ) -> Result<League> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(PersistenceError::InvalidInput(
                "League name cannot be empty".into(),
            ));
        }
        if algorithm.trim().is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Algorithm cannot be empty".into(),
            ));
        }
        if visibility.trim().is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Visibility cannot be empty".into(),
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();

        let desc_bind: Option<&str> = if description.is_empty() {
            None
        } else {
            Some(description)
        };

        sqlx::query(
            "INSERT INTO leagues (id, name, description, algorithm, visibility, is_active, is_archived, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, 1, 0, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(desc_bind)
        .bind(algorithm)
        .bind(visibility)
        .bind(&now_rfc3339)
        .bind(&now_rfc3339)
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(League {
            id,
            name: name.to_string(),
            description: if description.is_empty() {
                None
            } else {
                Some(description.to_string())
            },
            algorithm: algorithm.to_string(),
            visibility: visibility.to_string(),
            is_active: true,
            is_archived: false,
            created_at: now,
            updated_at: now,
        })
    }

    /// Get a league by ID.
    pub async fn get_league(pool: &SqlitePool, id: &str) -> Result<Option<League>> {
        let row = sqlx::query(
            "SELECT id, name, description, algorithm, visibility, is_active, is_archived, created_at, updated_at \
             FROM leagues WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(r) => {
                let league = row_to_league(&r)
                    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
                Ok(Some(league))
            }
            None => Ok(None),
        }
    }

    /// List leagues with dynamic filtering.
    pub async fn list_leagues(pool: &SqlitePool, filter: &LeagueFilter) -> Result<Vec<League>> {
        use sqlx::QueryBuilder;

        let mut builder = QueryBuilder::new(
            "SELECT id, name, description, algorithm, visibility, is_active, is_archived, created_at, updated_at \
             FROM leagues WHERE 1=1",
        );

        if let Some(is_active) = filter.is_active {
            builder.push(" AND is_active = ");
            builder.push_bind(is_active as i32);
        }
        if let Some(is_archived) = filter.is_archived {
            builder.push(" AND is_archived = ");
            builder.push_bind(is_archived as i32);
        }

        builder.push(" ORDER BY name ASC");

        if let Some(limit) = filter.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }
        if let Some(offset) = filter.offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }

        let rows = builder
            .build()
            .fetch_all(pool)
            .await
            .map_err(map_sqlx_error)?;

        let mut leagues = Vec::with_capacity(rows.len());
        for row in &rows {
            let league =
                row_to_league(row).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
            leagues.push(league);
        }
        Ok(leagues)
    }

    /// Update a league's fields.
    pub async fn update_league(pool: &SqlitePool, id: &str, patch: &LeaguePatch) -> Result<League> {
        if id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "League ID cannot be empty".into(),
            ));
        }

        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();

        // Build dynamic UPDATE SET clause
        let mut set_clauses: Vec<String> = Vec::new();
        let mut has_changes = false;

        // always update updated_at
        set_clauses.push("updated_at = ?".to_string());
        has_changes = true;

        if let Some(ref name) = patch.name {
            set_clauses.push("name = ?".to_string());
            has_changes = true;
        }
        if let Some(ref description) = patch.description {
            set_clauses.push("description = ?".to_string());
            has_changes = true;
        }
        if let Some(ref visibility) = patch.visibility {
            set_clauses.push("visibility = ?".to_string());
            has_changes = true;
        }
        if let Some(is_active) = patch.is_active {
            set_clauses.push("is_active = ?".to_string());
            has_changes = true;
        }

        if !has_changes {
            // Nothing to update; fetch and return current state
            return Self::get_league(pool, id)
                .await?
                .ok_or_else(|| PersistenceError::NotFound {
                    entity: "league".into(),
                    id: id.to_string(),
                });
        }

        // Build and execute the UPDATE
        let set_sql = set_clauses.join(", ");
        let mut sql = format!("UPDATE leagues SET {} WHERE id = ?", set_sql);

        let mut query = sqlx::query(&sql);

        // Bind updated_at
        query = query.bind(&now_rfc3339);

        // Bind optional fields
        if let Some(ref name) = patch.name {
            query = query.bind(name);
        }
        if let Some(ref description) = patch.description {
            query = query.bind(description);
        }
        if let Some(ref visibility) = patch.visibility {
            query = query.bind(visibility);
        }
        if let Some(is_active) = patch.is_active {
            query = query.bind(is_active as i32);
        }

        // Bind the WHERE id
        query = query.bind(id);

        let result = query.execute(pool).await.map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound {
                entity: "league".into(),
                id: id.to_string(),
            });
        }

        // Fetch and return the updated row
        let row = sqlx::query(
            "SELECT id, name, description, algorithm, visibility, is_active, is_archived, created_at, updated_at \
             FROM leagues WHERE id = ?",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error)?;

        let league =
            row_to_league(&row).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        Ok(league)
    }

    /// Archive a league.
    pub async fn archive_league(pool: &SqlitePool, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "League ID cannot be empty".into(),
            ));
        }

        let result = sqlx::query("UPDATE leagues SET is_archived = 1, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(pool)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound {
                entity: "league".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    /// Unarchive a league.
    pub async fn unarchive_league(pool: &SqlitePool, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "League ID cannot be empty".into(),
            ));
        }

        let result = sqlx::query("UPDATE leagues SET is_archived = 0, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(pool)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound {
                entity: "league".into(),
                id: id.to_string(),
            });
        }

        Ok(())
    }

    /// Assign an operator to a league.
    pub async fn assign_operator(
        pool: &SqlitePool,
        league_id: &str,
        user_id: &str,
        granted_by: &str,
    ) -> Result<()> {
        // Validate inputs
        if league_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "league_id cannot be empty".into(),
            ));
        }
        if user_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "user_id cannot be empty".into(),
            ));
        }

        // Validate that granted_by user exists
        let granted_by_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
            .bind(granted_by)
            .fetch_one(pool)
            .await
            .map_err(map_sqlx_error)?;

        if granted_by_count == 0 {
            return Err(PersistenceError::InvalidInput(format!(
                "granted_by user '{}' does not exist",
                granted_by
            )));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO league_operators (id, league_id, user_id, granted_by, granted_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(league_id)
        .bind(user_id)
        .bind(granted_by)
        .bind(&now_rfc3339)
        .execute(pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if db_err.is_unique_violation() {
                        // Idempotent: user is already an operator
                        return Ok(());
                    }
                    if db_err.is_foreign_key_violation() {
                        return Err(PersistenceError::InvalidInput(format!(
                            "Referenced entity does not exist: {}",
                            db_err.message()
                        )));
                    }
                }
                Err(PersistenceError::DatabaseError(e.to_string()))
            }
        }
    }

    /// Remove an operator from a league.
    pub async fn remove_operator(pool: &SqlitePool, league_id: &str, user_id: &str) -> Result<()> {
        // Validate inputs
        if league_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "league_id cannot be empty".into(),
            ));
        }
        if user_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "user_id cannot be empty".into(),
            ));
        }

        // Validate that the league exists
        let league_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leagues WHERE id = ?")
            .bind(league_id)
            .fetch_one(pool)
            .await
            .map_err(map_sqlx_error)?;

        if league_count == 0 {
            return Err(PersistenceError::InvalidInput(format!(
                "League '{}' does not exist",
                league_id
            )));
        }

        // Validate that the user exists
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(map_sqlx_error)?;

        if user_count == 0 {
            return Err(PersistenceError::InvalidInput(format!(
                "User '{}' does not exist",
                user_id
            )));
        }

        // Delete the operator assignment (idempotent if not assigned)
        sqlx::query("DELETE FROM league_operators WHERE league_id = ? AND user_id = ?")
            .bind(league_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    /// Get all operators for a league.
    pub async fn get_operators(pool: &SqlitePool, league_id: &str) -> Result<Vec<LeagueOperator>> {
        let rows = sqlx::query(
            "SELECT league_id, user_id, granted_by, granted_at FROM league_operators WHERE league_id = ?",
        )
        .bind(league_id)
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut operators = Vec::with_capacity(rows.len());
        for row in &rows {
            let op =
                row_to_operator(row).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
            operators.push(op);
        }
        Ok(operators)
    }

    /// Check if a user is an operator of a league.
    pub async fn is_operator(pool: &SqlitePool, league_id: &str, user_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM league_operators WHERE league_id = ? AND user_id = ?",
        )
        .bind(league_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(count > 0)
    }
}
