//! Performance benchmarks for WASM module
//!
//! This module contains performance tests to establish baseline performance metrics.

use wasm_bindgen_test::*;
use ladder_rs_wasm::{WasmRatingSystem, WasmTeam, PlayerManager, test_utils::PerformanceTimer};
use wasm_bindgen::JsValue;

wasm_bindgen_test_configure!(run_in_browser);

/// Performance requirements (in milliseconds)
struct PerformanceRequirements {
    player_creation: f64,        // Per 1000 players
    match_processing: f64,       // Per 1000 matches
    leaderboard_generation: f64, // For 1000 players
    bulk_import: f64,           // For 1000 players
    search_operation: f64,      // For 1000 players
}

impl Default for PerformanceRequirements {
    fn default() -> Self {
        Self {
            player_creation: 100.0,        // 100ms for 1000 players
            match_processing: 500.0,       // 500ms for 1000 matches
            leaderboard_generation: 50.0,  // 50ms to generate leaderboard
            bulk_import: 200.0,           // 200ms to import 1000 players
            search_operation: 20.0,       // 20ms to search 1000 players
        }
    }
}

#[wasm_bindgen_test]
fn test_player_creation_performance() {
    let requirements = PerformanceRequirements::default();
    let mut timer = PerformanceTimer::new();
    let mut manager = PlayerManager::new();
    
    // Create 1000 players
    for i in 0..1000 {
        let id = format!("player_{}", i);
        manager.register_player(&id, None, None).unwrap();
    }
    
    let elapsed = timer.elapsed();
    assert!(
        elapsed < requirements.player_creation,
        "Player creation took {}ms, requirement is {}ms",
        elapsed, requirements.player_creation
    );
}

#[wasm_bindgen_test]
fn test_rating_system_creation_performance() {
    let mut timer = PerformanceTimer::new();
    
    // Test Elo
    timer.reset();
    let _elo = WasmRatingSystem::new("elo").unwrap();
    let elo_time = timer.elapsed();
    assert!(elo_time < 10.0, "Elo system creation took {}ms", elo_time);
    
    // Test Glicko
    timer.reset();
    let _glicko = WasmRatingSystem::new("glicko").unwrap();
    let glicko_time = timer.elapsed();
    assert!(glicko_time < 10.0, "Glicko system creation took {}ms", glicko_time);
    
    // Test TrueSkill
    timer.reset();
    let _trueskill = WasmRatingSystem::new("trueskill").unwrap();
    let trueskill_time = timer.elapsed();
    assert!(trueskill_time < 10.0, "TrueSkill system creation took {}ms", trueskill_time);
}

#[wasm_bindgen_test]
fn test_match_processing_performance() {
    let requirements = PerformanceRequirements::default();
    let mut system = WasmRatingSystem::new("elo").unwrap();
    let mut manager = PlayerManager::new();
    let mut timer = PerformanceTimer::new();
    
    // Setup: Create 100 players
    let players: Vec<String> = (0..100).map(|i| format!("player_{}", i)).collect();
    for player in &players {
        manager.register_player(player, None, None).unwrap();
        system.create_player(player).unwrap();
    }
    
    timer.reset();
    
    // Process 1000 matches
    for i in 0..1000 {
        let p1_idx = i % players.len();
        let p2_idx = (i + 1) % players.len();
        
        let team1 = WasmTeam::new(vec![players[p1_idx].clone()].into_boxed_slice());
        let team2 = WasmTeam::new(vec![players[p2_idx].clone()].into_boxed_slice());
        
        system.update_ratings(team1, team2, (i % 3).min(2) as u32).unwrap();
    }
    
    let elapsed = timer.elapsed();
    assert!(
        elapsed < requirements.match_processing,
        "Match processing took {}ms, requirement is {}ms",
        elapsed, requirements.match_processing
    );
}

#[wasm_bindgen_test]
fn test_leaderboard_generation_performance() {
    let requirements = PerformanceRequirements::default();
    let mut system = WasmRatingSystem::new("elo").unwrap();
    let mut timer = PerformanceTimer::new();
    
    // Setup: Create 1000 players
    for i in 0..1000 {
        let id = format!("player_{}", i);
        system.create_player(&id).unwrap();
    }
    
    // Play some matches to create varied ratings
    for i in 0..500 {
        let p1 = format!("player_{}", i * 2);
        let p2 = format!("player_{}", i * 2 + 1);
        
        let team1 = WasmTeam::new(vec![p1].into_boxed_slice());
        let team2 = WasmTeam::new(vec![p2].into_boxed_slice());
        
        system.update_ratings(team1, team2, (i % 2 + 1) as u32).unwrap();
    }
    
    timer.reset();
    
    // Generate leaderboard
    let leaderboard = system.get_leaderboard(None).unwrap();
    
    let elapsed = timer.elapsed();
    assert!(
        elapsed < requirements.leaderboard_generation,
        "Leaderboard generation took {}ms, requirement is {}ms",
        elapsed, requirements.leaderboard_generation
    );
    
    // Verify leaderboard is complete
    assert_eq!(leaderboard.length(), 1000);
}

