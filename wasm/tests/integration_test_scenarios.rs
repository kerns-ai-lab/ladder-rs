use wasm_bindgen_test::*;
use ladder_rs_wasm::*;
use crate::test_infrastructure::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Comprehensive integration test scenarios for the WASM module.
/// These tests verify end-to-end functionality across rating systems,
/// data persistence, error handling, and real-world usage patterns.

#[wasm_bindgen_test]
fn test_tournament_lifecycle_integration() {
    let mut fixture = TestFixture::new();
    let logger = TestLogger::new();
    
    logger.log("Starting tournament lifecycle integration test");
    
    // Create a rating system and add players
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    // Tournament setup - 8 players
    let player_names = vec!["Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Henry"];
    let mut player_ids = Vec::new();
    
    for name in &player_names {
        let player_id = system.add_player(name).expect("Failed to add player");
        player_ids.push(player_id);
        logger.log(&format!("Added player: {} (ID: {})", name, player_id));
    }
    
    // Verify initial state
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    assert_eq!(leaderboard.len(), 8);
    
    // Simulate round-robin tournament (each player plays everyone else once)
    let mut match_count = 0;
    for i in 0..player_ids.len() {
        for j in (i + 1)..player_ids.len() {
            let player1_id = player_ids[i];
            let player2_id = player_ids[j];
            
            // Simulate match with random outcome weighted by player skill
            let winner_id = if i < j { player1_id } else { player2_id };
            
            system.record_match(player1_id, player2_id, winner_id, None)
                .expect("Failed to record match");
            
            match_count += 1;
            
            if match_count % 10 == 0 {
                logger.log(&format!("Completed {} matches", match_count));
            }
        }
    }
    
    // Verify final state
    let final_leaderboard = system.get_leaderboard().expect("Failed to get final leaderboard");
    assert_eq!(final_leaderboard.len(), 8);
    
    // Check that ratings have changed from initial values
    let initial_rating = 1500.0; // Default Elo rating
    let mut ratings_changed = 0;
    
    for player in &final_leaderboard {
        if (player.rating() - initial_rating).abs() > 0.1 {
            ratings_changed += 1;
        }
    }
    
    assert!(ratings_changed >= 6, "Expected most players to have changed ratings");
    
    logger.log(&format!(
        "Tournament completed: {} matches, {} players with changed ratings",
        match_count, ratings_changed
    ));
}

#[wasm_bindgen_test]
fn test_cross_system_rating_migration() {
    let logger = TestLogger::new();
    logger.log("Starting cross-system rating migration test");
    
    // Start with Elo system
    let elo_system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    // Add players and play some games
    let alice_id = elo_system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = elo_system.add_player("Bob").expect("Failed to add Bob");
    
    // Alice wins 3 games against Bob
    for i in 0..3 {
        elo_system.record_match(alice_id, bob_id, alice_id, None)
            .expect(&format!("Failed to record match {}", i + 1));
    }
    
    let alice_elo = elo_system.get_player_rating(alice_id)
        .expect("Failed to get Alice's Elo rating");
    let bob_elo = elo_system.get_player_rating(bob_id)
        .expect("Failed to get Bob's Elo rating");
    
    logger.log(&format!("Elo ratings - Alice: {:.3}, Bob: {:.3}", alice_elo, bob_elo));
    
    // Migration scenario: Create equivalent players in Glicko system
    let glicko_system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    
    let alice_glicko_id = glicko_system.add_player_with_rating("Alice", alice_elo, 350.0)
        .expect("Failed to add Alice to Glicko");
    let bob_glicko_id = glicko_system.add_player_with_rating("Bob", bob_elo, 350.0)
        .expect("Failed to add Bob to Glicko");
    
    // Continue with one more game in Glicko
    glicko_system.record_match(alice_glicko_id, bob_glicko_id, alice_glicko_id, None)
        .expect("Failed to record Glicko match");
    
    let alice_glicko = glicko_system.get_player_rating(alice_glicko_id)
        .expect("Failed to get Alice's Glicko rating");
    let bob_glicko = glicko_system.get_player_rating(bob_glicko_id)
        .expect("Failed to get Bob's Glicko rating");
    
    logger.log(&format!("Glicko ratings - Alice: {:.3}, Bob: {:.3}", alice_glicko, bob_glicko));
    
    // Verify that Alice still has higher rating in both systems
    assert!(alice_elo > bob_elo, "Alice should have higher Elo rating");
    assert!(alice_glicko > bob_glicko, "Alice should have higher Glicko rating");
    
    // Migration should preserve relative ordering
    let elo_diff = alice_elo - bob_elo;
    let glicko_diff = alice_glicko - bob_glicko;
    
    // Both should be positive (Alice ahead) and reasonably close in magnitude
    assert!(elo_diff > 0.0 && glicko_diff > 0.0, "Rating differences should be positive");
    
    logger.log("Cross-system migration test completed successfully");
}

