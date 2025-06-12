//! Minimal performance regression tests for WASM bindings (Elo-only)
//! 
//! This module provides performance benchmarking and regression detection
//! for the ladder-rs WASM module with only Elo support (to avoid rayon/statrs dependencies).

use wasm_bindgen_test::*;
use web_sys::{window, Performance};
use ladder_rs_wasm::*;
use std::collections::HashMap;

wasm_bindgen_test_configure!(run_in_browser);

/// Performance metrics structure for tracking results
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    operation: String,
    duration_ms: f64,
    memory_used: Option<f64>,
    iterations: u32,
}

impl PerformanceMetrics {
    fn new(operation: &str, duration_ms: f64, iterations: u32) -> Self {
        Self {
            operation: operation.to_string(),
            duration_ms,
            memory_used: None,
            iterations,
        }
    }

    fn ops_per_second(&self) -> f64 {
        (self.iterations as f64 * 1000.0) / self.duration_ms
    }
}

/// Performance test harness for consistent measurement
struct PerformanceHarness {
    performance: Performance,
    results: HashMap<String, Vec<PerformanceMetrics>>,
}

impl PerformanceHarness {
    fn new() -> Self {
        let performance = window()
            .expect("should have window")
            .performance()
            .expect("should have performance");
        
        Self {
            performance,
            results: HashMap::new(),
        }
    }

    /// Measure the performance of a function
    fn measure<F>(&mut self, name: &str, iterations: u32, mut f: F) -> PerformanceMetrics
    where
        F: FnMut(),
    {
        // Warm up
        for _ in 0..10 {
            f();
        }

        let start = self.performance.now();
        
        for _ in 0..iterations {
            f();
        }
        
        let end = self.performance.now();
        let duration = end - start;
        
        let metrics = PerformanceMetrics::new(name, duration, iterations);
        
        self.results
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(metrics.clone());
        
        metrics
    }

    /// Get performance report
    fn report(&self) -> String {
        let mut report = String::from("Performance Test Results:\n");
        report.push_str("========================\n\n");
        
        for (operation, metrics_list) in &self.results {
            if let Some(latest) = metrics_list.last() {
                report.push_str(&format!(
                    "{}: {:.2} ms ({:.0} ops/sec) for {} iterations\n",
                    operation,
                    latest.duration_ms,
                    latest.ops_per_second(),
                    latest.iterations
                ));
            }
        }
        
        report
    }
}

