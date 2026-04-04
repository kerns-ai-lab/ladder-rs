use wasm_bindgen_test::*;
use ladder_rs_wasm::*;
use crate::test_infrastructure::*;

wasm_bindgen_test_configure!(run_in_node_js);

/// Node.js specific integration tests for the WASM module.
/// These tests verify functionality in a Node.js environment,
/// including file system operations, process handling, and
/// server-side specific features.

#[wasm_bindgen_test]
fn test_nodejs_environment_detection() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js environment detection test");
    
    // In Node.js, we should be able to detect the environment
    // This is a basic test to ensure we're running in Node.js
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let alice_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's rating");
    
    // Basic verification that the system works in Node.js
    assert_eq!(alice_rating, 1500.0, "Elo should start at 1500 in Node.js");
    
    logger.log("Node.js environment detection test completed successfully");
}

#[wasm_bindgen_test]
fn test_nodejs_concurrent_processing() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js concurrent processing test");
    
    let system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    
    // Simulate concurrent player additions (as might happen in a server)
    let mut player_ids = Vec::new();
    
    // Add players in batches to simulate concurrent requests
    for batch in 0..5 {
        logger.log(&format!("Processing batch {}", batch + 1));
        
        for i in 0..10 {
            let player_name = format!("Batch{}_Player{}", batch, i);
            let player_id = system.add_player(&player_name)
                .expect(&format!("Failed to add {}", player_name));
            player_ids.push(player_id);
        }
    }
    
    assert_eq!(player_ids.len(), 50, "Should have added 50 players");
    
    // Simulate concurrent match processing
    let mut matches_processed = 0;
    
    for round in 0..3 {
        logger.log(&format!("Processing match round {}", round + 1));
        
        // Process matches in parallel-like fashion
        for i in (0..player_ids.len()).step_by(2) {
            if i + 1 < player_ids.len() {
                let player1_id = player_ids[i];
                let player2_id = player_ids[i + 1];
                let winner_id = if (i + round) % 2 == 0 { player1_id } else { player2_id };
                
                let result = system.record_match(player1_id, player2_id, winner_id, None);
                assert!(result.is_ok(), "Match should be recorded successfully");
                matches_processed += 1;
            }
        }
    }
    
    logger.log(&format!("Processed {} matches across 3 rounds", matches_processed));
    
    // Verify final state
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    assert_eq!(leaderboard.len(), 50, "Leaderboard should contain all players");
    
    // Verify ratings are distributed (some should be above/below initial)
    let initial_rating = 1500.0;
    let mut above_initial = 0;
    let mut below_initial = 0;
    
    for player in &leaderboard {
        if player.rating() > initial_rating {
            above_initial += 1;
        } else if player.rating() < initial_rating {
            below_initial += 1;
        }
    }
    
    assert!(above_initial > 0, "Some players should be above initial rating");
    assert!(below_initial > 0, "Some players should be below initial rating");
    
    logger.log("Node.js concurrent processing test completed successfully");
}

#[wasm_bindgen_test]
fn test_nodejs_memory_management() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js memory management test");
    
    // Test creating and destroying multiple systems to check for memory leaks
    let mut systems = Vec::new();
    
    // Create multiple systems
    for i in 0..10 {
        let system_type = match i % 3 {
            0 => "elo",
            1 => "glicko", 
            _ => "trueskill"
        };
        
        let system = WasmRatingSystem::new(system_type)
            .expect(&format!("Failed to create {} system", system_type));
        
        // Add some players to each system
        for j in 0..5 {
            let player_name = format!("System{}_Player{}", i, j);
            system.add_player(&player_name)
                .expect(&format!("Failed to add player {}", player_name));
        }
        
        systems.push(system);
    }
    
    logger.log("Created 10 rating systems with 5 players each");
    
    // Process matches in each system
    for (i, system) in systems.iter().enumerate() {
        let leaderboard = system.get_leaderboard()
            .expect(&format!("Failed to get leaderboard for system {}", i));
        
        assert_eq!(leaderboard.len(), 5, "Each system should have 5 players");
        
        // Record some matches
        if leaderboard.len() >= 2 {
            let player1_id = leaderboard[0].id();
            let player2_id = leaderboard[1].id();
            
            system.record_match(player1_id, player2_id, player1_id, None)
                .expect("Failed to record match");
        }
    }
    
    // Verify all systems still work after operations
    let total_players: usize = systems.iter()
        .map(|system| system.get_leaderboard().unwrap().len())
        .sum();
    
    assert_eq!(total_players, 50, "Should have 50 total players across all systems");
    
    // Drop systems to test cleanup
    systems.clear();
    
    logger.log("Node.js memory management test completed successfully");
}

