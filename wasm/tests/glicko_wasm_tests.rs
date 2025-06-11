//! Comprehensive tests for Glicko rating system WASM bindings
//!
//! This test module validates all aspects of the Glicko implementation
//! in the WASM context, including:
//! - Rating creation and initialization
//! - Match processing with RD updates
//! - Parameter configuration
//! - Serialization/deserialization
//! - Error handling
//! - Rating period handling

use wasm_bindgen_test::*;
use serde_json;

// These imports will be available once we implement the Glicko module
// use ladder_rs_wasm::{GlickoSystem, GlickoRating, GlickoUtils, MatchOutcome, MatchResult};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_glicko_system_creation_default() {
    // Test creating a Glicko system with default parameters
    // Default: c = 15.8, initial_rating = 1500.0, initial_rd = 350.0
    
    // let system = GlickoSystem::new();
    // assert_eq!(system.c(), 15.8);
    // assert_eq!(system.initial_rating(), 1500.0);
    // assert_eq!(system.initial_rd(), 350.0);
}

#[wasm_bindgen_test]
fn test_glicko_system_creation_custom() {
    // Test creating a Glicko system with custom parameters
    
    // let system = GlickoSystem::with_parameters(20.0, 1200.0, 300.0);
    // assert_eq!(system.c(), 20.0);
    // assert_eq!(system.initial_rating(), 1200.0);
    // assert_eq!(system.initial_rd(), 300.0);
}

#[wasm_bindgen_test]
fn test_glicko_rating_creation() {
    // Test creating Glicko ratings
    
    // let system = GlickoSystem::new();
    
    // // Test creating a new player rating
    // let rating = system.create_rating();
    // assert_eq!(rating.mu(), 1500.0);
    // assert_eq!(rating.rd(), 350.0);
    
    // // Test creating a rating with custom values
    // let custom_rating = system.create_rating_with_values(1800.0, 200.0);
    // assert_eq!(custom_rating.mu(), 1800.0);
    // assert_eq!(custom_rating.rd(), 200.0);
}

#[wasm_bindgen_test]
fn test_glicko_1v1_win() {
    // Test match processing where player1 wins
    
    // let system = GlickoSystem::new();
    
    // // Create two players with different ratings
    // let player1 = system.create_rating_with_values(1500.0, 200.0);
    // let player2 = system.create_rating_with_values(1400.0, 300.0);
    
    // // Process a match where player1 wins
    // let result = system.process_1v1(&player1, &player2, MatchOutcome::Player1Win).unwrap();
    
    // // Winner should gain rating, loser should lose rating
    // assert!(result.player1_rating() > player1.mu());
    // assert!(result.player2_rating() < player2.mu());
    
    // // RD should decrease for both players
    // assert!(result.player1_rd() < player1.rd());
    // assert!(result.player2_rd() < player2.rd());
}

#[wasm_bindgen_test]
fn test_glicko_1v1_draw() {
    // Test draw processing in Glicko
    
    // let system = GlickoSystem::new();
    
    // // Create two players with equal ratings
    // let player1 = system.create_rating();
    // let player2 = system.create_rating();
    
    // // Process a draw
    // let result = system.process_1v1(&player1, &player2, MatchOutcome::Draw).unwrap();
    
    // // Ratings should remain close for equal players
    // assert!((result.player1_rating() - player1.mu()).abs() < 1.0);
    // assert!((result.player2_rating() - player2.mu()).abs() < 1.0);
    
    // // RD should decrease for both players
    // assert!(result.player1_rd() < player1.rd());
    // assert!(result.player2_rd() < player2.rd());
}

#[wasm_bindgen_test]
fn test_glicko_rd_increase_over_time() {
    // Test that RD increases when no games are played
    
    // let system = GlickoSystem::new();
    
    // let rating = system.create_rating_with_values(1500.0, 200.0);
    
    // // Apply rating period without matches
    // let updated = system.apply_rating_period(&rating, 3).unwrap();
    
    // // RD should increase
    // assert!(updated.rd() > rating.rd());
    // // Rating should remain the same
    // assert_eq!(updated.mu(), rating.mu());
}

