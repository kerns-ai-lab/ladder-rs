//! Core conversion utilities between Rust and JavaScript types
//!
//! This module provides fundamental conversions that are used across
//! all rating system implementations.

use crate::types::{JsRating, JsPlayer, JsOutcome, JsMatchResult, JsError as CustomJsError};
use ladder_rs::{Rating, core::GameOutcome};
use wasm_bindgen::prelude::*;

/// Convert a Rust Rating trait object to JsRating
pub fn rating_to_js<R: Rating>(rating: &R) -> Result<JsRating, JsValue> {
    JsRating::new(rating.mean(), rating.variance())
}

/// Convert JsRating to a generic rating with mean and variance
pub fn js_to_rating_values(js_rating: &JsRating) -> (f64, f64) {
    (js_rating.mean(), js_rating.variance())
}

/// Convert JsOutcome to ladder_rs GameOutcome
pub fn js_outcome_to_game_outcome(outcome: JsOutcome, team_count: usize) -> GameOutcome {
    match outcome {
        JsOutcome::Win => GameOutcome::win(0, team_count),
        JsOutcome::Loss => GameOutcome::win(1, team_count),
        JsOutcome::Draw => GameOutcome::draw(team_count),
    }
}

/// Convert ranks array to GameOutcome
pub fn ranks_to_game_outcome(ranks: Vec<usize>) -> GameOutcome {
    GameOutcome::new(ranks)
}

/// Convert JsMatchResult to updated ratings
pub fn match_result_to_ratings(result: &JsMatchResult) -> Vec<JsRating> {
    result.ratings()
}

/// Convert player IDs and ratings to JsPlayers
pub fn create_js_players(ids: Vec<String>, ratings: Vec<JsRating>) -> Result<Vec<JsPlayer>, JsValue> {
    if ids.len() != ratings.len() {
        return Err(JsValue::from_str("Player IDs and ratings length mismatch"));
    }
    
    Ok(ids.into_iter()
        .zip(ratings.into_iter())
        .map(|(id, rating)| JsPlayer::new(id, None, rating))
        .collect())
}

/// Extract ratings from JsPlayers
pub fn extract_ratings_from_players(players: &[JsPlayer]) -> Vec<JsRating> {
    players.iter().map(|p| p.rating()).collect()
}

/// Validate rating parameters
pub fn validate_rating_params(mean: f64, variance: f64) -> Result<(), JsValue> {
    if variance <= 0.0 {
        return Err(JsValue::from_str("Variance must be positive"));
    }
    
    if !mean.is_finite() || !variance.is_finite() {
        return Err(JsValue::from_str("Rating parameters must be finite numbers"));
    }
    
    Ok(())
}

/// Convert optional string to Option<String>
pub fn js_string_to_option(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.is_empty())
}

/// Create error response
pub fn create_js_error(message: &str, error_type: &str) -> CustomJsError {
    CustomJsError::new(message.to_string(), error_type.to_string())
}

/// Convert Result to JS-friendly result
pub fn result_to_js<T, E: std::fmt::Display>(result: Result<T, E>) -> Result<T, JsValue> {
    result.map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rating_validation() {
        assert!(validate_rating_params(1500.0, 200.0).is_ok());
        assert!(validate_rating_params(1500.0, 0.0).is_err());
        assert!(validate_rating_params(1500.0, -100.0).is_err());
        assert!(validate_rating_params(f64::INFINITY, 200.0).is_err());
        assert!(validate_rating_params(1500.0, f64::NAN).is_err());
    }
    
    #[test]
    fn test_outcome_conversion() {
        let win_outcome = js_outcome_to_game_outcome(JsOutcome::Win, 2);
        assert_eq!(win_outcome.ranks(), &[1, 2]);
        
        let loss_outcome = js_outcome_to_game_outcome(JsOutcome::Loss, 2);
        assert_eq!(loss_outcome.ranks(), &[2, 1]);
        
        let draw_outcome = js_outcome_to_game_outcome(JsOutcome::Draw, 2);
        assert_eq!(draw_outcome.ranks(), &[1, 1]);
    }
    
    #[test]
    fn test_string_option_conversion() {
        assert_eq!(js_string_to_option(Some("test".to_string())), Some("test".to_string()));
        assert_eq!(js_string_to_option(Some("".to_string())), None);
        assert_eq!(js_string_to_option(None), None);
    }
}