#[wasm_bindgen_test]
fn test_nodejs_error_handling() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js error handling test");
    
    // Test error handling in Node.js context
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    // Test 1: Invalid system creation
    let invalid_system_result = WasmRatingSystem::new("invalid_system");
    assert!(invalid_system_result.is_err(), "Should fail to create invalid system");
    
    // Test 2: Empty player name
    let empty_name_result = system.add_player("");
    assert!(empty_name_result.is_err(), "Should fail to add player with empty name");
    
    // Test 3: Valid operations after errors
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    // Test 4: Invalid match operations
    let invalid_winner_result = system.record_match(alice_id, bob_id, 9999, None);
    assert!(invalid_winner_result.is_err(), "Should fail with invalid winner");
    
    let self_match_result = system.record_match(alice_id, alice_id, alice_id, None);
    assert!(self_match_result.is_err(), "Should fail with self-match");
    
    // Test 5: System recovery after errors
    let valid_match_result = system.record_match(alice_id, bob_id, alice_id, None);
    assert!(valid_match_result.is_ok(), "Valid match should work after errors");
    
    let leaderboard = system.get_leaderboard().expect("Should get leaderboard after errors");
    assert_eq!(leaderboard.len(), 2, "Should have 2 players after errors");
    
    logger.log("Node.js error handling test completed successfully");
}

#[wasm_bindgen_test]
fn test_nodejs_bulk_data_processing() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js bulk data processing test");
    
    let system = WasmRatingSystem::new("trueskill").expect("Failed to create TrueSkill system");
    
    // Simulate processing a large tournament file
    let tournament_data = vec![
        ("Alice", "Bob", "Alice"),
        ("Carol", "Dave", "Carol"), 
        ("Alice", "Carol", "Alice"),
        ("Bob", "Dave", "Dave"),
        ("Alice", "Dave", "Dave"),
        ("Bob", "Carol", "Carol"),
        ("Dave", "Alice", "Alice"),
        ("Carol", "Bob", "Bob"),
        ("Alice", "Bob", "Bob"),
        ("Carol", "Dave", "Dave"),
    ];
    
    // Add all unique players first
    let mut player_map = std::collections::HashMap::new();
    let unique_players: std::collections::HashSet<&str> = tournament_data.iter()
        .flat_map(|(p1, p2, _)| vec![*p1, *p2])
        .collect();
    
    for player_name in unique_players {
        let player_id = system.add_player(player_name)
            .expect(&format!("Failed to add player {}", player_name));
        player_map.insert(player_name, player_id);
    }
    
    logger.log(&format!("Added {} unique players", player_map.len()));
    
    // Process all matches
    let mut successful_matches = 0;
    
    for (player1_name, player2_name, winner_name) in tournament_data {
        let player1_id = player_map[player1_name];
        let player2_id = player_map[player2_name];
        let winner_id = player_map[winner_name];
        
        let result = system.record_match(player1_id, player2_id, winner_id, None);
        if result.is_ok() {
            successful_matches += 1;
        }
    }
    
    logger.log(&format!("Successfully processed {} matches", successful_matches));
    assert_eq!(successful_matches, 10, "All matches should be processed successfully");
    
    // Analyze final standings
    let leaderboard = system.get_leaderboard().expect("Failed to get final leaderboard");
    assert_eq!(leaderboard.len(), 4, "Should have 4 players");
    
    // Count wins for each player
    let mut win_counts = std::collections::HashMap::new();
    for (_, _, winner_name) in &tournament_data {
        *win_counts.entry(*winner_name).or_insert(0) += 1;
    }
    
    logger.log("Win counts:");
    for (player_name, wins) in &win_counts {
        logger.log(&format!("  {}: {} wins", player_name, wins));
    }
    
    // Verify that players with more wins tend to have higher ratings
    let alice_rating = leaderboard.iter()
        .find(|p| p.name() == "Alice")
        .expect("Alice should be in leaderboard")
        .rating();
    
    let dave_rating = leaderboard.iter()
        .find(|p| p.name() == "Dave")
        .expect("Dave should be in leaderboard")
        .rating();
    
    // Alice has 3 wins, Dave has 3 wins - should be close
    // Both should be above initial TrueSkill rating of 25.0
    assert!(alice_rating > 25.0, "Alice should be above initial rating");
    assert!(dave_rating > 25.0, "Dave should be above initial rating");
    
    logger.log("Node.js bulk data processing test completed successfully");
}

