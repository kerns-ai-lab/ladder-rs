//! Factory methods for creating test objects

use crate::{EloRating, EloSystem, TrueSkillRating, TrueSkillSystem};
use wasm_bindgen::prelude::*;

/// Create a test Elo rating with the specified value
pub fn create_test_elo_rating(value: f64) -> EloRating {
    EloRating::new(value)
}

/// Create a test Elo system with custom parameters
pub fn create_test_elo_system(k_factor: Option<f64>, initial_rating: Option<f64>) -> EloSystem {
    match (k_factor, initial_rating) {
        (Some(k), Some(r)) => EloSystem::with_parameters(k, r),
        _ => EloSystem::new(),
    }
}

/// Create a test Glicko rating (placeholder for when Glicko is enabled)
pub fn create_test_glicko_rating(_rating: f64, _rd: f64) -> js_sys::Object {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("rating"),
        &JsValue::from_f64(_rating),
    ).unwrap();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("rd"),
        &JsValue::from_f64(_rd),
    ).unwrap();
    obj
}

/// Create a test TrueSkill rating with the specified mean and variance
pub fn create_test_trueskill_rating(mean: f64, variance: f64) -> TrueSkillRating {
    TrueSkillRating::new(mean, variance)
}

/// Create a test TrueSkill system with custom parameters
pub fn create_test_trueskill_system(
    mu: Option<f64>,
    sigma: Option<f64>,
    beta: Option<f64>,
    tau: Option<f64>,
    draw_probability: Option<f64>,
) -> TrueSkillSystem {
    match (mu, sigma, beta, tau, draw_probability) {
        (Some(m), Some(s), Some(b), Some(t), Some(d)) => {
            TrueSkillSystem::with_parameters(m, s, b, t, d)
        }
        _ => TrueSkillSystem::new(),
    }
}

/// Factory for creating test match results
#[wasm_bindgen]
pub struct TestMatchFactory;

#[wasm_bindgen]
impl TestMatchFactory {
    /// Create a simple 1v1 match result
    pub fn create_1v1_result(
        player1_id: &str,
        player2_id: &str,
        player1_rating: f64,
        player2_rating: f64,
        outcome: u32,
    ) -> js_sys::Object {
        let result = js_sys::Object::new();
        
        let players = js_sys::Array::new();
        
        let p1 = js_sys::Object::new();
        js_sys::Reflect::set(&p1, &JsValue::from_str("id"), &JsValue::from_str(player1_id)).unwrap();
        js_sys::Reflect::set(&p1, &JsValue::from_str("rating"), &JsValue::from_f64(player1_rating)).unwrap();
        players.push(&p1);
        
        let p2 = js_sys::Object::new();
        js_sys::Reflect::set(&p2, &JsValue::from_str("id"), &JsValue::from_str(player2_id)).unwrap();
        js_sys::Reflect::set(&p2, &JsValue::from_str("rating"), &JsValue::from_f64(player2_rating)).unwrap();
        players.push(&p2);
        
        js_sys::Reflect::set(&result, &JsValue::from_str("players"), &players).unwrap();
        js_sys::Reflect::set(&result, &JsValue::from_str("outcome"), &JsValue::from_f64(outcome as f64)).unwrap();
        
        result
    }

    /// Create a team match result
    pub fn create_team_result(
        team1_players: js_sys::Array,
        team2_players: js_sys::Array,
        ranks: js_sys::Array,
    ) -> js_sys::Object {
        let result = js_sys::Object::new();
        
        let teams = js_sys::Array::new();
        teams.push(&team1_players);
        teams.push(&team2_players);
        
        js_sys::Reflect::set(&result, &JsValue::from_str("teams"), &teams).unwrap();
        js_sys::Reflect::set(&result, &JsValue::from_str("ranks"), &ranks).unwrap();
        
        result
    }

    /// Create a multi-team result
    pub fn create_multi_team_result(
        teams: js_sys::Array,
        ranks: js_sys::Array,
    ) -> js_sys::Object {
        let result = js_sys::Object::new();
        
        js_sys::Reflect::set(&result, &JsValue::from_str("teams"), &teams).unwrap();
        js_sys::Reflect::set(&result, &JsValue::from_str("ranks"), &ranks).unwrap();
        
        result
    }
}

/// Factory for creating test configurations
#[wasm_bindgen]
pub struct TestConfigFactory;

#[wasm_bindgen]
impl TestConfigFactory {
    /// Create default Elo configuration
    pub fn default_elo_config() -> js_sys::Object {
        let config = js_sys::Object::new();
        js_sys::Reflect::set(&config, &JsValue::from_str("k_factor"), &JsValue::from_f64(32.0)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("initial_rating"), &JsValue::from_f64(1500.0)).unwrap();
        config
    }

    /// Create custom Elo configuration
    pub fn custom_elo_config(k_factor: f64, initial_rating: f64) -> js_sys::Object {
        let config = js_sys::Object::new();
        js_sys::Reflect::set(&config, &JsValue::from_str("k_factor"), &JsValue::from_f64(k_factor)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("initial_rating"), &JsValue::from_f64(initial_rating)).unwrap();
        config
    }

    /// Create default Glicko configuration
    pub fn default_glicko_config() -> js_sys::Object {
        let config = js_sys::Object::new();
        js_sys::Reflect::set(&config, &JsValue::from_str("initial_rating"), &JsValue::from_f64(1500.0)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("initial_rd"), &JsValue::from_f64(350.0)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("c"), &JsValue::from_f64(63.2)).unwrap();
        config
    }

