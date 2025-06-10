//! WASM bindings for the Elo rating system
//!
//! This module provides JavaScript-friendly wrappers around the core Elo
//! rating system, with optimizations for WASM usage.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use ladder_rs::elo::{EloRating as CoreEloRating, EloSystem as CoreEloSystem, EloTeamRating};
use ladder_rs::core::{GameOutcome, RatingSystem, TeamRating};

/// Match outcome for 1v1 games
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum MatchOutcome {
    Player1Win,
    Player2Win,
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

/// Result of a 1v1 match processing
#[wasm_bindgen]
pub struct MatchResult {
    player1_rating: f64,
    player2_rating: f64,
}

#[wasm_bindgen]
impl MatchResult {
    /// Gets the updated rating for player 1
    #[wasm_bindgen(getter)]
    pub fn player1_rating(&self) -> f64 {
        self.player1_rating
    }

    /// Gets the updated rating for player 2
    #[wasm_bindgen(getter)]
    pub fn player2_rating(&self) -> f64 {
        self.player2_rating
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
    ) -> Result<MatchResult, JsValue> {
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

        Ok(MatchResult {
            player1_rating: new_p1,
            player2_rating: new_p2,
        })
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

    /// Processes a 1v1 match and returns updated ratings as JSON
    /// Returns: {"player1": 1520, "player2": 1480}
    pub fn process_1v1_json(
        &self,
        player1_rating: f64,
        player2_rating: f64,
        outcome: MatchOutcome,
    ) -> Result<String, JsValue> {
        let p1 = EloRating::new(player1_rating);
        let p2 = EloRating::new(player2_rating);
        
        let result = self.process_1v1(&p1, &p2, outcome)?;
        
        let json_result = serde_json::json!({
            "player1": result.player1_rating,
            "player2": result.player2_rating
        });
        
        serde_json::to_string(&json_result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
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
    /// Takes JSON strings: ratings array and matches array
    /// Match data format: [[player1_idx, player2_idx, outcome], ...]
    /// Returns updated ratings as JSON string
    pub fn batch_process(
        system: &EloSystem,
        ratings_json: &str,
        matches_json: &str,
    ) -> Result<String, JsValue> {
        // Parse ratings from JSON
        let mut ratings: Vec<EloRating> = serde_json::from_str(ratings_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid ratings JSON: {}", e)))?;

        // Parse matches from JSON
        let matches: Vec<Vec<i32>> = serde_json::from_str(matches_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid matches JSON: {}", e)))?;

        // Process each match
        for match_data in matches {
            if match_data.len() < 3 {
                return Err(JsValue::from_str("Match data must have [idx1, idx2, outcome]"));
            }

            let idx1 = match_data[0] as usize;
            let idx2 = match_data[1] as usize;
            let outcome_num = match_data[2];

            if idx1 >= ratings.len() || idx2 >= ratings.len() {
                return Err(JsValue::from_str("Player index out of bounds"));
            }

            let outcome = match outcome_num {
                1 => MatchOutcome::Player1Win,
                2 => MatchOutcome::Player2Win,
                _ => MatchOutcome::Draw,
            };

            // Process the match
            let result = system.process_1v1(&ratings[idx1], &ratings[idx2], outcome)?;
            
            ratings[idx1] = EloRating::new(result.player1_rating);
            ratings[idx2] = EloRating::new(result.player2_rating);
        }

        // Convert back to JSON
        serde_json::to_string(&ratings)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Creates a leaderboard from ratings JSON
    /// Returns JSON array of [index, rating] sorted by rating descending
    pub fn create_leaderboard(ratings_json: &str) -> Result<String, JsValue> {
        // Parse ratings
        let ratings: Vec<EloRating> = serde_json::from_str(ratings_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid ratings JSON: {}", e)))?;

        // Create indexed ratings
        let mut indexed: Vec<(usize, f64)> = ratings
            .iter()
            .enumerate()
            .map(|(idx, rating)| (idx, rating.value))
            .collect();

        // Sort by rating descending
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert to array format
        let result: Vec<Vec<serde_json::Value>> = indexed
            .into_iter()
            .map(|(idx, rating)| vec![
                serde_json::Value::from(idx),
                serde_json::Value::from(rating),
            ])
            .collect();

        // Return as JSON
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Helper to create a ratings array from values
    pub fn create_ratings_from_values(values_json: &str) -> Result<String, JsValue> {
        let values: Vec<f64> = serde_json::from_str(values_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid values JSON: {}", e)))?;

        let ratings: Vec<EloRating> = values.into_iter()
            .map(|v| EloRating::new(v))
            .collect();

        serde_json::to_string(&ratings)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
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

    #[test]
    fn test_batch_processing() {
        let system = EloSystem::new();
        let ratings_json = r#"[{"value":1500},{"value":1500},{"value":1500}]"#;
        let matches_json = r#"[[0,1,1],[1,2,2]]"#;
        
        let result = EloUtils::batch_process(&system, ratings_json, matches_json);
        assert!(result.is_ok());
    }
}