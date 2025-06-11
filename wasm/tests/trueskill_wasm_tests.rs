//! Comprehensive tests for TrueSkill rating system WASM bindings

use wasm_bindgen_test::*;
use serde_json;
use ladder_rs_wasm::{TrueSkillSystem, TrueSkillRating, TrueSkillTeam, TrueSkillUtils};
use js_sys::Array;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_trueskill_system_creation_default() {
    // Test creating a TrueSkill system with default parameters
    let system = TrueSkillSystem::new();
    assert_eq!(system.mu(), 25.0);
    assert!((system.sigma() - 8.333).abs() < 0.001);
    assert!((system.beta() - 4.166).abs() < 0.001);
    assert!((system.tau() - 0.0833).abs() < 0.001);
    assert_eq!(system.draw_probability(), 0.1);
}

#[wasm_bindgen_test]
fn test_trueskill_system_creation_custom() {
    // Test creating a TrueSkill system with custom parameters
    let system = TrueSkillSystem::with_parameters(30.0, 10.0, 5.0, 0.1, 0.05).unwrap();
    assert_eq!(system.mu(), 30.0);
    assert_eq!(system.sigma(), 10.0);
    assert_eq!(system.beta(), 5.0);
    assert_eq!(system.tau(), 0.1);
    assert_eq!(system.draw_probability(), 0.05);
}

#[wasm_bindgen_test]
fn test_trueskill_rating_creation() {
    // Test creating TrueSkill ratings
    let system = TrueSkillSystem::new();
    
    // Test creating a new player rating
    let rating = system.create_rating();
    assert_eq!(rating.mean(), 25.0);
    assert!((rating.variance() - (25.0/3.0)*(25.0/3.0)).abs() < 0.001);
    
    // Test creating a rating with custom values
    let custom_rating = system.create_rating_with_values(30.0, 100.0).unwrap();
    assert_eq!(custom_rating.mean(), 30.0);
    assert_eq!(custom_rating.variance(), 100.0);
}

#[wasm_bindgen_test]
fn test_trueskill_team_creation() {
    // Test creating teams in TrueSkill
    let system = TrueSkillSystem::new();
    
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    let player3 = system.create_rating();
    
    // Create a team with multiple players
    let team = TrueSkillTeam::from_ratings(vec![player1, player2, player3]).unwrap();
    assert_eq!(team.size(), 3);
    assert_eq!(team.mean_sum(), 75.0); // 3 * 25.0
}

#[wasm_bindgen_test]
fn test_trueskill_1v1_win() {
    // Test 1v1 match processing
    let system = TrueSkillSystem::new();
    
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    
    let team1 = TrueSkillTeam::from_ratings(vec![player1]).unwrap();
    let team2 = TrueSkillTeam::from_ratings(vec![player2]).unwrap();
    
    // Create JS arrays for teams and ranks
    let teams = Array::new();
    teams.push(&serde_json::to_string(&team1).unwrap().into());
    teams.push(&serde_json::to_string(&team2).unwrap().into());
    
    let ranks = Array::new();
    ranks.push(&1.into());
    ranks.push(&2.into());
    
    // Process a match where team1 wins
    let result = system.process_match(&teams, &ranks).unwrap();
    
    // Parse results
    let team1_json = js_sys::JSON::stringify(&result.get(0)).unwrap();
    let team2_json = js_sys::JSON::stringify(&result.get(1)).unwrap();
    
    let result_team1: TrueSkillTeam = serde_json::from_str(&team1_json.as_string().unwrap()).unwrap();
    let result_team2: TrueSkillTeam = serde_json::from_str(&team2_json.as_string().unwrap()).unwrap();
    
    // Winner should gain rating, loser should lose rating
    assert!(result_team1.ratings()[0].mean() > 25.0);
    assert!(result_team2.ratings()[0].mean() < 25.0);
    
    // Variance should decrease for both players
    assert!(result_team1.ratings()[0].variance() < (25.0/3.0)*(25.0/3.0));
    assert!(result_team2.ratings()[0].variance() < (25.0/3.0)*(25.0/3.0));
}

#[wasm_bindgen_test]
fn test_trueskill_1v1_draw() {
    // Test draw handling in TrueSkill
    let system = TrueSkillSystem::new();
    
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    
    let team1 = TrueSkillTeam::from_ratings(vec![player1]).unwrap();
    let team2 = TrueSkillTeam::from_ratings(vec![player2]).unwrap();
    
    // Create JS arrays for teams and ranks
    let teams = Array::new();
    teams.push(&serde_json::to_string(&team1).unwrap().into());
    teams.push(&serde_json::to_string(&team2).unwrap().into());
    
    let ranks = Array::new();
    ranks.push(&1.into());
    ranks.push(&1.into());
    
    // Process a draw
    let result = system.process_match(&teams, &ranks).unwrap();
    
    // Parse results
    let team1_json = js_sys::JSON::stringify(&result.get(0)).unwrap();
    let team2_json = js_sys::JSON::stringify(&result.get(1)).unwrap();
    
    let result_team1: TrueSkillTeam = serde_json::from_str(&team1_json.as_string().unwrap()).unwrap();
    let result_team2: TrueSkillTeam = serde_json::from_str(&team2_json.as_string().unwrap()).unwrap();
    
    // Ratings should remain close for equal players
    assert!((result_team1.ratings()[0].mean() - 25.0).abs() < 0.5);
    assert!((result_team2.ratings()[0].mean() - 25.0).abs() < 0.5);
    
    // Variance should decrease
    assert!(result_team1.ratings()[0].variance() < (25.0/3.0)*(25.0/3.0));
}

