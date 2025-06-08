use wasm_bindgen_test::*;
use ladder_rs_wasm::*;
use crate::test_infrastructure::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Performance integration tests for the WASM module.
/// These tests verify performance characteristics under various loads,
/// stress conditions, and real-world usage patterns.

#[wasm_bindgen_test]
fn test_rating_system_performance_comparison() {
    let logger = TestLogger::new();
    logger.log("Starting rating system performance comparison test");
    
    let mut timer = PerformanceTimer::new();
    let test_players = 100;
    let test_matches = 500;
    
    // Test each rating system
    let systems = vec![
        ("elo", "Elo"),
        ("glicko", "Glicko"),
        ("trueskill", "TrueSkill")
    ];
    
    let mut performance_results = Vec::new();
    
    for (system_type, system_name) in systems {
        logger.log(&format!("Testing {} system performance", system_name));
        
        let system = WasmRatingSystem::new(system_type)
            .expect(&format!("Failed to create {} system", system_name));
        
        // Measure player creation time
        timer.start("player_creation");
        
        let mut player_ids = Vec::new();
        for i in 0..test_players {
            let name = format!("Player_{:03}", i);
            let id = system.add_player(&name)
                .expect(&format!("Failed to add player {}", name));
            player_ids.push(id);
        }
        
        let creation_time = timer.end("player_creation");
        
        // Measure match processing time
        timer.start("match_processing");
        
        let mut matches_processed = 0;
        for i in 0..test_matches {
            let player1_idx = i % player_ids.len();
            let player2_idx = (i + 1) % player_ids.len();
            
            if player1_idx != player2_idx {
                let player1_id = player_ids[player1_idx];
                let player2_id = player_ids[player2_idx];
                let winner_id = if i % 2 == 0 { player1_id } else { player2_id };
                
                system.record_match(player1_id, player2_id, winner_id, None)
                    .expect("Failed to record match");
                matches_processed += 1;
            }
        }
        
        let processing_time = timer.end("match_processing");
        
        // Measure leaderboard generation time
        timer.start("leaderboard_generation");
        let leaderboard = system.get_leaderboard()
            .expect("Failed to get leaderboard");
        let leaderboard_time = timer.end("leaderboard_generation");
        
        // Calculate metrics
        let players_per_second = (test_players as f64) / (creation_time / 1000.0);
        let matches_per_second = (matches_processed as f64) / (processing_time / 1000.0);
        let leaderboard_players_per_ms = (leaderboard.len() as f64) / leaderboard_time;
        
        performance_results.push((
            system_name,
            creation_time,
            processing_time,
            leaderboard_time,
            players_per_second,
            matches_per_second,
            leaderboard_players_per_ms
        ));
        
        logger.log(&format!(
            "{} Performance - Creation: {:.2}ms, Processing: {:.2}ms, Leaderboard: {:.2}ms",
            system_name, creation_time, processing_time, leaderboard_time
        ));
        
        // Verify functionality
        assert_eq!(leaderboard.len(), test_players, "Should have all players");
        assert!(matches_processed > 0, "Should have processed matches");
    }
    
    // Compare performance across systems
    logger.log("Performance Comparison Summary:");
    logger.log("System      | Creation | Processing | Leaderboard | Players/s | Matches/s");
    logger.log("------------|----------|------------|-------------|-----------|----------");
    
    for (name, creation, processing, leaderboard, players_s, matches_s, _) in &performance_results {
        logger.log(&format!(
            "{:<11} | {:<8.1} | {:<10.1} | {:<11.1} | {:<9.0} | {:<8.0}",
            name, creation, processing, leaderboard, players_s, matches_s
        ));
    }
    
    // All systems should meet minimum performance requirements
    for (name, creation, processing, leaderboard, _, _, _) in &performance_results {
        assert!(*creation < 2000.0, "{} player creation should be < 2s", name);
        assert!(*processing < 5000.0, "{} match processing should be < 5s", name);
        assert!(*leaderboard < 100.0, "{} leaderboard should be < 100ms", name);
    }
    
    logger.log("Rating system performance comparison completed successfully");
}

