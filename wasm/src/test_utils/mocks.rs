//! Mock implementations for testing

use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Mock rating system for testing
#[wasm_bindgen]
pub struct MockRatingSystem {
    k_factor: f64,
    win_probability: Option<f64>,
    call_count: RefCell<HashMap<String, usize>>,
}

#[wasm_bindgen]
impl MockRatingSystem {
    /// Create a new mock rating system
    #[wasm_bindgen(constructor)]
    pub fn new(k_factor: f64) -> Self {
        Self {
            k_factor,
            win_probability: None,
            call_count: RefCell::new(HashMap::new()),
        }
    }

    /// Set a fixed win probability
    pub fn set_win_probability(&mut self, probability: f64) {
        self.win_probability = Some(probability);
    }

    /// Process a match (mock implementation)
    pub fn process_match(&self, rating1: f64, rating2: f64, outcome: u32) -> js_sys::Object {
        self.increment_call_count("process_match");
        
        let result = js_sys::Object::new();
        
        // Simple mock calculation
        let change = match outcome {
            1 => self.k_factor * 0.5,  // Player 1 wins
            2 => -self.k_factor * 0.5, // Player 2 wins
            _ => 0.0,                   // Draw
        };
        
        js_sys::Reflect::set(
            &result,
            &JsValue::from_str("player1"),
            &JsValue::from_f64(rating1 + change),
        ).unwrap();
        js_sys::Reflect::set(
            &result,
            &JsValue::from_str("player2"),
            &JsValue::from_f64(rating2 - change),
        ).unwrap();
        
        result
    }

    /// Get win probability (mock implementation)
    pub fn get_win_probability(&self, _rating1: f64, _rating2: f64) -> f64 {
        self.increment_call_count("get_win_probability");
        self.win_probability.unwrap_or(0.5)
    }

    /// Get call count for a method
    pub fn get_call_count(&self, method_name: &str) -> u32 {
        self.call_count.borrow()
            .get(method_name)
            .copied()
            .unwrap_or(0) as u32
    }

    /// Reset call counts
    pub fn reset_call_counts(&self) {
        self.call_count.borrow_mut().clear();
    }

    /// Get all call counts
    pub fn get_all_call_counts(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();
        for (method, count) in self.call_count.borrow().iter() {
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str(method),
                &JsValue::from_f64(*count as f64),
            ).unwrap();
        }
        obj
    }

    fn increment_call_count(&self, method: &str) {
        let mut counts = self.call_count.borrow_mut();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
}

/// Mock storage for testing persistence
#[wasm_bindgen]
pub struct MockStorage {
    data: RefCell<HashMap<String, String>>,
    fail_on_write: RefCell<bool>,
    fail_on_read: RefCell<bool>,
}

#[wasm_bindgen]
impl MockStorage {
    /// Create a new mock storage
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            data: RefCell::new(HashMap::new()),
            fail_on_write: RefCell::new(false),
            fail_on_read: RefCell::new(false),
        }
    }

    /// Set a value in storage
    pub fn set_item(&self, key: &str, value: &str) -> Result<(), JsValue> {
        if *self.fail_on_write.borrow() {
            return Err(JsValue::from_str("Mock storage write failure"));
        }
        self.data.borrow_mut().insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Get a value from storage
    pub fn get_item(&self, key: &str) -> Result<Option<String>, JsValue> {
        if *self.fail_on_read.borrow() {
            return Err(JsValue::from_str("Mock storage read failure"));
        }
        Ok(self.data.borrow().get(key).cloned())
    }

    /// Remove a value from storage
    pub fn remove_item(&self, key: &str) -> Result<(), JsValue> {
        self.data.borrow_mut().remove(key);
        Ok(())
    }

    /// Clear all storage
    pub fn clear(&self) -> Result<(), JsValue> {
        self.data.borrow_mut().clear();
        Ok(())
    }

    /// Get number of items in storage
    pub fn length(&self) -> u32 {
        self.data.borrow().len() as u32
    }

    /// Get all keys
    pub fn keys(&self) -> js_sys::Array {
        let arr = js_sys::Array::new();
        for key in self.data.borrow().keys() {
            arr.push(&JsValue::from_str(key));
        }
        arr
    }

    /// Configure to fail on write
    pub fn set_fail_on_write(&self, fail: bool) {
        *self.fail_on_write.borrow_mut() = fail;
    }

    /// Configure to fail on read
    pub fn set_fail_on_read(&self, fail: bool) {
        *self.fail_on_read.borrow_mut() = fail;
    }

    /// Get internal data as object
    pub fn get_all_data(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();
        for (key, value) in self.data.borrow().iter() {
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str(key),
                &JsValue::from_str(value),
            ).unwrap();
        }
        obj
    }
}

/// Mock random number generator for deterministic testing
#[wasm_bindgen]
pub struct MockRandom {
    seed: RefCell<u32>,
    values: RefCell<Vec<f64>>,
    use_fixed_values: RefCell<bool>,
    current_index: RefCell<usize>,
}

