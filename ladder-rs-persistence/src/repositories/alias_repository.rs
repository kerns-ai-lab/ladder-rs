//! Alias repository for player alias management
//!
//! Manages the `player_aliases` table. Creating or removing an alias triggers
//! recalculation job insertion for all affected seasons.

use crate::repositories::job_repository::JobRepository;
use crate::{PersistenceError, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// A player alias relationship
#[derive(Debug, Clone)]
pub struct PlayerAlias {
    pub id: String,
    pub primary_player_id: String,
    pub alias_player_id: String,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Repository for player alias operations
pub struct AliasRepository;

// ── Internal helpers ─────────────────────────────────────────────────────────

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

/// Parse a timestamp string from the database into DateTime<Utc>.
///
/// Handles both RFC 3339 format (with `T` separator and timezone suffix)
/// and SQLite's `CURRENT_TIMESTAMP` format (space separator, no timezone,
/// e.g. `2026-05-18 00:20:24`). SQLite timestamps are always in UTC.
fn parse_db_timestamp(s: &str) -> Result<chrono::DateTime<Utc>> {
    // First try RFC 3339 (with 'T' separator and timezone)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    // Try SQLite format: replace space with 'T', append 'Z' for UTC
    let normalized = format!("{}Z", s.replace(' ', "T"));
    chrono::DateTime::parse_from_rfc3339(&normalized)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| PersistenceError::DatabaseError(format!("Invalid timestamp '{}': {}", s, e)))
}

/// Find distinct season IDs where either of the given player IDs has match history.
async fn find_affected_seasons(
    pool: &SqlitePool,
    player_id_a: &str,
    player_id_b: &str,
) -> Result<Vec<String>> {
    #[derive(sqlx::FromRow)]
    struct SeasonRow {
        season_id: String,
    }

    let rows: Vec<SeasonRow> = sqlx::query_as::<_, SeasonRow>(
        r#"SELECT DISTINCT m.season_id
           FROM matches m
           INNER JOIN match_participants mp ON mp.match_id = m.id
           WHERE mp.player_id = ? OR mp.player_id = ?"#,
    )
    .bind(player_id_a)
    .bind(player_id_b)
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(rows.into_iter().map(|r| r.season_id).collect())
}

/// Insert recalculation jobs for the given seasons and return the job IDs.
async fn insert_recalc_jobs(
    pool: &SqlitePool,
    season_ids: &[String],
    triggered_by: &str,
) -> Result<Vec<String>> {
    let mut job_ids = Vec::with_capacity(season_ids.len());
    for season_id in season_ids {
        let job_id = JobRepository::insert_job(pool, season_id, triggered_by).await?;
        job_ids.push(job_id);
    }
    Ok(job_ids)
}

impl AliasRepository {
    /// Create an alias link between two players.
    /// Returns the job IDs for recalculation jobs inserted for affected seasons.
    pub async fn create_alias(
        pool: &SqlitePool,
        primary_player_id: &str,
        alias_player_id: &str,
        created_by: &str,
    ) -> Result<Vec<String>> {
        // Input validation
        if primary_player_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "primary_player_id cannot be empty".into(),
            ));
        }
        if alias_player_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "alias_player_id cannot be empty".into(),
            ));
        }
        if primary_player_id == alias_player_id {
            return Err(PersistenceError::InvalidInput(
                "Cannot create self-referencing alias: primary and alias are the same player"
                    .into(),
            ));
        }
        if created_by.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "created_by cannot be empty".into(),
            ));
        }

        // Insert the alias record
        let alias_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO player_aliases (id, primary_player_id, alias_player_id, created_by, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&alias_id)
        .bind(primary_player_id)
        .bind(alias_player_id)
        .bind(created_by)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?;

        // Find seasons where either player has match history
        let season_ids = find_affected_seasons(pool, primary_player_id, alias_player_id).await?;

        // Insert recalculation jobs for each affected season
        let job_ids = insert_recalc_jobs(pool, &season_ids, "alias_link").await?;

        Ok(job_ids)
    }

    /// Remove an alias link between two players.
    /// Returns the job IDs for recalculation jobs inserted for affected seasons.
    pub async fn remove_alias(
        pool: &SqlitePool,
        primary_player_id: &str,
        alias_player_id: &str,
    ) -> Result<Vec<String>> {
        // Input validation
        if primary_player_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "primary_player_id cannot be empty".into(),
            ));
        }
        if alias_player_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "alias_player_id cannot be empty".into(),
            ));
        }

        // Check if the alias link exists
        let existing: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM player_aliases WHERE primary_player_id = ? AND alias_player_id = ? LIMIT 1",
        )
        .bind(primary_player_id)
        .bind(alias_player_id)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error)?;

        let (alias_id,) = match existing {
            Some(row) => row,
            None => {
                // Idempotent: no link exists, nothing to do
                return Ok(Vec::new());
            }
        };

        // Find affected seasons BEFORE deleting the alias (player refs still valid)
        let season_ids = find_affected_seasons(pool, primary_player_id, alias_player_id).await?;

        // Delete the alias record
        sqlx::query("DELETE FROM player_aliases WHERE id = ?")
            .bind(&alias_id)
            .execute(pool)
            .await
            .map_err(map_sqlx_error)?;

        // Insert recalculation jobs for each affected season
        let job_ids = insert_recalc_jobs(pool, &season_ids, "alias_unlink").await?;

        Ok(job_ids)
    }

    /// Get all aliases for a player (where the player appears as primary or alias).
    pub async fn get_aliases(pool: &SqlitePool, player_id: &str) -> Result<Vec<PlayerAlias>> {
        #[derive(sqlx::FromRow)]
        struct AliasRow {
            id: String,
            primary_player_id: String,
            alias_player_id: String,
            created_by: String,
            created_at: String,
        }

        let rows: Vec<AliasRow> = sqlx::query_as::<_, AliasRow>(
            "SELECT id, primary_player_id, alias_player_id, created_by, created_at
             FROM player_aliases
             WHERE primary_player_id = ? OR alias_player_id = ?",
        )
        .bind(player_id)
        .bind(player_id)
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut aliases = Vec::with_capacity(rows.len());
        for row in rows {
            aliases.push(PlayerAlias {
                id: row.id,
                primary_player_id: row.primary_player_id,
                alias_player_id: row.alias_player_id,
                created_by: row.created_by,
                created_at: parse_db_timestamp(&row.created_at)?,
            });
        }

        Ok(aliases)
    }

    /// Resolve the full alias group for a player (all linked player IDs).
    ///
    /// Performs iterative graph traversal to find the transitive closure:
    /// starting from `player_id`, it repeatedly queries for directly linked
    /// players (as primary or alias) until no new players are found.
    ///
    /// Returns an error if the player does not exist in the `players` table.
    pub async fn resolve_alias_group(pool: &SqlitePool, player_id: &str) -> Result<Vec<String>> {
        // Check that the player exists
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM players WHERE id = ?")
            .bind(player_id)
            .fetch_one(pool)
            .await
            .map_err(map_sqlx_error)?;

        if exists.0 == 0 {
            return Err(PersistenceError::NotFound {
                entity: "player".into(),
                id: player_id.to_string(),
            });
        }

        // Build the connected component iteratively
        let mut group: std::collections::HashSet<String> = std::collections::HashSet::new();
        group.insert(player_id.to_string());

        let mut queue: Vec<String> = vec![player_id.to_string()];

        #[derive(sqlx::FromRow)]
        struct LinkedRow {
            linked_id: String,
        }

        while let Some(current) = queue.pop() {
            // Find all players directly linked to `current`
            let linked: Vec<LinkedRow> = sqlx::query_as::<_, LinkedRow>(
                "SELECT primary_player_id AS linked_id FROM player_aliases WHERE alias_player_id = ?
                 UNION
                 SELECT alias_player_id AS linked_id FROM player_aliases WHERE primary_player_id = ?",
            )
            .bind(&current)
            .bind(&current)
            .fetch_all(pool)
            .await
            .map_err(map_sqlx_error)?;

            for row in linked {
                if group.insert(row.linked_id.clone()) {
                    queue.push(row.linked_id);
                }
            }
        }

        let mut result: Vec<String> = group.into_iter().collect();
        result.sort();
        Ok(result)
    }
}
