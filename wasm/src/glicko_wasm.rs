//! WASM bindings for the Glicko rating system
//!
//! This module provides JavaScript-friendly wrappers around the core Glicko
//! rating system, with optimizations for WASM usage.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use ladder_rs::glicko::{GlickoRating as CoreGlickoRating, Glicko as CoreGlickoSystem, GlickoTeamRating, GlickoConfig};
use ladder_rs::core::{GameOutcome, RatingSystem, TeamRating};

use crate::elo_wasm::MatchOutcome;

/// WASM-friendly Glicko rating wrapper
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlickoRating {
    mu: f64,
    rd: f64,
}

#[wasm_bindgen]
impl GlickoRating {
    /// Creates a new Glicko rating with the specified values
    #[wasm_bindgen(constructor)]
    pub fn new(mu: f64, rd: f64) -> Result<GlickoRating, JsValue> {
        if rd < 0.0 {
            return Err(JsValue::from_str("RD must be non-negative"));
        }
        Ok(Self { mu, rd })
    }

    /// Gets the rating mean (μ)
    #[wasm_bindgen(getter)]
    pub fn mu(&self) -> f64 {
        self.mu
    }

    /// Gets the rating deviation (RD)
    #[wasm_bindgen(getter)]
    pub fn rd(&self) -> f64 {
        self.rd
    }

    /// Gets the conservative rating (μ - 2*RD)
    pub fn conservative_rating(&self) -> f64 {
        self.mu - 2.0 * self.rd
    }

    /// Serializes the rating to a string
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserializes a rating from a string
    pub fn deserialize(data: &str) -> Result<GlickoRating, JsValue> {
        serde_json::from_str(data)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))
    }

    /// Converts to JSON string for JavaScript interop
    pub fn to_json(&self) -> String {
        self.serialize()
    }

    /// Creates from JSON string
    pub fn from_json(json: &str) -> Result<GlickoRating, JsValue> {
        Self::deserialize(json)
    }
}

/// Result of a 1v1 match processing in Glicko
#[wasm_bindgen]
pub struct GlickoMatchResult {
    player1_rating: f64,
    player1_rd: f64,
    player2_rating: f64,
    player2_rd: f64,
}

#[wasm_bindgen]
impl GlickoMatchResult {
    /// Gets the updated rating for player 1
    #[wasm_bindgen(getter)]
    pub fn player1_rating(&self) -> f64 {
        self.player1_rating
    }

    /// Gets the updated RD for player 1
    #[wasm_bindgen(getter)]
    pub fn player1_rd(&self) -> f64 {
        self.player1_rd
    }

    /// Gets the updated rating for player 2
    #[wasm_bindgen(getter)]
    pub fn player2_rating(&self) -> f64 {
        self.player2_rating
    }

    /// Gets the updated RD for player 2
    #[wasm_bindgen(getter)]
    pub fn player2_rd(&self) -> f64 {
        self.player2_rd
    }
}

/// WASM-friendly Glicko system wrapper
#[wasm_bindgen]
pub struct GlickoSystem {
    c: f64,
    initial_rating: f64,
    initial_rd: f64,
    inner: CoreGlickoSystem,
}

