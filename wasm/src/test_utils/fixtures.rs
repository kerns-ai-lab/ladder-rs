//! Test fixtures for setting up common test scenarios

use crate::{EloSystem, TrueSkillSystem};
use js_sys::{Array, Date};
use wasm_bindgen::prelude::*;

/// Test fixture for setting up common test scenarios
#[wasm_bindgen]
pub struct TestFixture {
    players: Vec<String>,
    match_history: Vec<MatchRecord>,
}

#[derive(Clone)]
struct MatchRecord {
    player1: String,
    player2: String,
    outcome: u32,
    timestamp: f64,
}

#[wasm_bindgen]
impl TestFixture {
    /// Create a new test fixture
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            match_history: Vec::new(),
        }
    }

    /// Add a test player
    pub fn add_player(&mut self, player_id: &str) -> Result<(), JsValue> {
        if self.players.contains(&player_id.to_string()) {
            return Err(JsValue::from_str("Player already exists"));
        }
        self.players.push(player_id.to_string());
        Ok(())
    }

    /// Add multiple test players
    pub fn add_players(&mut self, count: u32) -> Result<Array, JsValue> {
        let players = Array::new();
        for i in 0..count {
            let player_id = format!("player_{}", i);
            self.add_player(&player_id)?;
            players.push(&JsValue::from_str(&player_id));
        }
        Ok(players)
    }

    /// Record a match result
    pub fn record_match(
        &mut self,
        player1_id: &str,
        player2_id: &str,
        outcome: u32,
    ) -> Result<(), JsValue> {
        // Validate players exist
        if !self.players.contains(&player1_id.to_string()) {
            return Err(JsValue::from_str("Player 1 not found"));
        }
        if !self.players.contains(&player2_id.to_string()) {
            return Err(JsValue::from_str("Player 2 not found"));
        }

        self.match_history.push(MatchRecord {
            player1: player1_id.to_string(),
            player2: player2_id.to_string(),
            outcome,
            timestamp: Date::now(),
        });

        Ok(())
    }

    /// Get player count
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Get match count
    pub fn match_count(&self) -> usize {
        self.match_history.len()
    }

    /// Get all players
    pub fn get_players(&self) -> Array {
        let arr = Array::new();
        for player in &self.players {
            arr.push(&JsValue::from_str(player));
        }
        arr
    }

    /// Get match history
    pub fn get_match_history(&self) -> Array {
        let arr = Array::new();
        for record in &self.match_history {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("player1"),
                &JsValue::from_str(&record.player1),
            ).unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("player2"),
                &JsValue::from_str(&record.player2),
            ).unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("outcome"),
                &JsValue::from_f64(record.outcome as f64),
            ).unwrap();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("timestamp"),
                &JsValue::from_f64(record.timestamp),
            ).unwrap();
            arr.push(&obj);
        }
        arr
    }

    /// Apply match history to an Elo system
    pub fn apply_to_elo_system(&self, system: &EloSystem) -> Result<js_sys::Object, JsValue> {
        let ratings = js_sys::Object::new();
        
        // Initialize all players
        for player in &self.players {
            let rating = system.create_rating();
            js_sys::Reflect::set(
                &ratings,
                &JsValue::from_str(player),
                &JsValue::from_f64(rating.value()),
            )?;
        }

        // Apply matches
        for record in &self.match_history {
            let p1_rating = js_sys::Reflect::get(&ratings, &JsValue::from_str(&record.player1))?
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid rating"))?;
            let p2_rating = js_sys::Reflect::get(&ratings, &JsValue::from_str(&record.player2))?
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Invalid rating"))?;

            let result = system.process_1v1_json(p1_rating, p2_rating, record.outcome)?;
            let result_obj: js_sys::Object = js_sys::JSON::parse(&result)?
                .dyn_into()
                .map_err(|_| JsValue::from_str("Invalid result format"))?;

            // Update ratings
            let new_p1 = js_sys::Reflect::get(&result_obj, &JsValue::from_str("player1"))?;
            let new_p2 = js_sys::Reflect::get(&result_obj, &JsValue::from_str("player2"))?;
            
            js_sys::Reflect::set(&ratings, &JsValue::from_str(&record.player1), &new_p1)?;
            js_sys::Reflect::set(&ratings, &JsValue::from_str(&record.player2), &new_p2)?;
        }

        Ok(ratings)
    }

    /// Reset fixture to initial state
    pub fn reset(&mut self) {
        self.players.clear();
        self.match_history.clear();
    }
}