    /// Create default TrueSkill configuration
    pub fn default_trueskill_config() -> js_sys::Object {
        let config = js_sys::Object::new();
        js_sys::Reflect::set(&config, &JsValue::from_str("mu"), &JsValue::from_f64(25.0)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("sigma"), &JsValue::from_f64(8.333)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("beta"), &JsValue::from_f64(4.166)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("tau"), &JsValue::from_f64(0.0833)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("draw_probability"), &JsValue::from_f64(0.1)).unwrap();
        config
    }

    /// Create custom TrueSkill configuration
    pub fn custom_trueskill_config(
        mu: f64,
        sigma: f64,
        beta: f64,
        tau: f64,
        draw_probability: f64,
    ) -> js_sys::Object {
        let config = js_sys::Object::new();
        js_sys::Reflect::set(&config, &JsValue::from_str("mu"), &JsValue::from_f64(mu)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("sigma"), &JsValue::from_f64(sigma)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("beta"), &JsValue::from_f64(beta)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("tau"), &JsValue::from_f64(tau)).unwrap();
        js_sys::Reflect::set(&config, &JsValue::from_str("draw_probability"), &JsValue::from_f64(draw_probability)).unwrap();
        config
    }
}

/// Factory for creating test scenarios
#[wasm_bindgen]
pub struct TestScenarioFactory;

#[wasm_bindgen]
impl TestScenarioFactory {
    /// Create a basic ladder scenario with specified number of players
    pub fn create_ladder_scenario(player_count: u32) -> js_sys::Object {
        let scenario = js_sys::Object::new();
        
        let players = js_sys::Array::new();
        for i in 0..player_count {
            let player = js_sys::Object::new();
            js_sys::Reflect::set(&player, &JsValue::from_str("id"), &JsValue::from_str(&format!("player_{}", i))).unwrap();
            js_sys::Reflect::set(&player, &JsValue::from_str("name"), &JsValue::from_str(&format!("Player {}", i + 1))).unwrap();
            js_sys::Reflect::set(&player, &JsValue::from_str("rating"), &JsValue::from_f64(1500.0)).unwrap();
            players.push(&player);
        }
        
        js_sys::Reflect::set(&scenario, &JsValue::from_str("players"), &players).unwrap();
        js_sys::Reflect::set(&scenario, &JsValue::from_str("matches"), &js_sys::Array::new()).unwrap();
        
        scenario
    }

    /// Create a tournament scenario
    pub fn create_tournament_scenario(
        player_count: u32,
        rounds: u32,
        matches_per_round: u32,
    ) -> js_sys::Object {
        let scenario = Self::create_ladder_scenario(player_count);
        
        js_sys::Reflect::set(&scenario, &JsValue::from_str("type"), &JsValue::from_str("tournament")).unwrap();
        js_sys::Reflect::set(&scenario, &JsValue::from_str("rounds"), &JsValue::from_f64(rounds as f64)).unwrap();
        js_sys::Reflect::set(&scenario, &JsValue::from_str("matches_per_round"), &JsValue::from_f64(matches_per_round as f64)).unwrap();
        
        scenario
    }

    /// Create a team-based scenario
    pub fn create_team_scenario(team_count: u32, players_per_team: u32) -> js_sys::Object {
        let scenario = js_sys::Object::new();
        
        let teams = js_sys::Array::new();
        let all_players = js_sys::Array::new();
        
        for t in 0..team_count {
            let team = js_sys::Object::new();
            js_sys::Reflect::set(&team, &JsValue::from_str("id"), &JsValue::from_str(&format!("team_{}", t))).unwrap();
            js_sys::Reflect::set(&team, &JsValue::from_str("name"), &JsValue::from_str(&format!("Team {}", t + 1))).unwrap();
            
            let players = js_sys::Array::new();
            for p in 0..players_per_team {
                let player_id = format!("player_{}_{}", t, p);
                let player = js_sys::Object::new();
                js_sys::Reflect::set(&player, &JsValue::from_str("id"), &JsValue::from_str(&player_id)).unwrap();
                js_sys::Reflect::set(&player, &JsValue::from_str("name"), &JsValue::from_str(&format!("Player {}-{}", t + 1, p + 1))).unwrap();
                js_sys::Reflect::set(&player, &JsValue::from_str("team_id"), &JsValue::from_str(&format!("team_{}", t))).unwrap();
                js_sys::Reflect::set(&player, &JsValue::from_str("rating"), &JsValue::from_f64(1500.0)).unwrap();
                
                players.push(&player);
                all_players.push(&player);
            }
            
            js_sys::Reflect::set(&team, &JsValue::from_str("players"), &players).unwrap();
            teams.push(&team);
        }
        
        js_sys::Reflect::set(&scenario, &JsValue::from_str("type"), &JsValue::from_str("team")).unwrap();
        js_sys::Reflect::set(&scenario, &JsValue::from_str("teams"), &teams).unwrap();
        js_sys::Reflect::set(&scenario, &JsValue::from_str("players"), &all_players).unwrap();
        js_sys::Reflect::set(&scenario, &JsValue::from_str("matches"), &js_sys::Array::new()).unwrap();
        
        scenario
    }
}

/// Builder pattern for complex test objects
pub struct TestObjectBuilder<T> {
    object: Option<T>,
}

impl<T> TestObjectBuilder<T> {
    /// Create a new builder
    pub fn new() -> Self {
        Self { object: None }
    }

    /// Set the object being built
    pub fn with_object(mut self, object: T) -> Self {
        self.object = Some(object);
        self
    }

    /// Build and return the object
    pub fn build(self) -> Option<T> {
        self.object
    }
}

/// Create a builder for Elo ratings
pub fn elo_rating_builder() -> TestObjectBuilder<EloRating> {
    TestObjectBuilder::new()
}

/// Create a builder for TrueSkill ratings
pub fn trueskill_rating_builder() -> TestObjectBuilder<TrueSkillRating> {
    TestObjectBuilder::new()
}