#[wasm_bindgen]
impl GlickoSystem {
    /// Creates a new Glicko system with default parameters
    /// Default: c = 15.8, initial_rating = 1500.0, initial_rd = 350.0
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let config = GlickoConfig::default();
        Self {
            c: config.c,
            initial_rating: 1500.0,
            initial_rd: 350.0,
            inner: CoreGlickoSystem::new(),
        }
    }

    /// Creates a new Glicko system with custom parameters
    pub fn with_parameters(c: f64, initial_rating: f64, initial_rd: f64) -> Result<GlickoSystem, JsValue> {
        if c <= 0.0 {
            return Err(JsValue::from_str("c parameter must be positive"));
        }
        if initial_rd <= 0.0 {
            return Err(JsValue::from_str("initial RD must be positive"));
        }
        
        let config = GlickoConfig {
            c,
            q: (10.0_f64).ln() / 400.0,
        };
        
        Ok(Self {
            c,
            initial_rating,
            initial_rd,
            inner: CoreGlickoSystem::with_config(config),
        })
    }

    /// Gets the c parameter
    #[wasm_bindgen(getter)]
    pub fn c(&self) -> f64 {
        self.c
    }

    /// Gets the initial rating
    #[wasm_bindgen(getter)]
    pub fn initial_rating(&self) -> f64 {
        self.initial_rating
    }

    /// Gets the initial RD
    #[wasm_bindgen(getter)]
    pub fn initial_rd(&self) -> f64 {
        self.initial_rd
    }

    /// Creates a new rating with the default values
    pub fn create_rating(&self) -> GlickoRating {
        GlickoRating {
            mu: self.initial_rating,
            rd: self.initial_rd,
        }
    }

    /// Creates a rating with specific values
    pub fn create_rating_with_values(&self, mu: f64, rd: f64) -> Result<GlickoRating, JsValue> {
        GlickoRating::new(mu, rd)
    }

    /// Processes a 1v1 match and returns updated ratings
    pub fn process_1v1(
        &self,
        player1: &GlickoRating,
        player2: &GlickoRating,
        outcome: MatchOutcome,
    ) -> Result<GlickoMatchResult, JsValue> {
        // Convert to core types
        let team1 = GlickoTeamRating::from_player_ratings(vec![
            CoreGlickoRating::new(player1.mu, player1.rd)
        ]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![
            CoreGlickoRating::new(player2.mu, player2.rd)
        ]);

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
        let new_p1 = &result[0].player_ratings()[0];
        let new_p2 = &result[1].player_ratings()[0];

        Ok(GlickoMatchResult {
            player1_rating: new_p1.mu,
            player1_rd: new_p1.rd,
            player2_rating: new_p2.mu,
            player2_rd: new_p2.rd,
        })
    }

    /// Applies rating periods without matches (increases RD)
    pub fn apply_rating_period(&self, rating: &GlickoRating, periods: u32) -> Result<GlickoRating, JsValue> {
        let mut rd = rating.rd;
        
        // Apply RD increase for each period
        for _ in 0..periods {
            rd = (rd * rd + self.c * self.c).sqrt();
            // Cap RD at initial value
            if rd > self.initial_rd {
                rd = self.initial_rd;
                break;
            }
        }
        
        Ok(GlickoRating {
            mu: rating.mu,
            rd,
        })
    }

    /// Calculates the win probability for player1
    pub fn win_probability(&self, player1: &GlickoRating, player2: &GlickoRating) -> f64 {
        // Using the Glicko formula
        let q = (10.0_f64).ln() / 400.0;
        let g = 1.0 / (1.0 + 3.0 * q * q * player2.rd * player2.rd / (std::f64::consts::PI * std::f64::consts::PI)).sqrt();
        1.0 / (1.0 + 10.0_f64.powf(-g * (player1.mu - player2.mu) / 400.0))
    }

    /// Calculates match quality (0-1, higher is better)
    pub fn match_quality(&self, player1: &GlickoRating, player2: &GlickoRating) -> f64 {
        let team1 = GlickoTeamRating::from_player_ratings(vec![
            CoreGlickoRating::new(player1.mu, player1.rd)
        ]);
        let team2 = GlickoTeamRating::from_player_ratings(vec![
            CoreGlickoRating::new(player2.mu, player2.rd)
        ]);

        self.inner
            .calculate_match_quality(&[team1, team2])
            .unwrap_or(0.0)
    }

    /// Serializes the system configuration
    pub fn serialize(&self) -> String {
        serde_json::to_string(&SystemConfig {
            c: self.c,
            initial_rating: self.initial_rating,
            initial_rd: self.initial_rd,
        })
        .unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserializes system configuration
    pub fn deserialize(data: &str) -> Result<GlickoSystem, JsValue> {
        let config: SystemConfig = serde_json::from_str(data)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        Self::with_parameters(config.c, config.initial_rating, config.initial_rd)
    }

    /// Processes a 1v1 match and returns updated ratings as JSON
    /// Returns: {"player1": {"mu": 1520, "rd": 180}, "player2": {"mu": 1480, "rd": 190}}
    pub fn process_1v1_json(
        &self,
        player1_mu: f64,
        player1_rd: f64,
        player2_mu: f64,
        player2_rd: f64,
        outcome: MatchOutcome,
    ) -> Result<String, JsValue> {
        let p1 = GlickoRating::new(player1_mu, player1_rd)?;
        let p2 = GlickoRating::new(player2_mu, player2_rd)?;
        
        let result = self.process_1v1(&p1, &p2, outcome)?;
        
        let json_result = serde_json::json!({
            "player1": {
                "mu": result.player1_rating,
                "rd": result.player1_rd
            },
            "player2": {
                "mu": result.player2_rating,
                "rd": result.player2_rd
            }
        });
        
        serde_json::to_string(&json_result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

#[derive(Serialize, Deserialize)]
struct SystemConfig {
    c: f64,
    initial_rating: f64,
    initial_rd: f64,
}

/// Utility functions for batch operations with Glicko
#[wasm_bindgen]
pub struct GlickoUtils;

#[wasm_bindgen]
impl GlickoUtils {
    /// Processes multiple matches in batch
    /// Takes JSON strings: ratings array and matches array
    /// Match data format: [[player1_idx, player2_idx, outcome], ...]
    /// Returns updated ratings as JSON string
    pub fn batch_process(
        system: &GlickoSystem,
        ratings_json: &str,
        matches_json: &str,
    ) -> Result<String, JsValue> {
        // Parse ratings from JSON
        let mut ratings: Vec<GlickoRating> = serde_json::from_str(ratings_json)
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
            
            ratings[idx1] = GlickoRating {
                mu: result.player1_rating,
                rd: result.player1_rd,
            };
            ratings[idx2] = GlickoRating {
                mu: result.player2_rating,
                rd: result.player2_rd,
            };
        }

        // Convert back to JSON
        serde_json::to_string(&ratings)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Creates a leaderboard from ratings JSON
    /// Returns JSON array of [index, rating, rd] sorted by rating descending
    pub fn create_leaderboard(ratings_json: &str) -> Result<String, JsValue> {
        // Parse ratings
        let ratings: Vec<GlickoRating> = serde_json::from_str(ratings_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid ratings JSON: {}", e)))?;

        // Create indexed ratings
        let mut indexed: Vec<(usize, f64, f64)> = ratings
            .iter()
            .enumerate()
            .map(|(idx, rating)| (idx, rating.mu, rating.rd))
            .collect();

        // Sort by rating descending
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert to array format
        let result: Vec<Vec<serde_json::Value>> = indexed
            .into_iter()
            .map(|(idx, mu, rd)| vec![
                serde_json::Value::from(idx),
                serde_json::Value::from(mu),
                serde_json::Value::from(rd),
            ])
            .collect();

        // Return as JSON
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Helper to create a ratings array from values
    pub fn create_ratings_from_values(values_json: &str) -> Result<String, JsValue> {
        let values: Vec<Vec<f64>> = serde_json::from_str(values_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid values JSON: {}", e)))?;

        let ratings: Vec<GlickoRating> = values.into_iter()
            .map(|v| {
                if v.len() >= 2 {
                    GlickoRating::new(v[0], v[1])
                } else {
                    Err(JsValue::from_str("Each rating must have [mu, rd]"))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        serde_json::to_string(&ratings)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glicko_rating_creation() {
        let rating = GlickoRating::new(1600.0, 200.0).unwrap();
        assert_eq!(rating.mu(), 1600.0);
        assert_eq!(rating.rd(), 200.0);
    }

    #[test]
    fn test_glicko_system_defaults() {
        let system = GlickoSystem::new();
        assert_eq!(system.c(), 15.8);
        assert_eq!(system.initial_rating(), 1500.0);
        assert_eq!(system.initial_rd(), 350.0);
    }

    #[test]
    fn test_serialization() {
        let rating = GlickoRating::new(1700.0, 250.0).unwrap();
        let serialized = rating.serialize();
        let deserialized = GlickoRating::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.mu(), 1700.0);
        assert_eq!(deserialized.rd(), 250.0);
    }

    #[test]
    fn test_rd_increase() {
        let system = GlickoSystem::new();
        let rating = system.create_rating_with_values(1500.0, 200.0).unwrap();
        let updated = system.apply_rating_period(&rating, 2).unwrap();
        assert!(updated.rd() > rating.rd());
        assert_eq!(updated.mu(), rating.mu());
    }

    #[test]
    fn test_leaderboard_mixed_types() {
        let ratings_json = r#"[{"mu":1600,"rd":150},{"mu":1400,"rd":200},{"mu":1800,"rd":100}]"#;
        
        let leaderboard_json = GlickoUtils::create_leaderboard(ratings_json).unwrap();
        let leaderboard: Vec<Vec<serde_json::Value>> = serde_json::from_str(&leaderboard_json).unwrap();
        
        // Should be sorted by rating descending
        assert_eq!(leaderboard[0][0].as_u64(), Some(2)); // index 2 has rating 1800
        assert_eq!(leaderboard[0][1].as_f64(), Some(1800.0));
        assert_eq!(leaderboard[0][2].as_f64(), Some(100.0)); // RD
    }
}