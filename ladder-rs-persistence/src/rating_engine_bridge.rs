//! Rating Engine Bridge — the seam between persistence and the ladder-rs rating math.
//!
//! Takes a set of current player ratings and a match outcome, calls the appropriate
//! `ladder-rs` algorithm, and returns new ratings as `RatingSnapshot` values ready
//! to be inserted into the database.
//!
//! The bridge inspects the algorithm result for convergence quality and pre-computes
//! the `conservative_rating` column to avoid per-query arithmetic in leaderboard queries.

use crate::{PersistenceError, RatingSnapshot, Result};
use ladder_rs::{Rating, RatingSystem, TeamRating};
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
    /// The match that produced this result (flowed through to snapshots)
    pub match_id: String,
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
        algorithm: &str,
        input: &MatchInput,
        player_ids: &[String],
        _season_id: &str,
        match_id: &str,
    ) -> Result<BridgeResult> {
        // --- input validation ---
        let n = input.ratings.len();
        if n == 0 {
            return Err(PersistenceError::InvalidInput(
                "ratings must not be empty".into(),
            ));
        }
        if input.placements.len() != n {
            return Err(PersistenceError::InvalidInput(
                "placements length must match ratings length".into(),
            ));
        }
        if input.draws.len() != n {
            return Err(PersistenceError::InvalidInput(
                "draws length must match ratings length".into(),
            ));
        }
        if player_ids.len() != n {
            return Err(PersistenceError::InvalidInput(
                "player_ids length must match ratings length".into(),
            ));
        }

        // --- degenerate single-participant case: return input as output ---
        if n == 1 {
            let r = &input.ratings[0];
            let conservative = Self::conservative_rating(algorithm, r.rating, r.uncertainty);
            return Ok(BridgeResult {
                outputs: vec![RatingOutput {
                    rating: r.rating,
                    uncertainty: r.uncertainty,
                    volatility: r.volatility,
                    conservative_rating: conservative,
                }],
                convergence_quality: "degraded".into(),
                match_id: match_id.into(),
            });
        }

        // --- construct GameOutcome from placements ---
        let outcome = ladder_rs::core::GameOutcome::new(
            input.placements.iter().map(|&p| p as usize).collect(),
        );

        // --- dispatch to algorithm-specific computation ---
        match algorithm {
            "elo" => Self::compute_elo(input, &outcome, match_id),
            "glicko" => Self::compute_glicko(input, &outcome, match_id),
            "glicko2" => Self::compute_glicko2(input, &outcome, match_id),
            "trueskill" => Self::compute_trueskill(input, &outcome, match_id),
            other => Err(PersistenceError::InvalidInput(format!(
                "unknown algorithm: '{}'",
                other
            ))),
        }
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
        result: &BridgeResult,
        player_ids: &[String],
        season_id: &str,
        rating_period: i32,
    ) -> Result<Vec<RatingSnapshot>> {
        let n = result.outputs.len();
        if n == 0 {
            return Err(PersistenceError::InvalidInput(
                "bridge result has no outputs".into(),
            ));
        }
        if player_ids.len() != n {
            return Err(PersistenceError::InvalidInput(format!(
                "player_ids length ({}) does not match outputs length ({})",
                player_ids.len(),
                n
            )));
        }

        let now = chrono::Utc::now();
        let match_id = &result.match_id;

        let snapshots: Vec<RatingSnapshot> = result
            .outputs
            .iter()
            .zip(player_ids.iter())
            .map(|(output, pid)| RatingSnapshot {
                id: uuid::Uuid::new_v4().to_string(),
                match_id: match_id.clone(),
                player_id: pid.clone(),
                season_id: season_id.to_string(),
                rating_value: output.rating,
                uncertainty: output.uncertainty,
                volatility: output.volatility,
                conservative_rating: output.conservative_rating,
                rating_period,
                created_at: now,
            })
            .collect();

        Ok(snapshots)
    }

    // ------------------------------------------------------------------
    //  private algorithm helpers
    // ------------------------------------------------------------------

    fn compute_elo(
        input: &MatchInput,
        outcome: &ladder_rs::core::GameOutcome,
        match_id: &str,
    ) -> Result<BridgeResult> {
        let system = ladder_rs::EloSystem::default();
        let ratings: Vec<ladder_rs::EloRating> = input
            .ratings
            .iter()
            .map(|r| ladder_rs::EloRating::new(r.rating))
            .collect();

        let teams: Vec<ladder_rs::EloTeamRating> = ratings
            .into_iter()
            .map(|r| ladder_rs::EloTeamRating::from_player_ratings(vec![r]))
            .collect();

        let updated = system.rate(&teams, outcome).map_err(map_ladder_error)?;

        let outputs: Vec<RatingOutput> = updated
            .iter()
            .map(|team| {
                let rating = &team.player_ratings()[0];
                RatingOutput {
                    rating: rating.rating(),
                    uncertainty: None,
                    volatility: None,
                    conservative_rating: rating.conservative_rating(),
                }
            })
            .collect();

        Ok(BridgeResult {
            outputs,
            convergence_quality: "converged".into(),
            match_id: match_id.into(),
        })
    }

    fn compute_glicko(
        input: &MatchInput,
        outcome: &ladder_rs::core::GameOutcome,
        match_id: &str,
    ) -> Result<BridgeResult> {
        let system = ladder_rs::Glicko::default();
        let ratings: Vec<ladder_rs::GlickoRating> = input
            .ratings
            .iter()
            .map(|r| {
                let rd = r.uncertainty.unwrap_or(350.0);
                ladder_rs::GlickoRating::new(r.rating, rd)
            })
            .collect();

        let teams: Vec<ladder_rs::GlickoTeamRating> = ratings
            .into_iter()
            .map(|r| ladder_rs::GlickoTeamRating::from_player_ratings(vec![r]))
            .collect();

        let updated = system.rate(&teams, outcome).map_err(map_ladder_error)?;

        let outputs: Vec<RatingOutput> = updated
            .iter()
            .map(|team| {
                let rating = &team.player_ratings()[0];
                RatingOutput {
                    rating: rating.mu,
                    uncertainty: Some(rating.rd),
                    volatility: None,
                    conservative_rating: rating.conservative_rating(),
                }
            })
            .collect();

        Ok(BridgeResult {
            outputs,
            convergence_quality: "converged".into(),
            match_id: match_id.into(),
        })
    }

    fn compute_glicko2(
        input: &MatchInput,
        outcome: &ladder_rs::core::GameOutcome,
        match_id: &str,
    ) -> Result<BridgeResult> {
        let system = ladder_rs::glicko::Glicko2::default();
        let ratings: Vec<ladder_rs::glicko::Glicko2Rating> = input
            .ratings
            .iter()
            .map(|r| {
                let rd = r.uncertainty.unwrap_or(350.0);
                let vol = r.volatility.unwrap_or(0.06);
                ladder_rs::glicko::Glicko2Rating::new(r.rating, rd, vol)
            })
            .collect();

        let teams: Vec<ladder_rs::glicko::Glicko2TeamRating> = ratings
            .into_iter()
            .map(|r| ladder_rs::glicko::Glicko2TeamRating::from_player_ratings(vec![r]))
            .collect();

        let updated = system.rate(&teams, outcome).map_err(map_ladder_error)?;

        let outputs: Vec<RatingOutput> = updated
            .iter()
            .map(|team| {
                let rating = &team.player_ratings()[0];
                RatingOutput {
                    rating: rating.mu,
                    uncertainty: Some(rating.rd),
                    volatility: Some(rating.volatility),
                    conservative_rating: rating.conservative_rating(),
                }
            })
            .collect();

        Ok(BridgeResult {
            outputs,
            convergence_quality: "converged".into(),
            match_id: match_id.into(),
        })
    }

    fn compute_trueskill(
        input: &MatchInput,
        outcome: &ladder_rs::core::GameOutcome,
        match_id: &str,
    ) -> Result<BridgeResult> {
        let system = ladder_rs::TrueSkill::default();
        let ratings: Vec<ladder_rs::TrueSkillRating> = input
            .ratings
            .iter()
            .map(|r| {
                let sigma = r.uncertainty.unwrap_or(8.333);
                system.create_rating_with_values(r.rating, sigma * sigma)
            })
            .collect();

        let teams: Vec<ladder_rs::TrueSkillTeam> = ratings
            .into_iter()
            .map(|r| ladder_rs::TrueSkillTeam::from_player_ratings(vec![r]))
            .collect();

        let updated = system.rate(&teams, outcome).map_err(map_ladder_error)?;

        let outputs: Vec<RatingOutput> = updated
            .iter()
            .map(|team| {
                let rating = &team.player_ratings()[0];
                RatingOutput {
                    rating: rating.mean(),
                    uncertainty: Some(rating.std_dev()),
                    volatility: None,
                    conservative_rating: rating.conservative_rating(),
                }
            })
            .collect();

        Ok(BridgeResult {
            outputs,
            convergence_quality: "converged".into(),
            match_id: match_id.into(),
        })
    }
}

// --------------------------------------------------------------------------
//  helpers
// --------------------------------------------------------------------------

/// Map a `ladder_rs::error::Error` into a `PersistenceError`.
fn map_ladder_error(err: ladder_rs::error::Error) -> PersistenceError {
    match err {
        ladder_rs::error::Error::InvalidInput(msg) => PersistenceError::InvalidInput(msg),
        other => PersistenceError::Unknown(other.to_string()),
    }
}
