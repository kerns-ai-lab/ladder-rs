//! Rating Engine Bridge — the seam between persistence and the ladder-rs rating math.
//!
//! Takes a set of current player ratings and a match outcome, calls the appropriate
//! `ladder-rs` algorithm, and returns new ratings as `RatingSnapshot` values ready
//! to be inserted into the database.
//!
//! The bridge inspects the algorithm result for convergence quality and pre-computes
//! the `conservative_rating` column to avoid per-query arithmetic in leaderboard queries.

use crate::{PersistenceError, RatingSnapshot, Result};
use serde::{Deserialize, Serialize};

/// Input to the rating engine bridge: a single player's pre-match state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingInput {
    /// Player's current rating value
    pub rating: f64,
    /// Uncertainty (RD for Glicko-2, sigma for TrueSkill, None for Elo)
    pub uncertainty: Option<f64>,
    /// Volatility (Glicko-2 only, None for Elo/TrueSkill)
    pub volatility: Option<f64>,
}

/// Output from the rating engine bridge for a single player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingOutput {
    /// New rating value
    pub rating: f64,
    /// New uncertainty
    pub uncertainty: Option<f64>,
    /// New volatility (Glicko-2 only)
    pub volatility: Option<f64>,
    /// Pre-computed conservative rating (rating - K * uncertainty)
    pub conservative_rating: f64,
}

/// Result of a full match computation through the bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResult {
    /// One RatingOutput per participant in the match
    pub outputs: Vec<RatingOutput>,
    /// Whether the algorithm fully converged ("converged" or "degraded")
    pub convergence_quality: String,
}

/// Match input for the rating engine bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchInput {
    /// Pre-match ratings for each participant
    pub ratings: Vec<RatingInput>,
    /// Placement for each participant (1-indexed, lower = better)
    pub placements: Vec<u32>,
    /// Whether each participant drew (true if they tied)
    pub draws: Vec<bool>,
}

/// Bridge between persistence and ladder-rs rating computation.
pub struct RatingEngineBridge;

impl RatingEngineBridge {
    /// Compute new ratings for all participants in a match.
    ///
    /// Takes the pre-match state and match outcome, calls the appropriate
    /// `ladder-rs` algorithm, and returns `RatingSnapshot` values ready
    /// for database insertion.
    ///
    /// # Arguments
    /// * `algorithm` — "elo", "glicko", "glicko2", or "trueskill"
    /// * `input` — pre-match ratings and match placements
    /// * `player_ids` — corresponding player IDs (same order as ratings)
    /// * `season_id` — the season this match belongs to
    /// * `match_id` — the match this computation is for
    pub fn compute(
        _algorithm: &str,
        _input: &MatchInput,
        _player_ids: &[String],
        _season_id: &str,
        _match_id: &str,
    ) -> Result<BridgeResult> {
        Err(PersistenceError::Unknown(
            "compute not yet implemented".into(),
        ))
    }

    /// Compute the conservative rating from a rating value and uncertainty.
    ///
    /// Per-algorithm formula:
    /// | Algorithm | Conservative Rating        |
    /// |-----------|----------------------------|
    /// | Elo       | rating                     |
    /// | Glicko-2  | mu - 2 * RD                |
    /// | TrueSkill | mu - 3 * sigma             |
    pub fn conservative_rating(algorithm: &str, rating: f64, uncertainty: Option<f64>) -> f64 {
        match algorithm {
            "elo" => rating,
            "glicko" | "glicko2" => {
                if let Some(rd) = uncertainty {
                    rating - 2.0 * rd
                } else {
                    rating
                }
            }
            "trueskill" => {
                if let Some(sigma) = uncertainty {
                    rating - 3.0 * sigma
                } else {
                    rating
                }
            }
            _ => rating,
        }
    }

    /// Convert a `BridgeResult` into `RatingSnapshot` values for persistence.
    pub fn to_snapshots(
        _result: &BridgeResult,
        _player_ids: &[String],
        _season_id: &str,
        _rating_period: i32,
    ) -> Result<Vec<RatingSnapshot>> {
        Err(PersistenceError::Unknown(
            "to_snapshots not yet implemented".into(),
        ))
    }
}