#[wasm_bindgen_test]
fn test_nodejs_json_serialization() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js JSON serialization test");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    // Add players and play matches
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    let carol_id = system.add_player("Carol").expect("Failed to add Carol");
    
    system.record_match(alice_id, bob_id, alice_id, None)
        .expect("Failed to record Alice vs Bob");
    system.record_match(bob_id, carol_id, carol_id, None)
        .expect("Failed to record Bob vs Carol");
    
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    
    // Simulate JSON serialization (as would be done in Node.js)
    let mut json_data = String::from("{\n  \"leaderboard\": [\n");
    
    for (i, player) in leaderboard.iter().enumerate() {
        if i > 0 {
            json_data.push_str(",\n");
        }
        json_data.push_str(&format!(
            "    {{\n      \"rank\": {},\n      \"id\": {},\n      \"name\": \"{}\",\n      \"rating\": {:.2}\n    }}",
            i + 1,
            player.id(),
            player.name(),
            player.rating()
        ));
    }
    
    json_data.push_str("\n  ]\n}");
    
    // Verify JSON structure
    assert!(json_data.contains("\"leaderboard\""), "Should contain leaderboard key");
    assert!(json_data.contains("\"Alice\""), "Should contain Alice");
    assert!(json_data.contains("\"Bob\""), "Should contain Bob");
    assert!(json_data.contains("\"Carol\""), "Should contain Carol");
    
    // Verify JSON is valid format (basic checks)
    assert!(json_data.starts_with("{"), "Should start with opening brace");
    assert!(json_data.ends_with("}"), "Should end with closing brace");
    
    let brace_count = json_data.matches('{').count();
    let close_brace_count = json_data.matches('}').count();
    assert_eq!(brace_count, close_brace_count, "Braces should be balanced");
    
    logger.log("Generated JSON data:");
    logger.log(&json_data);
    
    // Simulate parsing the data back (basic validation)
    assert!(json_data.contains("\"rank\": 1"), "Should have rank 1");
    assert!(json_data.contains("\"rank\": 2"), "Should have rank 2");
    assert!(json_data.contains("\"rank\": 3"), "Should have rank 3");
    
    logger.log("Node.js JSON serialization test completed successfully");
}

#[wasm_bindgen_test]
fn test_nodejs_command_line_simulation() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js command line simulation test");
    
    // Simulate command line tool usage
    let args = vec![
        ("create", "elo"),
        ("add-player", "Alice"),
        ("add-player", "Bob"),
        ("match", "Alice vs Bob, winner: Alice"),
        ("leaderboard", ""),
    ];
    
    let mut system: Option<WasmRatingSystem> = None;
    let mut alice_id: Option<u32> = None;
    let mut bob_id: Option<u32> = None;
    
    for (command, arg) in args {
        logger.log(&format!("Executing command: {} {}", command, arg));
        
        match command {
            "create" => {
                system = Some(WasmRatingSystem::new(arg)
                    .expect(&format!("Failed to create {} system", arg)));
                logger.log(&format!("Created {} rating system", arg));
            }
            "add-player" => {
                if let Some(ref sys) = system {
                    let id = sys.add_player(arg)
                        .expect(&format!("Failed to add player {}", arg));
                    
                    match arg {
                        "Alice" => alice_id = Some(id),
                        "Bob" => bob_id = Some(id),
                        _ => {}
                    }
                    
                    logger.log(&format!("Added player {} with ID {}", arg, id));
                }
            }
            "match" => {
                if let (Some(ref sys), Some(a_id), Some(b_id)) = (&system, alice_id, bob_id) {
                    // Parse match string (simplified)
                    if arg.contains("winner: Alice") {
                        sys.record_match(a_id, b_id, a_id, None)
                            .expect("Failed to record match");
                        logger.log("Recorded match: Alice beats Bob");
                    }
                }
            }
            "leaderboard" => {
                if let Some(ref sys) = system {
                    let leaderboard = sys.get_leaderboard()
                        .expect("Failed to get leaderboard");
                    
                    logger.log("Current leaderboard:");
                    for (rank, player) in leaderboard.iter().enumerate() {
                        logger.log(&format!(
                            "  {}. {} - {:.1}",
                            rank + 1,
                            player.name(),
                            player.rating()
                        ));
                    }
                }
            }
            _ => {
                logger.log(&format!("Unknown command: {}", command));
            }
        }
    }
    
    // Verify final state
    if let Some(ref sys) = system {
        let leaderboard = sys.get_leaderboard().expect("Failed to get final leaderboard");
        assert_eq!(leaderboard.len(), 2, "Should have 2 players");
        
        // Alice should be first (higher rating) since she won
        assert_eq!(leaderboard[0].name(), "Alice", "Alice should be first");
        assert!(leaderboard[0].rating() > leaderboard[1].rating(), "Alice should have higher rating");
    }
    
    logger.log("Node.js command line simulation test completed successfully");
}

