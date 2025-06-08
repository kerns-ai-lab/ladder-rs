//! Test utilities and infrastructure for WASM testing
//!
//! This module provides comprehensive testing utilities for the ladder-rs WASM module,
//! including fixtures, performance timing, mock data generation, and logging facilities.

use crate::{PlayerManager, WasmRating, WasmRatingSystem, WasmTeam};
use js_sys::{Array, Date, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

/// Test fixture for setting up common test scenarios
#[wasm_bindgen]
pub struct TestFixture {
    player_manager: PlayerManager,
    rating_system: Option<WasmRatingSystem>,
    match_history: Vec<String>,
}

#[wasm_bindgen]
impl TestFixture {
    /// Create a new test fixture
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            player_manager: PlayerManager::new(),
            rating_system: None,
            match_history: Vec::new(),
        }
    }

    /// Set up a rating system for the fixture
    pub fn setup_rating_system(&mut self, system_type: &str) -> Result<(), JsValue> {
        self.rating_system = Some(WasmRatingSystem::new(system_type)?);
        Ok(())
    }

    /// Add a test player with default rating
    pub fn add_test_player(&mut self, player_id: &str) -> Result<JsValue, JsValue> {
        self.player_manager
            .register_player(player_id, Some(&format!("Test {}", player_id)), None)?;

        if let Some(ref mut system) = self.rating_system {
            system.create_player(player_id)?;
        }

        Ok(JsValue::from_str(player_id))
    }

    /// Add multiple test players
    pub fn add_test_players(&mut self, count: u32) -> Result<Array, JsValue> {
        let players = Array::new();
        for i in 0..count {
            let player_id = format!("player_{}", i);
            self.add_test_player(&player_id)?;
            players.push(&JsValue::from_str(&player_id));
        }
        Ok(players)
    }

    /// Simulate a match between two players
    pub fn simulate_match(
        &mut self,
        player1_id: &str,
        player2_id: &str,
        winner: u32,
    ) -> Result<JsValue, JsValue> {
        // Record in player manager
        let match_id = self.player_manager.add_match_record(
            vec![player1_id.to_string()].into_boxed_slice(),
            vec![player2_id.to_string()].into_boxed_slice(),
            winner as i32,
            None,
        )?;

        self.match_history.push(match_id.clone());

        // Update ratings if system is set up
        if let Some(ref mut system) = self.rating_system {
            let team1 = WasmTeam::new(vec![player1_id.to_string()].into_boxed_slice());
            let team2 = WasmTeam::new(vec![player2_id.to_string()].into_boxed_slice());
            system.update_ratings(team1, team2, winner)?;
        }

        Ok(JsValue::from_str(&match_id))
    }

    /// Get player count
    pub fn player_count(&self) -> u32 {
        self.player_manager.get_all_players().length()
    }

    /// Get match count
    pub fn match_count(&self) -> usize {
        self.match_history.len()
    }

    /// Get player manager reference
    pub fn get_player_manager(&self) -> PlayerManager {
        self.player_manager.clone()
    }

    /// Get rating system reference
    pub fn get_rating_system(&self) -> Option<WasmRatingSystem> {
        self.rating_system.clone()
    }

    /// Reset fixture to initial state
    pub fn reset(&mut self) {
        self.player_manager = PlayerManager::new();
        self.rating_system = None;
        self.match_history.clear();
    }
}

/// Performance timer for benchmarking
#[wasm_bindgen]
pub struct PerformanceTimer {
    start_time: f64,
    laps: Vec<(String, f64)>,
}

#[wasm_bindgen]
impl PerformanceTimer {
    /// Create a new performance timer
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            start_time: Date::now(),
            laps: Vec::new(),
        }
    }

    /// Record a lap time with label
    pub fn lap(&mut self, label: &str) -> f64 {
        let current_time = Date::now();
        let elapsed = current_time - self.start_time;
        self.laps.push((label.to_string(), elapsed));
        elapsed
    }

    /// Get total elapsed time
    pub fn elapsed(&self) -> f64 {
        Date::now() - self.start_time
    }

    /// Get all lap times as JavaScript object
    pub fn get_laps(&self) -> Result<Object, JsValue> {
        let obj = Object::new();
        for (label, time) in &self.laps {
            Reflect::set(&obj, &JsValue::from_str(label), &JsValue::from_f64(*time))?;
        }
        Ok(obj)
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = Date::now();
        self.laps.clear();
    }
}

/// Mock data generator for testing
#[wasm_bindgen]
pub struct MockDataGenerator {
    seed: u32,
}

#[wasm_bindgen]
impl MockDataGenerator {
    /// Create a new mock data generator
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// Generate a random player ID
    pub fn generate_player_id(&mut self) -> String {
        self.seed = self.seed.wrapping_add(1);
        format!("player_{:08x}", self.seed)
    }

    /// Generate a random player name
    pub fn generate_player_name(&mut self) -> String {
        let first_names = vec!["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"];
        let last_names = vec!["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"];

        self.seed = self.seed.wrapping_add(1);
        let first_idx = (self.seed % first_names.len() as u32) as usize;
        let last_idx = ((self.seed >> 8) % last_names.len() as u32) as usize;

        format!("{} {}", first_names[first_idx], last_names[last_idx])
    }

    /// Generate a random email
    pub fn generate_email(&mut self) -> String {
        let name = self.generate_player_name().to_lowercase().replace(' ', ".");
        format!("{}@example.com", name)
    }

