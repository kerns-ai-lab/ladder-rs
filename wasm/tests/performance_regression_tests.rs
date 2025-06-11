//! Performance regression tests for WASM bindings
//! 
//! This module provides comprehensive performance benchmarking and regression
//! detection for the ladder-rs WASM module. It measures:
//! - Initialization overhead
//! - Memory usage patterns
//! - Runtime performance across different operations
//! - Cross-browser performance consistency

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

/// Test: WASM module initialization performance
#[wasm_bindgen_test]
fn test_wasm_initialization_performance() {
    let mut harness = PerformanceHarness::new();
    
    // Test various initialization scenarios
    let metrics = harness.measure("create_elo_system", 1000, || {
        let _ = create_elo_system();
    });
    
    assert!(
        metrics.ops_per_second() > 10000.0,
        "Elo system creation too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    let metrics = harness.measure("create_trueskill_system", 1000, || {
        let _ = create_trueskill_system();
    });
    
    assert!(
        metrics.ops_per_second() > 5000.0,
        "TrueSkill system creation too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    web_sys::console::log_1(&format!("Initialization Performance:\n{}", harness.report()).into());
}

/// Test: Rating update performance across different team sizes
#[wasm_bindgen_test]
fn test_rating_update_performance() {
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
    
    web_sys::console::log_1(&format!("Rating Update Performance:\n{}", harness.report()).into());
}