#[wasm_bindgen_test]
fn test_scaling_behavior() {
    let logger = TestLogger::new();
    logger.log("Starting scaling behavior test");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    let mut timer = PerformanceTimer::new();
    
    // Test scaling with different player counts
    let player_counts = vec![10, 50, 100, 200, 500];
    let mut scaling_results = Vec::new();
    
    for &player_count in &player_counts {
        logger.log(&format!("Testing with {} players", player_count));
        
        // Add players
        timer.start("player_addition");
        
        let mut player_ids = Vec::new();
        for i in 0..player_count {
            let name = format!("ScaleTest_Player_{:04}", i);
            let id = system.add_player(&name)
                .expect(&format!("Failed to add player {}", name));
            player_ids.push(id);
        }
        
        let addition_time = timer.end("player_addition");
        
        // Process matches (each player plays against next player)
        timer.start("match_batch");
        
        let match_count = player_count.min(100); // Limit matches to prevent exponential growth
        for i in 0..match_count {
            let player1_idx = i % player_ids.len();
            let player2_idx = (i + 1) % player_ids.len();
            
            if player1_idx != player2_idx {
                let player1_id = player_ids[player1_idx];
                let player2_id = player_ids[player2_idx];
                let winner_id = if i % 2 == 0 { player1_id } else { player2_id };
                
                system.record_match(player1_id, player2_id, winner_id, None)
                    .expect("Failed to record match");
            }
        }
        
        let match_time = timer.end("match_batch");
        
        // Generate leaderboard
        timer.start("leaderboard_gen");
        let leaderboard = system.get_leaderboard()
            .expect("Failed to get leaderboard");
        let leaderboard_time = timer.end("leaderboard_gen");
        
        // Calculate metrics
        let players_per_ms = (player_count as f64) / addition_time;
        let matches_per_ms = (match_count as f64) / match_time;
        let leaderboard_per_ms = (leaderboard.len() as f64) / leaderboard_time;
        
        scaling_results.push((
            player_count,
            addition_time,
            match_time,
            leaderboard_time,
            players_per_ms,
            matches_per_ms,
            leaderboard_per_ms
        ));
        
        logger.log(&format!(
            "{} players: Add={:.1}ms, Match={:.1}ms, Board={:.1}ms",
            player_count, addition_time, match_time, leaderboard_time
        ));
        
        // Verify correctness
        assert!(leaderboard.len() >= player_count, "Leaderboard should contain all players");
    }
    
    // Analyze scaling characteristics
    logger.log("Scaling Analysis:");
    logger.log("Players | Add Time | Match Time | Board Time | Add Rate | Match Rate | Board Rate");
    logger.log("--------|----------|------------|------------|----------|------------|------------");
    
    for (count, add_time, match_time, board_time, add_rate, match_rate, board_rate) in &scaling_results {
        logger.log(&format!(
            "{:<7} | {:<8.1} | {:<10.1} | {:<10.1} | {:<8.2} | {:<10.2} | {:<10.2}",
            count, add_time, match_time, board_time, add_rate, match_rate, board_rate
        ));
    }
    
    // Check that performance doesn't degrade catastrophically
    // (Allow some degradation but not exponential)
    let first_result = &scaling_results[0];
    let last_result = &scaling_results[scaling_results.len() - 1];
    
    let player_ratio = (last_result.0 as f64) / (first_result.0 as f64);
    let time_ratio = last_result.2 / first_result.2; // Match time ratio
    
    logger.log(&format!(
        "Scaling factor: {:.1}x players, {:.1}x time (efficiency: {:.2})",
        player_ratio, time_ratio, player_ratio / time_ratio
    ));
    
    // Performance should not degrade more than quadratically
    assert!(time_ratio < player_ratio * player_ratio, 
           "Performance should not degrade worse than O(n²)");
    
    logger.log("Scaling behavior test completed successfully");
}

