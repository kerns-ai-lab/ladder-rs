//! Task 1.2.2: Conversion Implementations Test Suite
//!
//! This test suite validates comprehensive type conversions between Rust types
//! and JavaScript/WASM boundary types for all rating systems.

// In integration tests, we need to import from the crate
use ladder_rs_wasm;
use ladder_rs_wasm::types::*;
use ladder_rs_wasm::conversions::core::*;

// Test conversion utilities
#[test]
fn test_rating_validation() {
    assert!(validate_rating_params(1500.0, 200.0).is_ok());
    assert!(validate_rating_params(1500.0, 0.0).is_err());
    assert!(validate_rating_params(1500.0, -100.0).is_err());
    assert!(validate_rating_params(f64::INFINITY, 200.0).is_err());
    assert!(validate_rating_params(1500.0, f64::NAN).is_err());
}

#[test]
fn test_string_option_conversion() {
    assert_eq!(js_string_to_option(Some("test".to_string())), Some("test".to_string()));
    assert_eq!(js_string_to_option(Some("".to_string())), None);
    assert_eq!(js_string_to_option(None), None);
}

#[test]
fn test_error_creation() {
    let error = create_js_error("Test error", "TestError");
    assert_eq!(error.message(), "Test error");
    assert_eq!(error.error_type(), "TestError");
}

// Test JsRating functionality
#[test]
fn test_js_rating_creation() {
    let rating = JsRating::new_unchecked(1500.0, 200.0);
    assert_eq!(rating.mean(), 1500.0);
    assert_eq!(rating.variance(), 200.0);
}

#[test]
fn test_js_rating_json_serialization() {
    let rating = JsRating::new_unchecked(1600.0, 225.0);
    let json = rating.to_json().expect("Should serialize to JSON");
    let parsed = JsRating::from_json(&json).expect("Should parse from JSON");
    assert_eq!(parsed.mean(), rating.mean());
    assert_eq!(parsed.variance(), rating.variance());
}

// Test JsPlayer functionality
#[test]
fn test_js_player_creation() {
    let rating = JsRating::new_unchecked(1500.0, 200.0);
    let player = JsPlayer::new(
        "p1".to_string(),
        Some("Alice".to_string()),
        rating
    );
    
    assert_eq!(player.id(), "p1");
    assert_eq!(player.name(), Some("Alice".to_string()));
    assert_eq!(player.rating().mean(), 1500.0);
}

// Test JsMatchResult functionality
#[test]
fn test_js_match_result() {
    let ratings = vec![
        JsRating::new_unchecked(1520.0, 190.0),
        JsRating::new_unchecked(1480.0, 210.0)
    ];
    
    let result = JsMatchResult::new(Some("p1".to_string()), ratings);
    assert_eq!(result.winner(), Some("p1".to_string()));
    assert_eq!(result.ratings().len(), 2);
    assert_eq!(result.ratings()[0].mean(), 1520.0);
}

// Test JsOutcome functionality
#[test]
fn test_js_outcome_serialization() {
    use serde_json;
    
    let win = JsOutcome::Win;
    let loss = JsOutcome::Loss;
    let draw = JsOutcome::Draw;
    
    // Test serialization
    let win_json = serde_json::to_string(&win).expect("Should serialize Win");
    let loss_json = serde_json::to_string(&loss).expect("Should serialize Loss");
    let draw_json = serde_json::to_string(&draw).expect("Should serialize Draw");
    
    // Enum values
    assert_eq!(win_json, "0");
    assert_eq!(loss_json, "1");
    assert_eq!(draw_json, "2");
}

// Test configuration types
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

// Test error handling
#[test]
fn test_js_error() {
    let error = JsError::new(
        "Invalid rating parameters".to_string(),
        "ConversionError".to_string()
    );
    
    assert_eq!(error.message(), "Invalid rating parameters");
    assert_eq!(error.error_type(), "ConversionError");
    assert_eq!(error.to_string(), "ConversionError: Invalid rating parameters");
}

// Test collection handling
#[test]
fn test_collections() {
    let ratings: Vec<JsRating> = (0..10)
        .map(|i| JsRating::new_unchecked(1500.0 + i as f64, 200.0))
        .collect();
    
    assert_eq!(ratings.len(), 10);
    assert_eq!(ratings[0].mean(), 1500.0);
    assert_eq!(ratings[9].mean(), 1509.0);
}

// Test conversion between types
#[test]
fn test_type_conversions() {
    // Test that we can extract values from JsRating for conversion
    let js_rating = JsRating::new_unchecked(1600.0, 225.0);
    let (mean, variance) = js_to_rating_values(&js_rating);
    assert_eq!(mean, 1600.0);
    assert_eq!(variance, 225.0);
}

// Test outcome conversions
#[test]
fn test_outcome_conversions() {
    use ladder_rs::core::GameOutcome;
    
    // Test creating game outcomes from different sources
    let win_outcome = js_outcome_to_game_outcome(JsOutcome::Win, 2);
    assert_eq!(win_outcome.ranks(), &[1, 2]);
    
    let loss_outcome = js_outcome_to_game_outcome(JsOutcome::Loss, 2);
    assert_eq!(loss_outcome.ranks(), &[2, 1]);
    
    let draw_outcome = js_outcome_to_game_outcome(JsOutcome::Draw, 2);
    assert_eq!(draw_outcome.ranks(), &[1, 1]);
}

// Test match configuration
#[test]
fn test_match_config() {
    use wasm_bindgen::JsValue;
    
    let config = JsMatchConfig::new(
        "elo".to_string(),
        JsValue::from_str("{\"k_factor\": 32}")
    );
    
    assert_eq!(config.algorithm(), "elo");
}

// Test boundary conditions and error cases
#[test]
fn test_boundary_conditions() {
    // Test very small positive variance
    let small_variance = JsRating::new_unchecked(1500.0, 0.001);
    assert_eq!(small_variance.variance(), 0.001);
    
    // Test large values
    let large_values = JsRating::new_unchecked(10000.0, 5000.0);
    assert_eq!(large_values.mean(), 10000.0);
    assert_eq!(large_values.variance(), 5000.0);
    
    // Test negative mean (valid for some systems)
    let negative_mean = JsRating::new_unchecked(-500.0, 100.0);
    assert_eq!(negative_mean.mean(), -500.0);
}

// Test precision and floating point handling
#[test]
fn test_precision() {
    let precise = JsRating::new_unchecked(1500.123456789, 200.987654321);
    assert!((precise.mean() - 1500.123456789).abs() < 1e-9);
    assert!((precise.variance() - 200.987654321).abs() < 1e-9);
}