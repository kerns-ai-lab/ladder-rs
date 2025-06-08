//! Test configuration and setup utilities
//!
//! This module provides configuration options and setup utilities for the test suite.

use wasm_bindgen_test::*;
use ladder_rs_wasm::{WasmRatingSystem, PlayerManager};
use wasm_bindgen::JsValue;

wasm_bindgen_test_configure!(run_in_browser);

/// Default test configuration values
pub struct TestConfig {
    pub default_elo_k_factor: f64,
    pub default_glicko_rating_period: f64,
    pub default_trueskill_beta: f64,
    pub default_trueskill_tau: f64,
    pub test_player_count: u32,
    pub test_match_count: u32,
    pub performance_iteration_count: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            default_elo_k_factor: 32.0,
            default_glicko_rating_period: 15.0,
            default_trueskill_beta: 4.166666666666667, // 25/6
            default_trueskill_tau: 0.08333333333333333, // 25/300
            test_player_count: 100,
            test_match_count: 1000,
            performance_iteration_count: 10000,
        }
    }
}

/// Create a configured Elo rating system
pub fn create_elo_system(k_factor: Option<f64>) -> Result<WasmRatingSystem, JsValue> {
    let config = TestConfig::default();
    let k = k_factor.unwrap_or(config.default_elo_k_factor);
    
    let mut system = WasmRatingSystem::new("elo")?;
    // Note: In a real implementation, we'd set the k_factor through configuration
    Ok(system)
}

/// Create a configured Glicko rating system
pub fn create_glicko_system(rating_period: Option<f64>) -> Result<WasmRatingSystem, JsValue> {
    let config = TestConfig::default();
    let period = rating_period.unwrap_or(config.default_glicko_rating_period);
    
    let mut system = WasmRatingSystem::new("glicko")?;
    // Note: In a real implementation, we'd set the rating_period through configuration
    Ok(system)
}

/// Create a configured TrueSkill rating system
pub fn create_trueskill_system(beta: Option<f64>, tau: Option<f64>) -> Result<WasmRatingSystem, JsValue> {
    let config = TestConfig::default();
    let beta_val = beta.unwrap_or(config.default_trueskill_beta);
    let tau_val = tau.unwrap_or(config.default_trueskill_tau);
    
    let mut system = WasmRatingSystem::new("trueskill")?;
    // Note: In a real implementation, we'd set beta and tau through configuration
    Ok(system)
}

/// Create a player manager with test data
pub fn create_test_player_manager(player_count: u32) -> Result<PlayerManager, JsValue> {
    let mut manager = PlayerManager::new();
    
    for i in 0..player_count {
        let id = format!("player_{}", i);
        let name = format!("Player {}", i);
        let email = format!("player{}@test.com", i);
        
        manager.register_player(&id, Some(&name), Some(&email))?;
    }
    
    Ok(manager)
}

/// Set up a complete test environment
pub fn setup_test_environment(
    system_type: &str,
    player_count: u32,
) -> Result<(WasmRatingSystem, PlayerManager), JsValue> {
    let system = match system_type {
        "elo" => create_elo_system(None)?,
        "glicko" => create_glicko_system(None)?,
        "trueskill" => create_trueskill_system(None, None)?,
        _ => return Err(JsValue::from_str("Invalid rating system type")),
    };
    
    let manager = create_test_player_manager(player_count)?;
    
    Ok((system, manager))
}

#[wasm_bindgen_test]
fn test_default_config() {
    let config = TestConfig::default();
    
    assert_eq!(config.default_elo_k_factor, 32.0);
    assert_eq!(config.default_glicko_rating_period, 15.0);
    assert_eq!(config.default_trueskill_beta, 25.0 / 6.0);
    assert_eq!(config.default_trueskill_tau, 25.0 / 300.0);
    assert_eq!(config.test_player_count, 100);
    assert_eq!(config.test_match_count, 1000);
    assert_eq!(config.performance_iteration_count, 10000);
}

#[wasm_bindgen_test]
fn test_create_rating_systems() {
    // Test Elo creation
    let elo = create_elo_system(None);
    assert!(elo.is_ok());
    
    let elo_custom = create_elo_system(Some(16.0));
    assert!(elo_custom.is_ok());
    
    // Test Glicko creation
    let glicko = create_glicko_system(None);
    assert!(glicko.is_ok());
    
    let glicko_custom = create_glicko_system(Some(30.0));
    assert!(glicko_custom.is_ok());
    
    // Test TrueSkill creation
    let trueskill = create_trueskill_system(None, None);
    assert!(trueskill.is_ok());
    
    let trueskill_custom = create_trueskill_system(Some(5.0), Some(0.1));
    assert!(trueskill_custom.is_ok());
}

#[wasm_bindgen_test]
fn test_create_player_manager() {
    let manager = create_test_player_manager(10).unwrap();
    let players = manager.get_all_players();
    
    assert_eq!(players.length(), 10);
    
    // Check first player
    let player = manager.get_player("player_0").unwrap();
    let name = js_sys::Reflect::get(&player, &JsValue::from_str("name")).unwrap();
    assert_eq!(name.as_string().unwrap(), "Player 0");
}

#[wasm_bindgen_test]
fn test_setup_environment() {
    // Test with Elo
    let (elo_system, elo_manager) = setup_test_environment("elo", 5).unwrap();
    assert_eq!(elo_manager.get_all_players().length(), 5);
    
    // Test with Glicko
    let (glicko_system, glicko_manager) = setup_test_environment("glicko", 10).unwrap();
    assert_eq!(glicko_manager.get_all_players().length(), 10);
    
    // Test with TrueSkill
    let (trueskill_system, trueskill_manager) = setup_test_environment("trueskill", 20).unwrap();
    assert_eq!(trueskill_manager.get_all_players().length(), 20);
    
    // Test with invalid system
    let invalid = setup_test_environment("invalid", 5);
    assert!(invalid.is_err());
}