#[wasm_bindgen_test]
fn test_glicko_upset_scenario() {
    // Test when lower rated player wins
    
    // let system = GlickoSystem::new();
    
    // // Create players with significant rating difference
    // let strong_player = system.create_rating_with_values(1700.0, 100.0);
    // let weak_player = system.create_rating_with_values(1300.0, 200.0);
    
    // // Weak player wins (upset)
    // let result = system.process_1v1(&strong_player, &weak_player, MatchOutcome::Player2Win).unwrap();
    
    // // Weak player should gain significantly
    // let weak_gain = result.player2_rating() - weak_player.mu();
    // let strong_loss = strong_player.mu() - result.player1_rating();
    
    // assert!(weak_gain > strong_loss); // Asymmetric gains/losses
}

#[wasm_bindgen_test]
fn test_glicko_win_probability() {
    // Test win probability calculations
    
    // let system = GlickoSystem::new();
    
    // let player1 = system.create_rating_with_values(1600.0, 150.0);
    // let player2 = system.create_rating_with_values(1400.0, 200.0);
    
    // // Player 1 should have higher win probability
    // let prob1 = system.win_probability(&player1, &player2);
    // assert!(prob1 > 0.5);
    // assert!(prob1 < 1.0);
    
    // // Probabilities should sum to 1
    // let prob2 = system.win_probability(&player2, &player1);
    // assert!((prob1 + prob2 - 1.0).abs() < 0.001);
}

#[wasm_bindgen_test]
fn test_glicko_batch_processing() {
    // Test processing multiple matches in batch
    
    // let system = GlickoSystem::new();
    
    // // Create ratings JSON with RD values
    // let ratings_json = r#"[
    //     {"mu":1500,"rd":350},
    //     {"mu":1500,"rd":350},
    //     {"mu":1500,"rd":350},
    //     {"mu":1500,"rd":350}
    // ]"#;
    
    // // Create matches
    // let matches_json = r#"[[0,1,1],[2,3,2],[0,2,0]]"#;
    
    // let result_json = GlickoUtils::batch_process(&system, ratings_json, matches_json).unwrap();
    // let results: Vec<GlickoRating> = serde_json::from_str(&result_json).unwrap();
    
    // // Verify all RDs decreased
    // assert!(results.iter().all(|r| r.rd() < 350.0));
}

#[wasm_bindgen_test]
fn test_glicko_serialization() {
    // Test rating serialization/deserialization
    
    // let system = GlickoSystem::new();
    
    // // Create and serialize a rating
    // let rating = system.create_rating_with_values(1650.0, 250.0);
    // let serialized = rating.serialize();
    
    // // Deserialize and verify
    // let deserialized = GlickoRating::deserialize(&serialized).unwrap();
    // assert_eq!(deserialized.mu(), rating.mu());
    // assert_eq!(deserialized.rd(), rating.rd());
}

#[wasm_bindgen_test]
fn test_glicko_system_serialization() {
    // Test system configuration serialization
    
    // let system = GlickoSystem::with_parameters(20.0, 1400.0, 300.0);
    
    // // Serialize the system
    // let serialized = system.serialize();
    
    // // Deserialize and verify parameters
    // let deserialized = GlickoSystem::deserialize(&serialized).unwrap();
    // assert_eq!(deserialized.c(), 20.0);
    // assert_eq!(deserialized.initial_rating(), 1400.0);
    // assert_eq!(deserialized.initial_rd(), 300.0);
}

#[wasm_bindgen_test]
fn test_glicko_javascript_interop() {
    // Test JavaScript-friendly interfaces
    
    // let system = GlickoSystem::new();
    
    // // Test JSON serialization for JS
    // let rating = system.create_rating();
    // let json = rating.to_json();
    // assert!(json.contains("\"mu\":1500"));
    // assert!(json.contains("\"rd\":350"));
    
    // // Test creating from JSON
    // let from_json = GlickoRating::from_json(r#"{"mu":1750,"rd":200}"#).unwrap();
    // assert_eq!(from_json.mu(), 1750.0);
    // assert_eq!(from_json.rd(), 200.0);
}

