//! WASM bindings for the Elo rating system
//!
//! This module provides JavaScript-friendly wrappers around the core Elo
//! rating system, with optimizations for WASM usage.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use ladder_rs::elo::{EloRating as CoreEloRating, EloSystem as CoreEloSystem, EloTeamRating};
use ladder_rs::core::{GameOutcome, Rating, RatingSystem, TeamRating};

/// Match outcome for 1v1 games
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum MatchOutcome {
    Player1Win,
    Player2Win,
    Draw,
}

/// Team match outcome
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum TeamOutcome {
    Team1Win,
    Team2Win,
    Draw,
}

/// WASM-friendly Elo rating wrapper
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EloRating {
    value: f64,
}

#[wasm_bindgen]
impl EloRating {
    /// Creates a new Elo rating with the specified value
    #[wasm_bindgen(constructor)]
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    /// Gets the rating value
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Serializes the rating to a string
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserializes a rating from a string
    pub fn deserialize(data: &str) -> Result<EloRating, JsValue> {
        serde_json::from_str(data)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))
    }

    /// Converts to JSON string for JavaScript interop
    pub fn to_json(&self) -> String {
        self.serialize()
    }

    /// Creates from JSON string
    pub fn from_json(json: &str) -> Result<EloRating, JsValue> {
        Self::deserialize(json)
    }
}

/// WASM-friendly Elo system wrapper
#[wasm_bindgen]
pub struct EloSystem {
    k_factor: f64,
    initial_rating: f64,
    inner: CoreEloSystem,
}

#[wasm_bindgen]
impl EloSystem {
    /// Creates a new Elo system with default parameters
    /// Default: k_factor = 32.0, initial_rating = 1500.0
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            k_factor: 32.0,
            initial_rating: 1500.0,
            inner: CoreEloSystem::new(),
        }
    }

    /// Creates a new Elo system with custom parameters
    pub fn with_parameters(k_factor: f64, initial_rating: f64) -> Self {
        // Ensure k_factor is positive
        let k_factor = k_factor.abs().max(1.0);
        
        // Note: The core EloSystem has different parameters
        // We'll use reasonable defaults for alpha and beta_elo
        let alpha = 0.1;
        let beta_elo = 200.0;
        
        Self {
            k_factor,
            initial_rating,
            inner: CoreEloSystem::with_parameters(k_factor, alpha, beta_elo, initial_rating),
        }
    }

    /// Gets the k-factor
    #[wasm_bindgen(getter)]
    pub fn k_factor(&self) -> f64 {
        self.k_factor
    }

    /// Gets the initial rating
    #[wasm_bindgen(getter)]
    pub fn initial_rating(&self) -> f64 {
        self.initial_rating
    }

    /// Creates a new rating with the default value
    pub fn create_rating(&self) -> EloRating {
        let core_rating = self.inner.create_rating();
        EloRating::new(core_rating.rating())
    }

    /// Creates a rating with a specific value
    pub fn create_rating_with_value(&self, value: f64) -> EloRating {
        EloRating::new(value)
    }

    /// Processes a 1v1 match and returns updated ratings
    pub fn process_1v1(
        &self,
        player1: &EloRating,
        player2: &EloRating,
        outcome: MatchOutcome,
    ) -> Result<Vec<EloRating>, JsValue> {
        // Convert to core types
        let team1 = EloTeamRating::new(CoreEloRating::new(player1.value));
        let team2 = EloTeamRating::new(CoreEloRating::new(player2.value));

        // Convert outcome
        let game_outcome = match outcome {
            MatchOutcome::Player1Win => GameOutcome::win(0, 2),
            MatchOutcome::Player2Win => GameOutcome::win(1, 2),
            MatchOutcome::Draw => GameOutcome::draw(2),
        };

        // Process the match
        let result = self.inner
            .rate(&[team1, team2], &game_outcome)
            .map_err(|e| JsValue::from_str(&format!("Rating error: {}", e)))?;

        // Extract updated ratings
        let new_p1 = result[0].player_ratings()[0].rating();
        let new_p2 = result[1].player_ratings()[0].rating();

        Ok(vec![EloRating::new(new_p1), EloRating::new(new_p2)])
    }

    /// Processes a team match and returns updated ratings
    pub fn process_team_match(
        &self,
        team1: &JsValue,
        team2: &JsValue,
        outcome: TeamOutcome,
    ) -> Result<Vec<Vec<EloRating>>, JsValue> {
        // Convert JavaScript arrays to Vec<EloRating>
        let team1_ratings: Vec<EloRating> = team1.into_serde()
            .map_err(|_| JsValue::from_str("Invalid team1 array"))?;
        let team2_ratings: Vec<EloRating> = team2.into_serde()
            .map_err(|_| JsValue::from_str("Invalid team2 array"))?;

        // For Elo, we need exactly one player per team
        if team1_ratings.len() != 1 || team2_ratings.len() != 1 {
            return Err(JsValue::from_str("Elo only supports 1v1 matches"));
        }

        // Convert to core types
        let core_team1 = EloTeamRating::new(CoreEloRating::new(team1_ratings[0].value));
        let core_team2 = EloTeamRating::new(CoreEloRating::new(team2_ratings[0].value));

        // Convert outcome
        let game_outcome = match outcome {
            TeamOutcome::Team1Win => GameOutcome::win(0, 2),
            TeamOutcome::Team2Win => GameOutcome::win(1, 2),
            TeamOutcome::Draw => GameOutcome::draw(2),
        };

        // Process the match
        let result = self.inner
            .rate(&[core_team1, core_team2], &game_outcome)
            .map_err(|e| JsValue::from_str(&format!("Rating error: {}", e)))?;

        // Extract updated ratings
        let new_team1 = vec![EloRating::new(result[0].player_ratings()[0].rating())];
        let new_team2 = vec![EloRating::new(result[1].player_ratings()[0].rating())];

        Ok(vec![new_team1, new_team2])
    }

    /// Calculates the win probability for player1
    pub fn win_probability(&self, player1: &EloRating, player2: &EloRating) -> f64 {
        // Using the standard Elo formula: 1 / (1 + 10^((R2-R1)/400))
        let rating_diff = player2.value - player1.value;
        1.0 / (1.0 + 10.0_f64.powf(rating_diff / 400.0))
    }

    /// Calculates match quality (0-1, higher is better)
    pub fn match_quality(&self, player1: &EloRating, player2: &EloRating) -> f64 {
        let team1 = EloTeamRating::new(CoreEloRating::new(player1.value));
        let team2 = EloTeamRating::new(CoreEloRating::new(player2.value));

        self.inner
            .calculate_match_quality(&[team1, team2])
            .unwrap_or(0.0)
    }

    /// Serializes the system configuration
    pub fn serialize(&self) -> String {
        serde_json::to_string(&SystemConfig {
            k_factor: self.k_factor,
            initial_rating: self.initial_rating,
        })
        .unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserializes system configuration
    pub fn deserialize(data: &str) -> Result<EloSystem, JsValue> {
        let config: SystemConfig = serde_json::from_str(data)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        Ok(Self::with_parameters(config.k_factor, config.initial_rating))
    }
}