#[wasm_bindgen_test]
fn test_stress_conditions() {
    let logger = TestLogger::new();
    logger.log("Starting stress conditions test");
    
    let system = WasmRatingSystem::new("glicko").expect("Failed to create Glicko system");
    let mut timer = PerformanceTimer::new();
    
    // Stress test 1: Rapid player additions
    timer.start("rapid_additions");
    
    let mut player_ids = Vec::new();
    for i in 0..1000 {
        let name = format!("Stress_Player_{:04}", i);
        let id = system.add_player(&name)
            .expect(&format!("Failed to add stress player {}", name));
        player_ids.push(id);
    }
    
    let addition_time = timer.end("rapid_additions");
    logger.log(&format!("Added 1000 players in {:.2}ms", addition_time));
    
    // Stress test 2: Burst match processing
    timer.start("burst_matches");
    
    let mut matches_processed = 0;
    for burst in 0..10 {
        logger.log(&format!("Processing burst {} of matches", burst + 1));
        
        for i in 0..100 {
            let player1_idx = (burst * 100 + i) % player_ids.len();
            let player2_idx = (burst * 100 + i + 1) % player_ids.len();
            
            if player1_idx != player2_idx {
                let player1_id = player_ids[player1_idx];
                let player2_id = player_ids[player2_idx];
                let winner_id = if i % 2 == 0 { player1_id } else { player2_id };
                
                let result = system.record_match(player1_id, player2_id, winner_id, None);
                if result.is_ok() {
                    matches_processed += 1;
                }
            }
        }
    }
    
    let burst_time = timer.end("burst_matches");
    logger.log(&format!("Processed {} matches in bursts: {:.2}ms", matches_processed, burst_time));
    
    // Stress test 3: Repeated leaderboard generation
    timer.start("repeated_leaderboards");
    
    let mut successful_generations = 0;
    for i in 0..50 {
        let leaderboard_result = system.get_leaderboard();
        if leaderboard_result.is_ok() {
            successful_generations += 1;
            
            if i % 10 == 0 {
                let leaderboard = leaderboard_result.unwrap();
                logger.log(&format!("Leaderboard generation {}: {} players", 
                                   i + 1, leaderboard.len()));
            }
        }
    }
    
    let leaderboard_time = timer.end("repeated_leaderboards");
    logger.log(&format!("Generated {} leaderboards in {:.2}ms", 
                       successful_generations, leaderboard_time));
    
    // Verify system integrity after stress
    let final_leaderboard = system.get_leaderboard()
        .expect("System should still work after stress test");
    
    assert_eq!(final_leaderboard.len(), 1000, "Should have all 1000 players");
    assert!(matches_processed > 900, "Should have processed most matches");
    assert_eq!(successful_generations, 50, "All leaderboard generations should succeed");
    
    // Check data integrity
    let mut seen_ids = std::collections::HashSet::new();
    for player in &final_leaderboard {
        assert!(seen_ids.insert(player.id()), "No duplicate player IDs");
        assert!(!player.name().is_empty(), "Player names should not be empty");
        assert!(player.rating().is_finite(), "Ratings should be finite");
    }
    
    // Performance requirements under stress
    assert!(addition_time < 5000.0, "Player addition should complete within 5s");
    assert!(burst_time < 10000.0, "Match processing should complete within 10s");
    assert!(leaderboard_time < 2000.0, "Leaderboard generation should complete within 2s");
    
    logger.log("Stress conditions test completed successfully");
}