#[wasm_bindgen_test]
fn test_glicko_error_handling() {
    // Test error handling for invalid inputs
    
    // // Test invalid JSON deserialization
    // assert!(GlickoRating::from_json("invalid json").is_err());
    // assert!(GlickoRating::from_json("{}").is_err()); // missing required fields
    
    // // Test invalid RD values
    // let system = GlickoSystem::new();
    // let result = system.create_rating_with_values(1500.0, -100.0); // negative RD
    // assert!(result.is_err() || result.unwrap().rd() >= 0.0);
}

#[wasm_bindgen_test]
fn test_glicko_edge_cases() {
    // Test edge cases and boundary conditions
    
    // let system = GlickoSystem::new();
    
    // // Test with very high RD (uncertain player)
    // let uncertain = system.create_rating_with_values(1500.0, 500.0);
    // let certain = system.create_rating_with_values(1500.0, 100.0);
    
    // let result = system.process_1v1(&uncertain, &certain, MatchOutcome::Player1Win).unwrap();
    
    // // Uncertain player's RD should decrease more
    // let uncertain_rd_change = uncertain.rd() - result.player1_rd();
    // let certain_rd_change = certain.rd() - result.player2_rd();
    // assert!(uncertain_rd_change > certain_rd_change);
}

#[wasm_bindgen_test]
fn test_glicko_rating_periods() {
    // Test handling of rating periods
    
    // let system = GlickoSystem::new();
    
    // // Create ratings with different last update times
    // let active = system.create_rating();
    // let inactive = system.create_rating();
    
    // // Process multiple rating periods for inactive player
    // let updated_inactive = system.apply_rating_period(&inactive, 5).unwrap();
    
    // // RD should increase significantly
    // assert!(updated_inactive.rd() > inactive.rd() + 50.0);
    
    // // Process match for active player
    // let opponent = system.create_rating();
    // let result = system.process_1v1(&active, &opponent, MatchOutcome::Draw).unwrap();
    
    // // Active player's RD should be lower than inactive
    // assert!(result.player1_rd() < updated_inactive.rd());
}

#[wasm_bindgen_test]
fn test_glicko_leaderboard() {
    // Test leaderboard creation with Glicko ratings
    
    // let ratings_json = r#"[
    //     {"mu":1600,"rd":150},
    //     {"mu":1400,"rd":200},
    //     {"mu":1800,"rd":100},
    //     {"mu":1500,"rd":350}
    // ]"#;
    
    // let leaderboard_json = GlickoUtils::create_leaderboard(ratings_json).unwrap();
    // let leaderboard: Vec<Vec<serde_json::Value>> = serde_json::from_str(&leaderboard_json).unwrap();
    
    // // Should be sorted by rating descending
    // assert_eq!(leaderboard[0][0].as_u64(), Some(2)); // index 2 has rating 1800
    // assert_eq!(leaderboard[0][1].as_f64(), Some(1800.0));
}

#[wasm_bindgen_test]
fn test_glicko_match_quality() {
    // Test match quality calculations
    
    // let system = GlickoSystem::new();
    
    // // Equal players with low RD should have high match quality
    // let player1 = system.create_rating_with_values(1500.0, 100.0);
    // let player2 = system.create_rating_with_values(1500.0, 100.0);
    // let quality1 = system.match_quality(&player1, &player2);
    // assert!(quality1 > 0.8);
    
    // // Very different players should have low match quality
    // let strong = system.create_rating_with_values(2000.0, 50.0);
    // let weak = system.create_rating_with_values(1000.0, 300.0);
    // let quality2 = system.match_quality(&strong, &weak);
    // assert!(quality2 < 0.3);
}

#[wasm_bindgen_test]
fn test_glicko_c_parameter_effect() {
    // Test the effect of the c parameter on RD increase
    
    // let system_low_c = GlickoSystem::with_parameters(10.0, 1500.0, 350.0);
    // let system_high_c = GlickoSystem::with_parameters(30.0, 1500.0, 350.0);
    
    // let rating = system_low_c.create_rating_with_values(1500.0, 200.0);
    
    // // Apply same number of rating periods
    // let updated_low_c = system_low_c.apply_rating_period(&rating, 3).unwrap();
    // let updated_high_c = system_high_c.apply_rating_period(&rating, 3).unwrap();
    
    // // Higher c should result in faster RD increase
    // assert!(updated_high_c.rd() > updated_low_c.rd());
}