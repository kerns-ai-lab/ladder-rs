//! Season repository for season lifecycle operations
//!
//! Manages season creation, closing, algorithm parameter updates, and seeding.

use crate::{PersistenceError, Result, Season};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

/// Algorithm parameters for a season
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmParams {
    /// Initial rating value for new players
    pub initial_rating: f64,
    /// Rating deviation (Glicko-2) or sigma (TrueSkill) for new players
    pub initial_deviation: Option<f64>,
    /// Optional per-algorithm parameters as JSON
    pub extra: Option<serde_json::Value>,
}

/// Seeding strategy for season transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeedingChoice {
    /// Map ordinal rank from prior season to new algorithm default-distribution range
    Ordinal,
    /// All players start at algorithm defaults
    Reset,
}

/// Repository for season operations
pub struct SeasonRepository;

/// Internal helper: map a sqlite row to a Season value.
fn row_to_season(row: &sqlx::sqlite::SqliteRow) -> std::result::Result<Season, sqlx::Error> {
    Ok(Season {
        id: row.try_get("id")?,
        league_id: row.try_get("league_id")?,
        number: row.try_get("number")?,
        algorithm: row.try_get("algorithm")?,
        is_open: row.try_get("is_open")?,
        start_date: row.try_get("start_date")?,
        end_date: row.try_get("end_date")?,
        created_at: row.try_get("created_at")?,
    })
}

