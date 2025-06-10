//! Comprehensive tests for the unified rating system interface

extern crate ladder_rs_wasm;

use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;
use ladder_rs_wasm::{UnifiedRatingSystem, RatingSystemType, PlayerInfo, MatchResult};
use serde_json::json;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_unified_system_elo() {
    let config = json!({
        "system": "elo",
        "k_factor": 32.0
    });
    
    let system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create Elo system");
    
    assert_eq!(system.system_type(), RatingSystemType::Elo);
}

#[wasm_bindgen_test]
fn test_create_unified_system_unsupported() {
    // For now, Glicko and TrueSkill are not supported
    let config = json!({
        "system": "glicko"
    });
    
    let result = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap());
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_invalid_system_type() {
    let config = json!({
        "system": "invalid_system"
    });
    
    let result = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap());
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_create_player() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    let player = system.create_player("player1".to_string())
        .expect("Should create player");
    
    assert_eq!(player.id(), "player1");
    assert!(player.rating() > 0.0);
    assert!(player.uncertainty() > 0.0);
}

#[wasm_bindgen_test]
fn test_get_player_info() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    system.create_player("player1".to_string()).expect("Should create player");
    
    let info = system.get_player("player1".to_string())
        .expect("Should get player info");
    
    assert_eq!(info.id(), "player1");
}

#[wasm_bindgen_test]
fn test_get_nonexistent_player() {
    let config = json!({ "system": "elo" });
    let system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    let result = system.get_player("nonexistent".to_string());
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_simple_1v1_match() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    system.create_player("player1".to_string()).unwrap();
    system.create_player("player2".to_string()).unwrap();
    
    let initial_rating1 = system.get_player("player1".to_string()).unwrap().rating();
    let initial_rating2 = system.get_player("player2".to_string()).unwrap().rating();
    
    // Player 1 wins
    let result = system.process_match(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1  // Team 1 wins
    ).expect("Should process match");
    
    assert_eq!(result.winner_team(), 1);
    assert_eq!(result.updated_ratings().len(), 2);
    
    let final_rating1 = system.get_player("player1".to_string()).unwrap().rating();
    let final_rating2 = system.get_player("player2".to_string()).unwrap().rating();
    
    assert!(final_rating1 > initial_rating1, "Winner rating should increase");
    assert!(final_rating2 < initial_rating2, "Loser rating should decrease");
}

#[wasm_bindgen_test]
fn test_team_match() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    // Create 4 players
    for i in 1..=4 {
        system.create_player(format!("player{}", i)).unwrap();
    }
    
    // Team 1: player1, player2
    // Team 2: player3, player4
    let result = system.process_match(
        vec!["player1".to_string(), "player2".to_string()],
        vec!["player3".to_string(), "player4".to_string()],
        2  // Team 2 wins
    ).expect("Should process team match");
    
    assert_eq!(result.winner_team(), 2);
    assert_eq!(result.updated_ratings().len(), 4);
}

#[wasm_bindgen_test]
fn test_match_quality_calculation() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    system.create_player("player1".to_string()).unwrap();
    system.create_player("player2".to_string()).unwrap();
    
    let quality = system.calculate_match_quality(
        vec!["player1".to_string()],
        vec!["player2".to_string()]
    ).expect("Should calculate match quality");
    
    assert!(quality > 0.0 && quality <= 1.0, "Match quality should be between 0 and 1");
}

#[wasm_bindgen_test]
fn test_win_probability() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    system.create_player("player1".to_string()).unwrap();
    system.create_player("player2".to_string()).unwrap();
    
    let prob = system.predict_win_probability(
        vec!["player1".to_string()],
        vec!["player2".to_string()]
    ).expect("Should calculate win probability");
    
    // With equal ratings, probability should be close to 0.5
    assert!((prob - 0.5).abs() < 0.01, "Equal ratings should give ~50% win probability");
}

#[wasm_bindgen_test]
fn test_leaderboard() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    // Create players
    for i in 1..=5 {
        system.create_player(format!("player{}", i)).unwrap();
    }
    
    // Simulate some matches to create rating differences
    system.process_match(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1
    ).unwrap();
    
    system.process_match(
        vec!["player1".to_string()],
        vec!["player3".to_string()],
        1
    ).unwrap();
    
    let leaderboard = system.get_leaderboard(None);
    
    assert_eq!(leaderboard.len(), 5);
    // Check that it's sorted by rating (descending)
    for i in 0..leaderboard.len() - 1 {
        assert!(leaderboard[i].rating() >= leaderboard[i + 1].rating());
    }
}

#[wasm_bindgen_test]
fn test_leaderboard_with_limit() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    // Create 10 players
    for i in 1..=10 {
        system.create_player(format!("player{}", i)).unwrap();
    }
    
    let top5 = system.get_leaderboard(Some(5));
    assert_eq!(top5.len(), 5);
}

#[wasm_bindgen_test]
fn test_serialization_roundtrip() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    // Create players and process matches
    system.create_player("player1".to_string()).unwrap();
    system.create_player("player2".to_string()).unwrap();
    system.process_match(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        1
    ).unwrap();
    
    // Serialize
    let serialized = system.serialize().expect("Should serialize");
    
    // Create new system from serialized data
    let restored = UnifiedRatingSystem::deserialize(serialized)
        .expect("Should deserialize");
    
    // Verify state is preserved
    let player1 = restored.get_player("player1".to_string()).unwrap();
    let player2 = restored.get_player("player2".to_string()).unwrap();
    
    assert!(player1.rating() > player2.rating());
}

#[wasm_bindgen_test]
fn test_error_handling() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    // Empty player ID
    let result = system.create_player("".to_string());
    assert!(result.is_err());
    
    // Match with non-existent player
    let result = system.process_match(
        vec!["nonexistent".to_string()],
        vec!["alsonothere".to_string()],
        1
    );
    assert!(result.is_err());
    
    // Invalid winner team
    system.create_player("player1".to_string()).unwrap();
    system.create_player("player2".to_string()).unwrap();
    let result = system.process_match(
        vec!["player1".to_string()],
        vec!["player2".to_string()],
        3  // Invalid team number
    );
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_batch_operations() {
    let config = json!({ "system": "elo" });
    let mut system = UnifiedRatingSystem::new(serde_wasm_bindgen::to_value(&config).unwrap())
        .expect("Should create system");
    
    // Batch create players
    let player_ids: Vec<String> = (1..=10).map(|i| format!("player{}", i)).collect();
    system.create_players(player_ids.clone())
        .expect("Should create players in batch");
    
    // Verify all were created
    for id in &player_ids {
        assert!(system.get_player(id.clone()).is_ok());
    }
    
    // Batch match processing
    let matches = vec![
        serde_wasm_bindgen::to_value(&json!({
            "team1": ["player1"],
            "team2": ["player2"],
            "winner": 1
        })).unwrap(),
        serde_wasm_bindgen::to_value(&json!({
            "team1": ["player3"],
            "team2": ["player4"],
            "winner": 2
        })).unwrap(),
        serde_wasm_bindgen::to_value(&json!({
            "team1": ["player5"],
            "team2": ["player6"],
            "winner": 1
        })).unwrap(),
    ];
    
    let results = system.process_matches(matches)
        .expect("Should process matches in batch");
    
    assert_eq!(results.len(), 3);
}