#[wasm_bindgen_test]
fn test_trueskill_serialization() {
    // Test rating serialization/deserialization
    let system = TrueSkillSystem::new();
    
    // Create and serialize a rating
    let rating = system.create_rating_with_values(30.0, 100.0).unwrap();
    let serialized = rating.serialize();
    
    // Deserialize and verify
    let deserialized = TrueSkillRating::deserialize(&serialized).unwrap();
    assert_eq!(deserialized.mean(), 30.0);
    assert_eq!(deserialized.variance(), 100.0);
}

#[wasm_bindgen_test]
fn test_trueskill_conservative_rating() {
    // Test conservative rating calculations
    let system = TrueSkillSystem::new();
    
    let rating = system.create_rating_with_values(25.0, 64.0).unwrap();
    // Conservative rating = mean - 3 * std_dev = 25 - 3 * 8 = 1
    assert_eq!(rating.conservative_rating(), 1.0);
}

#[wasm_bindgen_test]
fn test_trueskill_match_quality() {
    // Test match quality calculations
    let system = TrueSkillSystem::new();
    
    // Equal players should have high match quality
    let player1 = system.create_rating();
    let player2 = system.create_rating();
    let team1 = TrueSkillTeam::from_ratings(vec![player1]).unwrap();
    let team2 = TrueSkillTeam::from_ratings(vec![player2]).unwrap();
    
    let teams = Array::new();
    teams.push(&serde_json::to_string(&team1).unwrap().into());
    teams.push(&serde_json::to_string(&team2).unwrap().into());
    
    let quality1 = system.match_quality(&teams).unwrap();
    assert!(quality1 > 0.8);
}

#[wasm_bindgen_test]
fn test_trueskill_error_handling() {
    // Test error handling for invalid inputs
    
    // Test invalid variance
    let system = TrueSkillSystem::new();
    assert!(system.create_rating_with_values(25.0, -100.0).is_err());
    assert!(system.create_rating_with_values(25.0, 0.0).is_err());
    
    // Test invalid team compositions
    let empty_team = TrueSkillTeam::from_ratings(vec![]);
    assert!(empty_team.is_err());
}

#[wasm_bindgen_test]
fn test_trueskill_batch_processing() {
    // Test processing multiple matches in batch
    let system = TrueSkillSystem::new();
    
    // Create ratings JSON
    let ratings_json = r#"[
        {"mean":25,"variance":69.44},
        {"mean":25,"variance":69.44},
        {"mean":25,"variance":69.44},
        {"mean":25,"variance":69.44}
    ]"#;
    
    // Create matches (team compositions and results)
    let matches_json = r#"[
        {"teams":[[0],[1]],"ranks":[1,2]},
        {"teams":[[2],[3]],"ranks":[2,1]},
        {"teams":[[0],[2]],"ranks":[1,1]}
    ]"#;
    
    let result_json = TrueSkillUtils::batch_process(&system, ratings_json, matches_json).unwrap();
    let results: Vec<TrueSkillRating> = serde_json::from_str(&result_json).unwrap();
    
    // Verify ratings changed appropriately
    assert_ne!(results[0].mean(), 25.0);
    assert!(results[0].variance() < 69.44);
}

#[wasm_bindgen_test]
fn test_trueskill_javascript_interop() {
    // Test JavaScript-friendly interfaces
    let system = TrueSkillSystem::new();
    
    // Test JSON serialization for JS
    let rating = system.create_rating();
    let json = rating.to_json();
    assert!(json.contains("\"mean\":25"));
    assert!(json.contains("\"variance\":"));
    
    // Test creating from JSON
    let from_json = TrueSkillRating::from_json(r#"{"mean":30,"variance":100}"#).unwrap();
    assert_eq!(from_json.mean(), 30.0);
    assert_eq!(from_json.variance(), 100.0);
}

#[wasm_bindgen_test]
fn test_trueskill_leaderboard() {
    // Test leaderboard creation with TrueSkill ratings
    let ratings_json = r#"[
        {"mean":30,"variance":25},
        {"mean":20,"variance":36},
        {"mean":35,"variance":16},
        {"mean":25,"variance":64}
    ]"#;
    
    let leaderboard_json = TrueSkillUtils::create_leaderboard(ratings_json, true).unwrap();
    let leaderboard: Vec<Vec<serde_json::Value>> = serde_json::from_str(&leaderboard_json).unwrap();
    
    // Should be sorted by conservative rating descending
    // Conservative ratings: 30-3*5=15, 20-3*6=2, 35-3*4=23, 25-3*8=1
    assert_eq!(leaderboard[0][0].as_u64(), Some(2)); // index 2 has highest conservative rating (23)
}