#[wasm_bindgen_test]
fn test_memory_usage_patterns() {
    let logger = TestLogger::new();
    logger.log("Starting memory usage patterns test");
    
    // Test memory behavior with different usage patterns
    
    // Pattern 1: Gradual growth
    logger.log("Testing gradual growth pattern");
    let system1 = WasmRatingSystem::new("elo").expect("Failed to create system");
    
    let mut player_ids = Vec::new();
    for batch in 0..10 {
        // Add players in batches
        for i in 0..20 {
            let name = format!("Gradual_B{}_P{}", batch, i);
            let id = system1.add_player(&name)
                .expect(&format!("Failed to add player {}", name));
            player_ids.push(id);
        }
        
        // Play some matches
        for i in 0..10 {
            let p1_idx = (batch * 10 + i) % player_ids.len();
            let p2_idx = (batch * 10 + i + 1) % player_ids.len();
            
            if p1_idx != p2_idx {
                let winner_idx = if i % 2 == 0 { p1_idx } else { p2_idx };
                system1.record_match(
                    player_ids[p1_idx],
                    player_ids[p2_idx],
                    player_ids[winner_idx],
                    None
                ).expect("Failed to record match");
            }
        }
        
        // Check leaderboard periodically
        if batch % 3 == 0 {
            let leaderboard = system1.get_leaderboard()
                .expect("Failed to get leaderboard");
            logger.log(&format!("Batch {}: {} players", batch, leaderboard.len()));
        }
    }
    
    let final_count1 = system1.get_leaderboard()
        .expect("Failed to get final leaderboard")
        .len();
    
    // Pattern 2: Rapid burst
    logger.log("Testing rapid burst pattern");
    let system2 = WasmRatingSystem::new("trueskill").expect("Failed to create system");
    
    let mut burst_players = Vec::new();
    for i in 0..200 {
        let name = format!("Burst_Player_{:03}", i);
        let id = system2.add_player(&name)
            .expect(&format!("Failed to add burst player {}", name));
        burst_players.push(id);
    }
    
    // Rapid match processing
    for i in 0..300 {
        let p1_idx = i % burst_players.len();
        let p2_idx = (i + 7) % burst_players.len(); // Use step to avoid adjacent pairs
        
        if p1_idx != p2_idx {
            let winner_idx = if i % 3 == 0 { p1_idx } else { p2_idx };
            system2.record_match(
                burst_players[p1_idx],
                burst_players[p2_idx],
                burst_players[winner_idx],
                None
            ).expect("Failed to record burst match");
        }
    }
    
    let final_count2 = system2.get_leaderboard()
        .expect("Failed to get burst leaderboard")
        .len();
    
    // Pattern 3: Mixed operations
    logger.log("Testing mixed operations pattern");
    let system3 = WasmRatingSystem::new("glicko").expect("Failed to create system");
    
    let mut mixed_players = Vec::new();
    let mut operations_completed = 0;
    
    for cycle in 0..20 {
        // Add some players
        for i in 0..5 {
            let name = format!("Mixed_C{}_P{}", cycle, i);
            let id = system3.add_player(&name)
                .expect(&format!("Failed to add mixed player {}", name));
            mixed_players.push(id);
            operations_completed += 1;
        }
        
        // Play some matches
        if mixed_players.len() >= 2 {
            for i in 0..3 {
                let p1_idx = (cycle * 3 + i) % mixed_players.len();
                let p2_idx = (cycle * 3 + i + 1) % mixed_players.len();
                
                if p1_idx != p2_idx {
                    let winner_idx = if i % 2 == 0 { p1_idx } else { p2_idx };
                    system3.record_match(
                        mixed_players[p1_idx],
                        mixed_players[p2_idx],
                        mixed_players[winner_idx],
                        None
                    ).expect("Failed to record mixed match");
                    operations_completed += 1;
                }
            }
        }
        
        // Get leaderboard
        let leaderboard = system3.get_leaderboard()
            .expect("Failed to get mixed leaderboard");
        operations_completed += 1;
        
        if cycle % 5 == 0 {
            logger.log(&format!("Mixed cycle {}: {} players, {} ops", 
                               cycle, leaderboard.len(), operations_completed));
        }
    }
    
    let final_count3 = mixed_players.len();
    
    // Verify all patterns worked correctly
    assert_eq!(final_count1, 200, "Gradual pattern should have 200 players");
    assert_eq!(final_count2, 200, "Burst pattern should have 200 players");
    assert_eq!(final_count3, 100, "Mixed pattern should have 100 players");
    
    logger.log(&format!(
        "Memory usage patterns completed - Gradual: {}, Burst: {}, Mixed: {}",
        final_count1, final_count2, final_count3
    ));
    
    // All systems should still be functional
    for (i, system) in [&system1, &system2, &system3].iter().enumerate() {
        let leaderboard = system.get_leaderboard()
            .expect(&format!("System {} should still work", i + 1));
        assert!(!leaderboard.is_empty(), "Systems should have players");
        
        // Verify data integrity
        for player in &leaderboard {
            assert!(player.rating().is_finite(), "Ratings should be finite");
            assert!(!player.name().is_empty(), "Names should not be empty");
        }
    }
    
    logger.log("Memory usage patterns test completed successfully");
}

