//! WASM-specific type definitions and conversions
//!
//! This module contains type conversions between Rust types and
//! JavaScript/WASM boundary types, providing a clean API for JavaScript consumers.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// JavaScript-friendly rating representation
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsRating {
    /// Mean skill value (μ)
    mean: f64,
    /// Variance of skill (σ²)
    variance: f64,
}

#[wasm_bindgen]
impl JsRating {
    /// Create a new rating
    #[wasm_bindgen(constructor)]
    pub fn new(mean: f64, variance: f64) -> Result<JsRating, JsValue> {
        if variance <= 0.0 {
            return Err(JsValue::from_str("Variance must be positive"));
        }
        Ok(Self { mean, variance })
    }
    
    /// Create a new rating (for internal use, not exposed to JS)
    #[cfg(test)]
    pub fn new_unchecked(mean: f64, variance: f64) -> Self {
        Self { mean, variance }
    }

    /// Get the mean value
    #[wasm_bindgen(getter)]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Get the variance value
    #[wasm_bindgen(getter)]
    pub fn variance(&self) -> f64 {
        self.variance
    }

    /// Convert to JSON string
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create from JSON string
    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json(json: &str) -> Result<JsRating, JsValue> {
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// JavaScript-friendly player representation
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsPlayer {
    /// Unique player identifier
    id: String,
    /// Optional display name
    name: Option<String>,
    /// Current rating
    rating: JsRating,
}

#[wasm_bindgen]
impl JsPlayer {
    /// Create a new player
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, name: Option<String>, rating: JsRating) -> Self {
        Self { id, name, rating }
    }

    /// Get player ID
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Get player name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Get player rating
    #[wasm_bindgen(getter)]
    pub fn rating(&self) -> JsRating {
        self.rating.clone()
    }

    /// Convert to JSON string
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create from JSON string
    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json(json: &str) -> Result<JsPlayer, JsValue> {
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// JavaScript-friendly match outcome
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JsOutcome {
    Win = 0,
    Loss = 1,
    Draw = 2,
}

/// Match configuration for different algorithms
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsMatchConfig {
    /// Algorithm to use
    algorithm: String,
    /// Algorithm-specific parameters
    params: JsValue,
}

#[wasm_bindgen]
impl JsMatchConfig {
    /// Create match configuration
    #[wasm_bindgen(constructor)]
    pub fn new(algorithm: String, params: JsValue) -> Self {
        Self { algorithm, params }
    }

    /// Get algorithm name
    #[wasm_bindgen(getter)]
    pub fn algorithm(&self) -> String {
        self.algorithm.clone()
    }

    /// Get parameters
    #[wasm_bindgen(getter)]
    pub fn params(&self) -> JsValue {
        self.params.clone()
    }
}

/// Match result between two players
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsMatchResult {
    /// Winner's player ID (None for draw)
    winner: Option<String>,
    /// Updated ratings for both players
    ratings: Vec<JsRating>,
}

#[wasm_bindgen]
impl JsMatchResult {
    /// Create match result
    #[wasm_bindgen(constructor)]
    pub fn new(winner: Option<String>, ratings: Vec<JsRating>) -> Self {
        Self { winner, ratings }
    }

    /// Get winner ID
    #[wasm_bindgen(getter)]
    pub fn winner(&self) -> Option<String> {
        self.winner.clone()
    }

    /// Get updated ratings
    #[wasm_bindgen(getter)]
    pub fn ratings(&self) -> Vec<JsRating> {
        self.ratings.clone()
    }

    /// Convert to JSON string
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create from JSON string
    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json(json: &str) -> Result<JsMatchResult, JsValue> {
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Configuration for Elo algorithm
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsEloConfig {
    /// K-factor for rating adjustments
    k_factor: f64,
    /// Initial rating for new players
    initial_rating: f64,
    /// Initial variance
    initial_variance: f64,
}

#[wasm_bindgen]
impl JsEloConfig {
    /// Create Elo configuration
    #[wasm_bindgen(constructor)]
    pub fn new(k_factor: f64, initial_rating: f64, initial_variance: f64) -> Self {
        Self {
            k_factor,
            initial_rating,
            initial_variance,
        }
    }

    /// Get K-factor
    #[wasm_bindgen(getter, js_name = kFactor)]
    pub fn k_factor(&self) -> f64 {
        self.k_factor
    }

    /// Get initial rating
    #[wasm_bindgen(getter, js_name = initialRating)]
    pub fn initial_rating(&self) -> f64 {
        self.initial_rating
    }

    /// Get initial variance
    #[wasm_bindgen(getter, js_name = initialVariance)]
    pub fn initial_variance(&self) -> f64 {
        self.initial_variance
    }
}

/// Configuration for Glicko algorithm
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsGlickoConfig {
    /// Initial rating
    initial_rating: f64,
    /// Initial rating deviation
    initial_deviation: f64,
    /// Rating period constant
    c: f64,
}

#[wasm_bindgen]
impl JsGlickoConfig {
    /// Create Glicko configuration
    #[wasm_bindgen(constructor)]
    pub fn new(initial_rating: f64, initial_deviation: f64, c: f64) -> Self {
        Self {
            initial_rating,
            initial_deviation,
            c,
        }
    }

    /// Get initial rating
    #[wasm_bindgen(getter, js_name = initialRating)]
    pub fn initial_rating(&self) -> f64 {
        self.initial_rating
    }

    /// Get initial deviation
    #[wasm_bindgen(getter, js_name = initialDeviation)]
    pub fn initial_deviation(&self) -> f64 {
        self.initial_deviation
    }

    /// Get c constant
    #[wasm_bindgen(getter)]
    pub fn c(&self) -> f64 {
        self.c
    }
}

/// Configuration for TrueSkill algorithm
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTrueSkillConfig {
    /// Initial mean skill
    initial_mean: f64,
    /// Initial standard deviation
    initial_std_dev: f64,
    /// Performance variance factor (beta)
    beta: f64,
    /// Dynamics factor (tau)
    tau: f64,
    /// Draw probability
    draw_probability: f64,
}

#[wasm_bindgen]
impl JsTrueSkillConfig {
    /// Create TrueSkill configuration
    #[wasm_bindgen(constructor)]
    pub fn new(
        initial_mean: f64,
        initial_std_dev: f64,
        beta: f64,
        tau: f64,
        draw_probability: f64,
    ) -> Self {
        Self {
            initial_mean,
            initial_std_dev,
            beta,
            tau,
            draw_probability,
        }
    }

    /// Get initial mean
    #[wasm_bindgen(getter, js_name = initialMean)]
    pub fn initial_mean(&self) -> f64 {
        self.initial_mean
    }

    /// Get initial standard deviation
    #[wasm_bindgen(getter, js_name = initialStdDev)]
    pub fn initial_std_dev(&self) -> f64 {
        self.initial_std_dev
    }

    /// Get beta
    #[wasm_bindgen(getter)]
    pub fn beta(&self) -> f64 {
        self.beta
    }

    /// Get tau
    #[wasm_bindgen(getter)]
    pub fn tau(&self) -> f64 {
        self.tau
    }

    /// Get draw probability
    #[wasm_bindgen(getter, js_name = drawProbability)]
    pub fn draw_probability(&self) -> f64 {
        self.draw_probability
    }
}

/// Error type for WASM operations
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsError {
    /// Error message
    message: String,
    /// Error type/category
    error_type: String,
}

#[wasm_bindgen]
impl JsError {
    /// Create an error
    #[wasm_bindgen(constructor)]
    pub fn new(message: String, error_type: String) -> Self {
        Self {
            message,
            error_type,
        }
    }

    /// Get error message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get error type
    #[wasm_bindgen(getter, js_name = errorType)]
    pub fn error_type(&self) -> String {
        self.error_type.clone()
    }

    /// Convert to string
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("{}: {}", self.error_type, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ladder_rs::Rating;

    #[test]
    fn test_js_rating() {
        let rating = JsRating::new_unchecked(1500.0, 200.0);
        assert_eq!(rating.mean(), 1500.0);
        assert_eq!(rating.variance(), 200.0);

        // Test JSON serialization
        let json = rating.to_json().unwrap();
        let parsed = JsRating::from_json(&json).unwrap();
        assert_eq!(parsed.mean(), rating.mean());
        assert_eq!(parsed.variance(), rating.variance());
        
        // Test variance validation logic
        let rating_with_negative = JsRating::new_unchecked(1500.0, -100.0);
        assert!(rating_with_negative.variance < 0.0); // Would be invalid
        
        let rating_with_zero = JsRating::new_unchecked(1500.0, 0.0);
        assert_eq!(rating_with_zero.variance, 0.0); // Would be invalid
        
        let rating_valid = JsRating::new_unchecked(1500.0, 0.001);
        assert!(rating_valid.variance > 0.0); // Valid
    }

    #[test]
    fn test_js_player() {
        let rating = JsRating::new_unchecked(1200.0, 100.0);
        let player = JsPlayer::new("p1".to_string(), Some("Alice".to_string()), rating);

        assert_eq!(player.id(), "p1");
        assert_eq!(player.name(), Some("Alice".to_string()));
        assert_eq!(player.rating().mean(), 1200.0);

        // Test JSON serialization
        let json = player.to_json().unwrap();
        let parsed = JsPlayer::from_json(&json).unwrap();
        assert_eq!(parsed.id(), player.id());
        assert_eq!(parsed.name(), player.name());
    }

    #[test]
    fn test_match_result() {
        let ratings = vec![
            JsRating::new_unchecked(1520.0, 190.0), 
            JsRating::new_unchecked(1480.0, 210.0)
        ];

        let result = JsMatchResult::new(Some("p1".to_string()), ratings);

        assert_eq!(result.winner(), Some("p1".to_string()));
        assert_eq!(result.ratings().len(), 2);
        assert_eq!(result.ratings()[0].mean(), 1520.0);

        // Test JSON serialization
        let json = result.to_json().unwrap();
        let parsed = JsMatchResult::from_json(&json).unwrap();
        assert_eq!(parsed.winner(), result.winner());
        assert_eq!(parsed.ratings().len(), result.ratings().len());
        
        // Test draw (None winner)
        let draw_result = JsMatchResult::new(None, vec![
            JsRating::new_unchecked(1500.0, 200.0),
            JsRating::new_unchecked(1500.0, 200.0)
        ]);
        assert_eq!(draw_result.winner(), None);
    }

    #[test]
    fn test_config_types() {
        let elo_config = JsEloConfig::new(32.0, 1500.0, 300.0);
        assert_eq!(elo_config.k_factor(), 32.0);
        assert_eq!(elo_config.initial_rating(), 1500.0);

        let glicko_config = JsGlickoConfig::new(1500.0, 350.0, 15.0);
        assert_eq!(glicko_config.initial_rating(), 1500.0);
        assert_eq!(glicko_config.initial_deviation(), 350.0);

        let trueskill_config = JsTrueSkillConfig::new(25.0, 8.333, 4.166, 0.083, 0.1);
        assert_eq!(trueskill_config.initial_mean(), 25.0);
        assert_eq!(trueskill_config.beta(), 4.166);
    }

    #[test]
    fn test_error_type() {
        let error = JsError::new("Invalid player ID".to_string(), "ValidationError".to_string());
        assert_eq!(error.message(), "Invalid player ID");
        assert_eq!(error.error_type(), "ValidationError");
        assert_eq!(error.to_string(), "ValidationError: Invalid player ID");
    }

    #[test]
    fn test_rating_conversion() {
        // Test rating conversion
        #[derive(Debug, Clone)]
        struct TestRating {
            mean: f64,
            variance: f64,
        }
        impl ladder_rs::Rating for TestRating {
            fn mean(&self) -> f64 {
                self.mean
            }
            fn variance(&self) -> f64 {
                self.variance
            }
        }

        let test_rating = TestRating {
            mean: 1600.0,
            variance: 225.0,
        };

        // Convert to JsRating
        let js_rating = JsRating::new_unchecked(test_rating.mean(), test_rating.variance());
        assert_eq!(js_rating.mean(), 1600.0);
        assert_eq!(js_rating.variance(), 225.0);
    }
}