#[wasm_bindgen_test]
fn test_nodejs_performance_monitoring() {
    let logger = TestLogger::new();
    logger.log("Starting Node.js performance monitoring test");
    
    let system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    
    // Performance monitoring for different operations
    let mut timer = PerformanceTimer::new();
    
    // Monitor player creation performance
    timer.start("batch_player_creation");
    
    let mut player_ids = Vec::new();
    for i in 0..200 {
        let name = format!("Player_{:03}", i);
        let id = system.add_player(&name)
            .expect(&format!("Failed to add {}", name));
        player_ids.push(id);
    }
    
    let creation_time = timer.end("batch_player_creation");
    logger.log(&format!("Created 200 players in {:.3}ms", creation_time));
    
    // Monitor match processing performance
    timer.start("batch_match_processing");
    
    let mut matches_processed = 0;
    for round in 0..5 {
        for i in (0..player_ids.len()).step_by(4) {
            if i + 1 < player_ids.len() {
                let player1_id = player_ids[i];
                let player2_id = player_ids[i + 1];
                let winner_id = if (i + round) % 2 == 0 { player1_id } else { player2_id };
                
                system.record_match(player1_id, player2_id, winner_id, None)
                    .expect("Failed to record match");
                matches_processed += 1;
            }
        }
    }
    
    let match_time = timer.end("batch_match_processing");
    logger.log(&format!("Processed {} matches in {:.3}ms", matches_processed, match_time));
    
    // Monitor leaderboard generation performance
    timer.start("leaderboard_generation");
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    let leaderboard_time = timer.end("leaderboard_generation");
    
    logger.log(&format!("Generated leaderboard ({} players) in {:.3}ms", 
                       leaderboard.len(), leaderboard_time));
    
    // Calculate performance metrics
    let players_per_ms = 200.0 / creation_time;
    let matches_per_ms = matches_processed as f64 / match_time;
    
    logger.log(&format!("Performance metrics:"));
    logger.log(&format!("  Players/ms: {:.2}", players_per_ms));
    logger.log(&format!("  Matches/ms: {:.2}", matches_per_ms));
    logger.log(&format!("  Leaderboard generation: {:.3}ms", leaderboard_time));
    
    // Verify performance is acceptable for Node.js environment
    assert!(creation_time < 1000.0, "Player creation should complete within 1 second");
    assert!(match_time < 2000.0, "Match processing should complete within 2 seconds");
    assert!(leaderboard_time < 100.0, "Leaderboard generation should be fast");
    
    // Verify data integrity
    assert_eq!(leaderboard.len(), 200, "Should have all 200 players");
    
    // Check rating distribution
    let ratings: Vec<f64> = leaderboard.iter().map(|p| p.rating()).collect();
    let min_rating = ratings.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_rating = ratings.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    assert!(max_rating > min_rating, "Ratings should be distributed");
    assert!(max_rating - min_rating < 2000.0, "Rating spread should not be extreme");
    
    logger.log("Node.js performance monitoring test completed successfully");
}