// Error Handling Framework Tests for Task 1.2.5

use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;

// Configure tests to run in browser environment
wasm_bindgen_test_configure!(run_in_browser);

// Import the error handling types and functions from the crate
use ladder_rs_wasm::errors::*;
use ladder_rs_wasm::types::{JsRating, JsPlayer, JsMatchConfig, JsEloConfig};
use ladder_rs_wasm::rating_system::JsRatingSystem;
use ladder_rs_wasm::js_interface::systems::JsOutcome;
use std::collections::HashMap;

// Test Error Type System
#[wasm_bindgen_test]
fn test_error_types() {
    // Test validation error
    let error = JsRatingError::validation_error("Invalid rating value");
    assert_eq!(error.error_type(), "ValidationError");
    assert!(error.message().contains("Invalid rating value"));
    
    // Test calculation error
    let error = JsRatingError::calculation_error("Division by zero");
    assert_eq!(error.error_type(), "CalculationError");
    
    // Test configuration error
    let error = JsRatingError::configuration_error("Invalid K-factor");
    assert_eq!(error.error_type(), "ConfigurationError");
    
    // Test convergence error
    let error = JsRatingError::convergence_error("Failed to converge", 100);
    assert_eq!(error.error_type(), "ConvergenceError");
    
    // Check context is properly stored
    let context_js = error.context();
    assert!(!context_js.is_null());
}

// Test Error Context and Chaining
#[wasm_bindgen_test]
fn test_error_context_and_chaining() {
    let base_error = JsRatingError::validation_error("Invalid player ID");
    let enhanced_error = base_error
        .with_context("player_id", "player_123")
        .with_context("operation", "create_player");
    
    // Verify context was added
    let context_js = enhanced_error.context();
    assert!(!context_js.is_null());
    
    // Test error chaining
    let cause = JsRatingError::validation_error("Negative variance");
    let error = JsRatingError::calculation_error("Cannot calculate rating")
        .with_cause(Box::new(cause));
        
    assert!(error.cause().is_some());
}

// Test Error Severity Levels
#[wasm_bindgen_test]
fn test_error_levels() {
    let debug_error = JsRatingError::validation_error("Minor issue")
        .with_level(ErrorLevel::Debug);
    assert_eq!(debug_error.level(), ErrorLevel::Debug);
    
    let critical_error = JsRatingError::calculation_error("System failure")
        .with_level(ErrorLevel::Critical);
    assert_eq!(critical_error.level(), ErrorLevel::Critical);
}

// Test Recovery Suggestions
#[wasm_bindgen_test]
fn test_recovery_suggestions() {
    let error = JsRatingError::validation_error("Invalid variance")
        .with_recovery_suggestion("Variance must be positive. Try using a value greater than 0.");
        
    assert!(error.recovery_suggestion().is_some());
    assert!(error.recovery_suggestion().unwrap().contains("positive"));
}

// Test Rating Creation Errors
#[wasm_bindgen_test]
fn test_rating_creation_errors() {
    // Invalid variance should return proper error
    match JsRating::new(1500.0, -100.0) {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            let error_str = e.as_string().unwrap();
            assert!(error_str.contains("variance"));
            assert!(error_str.contains("positive"));
        }
    }
    
    // NaN values should be rejected
    match JsRating::new(f64::NAN, 100.0) {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            let error_str = e.as_string().unwrap();
            assert!(error_str.contains("mean"));
            assert!(error_str.contains("finite"));
        }
    }
    
    // Infinity should be rejected
    match JsRating::new(1500.0, f64::INFINITY) {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            let error_str = e.as_string().unwrap();
            assert!(error_str.contains("variance"));
            assert!(error_str.contains("finite"));
        }
    }
}

// Test Player Creation Errors
#[wasm_bindgen_test]
fn test_player_creation_errors() {
    // Empty player ID should fail
    let rating = JsRating::new(1500.0, 200.0).unwrap();
    match JsPlayer::new("", None, rating) {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            let error_str = e.as_string().unwrap();
            assert!(error_str.contains("Player ID"));
            assert!(error_str.contains("empty"));
        }
    }
}

// Test Match Processing Errors
#[wasm_bindgen_test]
fn test_match_processing_errors() {
    let config = JsMatchConfig::new("elo", JsValue::from(JsEloConfig::default()));
    let system = JsRatingSystem::new(config).unwrap();
    
    // Add a player
    let rating = JsRating::new(1500.0, 200.0).unwrap();
    let player = JsPlayer::new("player1", Some("Alice".to_string()), rating).unwrap();
    system.add_player(player).unwrap();
    
    // Try to process match with not enough players
    let players: Vec<JsPlayer> = vec![];
    match system.process_match(players.into_boxed_slice(), JsOutcome::Win) {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            let error_str = e.as_string().unwrap();
            assert!(error_str.contains("At least two players"));
        }
    }
}

// Test Configuration Errors
#[wasm_bindgen_test]
fn test_configuration_errors() {
    // Invalid algorithm should fail
    let config = JsMatchConfig::new("invalid_algorithm", JsValue::NULL);
    match JsRatingSystem::new(config) {
        Ok(_) => panic!("Should have failed"),
        Err(e) => {
            let error_str = e.as_string().unwrap();
            assert!(error_str.contains("algorithm"));
        }
    }
}

// Test Graceful Degradation
#[wasm_bindgen_test]
fn test_graceful_degradation() {
    let config = JsMatchConfig::new("elo", JsValue::from(JsEloConfig::default()));
    let system = JsRatingSystem::new(config).unwrap();
    
    // Add valid players
    let rating1 = JsRating::new(1500.0, 200.0).unwrap();
    let player1 = JsPlayer::new("player1", Some("Alice".to_string()), rating1).unwrap();
    system.add_player(player1).unwrap();
    
    let rating2 = JsRating::new(1600.0, 150.0).unwrap();
    let player2 = JsPlayer::new("player2", Some("Bob".to_string()), rating2).unwrap();
    system.add_player(player2).unwrap();
    
    // Test process_match_safe for graceful degradation
    match system.process_match_safe(vec!["player1".to_string(), "player2".to_string()], JsOutcome::Win) {
        Ok(result) => {
            assert!(result.success());
            assert!(result.error().is_none());
        }
        Err(_) => panic!("Safe processing should not fail"),
    }
    
    // Test with invalid player
    match system.process_match_safe(vec!["player1".to_string(), "invalid_player".to_string()], JsOutcome::Win) {
        Ok(result) => {
            assert!(!result.success());
            assert!(result.error().is_some());
        }
        Err(_) => panic!("Safe processing should not fail"),
    }
}

// Test Error Serialization
#[wasm_bindgen_test]
fn test_error_serialization() {
    let error = JsRatingError::validation_error("Test error")
        .with_context("field", "rating")
        .with_code("E001")
        .with_level(ErrorLevel::Warning);
        
    // Convert to JsValue
    let js_value = error.to_js_value();
    
    // Should be able to convert to string
    assert!(!js_value.is_null());
    assert!(!js_value.is_undefined());
    
    // Test JSON serialization
    let json = error.to_json();
    assert!(json.contains("ValidationError"));
    assert!(json.contains("Test error"));
    assert!(json.contains("E001"));
    assert!(json.contains("Warning"));
}

// Test JavaScript Error Compatibility
#[wasm_bindgen_test]
fn test_js_error_compatibility() {
    let error = JsRatingError::validation_error("JS compatible error");
    let js_value = error.to_js_value();
    
    // Should have Error-like properties
    assert!(js_value.is_object());
    
    // The error should be usable in JavaScript context
    assert!(!js_value.is_null());
}