//! Comprehensive tests for Elo rating system WASM bindings
//!
//! This test module validates all aspects of the Elo implementation
//! in the WASM context, including:
//! - Rating creation and initialization
//! - Match processing (wins, losses, draws)
//! - Multi-team support
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
    let (new_p1, new_p2) = system.process_1v1(&player1, &player2, MatchOutcome::Player1Win);
    
    // Winner should gain rating, loser should lose rating
    assert!(new_p1.value() > player1.value());
    assert!(new_p2.value() < player2.value());
    
    // The sum of rating changes should be zero (conservation)
    let p1_change = new_p1.value() - player1.value();
    let p2_change = new_p2.value() - player2.value();
    assert!((p1_change + p2_change).abs() < 0.001);
}

#[wasm_bindgen_test]
fn test_elo_1v1_draw() {
    let system = EloSystem::new();
    
    // Create two players with equal ratings
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    
    // Process a draw
    let (new_p1, new_p2) = system.process_1v1(&player1, &player2, MatchOutcome::Draw);
    
    // Both ratings should remain the same for equal players
    assert!((new_p1.value() - player1.value()).abs() < 0.001);
    assert!((new_p2.value() - player2.value()).abs() < 0.001);
}

#[wasm_bindgen_test]
fn test_elo_1v1_upset() {
    let system = EloSystem::new();
    
    // Create two players with different ratings
    let strong_player = system.create_rating_with_value(1700.0);
    let weak_player = system.create_rating_with_value(1300.0);
    
    // Weak player wins (upset)
    let (new_strong, new_weak) = system.process_1v1(&strong_player, &weak_player, MatchOutcome::Player2Win);
    
    // Weak player should gain more than strong player loses
    let strong_loss = strong_player.value() - new_strong.value();
    let weak_gain = new_weak.value() - weak_player.value();
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
fn test_elo_team_match() {
    let system = EloSystem::new();
    
    // Create teams with multiple players
    let team1 = vec![
        system.create_rating_with_value(1600.0),
        system.create_rating_with_value(1400.0),
    ];
    let team2 = vec![
        system.create_rating_with_value(1500.0),
        system.create_rating_with_value(1500.0),
    ];
    
    // Process team match
    let (new_team1, new_team2) = system.process_team_match(&team1, &team2, TeamOutcome::Team1Win);
    
    // All team1 players should gain rating
    assert!(new_team1[0].value() > team1[0].value());
    assert!(new_team1[1].value() > team1[1].value());
    
    // All team2 players should lose rating
    assert!(new_team2[0].value() < team2[0].value());
    assert!(new_team2[1].value() < team2[1].value());
}

#[wasm_bindgen_test]
fn test_elo_k_factor_effect() {
    // Test with different k-factors
    let system_high_k = EloSystem::with_parameters(40.0, 1500.0);
    let system_low_k = EloSystem::with_parameters(10.0, 1500.0);
    
    let player1 = system_high_k.create_rating();
    let player2 = system_high_k.create_rating();
    
    // Process same match with different k-factors
    let (new_p1_high, _) = system_high_k.process_1v1(&player1, &player2, MatchOutcome::Player1Win);
    let (new_p1_low, _) = system_low_k.process_1v1(&player1, &player2, MatchOutcome::Player1Win);
    
    // Higher k-factor should result in larger rating change
    let change_high = new_p1_high.value() - player1.value();
    let change_low = new_p1_low.value() - player1.value();
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
    
    // Create multiple players
    let mut players = vec![
        ("player1", system.create_rating()),
        ("player2", system.create_rating()),
        ("player3", system.create_rating()),
        ("player4", system.create_rating()),
    ];
    
    // Process multiple matches
    let matches = vec![
        (0, 1, MatchOutcome::Player1Win),  // player1 beats player2
        (2, 3, MatchOutcome::Player2Win),  // player4 beats player3
        (0, 3, MatchOutcome::Draw),        // player1 draws with player4
    ];
    
    for (idx1, idx2, outcome) in matches {
        let (new_p1, new_p2) = system.process_1v1(&players[idx1].1, &players[idx2].1, outcome);
        players[idx1].1 = new_p1;
        players[idx2].1 = new_p2;
    }
    
    // Verify expected rating order: player1 > player4 > player3 > player2
    assert!(players[0].1.value() > players[3].1.value());
    assert!(players[3].1.value() > players[2].1.value());
    assert!(players[2].1.value() > players[1].1.value());
}

#[wasm_bindgen_test]
fn test_elo_edge_cases() {
    let system = EloSystem::new();
    
    // Test with extreme rating differences
    let very_strong = system.create_rating_with_value(3000.0);
    let very_weak = system.create_rating_with_value(100.0);
    
    // Even with extreme difference, ratings should update reasonably
    let (new_strong, new_weak) = system.process_1v1(&very_strong, &very_weak, MatchOutcome::Player1Win);
    assert!(new_strong.value() > very_strong.value());
    assert!(new_weak.value() < very_weak.value());
    
    // Test with negative ratings
    let negative = system.create_rating_with_value(-500.0);
    let positive = system.create_rating_with_value(500.0);
    
    let (new_neg, new_pos) = system.process_1v1(&negative, &positive, MatchOutcome::Player2Win);
    assert!(new_neg.value() < negative.value());
    assert!(new_pos.value() > positive.value());
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
}

#[wasm_bindgen_test]
fn test_elo_performance_characteristics() {
    let system = EloSystem::new();
    
    // Create many players
    let players: Vec<_> = (0..1000)
        .map(|i| system.create_rating_with_value(1200.0 + (i as f64) * 0.6))
        .collect();
    
    // Process many matches - should be fast even with many operations
    let start = js_sys::Date::now();
    
    for i in 0..100 {
        let p1_idx = (i * 7) % players.len();
        let p2_idx = (i * 13) % players.len();
        if p1_idx != p2_idx {
            system.process_1v1(&players[p1_idx], &players[p2_idx], MatchOutcome::Player1Win);
        }
    }
    
    let elapsed = js_sys::Date::now() - start;
    // Should process 100 matches in less than 10ms
    assert!(elapsed < 10.0);
}

#[wasm_bindgen_test]
fn test_elo_error_handling() {
    let system = EloSystem::new();
    
    // Test invalid JSON deserialization
    assert!(EloRating::from_json("invalid json").is_err());
    assert!(EloRating::from_json("{}").is_err()); // missing value field
    
    // Test invalid system parameters
    assert!(EloSystem::with_parameters(-10.0, 1500.0).k_factor() > 0.0); // Should handle negative k-factor
    
    // Test deserialization of corrupted data
    assert!(EloRating::deserialize("corrupted").is_err());
    assert!(EloSystem::deserialize("corrupted").is_err());
}