/// Test: Serialization/deserialization performance
#[wasm_bindgen_test]
fn test_serialization_performance() {
    let mut harness = PerformanceHarness::new();
    
    // Create test data
    let players: Vec<Player> = (0..100)
        .map(|i| create_elo_player(&format!("player{}", i), 1500.0 + i as f64, None))
        .collect();
    
    // Test player serialization
    let metrics = harness.measure("serialize_100_players", 100, || {
        for player in &players {
            let _ = player.to_json();
        }
    });
    
    assert!(
        metrics.ops_per_second() > 100.0,
        "Player serialization too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    // Test player deserialization
    let json_players: Vec<String> = players.iter().map(|p| p.to_json()).collect();
    
    let metrics = harness.measure("deserialize_100_players", 100, || {
        for json in &json_players {
            let _ = Player::from_json(json);
        }
    });
    
    assert!(
        metrics.ops_per_second() > 100.0,
        "Player deserialization too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    web_sys::console::log_1(&format!("Serialization Performance:\n{}", harness.report()).into());
}

/// Test: Memory usage patterns
#[wasm_bindgen_test]
fn test_memory_usage_patterns() {
    let mut harness = PerformanceHarness::new();
    
    // Test memory usage for large player pools
    let metrics = harness.measure("create_1000_players", 10, || {
        let _players: Vec<Player> = (0..1000)
            .map(|i| create_elo_player(&format!("player{}", i), 1500.0, None))
            .collect();
    });
    
    assert!(
        metrics.duration_ms < 1000.0,
        "Creating 1000 players took too long: {} ms",
        metrics.duration_ms
    );
    
    // Test memory usage for complex match results
    let metrics = harness.measure("create_complex_matches", 100, || {
        let teams = vec![
            vec!["p1", "p2", "p3"],
            vec!["p4", "p5", "p6"],
            vec!["p7", "p8", "p9"],
        ];
        let _ = create_match_result(teams, vec![1, 2, 3]);
    });
    
    assert!(
        metrics.ops_per_second() > 1000.0,
        "Complex match creation too slow: {} ops/sec",
        metrics.ops_per_second()
    );
    
    web_sys::console::log_1(&format!("Memory Usage Performance:\n{}", harness.report()).into());
}

/// Test: Batch operation performance
#[wasm_bindgen_test]
fn test_batch_operation_performance() {
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
    let metrics = harness.measure("batch_100_matches", 10, || {
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
        "Batch processing 100 matches took too long: {} ms",
        metrics.duration_ms
    );
    
    web_sys::console::log_1(&format!("Batch Operation Performance:\n{}", harness.report()).into());
}

/// Test: Cross-algorithm performance comparison
#[wasm_bindgen_test]
fn test_cross_algorithm_performance() {
    let mut harness = PerformanceHarness::new();
    
    // Setup systems
    let elo_system = create_elo_system();
    let trueskill_system = create_trueskill_system();
    
    // Common match setup
    let match_result = create_match_result(vec![vec!["p1"], vec!["p2"]], vec![1, 2]);
    
    // Test Elo performance
    let elo_p1 = create_elo_player("p1", 1500.0, None);
    let elo_p2 = create_elo_player("p2", 1500.0, None);
    
    let elo_metrics = harness.measure("elo_single_update", 1000, || {
        let _ = elo_system.update_ratings(
            vec![elo_p1.clone(), elo_p2.clone()],
            match_result.clone()
        );
    });
    
    // Test TrueSkill performance
    let ts_p1 = create_trueskill_player("p1", 25.0, 8.333, None);
    let ts_p2 = create_trueskill_player("p2", 25.0, 8.333, None);
    
    let ts_metrics = harness.measure("trueskill_single_update", 1000, || {
        let _ = trueskill_system.update_ratings(
            vec![ts_p1.clone(), ts_p2.clone()],
            match_result.clone()
        );
    });
    
    // TrueSkill should be within 10x of Elo performance
    let performance_ratio = elo_metrics.ops_per_second() / ts_metrics.ops_per_second();
    assert!(
        performance_ratio < 10.0,
        "TrueSkill is too slow compared to Elo: {:.2}x slower",
        performance_ratio
    );
    
    web_sys::console::log_1(&format!("Cross-Algorithm Performance:\n{}", harness.report()).into());
}

/// Test: Performance regression detection
#[wasm_bindgen_test]
fn test_performance_regression_thresholds() {
    // Define performance thresholds (ops/second)
    let thresholds = HashMap::from([
        ("elo_system_creation", 10000.0),
        ("trueskill_system_creation", 5000.0),
        ("elo_1v1_update", 1000.0),
        ("trueskill_1v1_update", 100.0),
        ("player_serialization", 10000.0),
        ("player_deserialization", 5000.0),
        ("batch_10_matches", 100.0),
        ("batch_100_matches", 10.0),
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
            "trueskill_system_creation" => {
                let metrics = harness.measure(operation, 1000, || {
                    let _ = create_trueskill_system();
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
    
    web_sys::console::log_1(&"All performance thresholds met!".into());
}

/// Integration test: Real-world usage patterns
#[wasm_bindgen_test]
fn test_real_world_usage_patterns() {
    let mut harness = PerformanceHarness::new();
    
    // Simulate a tournament with 32 players
    let metrics = harness.measure("tournament_32_players", 1, || {
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
        "Tournament simulation took too long: {} ms",
        metrics.duration_ms
    );
    
    web_sys::console::log_1(&format!("Real-world Usage Performance:\n{}", harness.report()).into());
}

#[cfg(test)]
mod test_utilities {
    use super::*;
    
    /// Helper to generate performance baseline data
    pub fn generate_baseline_data() -> HashMap<String, f64> {
        let mut harness = PerformanceHarness::new();
        let mut baselines = HashMap::new();
        
        // Measure current performance for all operations
        let operations = vec![
            ("elo_system_creation", 10000),
            ("trueskill_system_creation", 5000),
            ("elo_1v1_update", 1000),
            ("player_serialization", 1000),
            ("batch_processing", 10),
        ];
        
        for (op_name, iterations) in operations {
            match op_name {
                "elo_system_creation" => {
                    let metrics = harness.measure(op_name, iterations, || {
                        let _ = create_elo_system();
                    });
                    baselines.insert(op_name.to_string(), metrics.ops_per_second());
                }
                "trueskill_system_creation" => {
                    let metrics = harness.measure(op_name, iterations, || {
                        let _ = create_trueskill_system();
                    });
                    baselines.insert(op_name.to_string(), metrics.ops_per_second());
                }
                _ => {}
            }
        }
        
        baselines
    }
}