impl SeasonRepository {
    /// Create a new season for a league
    pub async fn create_season(
        pool: &SqlitePool,
        league_id: &str,
        algorithm: &str,
        params: &AlgorithmParams,
        _seeding_choice: SeedingChoice,
    ) -> Result<Season> {
        // Validate inputs
        if league_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "league_id cannot be empty".into(),
            ));
        }
        if algorithm.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "algorithm cannot be empty".into(),
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();
        let params_json = serde_json::to_string(params).map_err(|e| {
            PersistenceError::InvalidInput(format!("Failed to serialize params: {}", e))
        })?;

        // Calculate the next season number for this league
        let number_row =
            sqlx::query("SELECT COALESCE(MAX(number), 0) + 1 FROM seasons WHERE league_id = ?")
                .bind(league_id)
                .fetch_one(pool)
                .await
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let number: i32 = number_row
            .try_get::<i32, _>(0)
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        // Insert the season
        sqlx::query(
            "INSERT INTO seasons (id, league_id, number, algorithm, params_json, is_open, start_date, end_date, created_at) \
             VALUES (?, ?, ?, ?, ?, 1, ?, NULL, ?)",
        )
        .bind(&id)
        .bind(league_id)
        .bind(number)
        .bind(algorithm)
        .bind(&params_json)
        .bind(&now_rfc3339)
        .bind(&now_rfc3339)
        .execute(pool)
        .await
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("FOREIGN KEY") {
                PersistenceError::InvalidInput(format!(
                    "League '{}' does not exist",
                    league_id
                ))
            } else {
                PersistenceError::DatabaseError(err_str)
            }
        })?;

        Ok(Season {
            id,
            league_id: league_id.to_string(),
            number,
            algorithm: algorithm.to_string(),
            is_open: true,
            start_date: now,
            end_date: None,
            created_at: now,
        })
    }

    /// Get a season by ID
    pub async fn get_season(pool: &SqlitePool, id: &str) -> Result<Option<Season>> {
        if id.is_empty() {
            return Ok(None);
        }

        let row = sqlx::query("SELECT * FROM seasons WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let season = row_to_season(&r)
                    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
                Ok(Some(season))
            }
            None => Ok(None),
        }
    }

    /// Get the current open season for a league
    pub async fn get_current_season(pool: &SqlitePool, league_id: &str) -> Result<Option<Season>> {
        let row = sqlx::query(
            "SELECT * FROM seasons WHERE league_id = ? AND is_open = 1 ORDER BY number DESC LIMIT 1",
        )
        .bind(league_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => {
                let season = row_to_season(&r)
                    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
                Ok(Some(season))
            }
            None => Ok(None),
        }
    }

    /// List all seasons for a league
    pub async fn list_seasons(pool: &SqlitePool, league_id: &str) -> Result<Vec<Season>> {
        let rows = sqlx::query("SELECT * FROM seasons WHERE league_id = ? ORDER BY number")
            .bind(league_id)
            .fetch_all(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let mut seasons = Vec::with_capacity(rows.len());
        for row in &rows {
            let season =
                row_to_season(row).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
            seasons.push(season);
        }

        Ok(seasons)
    }

    /// Close a season (sets is_open = false, sets end_date)
    pub async fn close_season(pool: &SqlitePool, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "season id cannot be empty".into(),
            ));
        }

        // Check if season exists and is already closed
        let existing = sqlx::query("SELECT is_open FROM seasons WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        match existing {
            Some(row) => {
                let is_open: bool = row
                    .try_get::<bool, _>(0)
                    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

                if !is_open {
                    // Already closed — idempotent: return Ok
                    return Ok(());
                }
            }
            None => {
                return Err(PersistenceError::NotFound {
                    entity: "season".into(),
                    id: id.to_string(),
                });
            }
        }

        let now_rfc3339 = Utc::now().to_rfc3339();

        sqlx::query("UPDATE seasons SET is_open = 0, end_date = ? WHERE id = ? AND is_open = 1")
            .bind(&now_rfc3339)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Update algorithm parameters for a season
    pub async fn update_season_params(
        pool: &SqlitePool,
        id: &str,
        params: &AlgorithmParams,
    ) -> Result<Season> {
        if id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "season id cannot be empty".into(),
            ));
        }

        let params_json = serde_json::to_string(params).map_err(|e| {
            PersistenceError::InvalidInput(format!("Failed to serialize params: {}", e))
        })?;

        // Only update if the season exists and is open
        let result = sqlx::query("UPDATE seasons SET params_json = ? WHERE id = ? AND is_open = 1")
            .bind(&params_json)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            // Check if season exists at all (to differentiate NotFound from closed)
            let exists = sqlx::query("SELECT is_open FROM seasons WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

            match exists {
                Some(row) => {
                    let is_open: bool = row
                        .try_get::<bool, _>(0)
                        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
                    if !is_open {
                        return Err(PersistenceError::Conflict(
                            "Cannot update params on a closed season".into(),
                        ));
                    }
                    // Unreachable in practice (rows_affected would be > 0 if is_open)
                    Err(PersistenceError::Unknown(
                        "Update succeeded but no rows affected".into(),
                    ))
                }
                None => Err(PersistenceError::NotFound {
                    entity: "season".into(),
                    id: id.to_string(),
                }),
            }
        } else {
            // Fetch the updated season
            let row = sqlx::query("SELECT * FROM seasons WHERE id = ?")
                .bind(id)
                .fetch_one(pool)
                .await
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

            let season =
                row_to_season(&row).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
            Ok(season)
        }
    }

    /// Apply seeding from a prior season to a new season
    pub async fn apply_seeding(
        pool: &SqlitePool,
        from_season_id: &str,
        to_season_id: &str,
        _choice: SeedingChoice,
    ) -> Result<()> {
        if from_season_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "from_season_id cannot be empty".into(),
            ));
        }
        if to_season_id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "to_season_id cannot be empty".into(),
            ));
        }
        if from_season_id == to_season_id {
            return Err(PersistenceError::InvalidInput(
                "Cannot seed from a season to itself".into(),
            ));
        }

        // Fetch both seasons
        let from_row = sqlx::query("SELECT league_id FROM seasons WHERE id = ?")
            .bind(from_season_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let from_league = match from_row {
            Some(row) => row
                .try_get::<String, _>(0)
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?,
            None => {
                return Err(PersistenceError::NotFound {
                    entity: "season".into(),
                    id: from_season_id.to_string(),
                });
            }
        };

        let to_row = sqlx::query("SELECT league_id FROM seasons WHERE id = ?")
            .bind(to_season_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let to_league = match to_row {
            Some(row) => row
                .try_get::<String, _>(0)
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?,
            None => {
                return Err(PersistenceError::NotFound {
                    entity: "season".into(),
                    id: to_season_id.to_string(),
                });
            }
        };

        if from_league != to_league {
            return Err(PersistenceError::InvalidInput(format!(
                "Cannot seed across leagues: from league '{}' to league '{}'",
                from_league, to_league
            )));
        }

        // Seeding is validated and acknowledged.
        // Actual rating propagation (Ordinal) or reset is a future concern;
        // the method succeeds once the seasons are confirmed to exist in the same league.
        Ok(())
    }
}
