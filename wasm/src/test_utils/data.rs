//! Test data generation utilities

use wasm_bindgen::prelude::*;
use std::collections::HashMap;

/// Skill distribution for generating test players
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum SkillDistribution {
    /// All players have the same skill
    Uniform,
    /// Normal distribution of skills
    Normal,
    /// Skewed distribution (few high skilled players)
    Skewed,
    /// Bimodal distribution (two skill clusters)
    Bimodal,
}

/// Test player data
#[wasm_bindgen]
#[derive(Clone)]
pub struct TestPlayer {
    id: String,
    name: String,
    true_skill: f64,
    current_rating: f64,
}

#[wasm_bindgen]
impl TestPlayer {
    /// Get player ID
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Get player name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get true skill (for simulation)
    pub fn true_skill(&self) -> f64 {
        self.true_skill
    }

    /// Get current rating
    pub fn current_rating(&self) -> f64 {
        self.current_rating
    }

    /// Set current rating
    pub fn set_current_rating(&mut self, rating: f64) {
        self.current_rating = rating;
    }

    /// Convert to JavaScript object
    pub fn to_object(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("id"),
            &JsValue::from_str(&self.id),
        ).unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("name"),
            &JsValue::from_str(&self.name),
        ).unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("true_skill"),
            &JsValue::from_f64(self.true_skill),
        ).unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("current_rating"),
            &JsValue::from_f64(self.current_rating),
        ).unwrap();
        obj
    }
}

/// Generate a pool of test players
pub fn generate_player_pool(count: usize, distribution: SkillDistribution) -> Vec<TestPlayer> {
    let mut players = Vec::new();
    
    for i in 0..count {
        let true_skill = match distribution {
            SkillDistribution::Uniform => 1500.0,
            SkillDistribution::Normal => {
                // Simple box-muller transform for normal distribution
                let u1 = ((i * 7919) % 1000) as f64 / 1000.0;
                let u2 = ((i * 7927) % 1000) as f64 / 1000.0;
                let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                1500.0 + z * 200.0 // mean=1500, std=200
            },
            SkillDistribution::Skewed => {
                // Exponential-like distribution
                let u = ((i * 7919) % 1000) as f64 / 1000.0;
                1200.0 + 600.0 * (1.0 - (-3.0 * u).exp())
            },
            SkillDistribution::Bimodal => {
                // Two peaks at 1300 and 1700
                if i % 2 == 0 {
                    1300.0 + ((i * 7919) % 100) as f64
                } else {
                    1700.0 + ((i * 7927) % 100) as f64
                }
            },
        };

        let player = TestPlayer {
            id: format!("player_{}", i),
            name: generate_player_name(i),
            true_skill: true_skill.max(0.0), // Ensure non-negative
            current_rating: 1500.0, // Start everyone at default
        };
        
        players.push(player);
    }
    
    players
}

/// Generate a realistic player name
fn generate_player_name(index: usize) -> String {
    let first_names = vec![
        "Alex", "Blake", "Casey", "Drew", "Ellis", "Finley",
        "Gray", "Harper", "Indigo", "Jordan", "Kai", "Logan",
        "Morgan", "Nova", "Oakley", "Parker", "Quinn", "River",
        "Sage", "Taylor", "Unity", "Vale", "Winter", "Xander",
        "Yael", "Zion"
    ];
    
    let last_names = vec![
        "Chen", "Smith", "Johnson", "Williams", "Brown", "Jones",
        "Garcia", "Miller", "Davis", "Rodriguez", "Martinez", "Anderson",
        "Wilson", "Moore", "Jackson", "Martin", "Lee", "Thompson",
        "White", "Harris", "Clark", "Lewis", "Robinson", "Walker",
        "Hall", "Young"
    ];
    
    let first_idx = index % first_names.len();
    let last_idx = (index / first_names.len()) % last_names.len();
    
    format!("{} {}", first_names[first_idx], last_names[last_idx])
}