/// Test data snapshot for comparison
#[wasm_bindgen]
pub struct TestSnapshot {
    data: String,
    timestamp: f64,
    metadata: js_sys::Object,
}

#[wasm_bindgen]
impl TestSnapshot {
    /// Create a new snapshot
    #[wasm_bindgen(constructor)]
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
            timestamp: Date::now(),
            metadata: js_sys::Object::new(),
        }
    }

    /// Create a snapshot with metadata
    pub fn with_metadata(data: &str, metadata: js_sys::Object) -> Self {
        Self {
            data: data.to_string(),
            timestamp: Date::now(),
            metadata,
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

    /// Get metadata
    pub fn get_metadata(&self) -> js_sys::Object {
        self.metadata.clone()
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

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, JsValue> {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("data"),
            &JsValue::from_str(&self.data),
        )?;
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("timestamp"),
            &JsValue::from_f64(self.timestamp),
        )?;
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("metadata"),
            &self.metadata,
        )?;
        
        js_sys::JSON::stringify(&obj)
            .map(|s| s.as_string().unwrap_or_default())
    }

    /// Import from JSON
    pub fn from_json(json: &str) -> Result<TestSnapshot, JsValue> {
        let obj: js_sys::Object = js_sys::JSON::parse(json)?
            .dyn_into()
            .map_err(|_| JsValue::from_str("Invalid JSON format"))?;

        let data = js_sys::Reflect::get(&obj, &JsValue::from_str("data"))?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Missing data field"))?;
        
        let timestamp = js_sys::Reflect::get(&obj, &JsValue::from_str("timestamp"))?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Missing timestamp field"))?;

        let metadata = js_sys::Reflect::get(&obj, &JsValue::from_str("metadata"))?
            .dyn_into()
            .unwrap_or_else(|_| js_sys::Object::new());

        Ok(TestSnapshot {
            data,
            timestamp,
            metadata,
        })
    }
}

/// Fixture builder for complex test scenarios
pub struct FixtureBuilder {
    fixture: TestFixture,
    default_outcome_distribution: Vec<(u32, f32)>, // (outcome, probability)
}

impl FixtureBuilder {
    /// Create a new fixture builder
    pub fn new() -> Self {
        Self {
            fixture: TestFixture::new(),
            default_outcome_distribution: vec![
                (1, 0.45), // Player 1 wins 45%
                (2, 0.45), // Player 2 wins 45%
                (0, 0.10), // Draw 10%
            ],
        }
    }

    /// Add players to the fixture
    pub fn with_players(mut self, count: u32) -> Result<Self, JsValue> {
        self.fixture.add_players(count)?;
        Ok(self)
    }

    /// Generate random matches between players
    pub fn with_random_matches(mut self, count: u32, seed: u32) -> Result<Self, JsValue> {
        let players = self.fixture.get_players();
        let player_count = players.length();
        
        if player_count < 2 {
            return Err(JsValue::from_str("Need at least 2 players for matches"));
        }

        let mut rng_state = seed;
        for _ in 0..count {
            // Simple LCG for deterministic randomness
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let p1_idx = (rng_state % player_count) as u32;
            
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let mut p2_idx = (rng_state % player_count) as u32;
            
            // Ensure different players
            if p1_idx == p2_idx {
                p2_idx = (p2_idx + 1) % player_count;
            }

            let player1 = players.get(p1_idx).as_string().unwrap();
            let player2 = players.get(p2_idx).as_string().unwrap();

            // Determine outcome based on distribution
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let outcome_roll = (rng_state % 100) as f32 / 100.0;
            
            let mut cumulative_prob = 0.0;
            let mut outcome = 1;
            for (o, prob) in &self.default_outcome_distribution {
                cumulative_prob += prob;
                if outcome_roll < cumulative_prob {
                    outcome = *o;
                    break;
                }
            }

            self.fixture.record_match(&player1, &player2, outcome)?;
        }

        Ok(self)
    }

    /// Set custom outcome distribution
    pub fn with_outcome_distribution(mut self, distribution: Vec<(u32, f32)>) -> Self {
        self.default_outcome_distribution = distribution;
        self
    }

    /// Build the fixture
    pub fn build(self) -> TestFixture {
        self.fixture
    }
}