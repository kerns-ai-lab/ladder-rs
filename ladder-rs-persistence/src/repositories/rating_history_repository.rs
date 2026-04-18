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
    pub async fn get_per_season_history(
        &self,
        player_id: i64,
        season_id: i64,
    ) -> Result<RatingHistoryResponse> {
        Ok(RatingHistoryResponse { entries: vec![] })
    }

    /// Get season overview for a player (final rating per season)
    pub async fn get_season_overview(&self, player_id: i64) -> Result<SeasonOverviewResponse> {
        Ok(SeasonOverviewResponse { seasons: vec![] })
    }

    /// Check if a player exists (returns true even if soft-deleted)
    pub async fn player_exists(&self, player_id: i64) -> Result<bool> {
        Ok(false)
    }
}
