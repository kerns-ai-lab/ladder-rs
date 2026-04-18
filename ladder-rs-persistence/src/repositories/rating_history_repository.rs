//! Rating history repository for querying player rating progression

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::{PersistenceError, Result};

/// A single rating snapshot entry in the history (after a match)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingHistoryEntry {
    /// Match ID that triggered this rating update
    pub match_id: i64,
    /// Timestamp when the match was recorded (ISO 8601 string)
    pub recorded_at: String,
    /// Player's rating (mu in Glicko-2/TrueSkill, rating in Elo)
    pub rating: f64,
    /// Rating deviation (RD) for Glicko-2; None for Elo/TrueSkill
    pub deviation: Option<f64>,
    /// Uncertainty (sigma) for TrueSkill; None for Elo/Glicko-2
    pub uncertainty: Option<f64>,
    /// Conservative rating (pre-computed sort key)
    pub conservative_rating: f64,
}

/// Per-season rating history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingHistoryResponse {
    /// Chronologically ordered entries
    pub entries: Vec<RatingHistoryEntry>,
}

/// Season overview entry showing final rating achieved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonOverviewEntry {
    /// Season ID
    pub season_id: i64,
    /// Algorithm type (elo, glicko2, trueskill)
    pub algorithm: String,
    /// Season start date (ISO 8601 string)
    pub start_date: String,
    /// Season end date (None if still active, ISO 8601 string)
    pub end_date: Option<String>,
    /// Final rating achieved in this season
    pub final_rating: f64,
    /// Final conservative rating
    pub final_conservative_rating: f64,
    /// Number of matches played
    pub match_count: i64,
}

/// Season overview response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonOverviewResponse {
    /// List of seasons player participated in
    pub seasons: Vec<SeasonOverviewEntry>,
}

/// Repository for rating history queries
pub struct RatingHistoryRepository {
    pool: SqlitePool,
}

impl RatingHistoryRepository {
    /// Create a new rating history repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get per-season rating history for a player in a specific season
    ///
    /// Returns ratings after each match in chronological order.
    /// Returns empty list if player has no matches in season (not 404).
    pub async fn get_per_season_history(
        &self,
        player_id: i64,
        season_id: i64,
    ) -> Result<RatingHistoryResponse> {
        let rows: Vec<(i64, String, f64, Option<f64>, Option<f64>, f64)> = sqlx::query_as(
            r#"
            SELECT
                rs.match_id,
                m.recorded_at,
                rs.rating,
                rs.deviation,
                rs.uncertainty,
                rs.conservative_rating
            FROM rating_snapshots rs
            JOIN matches m ON rs.match_id = m.id
            WHERE rs.player_id = ? AND rs.season_id = ?
            ORDER BY m.recorded_at ASC
            "#,
        )
        .bind(player_id)
        .bind(season_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(format!("Failed to get per-season history: {}", e)))?;

        let entries = rows
            .into_iter()
            .map(|(match_id, recorded_at, rating, deviation, uncertainty, conservative_rating)| {
                RatingHistoryEntry {
                    match_id,
                    recorded_at,
                    rating,
                    deviation,
                    uncertainty,
                    conservative_rating,
                }
            })
            .collect();

        Ok(RatingHistoryResponse { entries })
    }

    /// Get season overview for a player (final rating per season)
    ///
    /// Returns all seasons player participated in with final ratings.
    pub async fn get_season_overview(&self, player_id: i64) -> Result<SeasonOverviewResponse> {
        let rows: Vec<(i64, String, String, Option<String>, f64, f64, i64)> = sqlx::query_as(
            r#"
            SELECT
                s.id as season_id,
                s.algorithm,
                s.start_date,
                s.end_date,
                COALESCE((
                    SELECT rs.rating FROM rating_snapshots rs
                    WHERE rs.player_id = ? AND rs.season_id = s.id
                    ORDER BY rs.match_id DESC
                    LIMIT 1
                ), 0.0) as final_rating,
                COALESCE((
                    SELECT rs.conservative_rating FROM rating_snapshots rs
                    WHERE rs.player_id = ? AND rs.season_id = s.id
                    ORDER BY rs.match_id DESC
                    LIMIT 1
                ), 0.0) as final_conservative_rating,
                COUNT(DISTINCT rs.match_id) as match_count
            FROM seasons s
            LEFT JOIN rating_snapshots rs ON s.id = rs.season_id AND rs.player_id = ?
            WHERE s.id IN (
                SELECT DISTINCT season_id FROM rating_snapshots WHERE player_id = ?
            )
            GROUP BY s.id
            ORDER BY s.start_date DESC
            "#,
        )
        .bind(player_id)
        .bind(player_id)
        .bind(player_id)
        .bind(player_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(format!("Failed to get season overview: {}", e)))?;

        let seasons = rows
            .into_iter()
            .map(|(season_id, algorithm, start_date, end_date, final_rating, final_conservative_rating, match_count)| {
                SeasonOverviewEntry {
                    season_id,
                    algorithm,
                    start_date,
                    end_date,
                    final_rating,
                    final_conservative_rating,
                    match_count,
                }
            })
            .collect();

        Ok(SeasonOverviewResponse { seasons })
    }

    /// Check if a player exists (returns true even if soft-deleted)
    pub async fn player_exists(&self, player_id: i64) -> Result<bool> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM players WHERE id = ?"
        )
        .bind(player_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(format!("Failed to check player existence: {}", e)))?;

        Ok(result.0 > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_history_for_player_with_no_matches() {
        // TODO: Implement with test database
        todo!("Setup test database and verify empty results");
    }

    #[tokio::test]
    async fn test_history_ordered_by_timestamp() {
        // TODO: Implement with test database
        todo!("Setup test database and verify chronological ordering");
    }

    #[tokio::test]
    async fn test_season_overview_includes_all_seasons() {
        // TODO: Implement with test database
        todo!("Setup test database and verify all seasons included");
    }
}