/// Match history entry
#[derive(Clone)]
pub struct MatchHistoryEntry {
    pub player1_id: String,
    pub player2_id: String,
    pub outcome: u32, // 0=draw, 1=p1 win, 2=p2 win
    pub timestamp: f64,
}

/// Generate match history based on true skills
pub fn generate_match_history(
    players: &[TestPlayer],
    match_count: usize,
    seed: u32,
) -> Vec<MatchHistoryEntry> {
    let mut history = Vec::new();
    let mut rng_state = seed;
    
    for i in 0..match_count {
        // Simple LCG for deterministic randomness
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let p1_idx = (rng_state % players.len() as u32) as usize;
        
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let mut p2_idx = (rng_state % players.len() as u32) as usize;
        
        // Ensure different players
        if p1_idx == p2_idx {
            p2_idx = (p2_idx + 1) % players.len();
        }
        
        let player1 = &players[p1_idx];
        let player2 = &players[p2_idx];
        
        // Calculate win probability based on true skills
        let skill_diff = player1.true_skill - player2.true_skill;
        let win_prob = 1.0 / (1.0 + 10.0_f64.powf(-skill_diff / 400.0));
        
        // Determine outcome
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let roll = (rng_state % 1000) as f64 / 1000.0;
        
        let outcome = if roll < win_prob - 0.05 {
            1 // Player 1 wins (with 5% draw margin)
        } else if roll > win_prob + 0.05 {
            2 // Player 2 wins (with 5% draw margin)
        } else {
            0 // Draw
        };
        
        history.push(MatchHistoryEntry {
            player1_id: player1.id.clone(),
            player2_id: player2.id.clone(),
            outcome,
            timestamp: i as f64,
        });
    }
    
    history
}

/// Tournament configuration
#[wasm_bindgen]
pub struct TournamentConfig {
    format: TournamentFormat,
    rounds: u32,
    matches_per_round: u32,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum TournamentFormat {
    RoundRobin,
    Swiss,
    SingleElimination,
    DoubleElimination,
}

#[wasm_bindgen]
impl TournamentConfig {
    /// Create a new tournament configuration
    #[wasm_bindgen(constructor)]
    pub fn new(format: TournamentFormat, rounds: u32, matches_per_round: u32) -> Self {
        Self {
            format,
            rounds,
            matches_per_round,
        }
    }