#[wasm_bindgen_test]
fn test_concurrent_operation_simulation() {
    let logger = TestLogger::new();
    logger.log("Starting concurrent operation simulation test");
    
    let system = WasmRatingSystem::new("elo").expect("Failed to create Elo system");
    let mut timer = PerformanceTimer::new();
    
    // Simulate concurrent operations by interleaving different types of operations
    timer.start("concurrent_simulation");
    
    let mut player_ids = Vec::new();
    let mut total_operations = 0;
    let mut successful_operations = 0;
    
    // Phase 1: Concurrent player additions
    logger.log("Phase 1: Simulating concurrent player additions");
    for batch in 0..5 {
        for i in 0..20 {
            let name = format!("Concurrent_B{}_P{:02}", batch, i);
            let result = system.add_player(&name);
            total_operations += 1;
            
            if let Ok(id) = result {
                player_ids.push(id);
                successful_operations += 1;
            }
        }
        
        // Interleave with leaderboard requests
        let leaderboard_result = system.get_leaderboard();
        total_operations += 1;
        if leaderboard_result.is_ok() {
            successful_operations += 1;
        }
    }
    
    logger.log(&format!("Phase 1 completed: {} players added", player_ids.len()));
    
    // Phase 2: Concurrent match processing
    logger.log("Phase 2: Simulating concurrent match processing");
    let mut matches_attempted = 0;
    let mut matches_successful = 0;
    
    for round in 0..10 {
        // Process multiple matches "concurrently"
        for i in 0..10 {
            if player_ids.len() >= 2 {
                let p1_idx = (round * 10 + i) % player_ids.len();
                let p2_idx = (round * 10 + i + 3) % player_ids.len();
                
                if p1_idx != p2_idx {
                    let player1_id = player_ids[p1_idx];
                    let player2_id = player_ids[p2_idx];
                    let winner_id = if i % 2 == 0 { player1_id } else { player2_id };
                    
                    let result = system.record_match(player1_id, player2_id, winner_id, None);
                    matches_attempted += 1;
                    total_operations += 1;
                    
                    if result.is_ok() {
                        matches_successful += 1;
                        successful_operations += 1;
                    }
                }
            }
        }
        
        // Interleave with rating queries
        for i in 0..3 {
            if i < player_ids.len() {
                let rating_result = system.get_player_rating(player_ids[i]);
                total_operations += 1;
                if rating_result.is_ok() {
                    successful_operations += 1;
                }
            }
        }
        
        // Interleave with leaderboard generation
        let leaderboard_result = system.get_leaderboard();
        total_operations += 1;
        if leaderboard_result.is_ok() {
            successful_operations += 1;
        }
    }
    
    let simulation_time = timer.end("concurrent_simulation");
    
    logger.log(&format!("Phase 2 completed: {}/{} matches successful", 
                       matches_successful, matches_attempted));
    
    // Phase 3: Mixed concurrent operations
    logger.log("Phase 3: Mixed concurrent operations");
    timer.start("mixed_operations");
    
    for cycle in 0..20 {
        // Add a player
        let name = format!("Mixed_Player_{:03}", cycle);
        if let Ok(id) = system.add_player(&name) {
            player_ids.push(id);
            successful_operations += 1;
        }
        total_operations += 1;
        
        // Record a match if possible
        if player_ids.len() >= 2 {
            let p1_idx = cycle % player_ids.len();
            let p2_idx = (cycle + 1) % player_ids.len();
            
            if p1_idx != p2_idx {
                let result = system.record_match(
                    player_ids[p1_idx],
                    player_ids[p2_idx],
                    player_ids[if cycle % 2 == 0 { p1_idx } else { p2_idx }],
                    None
                );
                total_operations += 1;
                if result.is_ok() {
                    successful_operations += 1;
                }
            }
        }
        
        // Get leaderboard
        if let Ok(_) = system.get_leaderboard() {
            successful_operations += 1;
        }
        total_operations += 1;
        
        // Get random player rating
        if !player_ids.is_empty() {
            let random_idx = cycle % player_ids.len();
            if let Ok(_) = system.get_player_rating(player_ids[random_idx]) {
                successful_operations += 1;
            }
            total_operations += 1;
        }
    }
    
    let mixed_time = timer.end("mixed_operations");
    
    // Calculate final metrics
    let success_rate = (successful_operations as f64) / (total_operations as f64) * 100.0;
    let operations_per_second = (total_operations as f64) / ((simulation_time + mixed_time) / 1000.0);
    
    logger.log("Concurrent Operation Simulation Results:");
    logger.log(&format!("  Total operations: {}", total_operations));
    logger.log(&format!("  Successful operations: {}", successful_operations));
    logger.log(&format!("  Success rate: {:.1}%", success_rate));
    logger.log(&format!("  Operations per second: {:.1}", operations_per_second));
    logger.log(&format!("  Total time: {:.1}ms", simulation_time + mixed_time));
    
    // Verify final state
    let final_leaderboard = system.get_leaderboard()
        .expect("System should work after concurrent operations");
    
    logger.log(&format!("Final state: {} players in leaderboard", final_leaderboard.len()));
    
    // Performance and correctness assertions
    assert!(success_rate > 95.0, "Success rate should be above 95%");
    assert!(operations_per_second > 100.0, "Should process at least 100 operations per second");
    assert!(!final_leaderboard.is_empty(), "Should have players in final leaderboard");
    assert!(final_leaderboard.len() <= player_ids.len(), "Leaderboard should not have more players than added");
    
    // Data integrity checks
    let mut seen_ids = std::collections::HashSet::new();
    for player in &final_leaderboard {
        assert!(seen_ids.insert(player.id()), "No duplicate player IDs");
        assert!(player.rating().is_finite(), "All ratings should be finite");
        assert!(!player.name().is_empty(), "All names should be non-empty");
    }
    
    logger.log("Concurrent operation simulation test completed successfully");
}