#[wasm_bindgen_test]
fn test_bulk_operations_performance() {
    let requirements = PerformanceRequirements::default();
    let mut manager = PlayerManager::new();
    let mut timer = PerformanceTimer::new();
    
    // Create test data
    let mut players_data = Vec::new();
    for i in 0..1000 {
        players_data.push(format!(
            r#"{{"id":"player_{}","name":"Player {}","email":"player{}@test.com"}}"#,
            i, i, i
        ));
    }
    let json_data = format!("[{}]", players_data.join(","));
    
    timer.reset();
    
    // Bulk import
    let result = manager.bulk_import_players(&json_data).unwrap();
    
    let elapsed = timer.elapsed();
    assert!(
        elapsed < requirements.bulk_import,
        "Bulk import took {}ms, requirement is {}ms",
        elapsed, requirements.bulk_import
    );
    
    // Verify import
    let imported = js_sys::Reflect::get(&result, &JsValue::from_str("imported")).unwrap();
    assert_eq!(imported.as_f64().unwrap(), 1000.0);
}

#[wasm_bindgen_test]
fn test_search_performance() {
    let requirements = PerformanceRequirements::default();
    let mut manager = PlayerManager::new();
    let mut timer = PerformanceTimer::new();
    
    // Setup: Create 1000 players
    for i in 0..1000 {
        let id = format!("player_{}", i);
        let name = format!("Player Number {}", i);
        manager.register_player(&id, Some(&name), None).unwrap();
    }
    
    timer.reset();
    
    // Search operations
    let results1 = manager.search_players("Player");
    let results2 = manager.search_players("Number");
    let results3 = manager.search_players("500");
    
    let elapsed = timer.elapsed();
    assert!(
        elapsed < requirements.search_operation,
        "Search operations took {}ms, requirement is {}ms",
        elapsed, requirements.search_operation
    );
    
    // Verify search works
    assert!(results1.length() > 0);
    assert!(results2.length() > 0);
    assert!(results3.length() > 0);
}

#[wasm_bindgen_test]
fn test_concurrent_operations_performance() {
    let mut timer = PerformanceTimer::new();
    let mut system = WasmRatingSystem::new("trueskill").unwrap();
    let mut manager = PlayerManager::new();
    
    // Create players
    for i in 0..50 {
        let id = format!("player_{}", i);
        manager.register_player(&id, None, None).unwrap();
        system.create_player(&id).unwrap();
    }
    
    timer.reset();
    
    // Simulate concurrent-like operations
    for round in 0..10 {
        // Multiple matches in parallel (simulated)
        for match_num in 0..5 {
            let p1_idx = (round * 5 + match_num * 2) % 50;
            let p2_idx = (round * 5 + match_num * 2 + 1) % 50;
            
            let p1 = format!("player_{}", p1_idx);
            let p2 = format!("player_{}", p2_idx);
            
            // Record match
            manager.add_match_record(
                vec![p1.clone()].into_boxed_slice(),
                vec![p2.clone()].into_boxed_slice(),
                1,
                None
            ).unwrap();
            
            // Update ratings
            let team1 = WasmTeam::new(vec![p1].into_boxed_slice());
            let team2 = WasmTeam::new(vec![p2].into_boxed_slice());
            system.update_ratings(team1, team2, 1).unwrap();
        }
        
        // Get leaderboard after each round
        let _leaderboard = system.get_leaderboard(Some(10)).unwrap();
    }
    
    let elapsed = timer.elapsed();
    assert!(
        elapsed < 200.0,
        "Concurrent operations took {}ms, expected < 200ms",
        elapsed
    );
}

#[wasm_bindgen_test]
fn test_memory_efficiency() {
    // This test verifies that we can handle large numbers of objects
    // without excessive memory usage (browser will fail if we use too much)
    
    let mut manager = PlayerManager::new();
    let mut system = WasmRatingSystem::new("glicko").unwrap();
    
    // Create many players
    for i in 0..5000 {
        let id = format!("p{}", i);
        manager.register_player(&id, None, None).unwrap();
        system.create_player(&id).unwrap();
    }
    
    // Process many matches
    for i in 0..2000 {
        let p1 = format!("p{}", i % 5000);
        let p2 = format!("p{}", (i + 1) % 5000);
        
        let team1 = WasmTeam::new(vec![p1].into_boxed_slice());
        let team2 = WasmTeam::new(vec![p2].into_boxed_slice());
        
        system.update_ratings(team1, team2, 1).unwrap();
    }
    
    // If we get here without crashing, memory usage is acceptable
    assert!(true);
}

#[wasm_bindgen_test]
fn test_performance_degradation() {
    // Test that performance doesn't degrade significantly with more data
    let mut timer = PerformanceTimer::new();
    let mut manager = PlayerManager::new();
    
    // Measure time for first 100 players
    timer.reset();
    for i in 0..100 {
        manager.register_player(&format!("player_{}", i), None, None).unwrap();
    }
    let first_batch_time = timer.elapsed();
    
    // Add 900 more players
    for i in 100..1000 {
        manager.register_player(&format!("player_{}", i), None, None).unwrap();
    }
    
    // Measure time for last 100 players
    timer.reset();
    for i in 1000..1100 {
        manager.register_player(&format!("player_{}", i), None, None).unwrap();
    }
    let last_batch_time = timer.elapsed();
    
    // Performance shouldn't degrade by more than 2x
    assert!(
        last_batch_time < first_batch_time * 2.0,
        "Performance degraded: first batch {}ms, last batch {}ms",
        first_batch_time, last_batch_time
    );
}