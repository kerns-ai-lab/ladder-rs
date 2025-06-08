#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;
use serde_json::json;

extern crate ladder_rs_wasm;
use ladder_rs_wasm::{WasmRatingSystem, WasmTeam};

wasm_bindgen_test_configure!(run_in_browser);

/// Test performance of rating calculations at scale
#[wasm_bindgen_test]
fn test_rating_calculation_performance() {
    console_error_panic_hook::set_once();
    
    // Test each rating system for performance
    let systems = vec![
        ("elo", json!({"type": "elo", "k_factor": 32.0})),
        ("glicko", json!({"type": "glicko", "initial_volatility": 0.06})),
        ("trueskill", json!({"type": "trueskill", "beta": 4.166666666666667, "tau": 0.08333333333333333})),
    ];
    
    for (system_name, config) in systems {
        let start_time = instant::Instant::now();
        
        let mut system = WasmRatingSystem::new(system_name, JsValue::from_serde(&config).unwrap())
            .expect(&format!("Failed to create {} system", system_name));
        
        // Create multiple players and run matches
        let mut players = Vec::new();
        for i in 0..20 {
            let player = system.create_player(format!("{}_{}", system_name, i))
                .expect("Failed to create player");
            players.push(player);
        }
        
        // Run 50 matches between random players
        for i in 0..50 {
            let player1_idx = i % players.len();
            let player2_idx = (i + 1) % players.len();
            
            let mut team1 = WasmTeam::new(1.0); // Winner
            let mut team2 = WasmTeam::new(2.0); // Loser
            
            team1.add_player(players[player1_idx].clone());
            team2.add_player(players[player2_idx].clone());
            
            let updated_teams = system.update_ratings(vec![team1, team2])
                .expect("Failed to update ratings");
            
            // Update our local player array with new ratings
            players[player1_idx] = updated_teams[0].players[0].clone();
            players[player2_idx] = updated_teams[1].players[0].clone();
        }
        
        let duration = start_time.elapsed();
        let msg = format!("{} system: 20 players, 50 matches in {:?}", 
                         system_name, duration);
        web_sys::console::log_1(&msg.into());
        
        // Performance should be reasonable (less than 1 second)
        assert!(duration.as_millis() < 1000, 
               "{} performance test failed: {:?} > 1000ms", system_name, duration);
    }
    
    web_sys::console::log_1(&"Rating calculation performance test completed!".into());
}

/// Test stress conditions with many players
#[wasm_bindgen_test]
fn test_stress_conditions() {
    console_error_panic_hook::set_once();
    
    // Use Elo for this stress test (simplest system)
    let config = json!({
        "type": "elo",
        "k_factor": 32.0
    });
    
    let mut system = WasmRatingSystem::new("elo", JsValue::from_serde(&config).unwrap())
        .expect("Failed to create Elo system");
    
    let start_time = instant::Instant::now();
    
    // Create 100 players
    let mut players = Vec::new();
    for i in 0..100 {
        let player = system.create_player(format!("stress_player_{}", i))
            .expect("Failed to create player");
        players.push(player);
    }
    
    // Run 500 matches
    for i in 0..500 {
        let player1_idx = i % players.len();
        let player2_idx = (i * 7 + 13) % players.len(); // Use a different pattern to avoid always same players
        
        if player1_idx == player2_idx {
            continue; // Skip self-matches
        }
        
        let mut team1 = WasmTeam::new(if i % 3 == 0 { 1.0 } else { 2.0 }); // Random winner
        let mut team2 = WasmTeam::new(if i % 3 == 0 { 2.0 } else { 1.0 });
        
        team1.add_player(players[player1_idx].clone());
        team2.add_player(players[player2_idx].clone());
        
        if let Ok(updated_teams) = system.update_ratings(vec![team1, team2]) {
            players[player1_idx] = updated_teams[0].players[0].clone();
            players[player2_idx] = updated_teams[1].players[0].clone();
        }
    }
    
    let duration = start_time.elapsed();
    let msg = format!("Stress test: 100 players, 500 matches in {:?}", duration);
    web_sys::console::log_1(&msg.into());
    
    // Verify rating distribution makes sense
    let total_rating: f64 = players.iter().map(|p| p.rating).sum();
    let avg_rating = total_rating / players.len() as f64;
    
    // Average should be close to initial rating (1500 for Elo)
    assert!((avg_rating - 1500.0).abs() < 50.0, 
           "Average rating drift too large: {}", avg_rating);
    
    // Should have some spread in ratings
    let min_rating = players.iter().map(|p| p.rating).fold(f64::INFINITY, f64::min);
    let max_rating = players.iter().map(|p| p.rating).fold(f64::NEG_INFINITY, f64::max);
    let rating_spread = max_rating - min_rating;
    
    assert!(rating_spread > 50.0, "Rating spread too small: {}", rating_spread);
    
    let msg = format!("Rating spread: {:.1} (min: {:.1}, max: {:.1}, avg: {:.1})", 
                     rating_spread, min_rating, max_rating, avg_rating);
    web_sys::console::log_1(&msg.into());
    
    web_sys::console::log_1(&"Stress test completed!".into());
}

/// Test memory usage and cleanup
#[wasm_bindgen_test] 
fn test_memory_usage() {
    console_error_panic_hook::set_once();
    
    // Create and destroy multiple systems to test memory cleanup
    for iteration in 0..10 {
        let config = json!({
            "type": "trueskill",
            "beta": 4.166666666666667,
            "tau": 0.08333333333333333
        });
        
        let mut system = WasmRatingSystem::new("trueskill", JsValue::from_serde(&config).unwrap())
            .expect("Failed to create TrueSkill system");
        
        // Create players and run matches
        let mut players = Vec::new();
        for i in 0..20 {
            let player = system.create_player(format!("mem_test_{}_{}", iteration, i))
                .expect("Failed to create player");
            players.push(player);
        }
        
        // Run some matches
        for i in 0..10 {
            let player1_idx = i % players.len();
            let player2_idx = (i + 1) % players.len();
            
            let mut team1 = WasmTeam::new(1.0);
            let mut team2 = WasmTeam::new(2.0);
            
            team1.add_player(players[player1_idx].clone());
            team2.add_player(players[player2_idx].clone());
            
            let _ = system.update_ratings(vec![team1, team2]);
        }
        
        // System will be dropped here - this tests that Drop implementations work correctly
    }
    
    web_sys::console::log_1(&"Memory usage test completed - created and destroyed 10 systems with 20 players each".into());
}