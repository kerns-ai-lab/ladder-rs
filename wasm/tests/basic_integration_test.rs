#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;
use serde_json::json;

extern crate ladder_rs_wasm;
use ladder_rs_wasm::{WasmRating, WasmRatingSystem, WasmTeam};

wasm_bindgen_test_configure!(run_in_browser);

/// Basic integration test to verify the WASM module works with current API
#[wasm_bindgen_test]
fn test_basic_elo_integration() {
    console_error_panic_hook::set_once();
    
    // Create Elo system with default config
    let config = json!({
        "type": "elo",
        "k_factor": 32.0
    });
    
    let mut system = WasmRatingSystem::new("elo", JsValue::from_serde(&config).unwrap())
        .expect("Failed to create Elo system");
    
    // Create two players
    let player1 = system.create_player("player1".to_string())
        .expect("Failed to create player1");
    let player2 = system.create_player("player2".to_string())
        .expect("Failed to create player2");
    
    // Verify initial ratings
    assert_eq!(player1.rating, 1500.0);
    assert_eq!(player2.rating, 1500.0);
    
    // Create teams for a match
    let mut team1 = WasmTeam::new(1.0); // Score 1 (winner)
    let mut team2 = WasmTeam::new(2.0); // Score 2 (loser)
    
    team1.add_player(player1);
    team2.add_player(player2);
    
    // Update ratings based on match result
    let updated_teams = system.update_ratings(vec![team1, team2])
        .expect("Failed to update ratings");
    
    // Verify that winner gained rating and loser lost rating
    assert!(updated_teams[0].players[0].rating > 1500.0, "Winner should gain rating");
    assert!(updated_teams[1].players[0].rating < 1500.0, "Loser should lose rating");
    
    web_sys::console::log_1(&"Basic Elo integration test passed!".into());
}

/// Test TrueSkill integration
#[wasm_bindgen_test]
fn test_basic_trueskill_integration() {
    console_error_panic_hook::set_once();
    
    // Create TrueSkill system
    let config = json!({
        "type": "trueskill",
        "beta": 4.166666666666667,
        "tau": 0.08333333333333333
    });
    
    let mut system = WasmRatingSystem::new("trueskill", JsValue::from_serde(&config).unwrap())
        .expect("Failed to create TrueSkill system");
    
    // Create two players
    let player1 = system.create_player("ts_player1".to_string())
        .expect("Failed to create player1");
    let player2 = system.create_player("ts_player2".to_string())
        .expect("Failed to create player2");
    
    // Verify initial TrueSkill ratings
    assert_eq!(player1.rating, 25.0);
    assert_eq!(player2.rating, 25.0);
    assert!(player1.uncertainty.is_some());
    assert!(player2.uncertainty.is_some());
    
    // Create teams for a match
    let mut team1 = WasmTeam::new(1.0); // Winner
    let mut team2 = WasmTeam::new(2.0); // Loser
    
    team1.add_player(player1);
    team2.add_player(player2);
    
    // Update ratings
    let updated_teams = system.update_ratings(vec![team1, team2])
        .expect("Failed to update ratings");
    
    // Verify rating changes
    assert!(updated_teams[0].players[0].rating > 25.0, "Winner should gain rating");
    assert!(updated_teams[1].players[0].rating < 25.0, "Loser should lose rating");
    
    web_sys::console::log_1(&"Basic TrueSkill integration test passed!".into());
}

/// Test Glicko integration
#[wasm_bindgen_test]
fn test_basic_glicko_integration() {
    console_error_panic_hook::set_once();
    
    // Create Glicko system
    let config = json!({
        "type": "glicko",
        "initial_volatility": 0.06
    });
    
    let mut system = WasmRatingSystem::new("glicko", JsValue::from_serde(&config).unwrap())
        .expect("Failed to create Glicko system");
    
    // Create two players
    let player1 = system.create_player("glicko_player1".to_string())
        .expect("Failed to create player1");
    let player2 = system.create_player("glicko_player2".to_string())
        .expect("Failed to create player2");
    
    // Verify initial Glicko ratings
    assert_eq!(player1.rating, 1500.0);
    assert_eq!(player2.rating, 1500.0);
    assert!(player1.uncertainty.is_some());
    assert!(player2.uncertainty.is_some());
    
    // Create teams for a match
    let mut team1 = WasmTeam::new(1.0); // Winner
    let mut team2 = WasmTeam::new(2.0); // Loser
    
    team1.add_player(player1);
    team2.add_player(player2);
    
    // Update ratings
    let updated_teams = system.update_ratings(vec![team1, team2])
        .expect("Failed to update ratings");
    
    // Verify rating changes
    assert!(updated_teams[0].players[0].rating > 1500.0, "Winner should gain rating");
    assert!(updated_teams[1].players[0].rating < 1500.0, "Loser should lose rating");
    
    web_sys::console::log_1(&"Basic Glicko integration test passed!".into());
}

/// Test error handling
#[wasm_bindgen_test]
fn test_error_handling_integration() {
    console_error_panic_hook::set_once();
    
    // Test invalid system type
    let config = json!({
        "type": "invalid_system"
    });
    
    let result = WasmRatingSystem::new("invalid_system", JsValue::from_serde(&config).unwrap());
    assert!(result.is_err(), "Should fail with invalid system type");
    
    // Test empty teams
    let config = json!({
        "type": "elo",
        "k_factor": 32.0
    });
    
    let mut system = WasmRatingSystem::new("elo", JsValue::from_serde(&config).unwrap())
        .expect("Failed to create system");
    
    let empty_team1 = WasmTeam::new(1.0);
    let empty_team2 = WasmTeam::new(2.0);
    
    let result = system.update_ratings(vec![empty_team1, empty_team2]);
    assert!(result.is_err(), "Should fail with empty teams");
    
    web_sys::console::log_1(&"Error handling integration test passed!".into());
}

/// Test match quality calculation
#[wasm_bindgen_test]
fn test_match_quality_integration() {
    console_error_panic_hook::set_once();
    
    // Create Elo system (supports match quality)
    let config = json!({
        "type": "elo",
        "k_factor": 32.0
    });
    
    let mut system = WasmRatingSystem::new("elo", JsValue::from_serde(&config).unwrap())
        .expect("Failed to create Elo system");
    
    // Create two players with same rating
    let player1 = system.create_player("quality_player1".to_string())
        .expect("Failed to create player1");
    let player2 = system.create_player("quality_player2".to_string())
        .expect("Failed to create player2");
    
    // Create teams
    let mut team1 = WasmTeam::new(0.0); // Doesn't matter for quality calculation
    let mut team2 = WasmTeam::new(0.0);
    
    team1.add_player(player1);
    team2.add_player(player2);
    
    // Calculate match quality
    let quality = system.get_match_quality(vec![team1, team2])
        .expect("Failed to calculate match quality");
    
    // Equal players should have high match quality
    assert!(quality > 0.9, "Equal players should have high match quality, got: {}", quality);
    
    web_sys::console::log_1(&"Match quality integration test passed!".into());
}