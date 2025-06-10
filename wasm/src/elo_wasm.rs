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
    /// Returns a JavaScript array with two EloRating objects
    pub fn process_1v1(
        &self,
        player1: &EloRating,
        player2: &EloRating,
        outcome: MatchOutcome,
    ) -> Result<js_sys::Array, JsValue> {
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

        // Extract updated ratings and convert to JS array
        let new_p1 = result[0].player_ratings()[0].rating();
        let new_p2 = result[1].player_ratings()[0].rating();

        let array = js_sys::Array::new();
        array.push(&JsValue::from(EloRating::new(new_p1)));
        array.push(&JsValue::from(EloRating::new(new_p2)));

        Ok(array)
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
    /// Takes an array of ratings and an array of match data
    /// Match data format: [[player1_idx, player2_idx, outcome], ...]
    /// Returns updated ratings array
    pub fn batch_process(
        system: &EloSystem,
        ratings: &js_sys::Array,
        matches: &js_sys::Array,
    ) -> Result<js_sys::Array, JsValue> {
        // Convert ratings from JS array
        let mut ratings: Vec<EloRating> = ratings
            .iter()
            .map(|val| {
                val.dyn_into::<EloRating>()
                    .map(|r| r.clone())
                    .or_else(|_| {
                        // Try to parse as object with value property
                        let obj = js_sys::Object::from(val);
                        let value = js_sys::Reflect::get(&obj, &"value".into())
                            .ok()?
                            .as_f64()?;
                        Some(EloRating::new(value))
                    })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| JsValue::from_str("Invalid ratings array"))?;

        // Process each match
        for i in 0..matches.length() {
            let match_data = matches.get(i);
            let match_array = match_data.dyn_into::<js_sys::Array>()
                .map_err(|_| JsValue::from_str("Invalid match data"))?;

            if match_array.length() < 3 {
                return Err(JsValue::from_str("Match data must have [idx1, idx2, outcome]"));
            }

            let idx1 = match_array.get(0).as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid player index"))? as usize;
            let idx2 = match_array.get(1).as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid player index"))? as usize;
            let outcome_num = match_array.get(2).as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid outcome"))? as i32;

            if idx1 >= ratings.len() || idx2 >= ratings.len() {
                return Err(JsValue::from_str("Player index out of bounds"));
            }

            let outcome = match outcome_num {
                1 => MatchOutcome::Player1Win,
                2 => MatchOutcome::Player2Win,
                _ => MatchOutcome::Draw,
            };

            let updated = system.process_1v1(&ratings[idx1], &ratings[idx2], outcome)?;
            ratings[idx1] = updated.get(0).dyn_into::<EloRating>()
                .map_err(|_| JsValue::from_str("Failed to update rating"))?
                .clone();
            ratings[idx2] = updated.get(1).dyn_into::<EloRating>()
                .map_err(|_| JsValue::from_str("Failed to update rating"))?
                .clone();
        }

        // Convert back to JS array
        let result = js_sys::Array::new();
        for rating in ratings {
            result.push(&JsValue::from(rating));
        }

        Ok(result)
    }

    /// Creates a leaderboard from ratings
    /// Returns array of [index, rating] sorted by rating descending
    pub fn create_leaderboard(ratings: &js_sys::Array) -> Result<js_sys::Array, JsValue> {
        let mut indexed: Vec<(usize, f64)> = Vec::new();

        for i in 0..ratings.length() {
            let rating = ratings.get(i);
            let value = if let Ok(elo_rating) = rating.dyn_ref::<EloRating>() {
                elo_rating.value()
            } else {
                // Try to extract value from object
                let obj = js_sys::Object::from(rating);
                js_sys::Reflect::get(&obj, &"value".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| JsValue::from_str("Invalid rating in array"))?
            };
            indexed.push((i as usize, value));
        }

        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let result = js_sys::Array::new();
        for (idx, rating) in indexed {
            let entry = js_sys::Array::new();
            entry.push(&JsValue::from(idx as u32));
            entry.push(&JsValue::from(rating));
            result.push(&entry);
        }

        Ok(result)
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