#[derive(Serialize, Deserialize)]
struct SystemConfig {
    k_factor: f64,
    initial_rating: f64,
}

/// Utility functions for batch operations
#[wasm_bindgen]
pub struct EloUtils;

#[wasm_bindgen]
impl EloUtils {
    /// Processes multiple matches in batch
    /// Takes an array of match data: [[player1_idx, player2_idx, outcome], ...]
    pub fn batch_process(
        system: &EloSystem,
        ratings: &JsValue,
        matches: &JsValue,
    ) -> Result<Vec<EloRating>, JsValue> {
        let mut ratings: Vec<EloRating> = ratings.into_serde()
            .map_err(|_| JsValue::from_str("Invalid ratings array"))?;

        let matches: Vec<(usize, usize, i32)> = matches.into_serde()
            .map_err(|_| JsValue::from_str("Invalid matches array"))?;

        for (idx1, idx2, outcome_num) in matches {
            if idx1 >= ratings.len() || idx2 >= ratings.len() {
                return Err(JsValue::from_str("Invalid player index"));
            }

            let outcome = match outcome_num {
                1 => MatchOutcome::Player1Win,
                2 => MatchOutcome::Player2Win,
                _ => MatchOutcome::Draw,
            };

            let updated = system.process_1v1(&ratings[idx1], &ratings[idx2], outcome)?;
            ratings[idx1] = updated[0].clone();
            ratings[idx2] = updated[1].clone();
        }

        Ok(ratings)
    }

    /// Creates a leaderboard from ratings
    /// Returns array of [index, rating] sorted by rating descending
    pub fn create_leaderboard(ratings: &JsValue) -> Result<Vec<JsValue>, JsValue> {
        let ratings: Vec<EloRating> = ratings.into_serde()
            .map_err(|_| JsValue::from_str("Invalid ratings array"))?;

        let mut indexed: Vec<(usize, f64)> = ratings
            .iter()
            .enumerate()
            .map(|(idx, rating)| (idx, rating.value))
            .collect();

        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(indexed
            .into_iter()
            .map(|(idx, rating)| {
                let arr = js_sys::Array::new();
                arr.push(&JsValue::from(idx as u32));
                arr.push(&JsValue::from(rating));
                arr.into()
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elo_rating_creation() {
        let rating = EloRating::new(1600.0);
        assert_eq!(rating.value(), 1600.0);
    }

    #[test]
    fn test_elo_system_defaults() {
        let system = EloSystem::new();
        assert_eq!(system.k_factor(), 32.0);
        assert_eq!(system.initial_rating(), 1500.0);
    }

    #[test]
    fn test_serialization() {
        let rating = EloRating::new(1700.0);
        let serialized = rating.serialize();
        let deserialized = EloRating::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.value(), 1700.0);
    }
}