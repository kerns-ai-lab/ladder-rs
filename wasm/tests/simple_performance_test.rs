//! Simple performance regression tests for WASM bindings (Elo-only)
//! 
//! This module provides basic performance benchmarking using only the
//! available WASM API to avoid missing function issues.

use wasm_bindgen_test::*;
use web_sys::{window, Performance};
use ladder_rs_wasm::{EloRating, EloSystem};

wasm_bindgen_test_configure!(run_in_browser);

/// Simple performance harness
struct SimpleHarness {
    performance: Performance,
}

impl SimpleHarness {
    fn new() -> Self {
        let performance = window()
            .expect("should have window")
            .performance()
            .expect("should have performance");
        
        Self { performance }
    }

    /// Measure the performance of a function
    fn measure<F>(&self, iterations: u32, mut f: F) -> f64
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
        
        (iterations as f64 * 1000.0) / duration // ops per second
    }
}

/// Test: Elo rating creation performance
#[wasm_bindgen_test]
fn test_elo_rating_creation() {
    let harness = SimpleHarness::new();
    
    let ops_per_sec = harness.measure(1000, || {
        let _ = EloRating::new(1500.0);
    });
    
    assert!(
        ops_per_sec > 10000.0,
        "Elo rating creation too slow: {} ops/sec",
        ops_per_sec
    );
    
    web_sys::console::log_1(&format!("Elo rating creation: {:.0} ops/sec", ops_per_sec).into());
}

/// Test: Elo system creation performance
#[wasm_bindgen_test]
fn test_elo_system_creation() {
    let harness = SimpleHarness::new();
    
    let ops_per_sec = harness.measure(1000, || {
        let _ = EloSystem::new();
    });
    
    assert!(
        ops_per_sec > 5000.0,
        "Elo system creation too slow: {} ops/sec",
        ops_per_sec
    );
    
    web_sys::console::log_1(&format!("Elo system creation: {:.0} ops/sec", ops_per_sec).into());
}

/// Test: Elo rating serialization performance
#[wasm_bindgen_test]
fn test_elo_serialization() {
    let harness = SimpleHarness::new();
    let rating = EloRating::new(1500.0);
    
    let ops_per_sec = harness.measure(1000, || {
        let _ = rating.serialize();
    });
    
    assert!(
        ops_per_sec > 1000.0,
        "Elo serialization too slow: {} ops/sec",
        ops_per_sec
    );
    
    web_sys::console::log_1(&format!("Elo serialization: {:.0} ops/sec", ops_per_sec).into());
}

/// Test: Basic performance regression check
#[wasm_bindgen_test]
fn test_performance_regression_check() {
    let harness = SimpleHarness::new();
    
    // Test object creation performance
    let rating_creation_ops = harness.measure(1000, || {
        let _ = EloRating::new(1500.0);
    });
    
    let system_creation_ops = harness.measure(500, || {
        let _ = EloSystem::new();
    });
    
    // Performance thresholds
    assert!(rating_creation_ops > 10000.0, "Rating creation regression");
    assert!(system_creation_ops > 5000.0, "System creation regression");
    
    web_sys::console::log_1(&"Performance regression tests passed!".into());
}