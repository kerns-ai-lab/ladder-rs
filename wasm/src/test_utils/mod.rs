//! Test utilities and infrastructure for WASM testing

// Sub-modules
pub mod assertions;
pub mod coverage;
pub mod data;
pub mod factories;
pub mod fixtures;
pub mod integration;
pub mod mocks;
pub mod performance;

// Re-export commonly used items
pub use assertions::{assert_ratings_approximately_equal, AssertionHelper};
pub use coverage::{CoverageTracker, CoverageReport, BranchCoverageTracker};
pub use data::{generate_player_pool, generate_match_history, SkillDistribution, TestPlayer, TestDatasetBuilder};
pub use factories::{
    create_test_elo_rating, create_test_glicko_rating, create_test_trueskill_rating,
    create_test_elo_system, create_test_trueskill_system,
    TestMatchFactory, TestConfigFactory, TestScenarioFactory
};
pub use fixtures::{TestFixture, TestSnapshot, FixtureBuilder};
pub use integration::{IntegrationTestHelper, BrowserEnvironment};
pub use mocks::{MockRatingSystem, MockStorage, MockRandom, MockMatchGenerator};
pub use performance::{PerformanceTimer, measure_performance, PerformanceResult, BenchmarkRunner, MemoryTracker};

// Common test utilities
use wasm_bindgen::prelude::*;
use web_sys::console;

/// Test logger for capturing and verifying log output
#[wasm_bindgen]
pub struct TestLogger {
    logs: Vec<(String, String)>, // (level, message)
    enabled: bool,
}

#[wasm_bindgen]
impl TestLogger {
    /// Create a new test logger
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            enabled: true,
        }
    }

    /// Log a debug message
    pub fn debug(&mut self, message: &str) {
        if self.enabled {
            self.logs.push(("debug".to_string(), message.to_string()));
            console::debug_1(&JsValue::from_str(message));
        }
    }

    /// Log an info message
    pub fn info(&mut self, message: &str) {
        if self.enabled {
            self.logs.push(("info".to_string(), message.to_string()));
            console::info_1(&JsValue::from_str(message));
        }
    }

    /// Log a warning message
    pub fn warn(&mut self, message: &str) {
        if self.enabled {
            self.logs.push(("warn".to_string(), message.to_string()));
            console::warn_1(&JsValue::from_str(message));
        }
    }

    /// Log an error message
    pub fn error(&mut self, message: &str) {
        if self.enabled {
            self.logs.push(("error".to_string(), message.to_string()));
            console::error_1(&JsValue::from_str(message));
        }
    }

    /// Get all logged messages
    pub fn get_logs(&self) -> js_sys::Array {
        let logs = js_sys::Array::new();
        for (level, message) in &self.logs {
            let entry = js_sys::Object::new();
            js_sys::Reflect::set(
                &entry,
                &JsValue::from_str("level"),
                &JsValue::from_str(level),
            )
            .unwrap();
            js_sys::Reflect::set(
                &entry,
                &JsValue::from_str("message"),
                &JsValue::from_str(message),
            )
            .unwrap();
            logs.push(&entry);
        }
        logs
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.logs.clear();
    }

    /// Enable or disable logging
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if a message was logged
    pub fn contains(&self, message: &str) -> bool {
        self.logs.iter().any(|(_, msg)| msg.contains(message))
    }

    /// Get log count by level
    pub fn count_by_level(&self, level: &str) -> usize {
        self.logs.iter().filter(|(lvl, _)| lvl == level).count()
    }
}

/// Common test configuration
pub struct TestConfig {
    pub enable_coverage: bool,
    pub enable_performance_tracking: bool,
    pub verbose_logging: bool,
    pub random_seed: Option<u32>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            enable_coverage: true,
            enable_performance_tracking: true,
            verbose_logging: false,
            random_seed: None,
        }
    }
}

/// Test execution context
pub struct TestContext {
    pub config: TestConfig,
    pub logger: TestLogger,
    pub coverage: Option<CoverageTracker>,
    pub performance: Option<PerformanceTimer>,
}

impl TestContext {
    /// Create a new test context
    pub fn new(config: TestConfig) -> Self {
        let logger = TestLogger::new();
        let coverage = if config.enable_coverage {
            Some(CoverageTracker::new())
        } else {
            None
        };
        let performance = if config.enable_performance_tracking {
            Some(PerformanceTimer::new())
        } else {
            None
        };

        Self {
            config,
            logger,
            coverage,
            performance,
        }
    }

    /// Run a test with the context
    pub fn run_test<F, T>(&mut self, test_name: &str, test_fn: F) -> Result<T, JsValue>
    where
        F: FnOnce(&mut Self) -> Result<T, JsValue>,
    {
        self.logger.info(&format!("Starting test: {}", test_name));
        
        if let Some(ref mut perf) = self.performance {
            perf.lap(&format!("{}_start", test_name));
        }

        let result = test_fn(self);

        if let Some(ref mut perf) = self.performance {
            perf.lap(&format!("{}_end", test_name));
        }

        match &result {
            Ok(_) => self.logger.info(&format!("Test passed: {}", test_name)),
            Err(e) => self.logger.error(&format!("Test failed: {} - {:?}", test_name, e)),
        }

        result
    }
}