    /// Generate tournament matches
    pub fn generate_matches(&self, players: js_sys::Array) -> js_sys::Array {
        let player_count = players.length();
        let matches = js_sys::Array::new();
        
        match self.format {
            TournamentFormat::RoundRobin => {
                // Everyone plays everyone
                for i in 0..player_count {
                    for j in (i + 1)..player_count {
                        let match_obj = js_sys::Object::new();
                        js_sys::Reflect::set(
                            &match_obj,
                            &JsValue::from_str("round"),
                            &JsValue::from_f64(1.0),
                        ).unwrap();
                        js_sys::Reflect::set(
                            &match_obj,
                            &JsValue::from_str("player1_idx"),
                            &JsValue::from_f64(i as f64),
                        ).unwrap();
                        js_sys::Reflect::set(
                            &match_obj,
                            &JsValue::from_str("player2_idx"),
                            &JsValue::from_f64(j as f64),
                        ).unwrap();
                        matches.push(&match_obj);
                    }
                }
            },
            TournamentFormat::Swiss => {
                // Simplified Swiss system
                for round in 0..self.rounds {
                    let mut paired = vec![false; player_count as usize];
                    
                    for i in 0..player_count {
                        if paired[i as usize] {
                            continue;
                        }
                        
                        // Find unpaired opponent
                        for j in (i + 1)..player_count {
                            if !paired[j as usize] {
                                paired[i as usize] = true;
                                paired[j as usize] = true;
                                
                                let match_obj = js_sys::Object::new();
                                js_sys::Reflect::set(
                                    &match_obj,
                                    &JsValue::from_str("round"),
                                    &JsValue::from_f64((round + 1) as f64),
                                ).unwrap();
                                js_sys::Reflect::set(
                                    &match_obj,
                                    &JsValue::from_str("player1_idx"),
                                    &JsValue::from_f64(i as f64),
                                ).unwrap();
                                js_sys::Reflect::set(
                                    &match_obj,
                                    &JsValue::from_str("player2_idx"),
                                    &JsValue::from_f64(j as f64),
                                ).unwrap();
                                matches.push(&match_obj);
                                break;
                            }
                        }
                    }
                }
            },
            _ => {
                // Not implemented for single/double elimination
            }
        }
        
        matches
    }
}

/// Create test dataset for specific scenarios
#[wasm_bindgen]
pub struct TestDatasetBuilder {
    players: Vec<TestPlayer>,
    matches: Vec<MatchHistoryEntry>,
}

#[wasm_bindgen]
impl TestDatasetBuilder {
    /// Create a new dataset builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            matches: Vec::new(),
        }
    }

    /// Add players with specified skill distribution
    pub fn add_players(&mut self, count: u32, distribution: SkillDistribution) -> &mut Self {
        let new_players = generate_player_pool(count as usize, distribution);
        self.players.extend(new_players);
        self
    }

    /// Add specific player
    pub fn add_player(&mut self, id: &str, name: &str, true_skill: f64) -> &mut Self {
        self.players.push(TestPlayer {
            id: id.to_string(),
            name: name.to_string(),
            true_skill,
            current_rating: 1500.0,
        });
        self
    }

    /// Add matches between all players
    pub fn add_round_robin_matches(&mut self) -> &mut Self {
        for i in 0..self.players.len() {
            for j in (i + 1)..self.players.len() {
                // Simulate based on true skill
                let skill_diff = self.players[i].true_skill - self.players[j].true_skill;
                let win_prob = 1.0 / (1.0 + 10.0_f64.powf(-skill_diff / 400.0));
                
                // Use deterministic outcome based on skill difference
                let outcome = if win_prob > 0.65 {
                    1
                } else if win_prob < 0.35 {
                    2
                } else {
                    0
                };
                
                self.matches.push(MatchHistoryEntry {
                    player1_id: self.players[i].id.clone(),
                    player2_id: self.players[j].id.clone(),
                    outcome,
                    timestamp: self.matches.len() as f64,
                });
            }
        }
        self
    }

    /// Add specific match
    pub fn add_match(&mut self, player1_id: &str, player2_id: &str, outcome: u32) -> &mut Self {
        self.matches.push(MatchHistoryEntry {
            player1_id: player1_id.to_string(),
            player2_id: player2_id.to_string(),
            outcome,
            timestamp: self.matches.len() as f64,
        });
        self
    }

    /// Build and return the dataset
    pub fn build(&self) -> js_sys::Object {
        let dataset = js_sys::Object::new();
        
        // Add players
        let players_arr = js_sys::Array::new();
        for player in &self.players {
            players_arr.push(&player.to_object());
        }
        js_sys::Reflect::set(
            &dataset,
            &JsValue::from_str("players"),
            &players_arr,
        ).unwrap();
        
        // Add matches
        let matches_arr = js_sys::Array::new();
        for match_entry in &self.matches {
            let match_obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &match_obj,
                &JsValue::from_str("player1_id"),
                &JsValue::from_str(&match_entry.player1_id),
            ).unwrap();
            js_sys::Reflect::set(
                &match_obj,
                &JsValue::from_str("player2_id"),
                &JsValue::from_str(&match_entry.player2_id),
            ).unwrap();
            js_sys::Reflect::set(
                &match_obj,
                &JsValue::from_str("outcome"),
                &JsValue::from_f64(match_entry.outcome as f64),
            ).unwrap();
            js_sys::Reflect::set(
                &match_obj,
                &JsValue::from_str("timestamp"),
                &JsValue::from_f64(match_entry.timestamp),
            ).unwrap();
            matches_arr.push(&match_obj);
        }
        js_sys::Reflect::set(
            &dataset,
            &JsValue::from_str("matches"),
            &matches_arr,
        ).unwrap();
        
        dataset
    }
}