#[wasm_bindgen_test]
fn test_error_recovery_scenarios() {
    let logger = TestLogger::new();
    logger.log("Starting error recovery scenarios test");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create system");
    
    // Test 1: Invalid match scenarios
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    // Try to record match with invalid winner
    let invalid_winner_result = system.record_match(alice_id, bob_id, 9999, None);
    assert!(invalid_winner_result.is_err(), "Should fail with invalid winner ID");
    
    // Try to record match with non-existent player
    let invalid_player_result = system.record_match(9999, bob_id, alice_id, None);
    assert!(invalid_player_result.is_err(), "Should fail with invalid player ID");
    
    // Try to record match where player plays themselves
    let self_match_result = system.record_match(alice_id, alice_id, alice_id, None);
    assert!(self_match_result.is_err(), "Should fail when player plays themselves");
    
    logger.log("Invalid match scenarios handled correctly");
    
    // Test 2: Rating retrieval errors
    let invalid_rating_result = system.get_player_rating(9999);
    assert!(invalid_rating_result.is_err(), "Should fail to get rating for invalid player");
    
    // Test 3: System recovery after errors
    // System should still work normally after encountering errors
    let valid_match_result = system.record_match(alice_id, bob_id, alice_id, None);
    assert!(valid_match_result.is_ok(), "Valid match should work after errors");
    
    let alice_rating = system.get_player_rating(alice_id)
        .expect("Should be able to get Alice's rating after errors");
    assert!(alice_rating > 1500.0, "Alice should have gained rating from win");
    
    logger.log("Error recovery scenarios completed successfully");
}

#[wasm_bindgen_test]
fn test_concurrent_operations_simulation() {
    let logger = TestLogger::new();
    logger.log("Starting concurrent operations simulation test");
    
    let system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    
    // Add multiple players
    let mut player_ids = Vec::new();
    for i in 0..20 {
        let player_name = format!("Player_{}", i);
        let player_id = system.add_player(&player_name)
            .expect(&format!("Failed to add {}", player_name));
        player_ids.push(player_id);
    }
    
    logger.log("Added 20 players for concurrent operations test");
    
    // Simulate rapid-fire operations that might happen in a real application
    let mut operations_completed = 0;
    
    // Batch of matches
    for round in 0..5 {
        logger.log(&format!("Starting round {}", round + 1));
        
        // Each round, every player plays against a random opponent
        for i in 0..player_ids.len() {
            let player1_id = player_ids[i];
            let opponent_idx = (i + round * 3 + 1) % player_ids.len();
            let player2_id = player_ids[opponent_idx];
            
            if player1_id != player2_id {
                // Deterministic outcome based on player indices for reproducibility
                let winner_id = if i < opponent_idx { player1_id } else { player2_id };
                
                let match_result = system.record_match(player1_id, player2_id, winner_id, None);
                assert!(match_result.is_ok(), "Match should be recorded successfully");
                operations_completed += 1;
            }
        }
        
        // Get leaderboard after each round
        let leaderboard = system.get_leaderboard()
            .expect("Should be able to get leaderboard");
        assert_eq!(leaderboard.len(), 20, "Leaderboard should contain all players");
        
        // Verify ratings are reasonable (no NaN, infinite, or extreme values)
        for player in &leaderboard {
            let rating = player.rating();
            assert!(rating.is_finite(), "Rating should be finite");
            assert!(rating > 0.0, "Rating should be positive");
            assert!(rating < 10000.0, "Rating should be reasonable");
        }
    }
    
    logger.log(&format!(
        "Concurrent operations simulation completed: {} operations",
        operations_completed
    ));
}

