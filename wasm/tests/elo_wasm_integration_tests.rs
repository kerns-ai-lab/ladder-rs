//! Comprehensive tests for Elo rating system WASM bindings
//!
//! This test module validates all aspects of the Elo implementation
//! in the WASM context, including:
//! - Rating creation and initialization
//! - Match processing (wins, losses, draws)
//! - Parameter configuration
//! - Serialization/deserialization
//! - Error handling
//! - Performance characteristics

use wasm_bindgen_test::*;
use ladder_rs_wasm::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_elo_system_creation_default() {
    // Test creating an Elo system with default parameters
    let system = EloSystem::new();
    
    // Default parameters should be:
    // - k_factor: 32.0
    // - initial_rating: 1500.0
    assert_eq!(system.k_factor(), 32.0);
    assert_eq!(system.initial_rating(), 1500.0);
}

#[wasm_bindgen_test]
fn test_elo_system_creation_custom() {
    // Test creating an Elo system with custom parameters
    let system = EloSystem::with_parameters(20.0, 1200.0);
    
    assert_eq!(system.k_factor(), 20.0);
    assert_eq!(system.initial_rating(), 1200.0);
}

#[wasm_bindgen_test]
fn test_elo_rating_creation() {
    let system = EloSystem::new();
    
    // Test creating a new player rating
    let rating = system.create_rating();
    assert_eq!(rating.value(), 1500.0);
    
    // Test creating a rating with custom value
    let custom_rating = system.create_rating_with_value(1800.0);
    assert_eq!(custom_rating.value(), 1800.0);
}

#[wasm_bindgen_test]
fn test_elo_1v1_win() {
    let system = EloSystem::new();
    
    // Create two players with equal ratings
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    
    // Process a match where player1 wins
    let result = system.process_1v1(&player1, &player2, MatchOutcome::Player1Win).unwrap();
    
    // Winner should gain rating, loser should lose rating
    assert!(result.player1_rating() > player1.value());
    assert!(result.player2_rating() < player2.value());
    
    // The sum of rating changes should be zero (conservation)
    let p1_change = result.player1_rating() - player1.value();
    let p2_change = result.player2_rating() - player2.value();
    assert!((p1_change + p2_change).abs() < 0.001);
}

#[wasm_bindgen_test]
fn test_elo_1v1_draw() {
    let system = EloSystem::new();
    
    // Create two players with equal ratings
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    
    // Process a draw
    let result = system.process_1v1(&player1, &player2, MatchOutcome::Draw).unwrap();
    
    // Both ratings should remain the same for equal players
    assert!((result.player1_rating() - player1.value()).abs() < 0.001);
    assert!((result.player2_rating() - player2.value()).abs() < 0.001);
}

#[wasm_bindgen_test]
fn test_elo_1v1_upset() {
    let system = EloSystem::new();
    
    // Create two players with different ratings
    let strong_player = system.create_rating_with_value(1700.0);
    let weak_player = system.create_rating_with_value(1300.0);
    
    // Weak player wins (upset)
    let result = system.process_1v1(&strong_player, &weak_player, MatchOutcome::Player2Win).unwrap();
    
    // Weak player should gain more than strong player loses
    let strong_loss = strong_player.value() - result.player1_rating();
    let weak_gain = result.player2_rating() - weak_player.value();
    assert!(weak_gain > strong_loss);
}

#[wasm_bindgen_test]
fn test_elo_win_probability() {
    let system = EloSystem::new();
    
    let player1 = system.create_rating_with_value(1600.0);
    let player2 = system.create_rating_with_value(1400.0);
    
    // Player 1 should have higher win probability
    let prob1 = system.win_probability(&player1, &player2);
    assert!(prob1 > 0.5);
    assert!(prob1 < 1.0);
    
    // Probabilities should sum to 1
    let prob2 = system.win_probability(&player2, &player1);
    assert!((prob1 + prob2 - 1.0).abs() < 0.001);
}

#[wasm_bindgen_test]
fn test_elo_match_quality() {
    let system = EloSystem::new();
    
    // Equal players should have high match quality
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    let quality1 = system.match_quality(&player1, &player2);
    assert!(quality1 > 0.9);
    
    // Very different players should have low match quality
    let strong = system.create_rating_with_value(2000.0);
    let weak = system.create_rating_with_value(1000.0);
    let quality2 = system.match_quality(&strong, &weak);
    assert!(quality2 < 0.3);
}

#[wasm_bindgen_test]
fn test_elo_k_factor_effect() {
    // Test with different k-factors
    let system_high_k = EloSystem::with_parameters(40.0, 1500.0);
    let system_low_k = EloSystem::with_parameters(10.0, 1500.0);
    
    let player1 = system_high_k.create_rating();
    let player2 = system_high_k.create_rating();
    
    // Process same match with different k-factors
    let result_high = system_high_k.process_1v1(&player1, &player2, MatchOutcome::Player1Win).unwrap();
    let result_low = system_low_k.process_1v1(&player1, &player2, MatchOutcome::Player1Win).unwrap();
    
    // Higher k-factor should result in larger rating change
    let change_high = result_high.player1_rating() - player1.value();
    let change_low = result_low.player1_rating() - player1.value();
    assert!(change_high > change_low);
}