#[wasm_bindgen_test]
fn test_performance_regression_detection() {
    let logger = TestLogger::new();
    logger.log("Starting performance regression detection test");
    
    let mut timer = PerformanceTimer::new();
    
    // Establish baseline performance with a standard workload
    let baseline_system = WasmRatingSystem::new("elo").expect("Failed to create baseline system");
    
    // Baseline workload
    let baseline_players = 100;
    let baseline_matches = 200;
    
    // Measure baseline player creation
    timer.start("baseline_players");
    let mut baseline_player_ids = Vec::new();
    for i in 0..baseline_players {
        let name = format!("Baseline_Player_{:03}", i);
        let id = baseline_system.add_player(&name)
            .expect(&format!("Failed to add baseline player {}", name));
        baseline_player_ids.push(id);
    }
    let baseline_creation_time = timer.end("baseline_players");
    
    // Measure baseline match processing
    timer.start("baseline_matches");
    for i in 0..baseline_matches {
        let p1_idx = i % baseline_player_ids.len();
        let p2_idx = (i + 1) % baseline_player_ids.len();
        
        if p1_idx != p2_idx {
            let winner_idx = if i % 2 == 0 { p1_idx } else { p2_idx };
            baseline_system.record_match(
                baseline_player_ids[p1_idx],
                baseline_player_ids[p2_idx],
                baseline_player_ids[winner_idx],
                None
            ).expect("Failed to record baseline match");
        }
    }
    let baseline_match_time = timer.end("baseline_matches");
    
    // Measure baseline leaderboard generation
    timer.start("baseline_leaderboard");
    let baseline_leaderboard = baseline_system.get_leaderboard()
        .expect("Failed to get baseline leaderboard");
    let baseline_leaderboard_time = timer.end("baseline_leaderboard");
    
    logger.log(&format!(
        "Baseline performance - Players: {:.1}ms, Matches: {:.1}ms, Leaderboard: {:.1}ms",
        baseline_creation_time, baseline_match_time, baseline_leaderboard_time
    ));
    
    // Run the same workload multiple times to check consistency
    let mut performance_samples = Vec::new();
    
    for run in 0..5 {
        logger.log(&format!("Performance run {}", run + 1));
        
        let test_system = WasmRatingSystem::new("elo").expect("Failed to create test system");
        
        // Player creation
        timer.start("test_players");
        let mut test_player_ids = Vec::new();
        for i in 0..baseline_players {
            let name = format!("Test_R{}_P{:03}", run, i);
            let id = test_system.add_player(&name)
                .expect(&format!("Failed to add test player {}", name));
            test_player_ids.push(id);
        }
        let test_creation_time = timer.end("test_players");
        
        // Match processing
        timer.start("test_matches");
        for i in 0..baseline_matches {
            let p1_idx = i % test_player_ids.len();
            let p2_idx = (i + 1) % test_player_ids.len();
            
            if p1_idx != p2_idx {
                let winner_idx = if i % 2 == 0 { p1_idx } else { p2_idx };
                test_system.record_match(
                    test_player_ids[p1_idx],
                    test_player_ids[p2_idx],
                    test_player_ids[winner_idx],
                    None
                ).expect("Failed to record test match");
            }
        }
        let test_match_time = timer.end("test_matches");
        
        // Leaderboard generation
        timer.start("test_leaderboard");
        let test_leaderboard = test_system.get_leaderboard()
            .expect("Failed to get test leaderboard");
        let test_leaderboard_time = timer.end("test_leaderboard");
        
        performance_samples.push((test_creation_time, test_match_time, test_leaderboard_time));
        
        // Verify correctness
        assert_eq!(test_leaderboard.len(), baseline_leaderboard.len(), 
                  "Test leaderboard should have same size as baseline");
    }
    
    // Analyze performance consistency
    let avg_creation = performance_samples.iter().map(|(c, _, _)| *c).sum::<f64>() / performance_samples.len() as f64;
    let avg_match = performance_samples.iter().map(|(_, m, _)| *m).sum::<f64>() / performance_samples.len() as f64;
    let avg_leaderboard = performance_samples.iter().map(|(_, _, l)| *l).sum::<f64>() / performance_samples.len() as f64;
    
    logger.log(&format!(
        "Average performance - Players: {:.1}ms, Matches: {:.1}ms, Leaderboard: {:.1}ms",
        avg_creation, avg_match, avg_leaderboard
    ));
    
    // Calculate variance
    let var_creation = performance_samples.iter()
        .map(|(c, _, _)| (c - avg_creation).powi(2))
        .sum::<f64>() / performance_samples.len() as f64;
    let var_match = performance_samples.iter()
        .map(|(_, m, _)| (m - avg_match).powi(2))
        .sum::<f64>() / performance_samples.len() as f64;
    let var_leaderboard = performance_samples.iter()
        .map(|(_, _, l)| (l - avg_leaderboard).powi(2))
        .sum::<f64>() / performance_samples.len() as f64;
    
    let std_creation = var_creation.sqrt();
    let std_match = var_match.sqrt();
    let std_leaderboard = var_leaderboard.sqrt();
    
    logger.log(&format!(
        "Performance std dev - Players: {:.1}ms, Matches: {:.1}ms, Leaderboard: {:.1}ms",
        std_creation, std_match, std_leaderboard
    ));
    
    // Regression detection
    let creation_regression = (avg_creation - baseline_creation_time) / baseline_creation_time * 100.0;
    let match_regression = (avg_match - baseline_match_time) / baseline_match_time * 100.0;
    let leaderboard_regression = (avg_leaderboard - baseline_leaderboard_time) / baseline_leaderboard_time * 100.0;
    
    logger.log("Performance regression analysis:");
    logger.log(&format!("  Player creation: {:.1}% change", creation_regression));
    logger.log(&format!("  Match processing: {:.1}% change", match_regression));
    logger.log(&format!("  Leaderboard generation: {:.1}% change", leaderboard_regression));
    
    // Assertions for regression detection
    assert!(creation_regression.abs() < 50.0, 
           "Player creation performance should not regress more than 50%");
    assert!(match_regression.abs() < 50.0, 
           "Match processing performance should not regress more than 50%");
    assert!(leaderboard_regression.abs() < 50.0, 
           "Leaderboard generation performance should not regress more than 50%");
    
    // Consistency checks (coefficient of variation should be reasonable)
    let cv_creation = (std_creation / avg_creation) * 100.0;
    let cv_match = (std_match / avg_match) * 100.0;
    let cv_leaderboard = (std_leaderboard / avg_leaderboard) * 100.0;
    
    logger.log(&format!(
        "Performance consistency (CV) - Players: {:.1}%, Matches: {:.1}%, Leaderboard: {:.1}%",
        cv_creation, cv_match, cv_leaderboard
    ));
    
    assert!(cv_creation < 30.0, "Player creation should be consistent (CV < 30%)");
    assert!(cv_match < 30.0, "Match processing should be consistent (CV < 30%)");
    assert!(cv_leaderboard < 30.0, "Leaderboard generation should be consistent (CV < 30%)");
    
    logger.log("Performance regression detection test completed successfully");
}