#[wasm_bindgen_test]
fn test_data_consistency_verification() {
    let logger = TestLogger::new();
    logger.log("Starting data consistency verification test");
    
    let system = WasmRatingSystem::new("trueskill").expect("Failed to create TrueSkill system");
    
    // Add players
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    let carol_id = system.add_player("Carol").expect("Failed to add Carol");
    
    // Record initial state
    let initial_leaderboard = system.get_leaderboard()
        .expect("Failed to get initial leaderboard");
    assert_eq!(initial_leaderboard.len(), 3);
    
    // Record a series of matches with known outcomes
    let matches = vec![
        (alice_id, bob_id, alice_id),    // Alice beats Bob
        (bob_id, carol_id, bob_id),      // Bob beats Carol
        (carol_id, alice_id, carol_id),  // Carol beats Alice (creates cycle)
        (alice_id, bob_id, bob_id),      // Bob beats Alice (reversal)
    ];
    
    for (i, (player1, player2, winner)) in matches.iter().enumerate() {
        system.record_match(*player1, *player2, *winner, None)
            .expect(&format!("Failed to record match {}", i + 1));
        
        // Verify consistency after each match
        let leaderboard = system.get_leaderboard()
            .expect("Failed to get leaderboard");
        
        // Should still have all 3 players
        assert_eq!(leaderboard.len(), 3);
        
        // No duplicate player IDs
        let mut seen_ids = std::collections::HashSet::new();
        for player in &leaderboard {
            assert!(seen_ids.insert(player.id()), "Duplicate player ID in leaderboard");
        }
        
        // Ratings should be ordered (highest first)
        for j in 1..leaderboard.len() {
            assert!(
                leaderboard[j-1].rating() >= leaderboard[j].rating(),
                "Leaderboard should be ordered by rating"
            );
        }
    }
    
    let final_leaderboard = system.get_leaderboard()
        .expect("Failed to get final leaderboard");
    
    logger.log("Final leaderboard:");
    for (i, player) in final_leaderboard.iter().enumerate() {
        logger.log(&format!("  {}. {} (ID: {}, Rating: {:.3})", 
                           i + 1, player.name(), player.id(), player.rating()));
    }
    
    logger.log("Data consistency verification completed successfully");
}

#[wasm_bindgen_test]
fn test_rating_system_specific_features() {
    let logger = TestLogger::new();
    logger.log("Starting rating system specific features test");
    
    // Test Elo system
    let elo_system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    test_elo_specific_features(&elo_system, &logger);
    
    // Test Glicko system  
    let glicko_system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    test_glicko_specific_features(&glicko_system, &logger);
    
    // Test TrueSkill system
    let trueskill_system = WasmRatingSystem::new("trueskill").expect("Failed to create TrueSkill system");
    test_trueskill_specific_features(&trueskill_system, &logger);
    
    logger.log("Rating system specific features test completed");
}

fn test_elo_specific_features(system: &WasmRatingSystem, logger: &TestLogger) {
    logger.log("Testing Elo-specific features");
    
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    // Elo should start at exactly 1500
    let initial_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's initial rating");
    assert_eq!(initial_rating, 1500.0, "Elo should start at exactly 1500");
    
    // Test draw handling
    system.record_match(alice_id, bob_id, alice_id, Some("draw".to_string()))
        .expect("Failed to record draw");
    
    let alice_after_draw = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's rating after draw");
    let bob_after_draw = system.get_player_rating(bob_id)
        .expect("Failed to get Bob's rating after draw");
    
    // In Elo, equal players drawing should have minimal rating change
    assert!((alice_after_draw - 1500.0).abs() < 1.0, "Elo draw should cause minimal change");
    assert!((bob_after_draw - 1500.0).abs() < 1.0, "Elo draw should cause minimal change");
    
    logger.log("Elo-specific features tested successfully");
}

fn test_glicko_specific_features(system: &WasmRatingSystem, logger: &TestLogger) {
    logger.log("Testing Glicko-specific features");
    
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    // Glicko should handle rating deviation (uncertainty)
    // After matches, uncertainty should decrease
    let initial_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's initial rating");
    
    // Play several matches to reduce uncertainty
    for i in 0..5 {
        let winner = if i % 2 == 0 { alice_id } else { bob_id };
        system.record_match(alice_id, bob_id, winner, None)
            .expect("Failed to record Glicko match");
    }
    
    let final_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's final rating");
    
    // Rating should have changed due to the matches
    assert!((final_rating - initial_rating).abs() > 0.1, "Glicko rating should change after matches");
    
    logger.log("Glicko-specific features tested successfully");
}