#[wasm_bindgen]
impl MockRandom {
    /// Create a new mock random generator
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            seed: RefCell::new(seed),
            values: RefCell::new(Vec::new()),
            use_fixed_values: RefCell::new(false),
            current_index: RefCell::new(0),
        }
    }

    /// Set fixed values to return
    pub fn set_fixed_values(&self, values: js_sys::Array) {
        let mut vec = Vec::new();
        for i in 0..values.length() {
            if let Some(val) = values.get(i).as_f64() {
                vec.push(val);
            }
        }
        *self.values.borrow_mut() = vec;
        *self.use_fixed_values.borrow_mut() = true;
        *self.current_index.borrow_mut() = 0;
    }

    /// Get next random value
    pub fn next(&self) -> f64 {
        if *self.use_fixed_values.borrow() {
            let values = self.values.borrow();
            if !values.is_empty() {
                let mut index = self.current_index.borrow_mut();
                let value = values[*index % values.len()];
                *index += 1;
                return value;
            }
        }

        // Simple LCG for deterministic randomness
        let mut seed = self.seed.borrow_mut();
        *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (*seed as f64) / (u32::MAX as f64)
    }

    /// Get random integer in range [min, max)
    pub fn next_int(&self, min: i32, max: i32) -> i32 {
        let range = (max - min) as f64;
        min + (self.next() * range) as i32
    }

    /// Get random boolean
    pub fn next_bool(&self) -> bool {
        self.next() < 0.5
    }

    /// Reset to initial seed
    pub fn reset(&self, seed: u32) {
        *self.seed.borrow_mut() = seed;
        *self.current_index.borrow_mut() = 0;
        *self.use_fixed_values.borrow_mut() = false;
    }

    /// Get current seed
    pub fn get_seed(&self) -> u32 {
        *self.seed.borrow()
    }
}

/// Mock match generator for testing
#[wasm_bindgen]
pub struct MockMatchGenerator {
    players: Vec<String>,
    outcome_distribution: Vec<f64>, // [p1_win, p2_win, draw]
    random: MockRandom,
}

#[wasm_bindgen]
impl MockMatchGenerator {
    /// Create a new match generator
    #[wasm_bindgen(constructor)]
    pub fn new(player_count: u32, seed: u32) -> Self {
        let players: Vec<String> = (0..player_count)
            .map(|i| format!("player_{}", i))
            .collect();
        
        Self {
            players,
            outcome_distribution: vec![0.45, 0.45, 0.10], // Default: 45% p1 win, 45% p2 win, 10% draw
            random: MockRandom::new(seed),
        }
    }

    /// Set custom outcome distribution
    pub fn set_outcome_distribution(&mut self, p1_win: f64, p2_win: f64, draw: f64) {
        self.outcome_distribution = vec![p1_win, p2_win, draw];
    }

    /// Generate a random match
    pub fn generate_match(&self) -> js_sys::Object {
        let p1_idx = self.random.next_int(0, self.players.len() as i32) as usize;
        let mut p2_idx = self.random.next_int(0, self.players.len() as i32) as usize;
        
        // Ensure different players
        while p2_idx == p1_idx && self.players.len() > 1 {
            p2_idx = self.random.next_int(0, self.players.len() as i32) as usize;
        }

        // Determine outcome
        let roll = self.random.next();
        let outcome = if roll < self.outcome_distribution[0] {
            1 // Player 1 wins
        } else if roll < self.outcome_distribution[0] + self.outcome_distribution[1] {
            2 // Player 2 wins
        } else {
            0 // Draw
        };

        let match_obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &match_obj,
            &JsValue::from_str("player1"),
            &JsValue::from_str(&self.players[p1_idx]),
        ).unwrap();
        js_sys::Reflect::set(
            &match_obj,
            &JsValue::from_str("player2"),
            &JsValue::from_str(&self.players[p2_idx]),
        ).unwrap();
        js_sys::Reflect::set(
            &match_obj,
            &JsValue::from_str("outcome"),
            &JsValue::from_f64(outcome as f64),
        ).unwrap();

        match_obj
    }

    /// Generate multiple matches
    pub fn generate_matches(&self, count: u32) -> js_sys::Array {
        let matches = js_sys::Array::new();
        for _ in 0..count {
            matches.push(&self.generate_match());
        }
        matches
    }

    /// Generate a tournament (all players play each other)
    pub fn generate_tournament(&self) -> js_sys::Array {
        let matches = js_sys::Array::new();
        
        for i in 0..self.players.len() {
            for j in (i + 1)..self.players.len() {
                // Determine outcome
                let roll = self.random.next();
                let outcome = if roll < self.outcome_distribution[0] {
                    1 // Player 1 wins
                } else if roll < self.outcome_distribution[0] + self.outcome_distribution[1] {
                    2 // Player 2 wins
                } else {
                    0 // Draw
                };

                let match_obj = js_sys::Object::new();
                js_sys::Reflect::set(
                    &match_obj,
                    &JsValue::from_str("player1"),
                    &JsValue::from_str(&self.players[i]),
                ).unwrap();
                js_sys::Reflect::set(
                    &match_obj,
                    &JsValue::from_str("player2"),
                    &JsValue::from_str(&self.players[j]),
                ).unwrap();
                js_sys::Reflect::set(
                    &match_obj,
                    &JsValue::from_str("outcome"),
                    &JsValue::from_f64(outcome as f64),
                ).unwrap();

                matches.push(&match_obj);
            }
        }
        
        matches
    }
}