#[wasm_bindgen_test]
fn test_elo_serialization() {
    let system = EloSystem::new();
    
    // Create and serialize a rating
    let rating = system.create_rating_with_value(1650.0);
    let serialized = rating.serialize();
    
    // Deserialize and verify
    let deserialized = EloRating::deserialize(&serialized).unwrap();
    assert_eq!(deserialized.value(), rating.value());
}

#[wasm_bindgen_test]
fn test_elo_system_serialization() {
    // Create system with custom parameters
    let system = EloSystem::with_parameters(25.0, 1400.0);
    
    // Serialize the system
    let serialized = system.serialize();
    
    // Deserialize and verify parameters
    let deserialized = EloSystem::deserialize(&serialized).unwrap();
    assert_eq!(deserialized.k_factor(), 25.0);
    assert_eq!(deserialized.initial_rating(), 1400.0);
}

#[wasm_bindgen_test]
fn test_elo_batch_processing() {
    let system = EloSystem::new();
    
    // Create ratings JSON
    let ratings_json = r#"[{"value":1500},{"value":1500},{"value":1500},{"value":1500}]"#;
    
    // Create matches: player1 beats player2, player4 beats player3, player1 draws with player4
    let matches_json = r#"[[0,1,1],[3,2,1],[0,3,0]]"#;
    
    let result_json = EloUtils::batch_process(&system, ratings_json, matches_json).unwrap();
    let results: Vec<EloRating> = serde_json::from_str(&result_json).unwrap();
    
    // Verify expected rating order: player1 > player4 > player3 > player2
    assert!(results[0].value() > results[3].value());
    assert!(results[3].value() > results[2].value());
    assert!(results[2].value() > results[1].value());
}

#[wasm_bindgen_test]
fn test_elo_edge_cases() {
    let system = EloSystem::new();
    
    // Test with extreme rating differences
    let very_strong = system.create_rating_with_value(3000.0);
    let very_weak = system.create_rating_with_value(100.0);
    
    // Even with extreme difference, ratings should update reasonably
    let result = system.process_1v1(&very_strong, &very_weak, MatchOutcome::Player1Win).unwrap();
    assert!(result.player1_rating() > very_strong.value());
    assert!(result.player2_rating() < very_weak.value());
    
    // Test with negative ratings
    let negative = system.create_rating_with_value(-500.0);
    let positive = system.create_rating_with_value(500.0);
    
    let result2 = system.process_1v1(&negative, &positive, MatchOutcome::Player2Win).unwrap();
    assert!(result2.player1_rating() < negative.value());
    assert!(result2.player2_rating() > positive.value());
}

#[wasm_bindgen_test]
fn test_elo_javascript_interop() {
    // Test that our types work well with JavaScript
    let system = EloSystem::new();
    
    // Test JSON serialization for JS
    let rating = system.create_rating();
    let json = rating.to_json();
    assert!(json.contains("\"value\":1500"));
    
    // Test creating from JSON
    let from_json = EloRating::from_json("{\"value\":1750}").unwrap();
    assert_eq!(from_json.value(), 1750.0);
    
    // Test process_1v1_json method
    let result_json = system.process_1v1_json(1500.0, 1500.0, MatchOutcome::Player1Win).unwrap();
    assert!(result_json.contains("player1"));
    assert!(result_json.contains("player2"));
}

#[wasm_bindgen_test]
fn test_elo_error_handling() {
    // Test invalid JSON deserialization
    assert!(EloRating::from_json("invalid json").is_err());
    assert!(EloRating::from_json("{}").is_err()); // missing value field
    
    // Test invalid system parameters
    let system = EloSystem::with_parameters(-10.0, 1500.0);
    assert!(system.k_factor() > 0.0); // Should handle negative k-factor
    
    // Test deserialization of corrupted data
    assert!(EloRating::deserialize("corrupted").is_err());
    assert!(EloSystem::deserialize("corrupted").is_err());
}

#[wasm_bindgen_test]
fn test_elo_leaderboard() {
    let ratings_json = r#"[{"value":1600},{"value":1400},{"value":1800},{"value":1500}]"#;
    
    let leaderboard_json = EloUtils::create_leaderboard(ratings_json).unwrap();
    let leaderboard: Vec<Vec<f64>> = serde_json::from_str(&leaderboard_json).unwrap();
    
    // Should be sorted by rating descending
    assert_eq!(leaderboard[0][0], 2.0); // index 2 has rating 1800
    assert_eq!(leaderboard[0][1], 1800.0);
    assert_eq!(leaderboard[1][0], 0.0); // index 0 has rating 1600
    assert_eq!(leaderboard[1][1], 1600.0);
}

#[wasm_bindgen_test]
fn test_elo_utils_helpers() {
    // Test creating ratings from values
    let values_json = "[1200, 1300, 1400, 1500]";
    let ratings_json = EloUtils::create_ratings_from_values(values_json).unwrap();
    
    let ratings: Vec<EloRating> = serde_json::from_str(&ratings_json).unwrap();
    assert_eq!(ratings.len(), 4);
    assert_eq!(ratings[0].value(), 1200.0);
    assert_eq!(ratings[3].value(), 1500.0);
}