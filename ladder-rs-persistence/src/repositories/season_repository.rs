//! Season repository for season lifecycle operations
//!
//! Manages season creation, closing, algorithm parameter updates, and seeding.

use crate::{PersistenceError, Result, Season};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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

impl SeasonRepository {
    /// Create a new season for a league
    pub async fn create_season(
        _pool: &SqlitePool,
        _league_id: &str,
        _algorithm: &str,
        _params: &AlgorithmParams,
        _seeding_choice: SeedingChoice,
    ) -> Result<Season> {
        Err(PersistenceError::Unknown(
            "create_season not yet implemented".into(),
        ))
    }

    /// Get a season by ID
    pub async fn get_season(_pool: &SqlitePool, _id: &str) -> Result<Option<Season>> {
        Err(PersistenceError::Unknown(
            "get_season not yet implemented".into(),
        ))
    }

    /// Get the current open season for a league
    pub async fn get_current_season(
        _pool: &SqlitePool,
        _league_id: &str,
    ) -> Result<Option<Season>> {
        Err(PersistenceError::Unknown(
            "get_current_season not yet implemented".into(),
        ))
    }

    /// List all seasons for a league
    pub async fn list_seasons(_pool: &SqlitePool, _league_id: &str) -> Result<Vec<Season>> {
        Err(PersistenceError::Unknown(
            "list_seasons not yet implemented".into(),
        ))
    }

    /// Close a season (sets is_open = false, sets end_date)
    pub async fn close_season(_pool: &SqlitePool, _id: &str) -> Result<()> {
        Err(PersistenceError::Unknown(
            "close_season not yet implemented".into(),
        ))
    }

    /// Update algorithm parameters for a season
    pub async fn update_season_params(
        _pool: &SqlitePool,
        _id: &str,
        _params: &AlgorithmParams,
    ) -> Result<Season> {
        Err(PersistenceError::Unknown(
            "update_season_params not yet implemented".into(),
        ))
    }

    /// Apply seeding from a prior season to a new season
    pub async fn apply_seeding(
        _pool: &SqlitePool,
        _from_season_id: &str,
        _to_season_id: &str,
        _choice: SeedingChoice,
    ) -> Result<()> {
        Err(PersistenceError::Unknown(
            "apply_seeding not yet implemented".into(),
        ))
    }
}