    /// Generate a random match outcome (0=draw, 1=team1 wins, 2=team2 wins)
    pub fn generate_match_outcome(&mut self) -> u32 {
        self.seed = self.seed.wrapping_add(1);
        if self.seed % 10 == 0 {
            0 // 10% draws
        } else if self.seed % 2 == 0 {
            1 // 45% team1 wins
        } else {
            2 // 45% team2 wins
        }
    }

    /// Generate a batch of test players
    pub fn generate_players(&mut self, count: u32) -> Array {
        let players = Array::new();
        for _ in 0..count {
            let player = Object::new();
            Reflect::set(
                &player,
                &JsValue::from_str("id"),
                &JsValue::from_str(&self.generate_player_id()),
            )
            .unwrap();
            Reflect::set(
                &player,
                &JsValue::from_str("name"),
                &JsValue::from_str(&self.generate_player_name()),
            )
            .unwrap();
            Reflect::set(
                &player,
                &JsValue::from_str("email"),
                &JsValue::from_str(&self.generate_email()),
            )
            .unwrap();
            players.push(&player);
        }
        players
    }
}

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
    pub fn get_logs(&self) -> Array {
        let logs = Array::new();
        for (level, message) in &self.logs {
            let entry = Object::new();
            Reflect::set(
                &entry,
                &JsValue::from_str("level"),
                &JsValue::from_str(level),
            )
            .unwrap();
            Reflect::set(
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

/// Assertion helper for WASM tests
#[wasm_bindgen]
pub struct AssertionHelper;

#[wasm_bindgen]
impl AssertionHelper {
    /// Assert that two values are equal
    pub fn assert_equals(actual: &JsValue, expected: &JsValue, message: &str) -> Result<(), JsValue> {
        if actual != expected {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected {:?}, got {:?}",
                message, expected, actual
            )));
        }
        Ok(())
    }

    /// Assert that a value is truthy
    pub fn assert_truthy(value: &JsValue, message: &str) -> Result<(), JsValue> {
        if !value.is_truthy() {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected truthy value, got {:?}",
                message, value
            )));
        }
        Ok(())
    }

    /// Assert that a value is falsy
    pub fn assert_falsy(value: &JsValue, message: &str) -> Result<(), JsValue> {
        if value.is_truthy() {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected falsy value, got {:?}",
                message, value
            )));
        }
        Ok(())
    }

    /// Assert that an array contains a value
    pub fn assert_contains(array: &Array, value: &JsValue, message: &str) -> Result<(), JsValue> {
        let length = array.length();
        for i in 0..length {
            if array.get(i) == *value {
                return Ok(());
            }
        }
        Err(JsValue::from_str(&format!(
            "Assertion failed: {}. Array does not contain {:?}",
            message, value
        )))
    }

    /// Assert that a number is within a range
    pub fn assert_in_range(value: f64, min: f64, max: f64, message: &str) -> Result<(), JsValue> {
        if value < min || value > max {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Value {} is not in range [{}, {}]",
                message, value, min, max
            )));
        }
        Ok(())
    }
}

/// Test data snapshot for comparison
#[wasm_bindgen]
pub struct TestSnapshot {
    data: String,
    timestamp: f64,
}

#[wasm_bindgen]
impl TestSnapshot {
    /// Create a new snapshot
    #[wasm_bindgen(constructor)]
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
            timestamp: Date::now(),
        }
    }

    /// Compare with another snapshot
    pub fn equals(&self, other: &TestSnapshot) -> bool {
        self.data == other.data
    }

    /// Get snapshot data
    pub fn get_data(&self) -> String {
        self.data.clone()
    }

    /// Get snapshot timestamp
    pub fn get_timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Create a diff with another snapshot
    pub fn diff(&self, other: &TestSnapshot) -> String {
        if self.data == other.data {
            "No differences".to_string()
        } else {
            format!(
                "Snapshots differ:\nOld ({}): {}\nNew ({}): {}",
                self.timestamp, self.data, other.timestamp, other.data
            )
        }
    }
}

/// Browser environment detector
#[wasm_bindgen]
pub struct BrowserEnvironment;

#[wasm_bindgen]
impl BrowserEnvironment {
    /// Check if running in a browser
    pub fn is_browser() -> bool {
        web_sys::window().is_some()
    }

    /// Check if running in Node.js
    pub fn is_node() -> bool {
        !Self::is_browser()
    }

    /// Get user agent string
    pub fn get_user_agent() -> Option<String> {
        web_sys::window()?
            .navigator()
            .user_agent()
            .ok()
    }

    /// Check if localStorage is available
    pub fn has_local_storage() -> bool {
        if let Some(window) = web_sys::window() {
            window.local_storage().ok().flatten().is_some()
        } else {
            false
        }
    }

    /// Check if WebWorkers are available
    pub fn has_web_workers() -> bool {
        if let Some(window) = web_sys::window() {
            let worker = js_sys::Reflect::get(&window, &JsValue::from_str("Worker")).ok();
            worker.is_some() && !worker.unwrap().is_undefined()
        } else {
            false
        }
    }
}

// Re-export for convenience
pub use self::{
    AssertionHelper, BrowserEnvironment, MockDataGenerator, PerformanceTimer, TestFixture,
    TestLogger, TestSnapshot,
};