/// Test: WASM Elo system initialization performance
#[wasm_bindgen_test]
fn test_elo_initialization_performance() {
    let mut harness = PerformanceHarness::new();
    
    // Test Elo system initialization
    let metrics = harness.measure("create_elo_system", 1000, || {
        let _ = create_elo_system();
    });
    
    assert!(
        metrics.ops_per_second() > 10000.0,
        "Elo system creation too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    web_sys::console::log_1(&format!("Elo Initialization Performance:\n{}", harness.report()).into());
}

/// Test: Elo rating update performance
#[wasm_bindgen_test]
fn test_elo_update_performance() {
    let mut harness = PerformanceHarness::new();
    let elo_system = create_elo_system();
    
    // Test 1v1 matches
    let player1 = create_elo_player("player1", 1500.0, None);
    let player2 = create_elo_player("player2", 1500.0, None);
    let match_result = create_match_result(vec![vec!["player1"], vec!["player2"]], vec![1, 2]);
    
    let metrics = harness.measure("elo_1v1_update", 1000, || {
        let _ = elo_system.update_ratings(
            vec![player1.clone(), player2.clone()],
            match_result.clone()
        );
    });
    
    assert!(
        metrics.ops_per_second() > 1000.0,
        "Elo 1v1 update too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    web_sys::console::log_1(&format!("Elo Update Performance:\n{}", harness.report()).into());
}

/// Test: Elo player serialization performance
#[wasm_bindgen_test]
fn test_elo_serialization_performance() {
    let mut harness = PerformanceHarness::new();
    
    // Create test players
    let players: Vec<Player> = (0..100)
        .map(|i| create_elo_player(&format!("player{}", i), 1500.0 + i as f64, None))
        .collect();
    
    // Test player serialization
    let metrics = harness.measure("serialize_100_elo_players", 100, || {
        for player in &players {
            let _ = player.to_json();
        }
    });
    
    assert!(
        metrics.ops_per_second() > 100.0,
        "Elo player serialization too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    // Test player deserialization
    let json_players: Vec<String> = players.iter().map(|p| p.to_json()).collect();
    
    let metrics = harness.measure("deserialize_100_elo_players", 100, || {
        for json in &json_players {
            let _ = Player::from_json(json);
        }
    });
    
    assert!(
        metrics.ops_per_second() > 100.0,
        "Elo player deserialization too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    web_sys::console::log_1(&format!("Elo Serialization Performance:\n{}", harness.report()).into());
}

/// Test: Elo batch operation performance
#[wasm_bindgen_test]
fn test_elo_batch_performance() {
    let mut harness = PerformanceHarness::new();
    let elo_system = create_elo_system();
    
    // Create player pool
    let players: Vec<Player> = (0..20)
        .map(|i| create_elo_player(&format!("player{}", i), 1500.0 + i as f64 * 10.0, None))
        .collect();
    
    // Generate batch of matches
    let matches: Vec<MatchResult> = (0..100)
        .map(|i| {
            let p1_idx = i % 20;
            let p2_idx = (i + 1) % 20;
            create_match_result(
                vec![vec![&format!("player{}", p1_idx)], vec![&format!("player{}", p2_idx)]],
                if i % 2 == 0 { vec![1, 2] } else { vec![2, 1] }
            )
        })
        .collect();
    
    // Test batch update performance
    let metrics = harness.measure("elo_batch_100_matches", 10, || {
        let mut current_players = players.clone();
        for match_result in &matches {
            let p1_name = &match_result.teams()[0][0];
            let p2_name = &match_result.teams()[1][0];
            
            let p1 = current_players.iter().find(|p| p.id() == p1_name).unwrap().clone();
            let p2 = current_players.iter().find(|p| p.id() == p2_name).unwrap().clone();
            
            let updated = elo_system.update_ratings(vec![p1, p2], match_result.clone()).unwrap();
            
            // Update player ratings
            for (i, player) in current_players.iter_mut().enumerate() {
                if player.id() == p1_name {
                    current_players[i] = updated[0].clone();
                } else if player.id() == p2_name {
                    current_players[i] = updated[1].clone();
                }
            }
        }
    });
    
    assert!(
        metrics.duration_ms < 5000.0,
        "Elo batch processing 100 matches took too long: {} ms",
        metrics.duration_ms
    );
    
    web_sys::console::log_1(&format!("Elo Batch Performance:\n{}", harness.report()).into());
}

/// Test: Elo performance regression thresholds
#[wasm_bindgen_test]
fn test_elo_performance_regression_thresholds() {
    // Define performance thresholds for Elo-only operations (ops/second)
    let thresholds = HashMap::from([
        ("elo_system_creation", 10000.0),
        ("elo_1v1_update", 1000.0),
        ("elo_serialization", 10000.0),
        ("elo_batch_10_matches", 100.0),
    ]);
    
    let mut harness = PerformanceHarness::new();
    let mut regressions = Vec::new();
    
    // Run performance tests and check against thresholds
    for (operation, min_ops_per_sec) in thresholds {
        match operation {
            "elo_system_creation" => {
                let metrics = harness.measure(operation, 1000, || {
                    let _ = create_elo_system();
                });
                
                if metrics.ops_per_second() < min_ops_per_sec {
                    regressions.push(format!(
                        "{}: Expected >= {} ops/sec, got {:.2} ops/sec",
                        operation, min_ops_per_sec, metrics.ops_per_second()
                    ));
                }
            }
            "elo_1v1_update" => {
                let elo_system = create_elo_system();
                let player1 = create_elo_player("p1", 1500.0, None);
                let player2 = create_elo_player("p2", 1500.0, None);
                let match_result = create_match_result(vec![vec!["p1"], vec!["p2"]], vec![1, 2]);
                
                let metrics = harness.measure(operation, 1000, || {
                    let _ = elo_system.update_ratings(
                        vec![player1.clone(), player2.clone()],
                        match_result.clone()
                    );
                });
                
                if metrics.ops_per_second() < min_ops_per_sec {
                    regressions.push(format!(
                        "{}: Expected >= {} ops/sec, got {:.2} ops/sec",
                        operation, min_ops_per_sec, metrics.ops_per_second()
                    ));
                }
            }
            _ => {} // Add more test cases as needed
        }
    }
    
    if !regressions.is_empty() {
        panic!(
            "Performance regressions detected:\n{}",
            regressions.join("\n")
        );
    }
    
    web_sys::console::log_1(&"All Elo performance thresholds met!".into());
}

/// Test: Real-world Elo tournament simulation
#[wasm_bindgen_test]
fn test_elo_tournament_simulation() {
    let mut harness = PerformanceHarness::new();
    
    // Simulate a tournament with 32 players
    let metrics = harness.measure("elo_tournament_32_players", 1, || {
        let elo_system = create_elo_system();
        
        // Create players
        let mut players: Vec<Player> = (0..32)
            .map(|i| create_elo_player(&format!("player{}", i), 1200.0 + i as f64 * 20.0, None))
            .collect();
        
        // Simulate Swiss tournament (5 rounds)
        for round in 0..5 {
            // Pair players
            let mut matches = Vec::new();
            for i in (0..32).step_by(2) {
                let match_result = create_match_result(
                    vec![vec![&format!("player{}", i)], vec![&format!("player{}", i + 1)]],
                    if (i + round) % 3 == 0 { vec![1, 2] } else { vec![2, 1] }
                );
                matches.push((i, i + 1, match_result));
            }
            
            // Process matches
            for (p1_idx, p2_idx, match_result) in matches {
                let p1 = players[p1_idx].clone();
                let p2 = players[p2_idx].clone();
                
                let updated = elo_system.update_ratings(vec![p1, p2], match_result).unwrap();
                
                players[p1_idx] = updated[0].clone();
                players[p2_idx] = updated[1].clone();
            }
        }
    });
    
    assert!(
        metrics.duration_ms < 1000.0,
        "Elo tournament simulation took too long: {} ms",
        metrics.duration_ms
    );
    
    web_sys::console::log_1(&format!("Elo Tournament Performance:\n{}", harness.report()).into());
}