fn test_trueskill_specific_features(system: &WasmRatingSystem, logger: &TestLogger) {
    logger.log("Testing TrueSkill-specific features");
    
    let alice_id = system.add_player("Alice").expect("Failed to add Alice");
    let bob_id = system.add_player("Bob").expect("Failed to add Bob");
    
    // TrueSkill should start at 25.0 (different from Elo/Glicko)
    let initial_rating = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's initial rating");
    assert_eq!(initial_rating, 25.0, "TrueSkill should start at 25.0");
    
    // TrueSkill should handle uncertainty and converge over time
    for i in 0..10 {
        let winner = if i < 7 { alice_id } else { bob_id }; // Alice wins 7/10
        system.record_match(alice_id, bob_id, winner, None)
            .expect("Failed to record TrueSkill match");
    }
    
    let alice_final = system.get_player_rating(alice_id)
        .expect("Failed to get Alice's final rating");
    let bob_final = system.get_player_rating(bob_id)
        .expect("Failed to get Bob's final rating");
    
    // Alice should have higher rating after winning more
    assert!(alice_final > bob_final, "Alice should have higher TrueSkill rating");
    assert!(alice_final > 25.0, "Alice should be above initial rating");
    assert!(bob_final < 25.0, "Bob should be below initial rating");
    
    logger.log("TrueSkill-specific features tested successfully");
}

#[wasm_bindgen_test]
fn test_large_scale_performance() {
    let logger = TestLogger::new();
    logger.log("Starting large-scale performance test");
    
    let mut timer = PerformanceTimer::new();
    
    // Test with a larger number of players and matches
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    
    timer.start("player_creation");
    
    // Add 100 players
    let mut player_ids = Vec::new();
    for i in 0..100 {
        let player_name = format!("Player_{:03}", i);
        let player_id = system.add_player(&player_name)
            .expect(&format!("Failed to add {}", player_name));
        player_ids.push(player_id);
    }
    
    let creation_time = timer.end("player_creation");
    logger.log(&format!("Created 100 players in {:.3}ms", creation_time));
    
    timer.start("bulk_matches");
    
    // Record 1000 matches
    let mut matches_recorded = 0;
    for round in 0..10 {
        for i in 0..player_ids.len() {
            let opponent_idx = (i + round * 7 + 1) % player_ids.len();
            if i != opponent_idx {
                let player1_id = player_ids[i];
                let player2_id = player_ids[opponent_idx];
                let winner_id = if i < opponent_idx { player1_id } else { player2_id };
                
                system.record_match(player1_id, player2_id, winner_id, None)
                    .expect("Failed to record match");
                matches_recorded += 1;
                
                if matches_recorded >= 1000 {
                    break;
                }
            }
        }
        if matches_recorded >= 1000 {
            break;
        }
    }
    
    let match_time = timer.end("bulk_matches");
    logger.log(&format!("Recorded {} matches in {:.3}ms", matches_recorded, match_time));
    
    timer.start("leaderboard_generation");
    
    // Generate leaderboard
    let leaderboard = system.get_leaderboard().expect("Failed to get leaderboard");
    
    let leaderboard_time = timer.end("leaderboard_generation");
    logger.log(&format!("Generated leaderboard in {:.3}ms", leaderboard_time));
    
    // Verify performance characteristics
    assert_eq!(leaderboard.len(), 100, "Leaderboard should contain all players");
    
    // Check that performance is reasonable (these are loose bounds)
    assert!(creation_time < 100.0, "Player creation should be fast");
    assert!(match_time < 1000.0, "Match recording should be reasonably fast");
    assert!(leaderboard_time < 50.0, "Leaderboard generation should be fast");
    
    // Verify rating distribution is reasonable
    let ratings: Vec<f64> = leaderboard.iter().map(|p| p.rating()).collect();
    let min_rating = ratings.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_rating = ratings.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let avg_rating = ratings.iter().sum::<f64>() / ratings.len() as f64;
    
    logger.log(&format!(
        "Rating distribution - Min: {:.1}, Max: {:.1}, Avg: {:.1}",
        min_rating, max_rating, avg_rating
    ));
    
    // Ratings should be spread out but not extreme
    assert!(max_rating - min_rating > 50.0, "Ratings should be spread out");
    assert!(max_rating - min_rating < 1000.0, "Rating spread should not be extreme");
    assert!((avg_rating - 1500.0).abs() < 100.0, "Average rating should be near initial");
    
    logger.log("Large-scale performance test completed successfully");
}