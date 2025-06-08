//! Integration test helpers
//!
//! This module provides utilities for complex integration test scenarios.

use wasm_bindgen_test::*;
use ladder_rs_wasm::{WasmRatingSystem, WasmTeam, PlayerManager};
use wasm_bindgen::JsValue;
use js_sys::Array;

wasm_bindgen_test_configure!(run_in_browser);

/// Scenario builder for complex test scenarios
pub struct ScenarioBuilder {
    rating_system: WasmRatingSystem,
    player_manager: PlayerManager,
    players: Vec<String>,
    matches: Vec<(String, String, u32)>, // (player1, player2, outcome)
}

impl ScenarioBuilder {
    /// Create a new scenario builder
    pub fn new(system_type: &str) -> Result<Self, JsValue> {
        Ok(Self {
            rating_system: WasmRatingSystem::new(system_type)?,
            player_manager: PlayerManager::new(),
            players: Vec::new(),
            matches: Vec::new(),
        })
    }
    
    /// Add a player to the scenario
    pub fn add_player(mut self, player_id: &str) -> Result<Self, JsValue> {
        self.player_manager.register_player(player_id, None, None)?;
        self.rating_system.create_player(player_id)?;
        self.players.push(player_id.to_string());
        Ok(self)
    }
    
    /// Add multiple players
    pub fn add_players(mut self, player_ids: Vec<&str>) -> Result<Self, JsValue> {
        for id in player_ids {
            self.player_manager.register_player(id, None, None)?;
            self.rating_system.create_player(id)?;
            self.players.push(id.to_string());
        }
        Ok(self)
    }
    
    /// Add a match to be played
    pub fn add_match(mut self, player1: &str, player2: &str, outcome: u32) -> Self {
        self.matches.push((player1.to_string(), player2.to_string(), outcome));
        self
    }
    
    /// Play all matches
    pub fn play_matches(mut self) -> Result<Self, JsValue> {
        for (p1, p2, outcome) in &self.matches {
            // Record in player manager
            self.player_manager.add_match_record(
                vec![p1.clone()].into_boxed_slice(),
                vec![p2.clone()].into_boxed_slice(),
                outcome.clone() as i32,
                None,
            )?;
            
            // Update ratings
            let team1 = WasmTeam::new(vec![p1.clone()].into_boxed_slice());
            let team2 = WasmTeam::new(vec![p2.clone()].into_boxed_slice());
            self.rating_system.update_ratings(team1, team2, *outcome)?;
        }
        Ok(self)
    }
    
    /// Get the final state
    pub fn build(self) -> (WasmRatingSystem, PlayerManager, Vec<String>) {
        (self.rating_system, self.player_manager, self.players)
    }
}

/// Create a round-robin tournament
pub fn create_round_robin_tournament(
    system_type: &str,
    players: Vec<&str>,
) -> Result<(WasmRatingSystem, PlayerManager), JsValue> {
    let mut builder = ScenarioBuilder::new(system_type)?
        .add_players(players.clone())?;
    
    // Generate all pairings
    for i in 0..players.len() {
        for j in i+1..players.len() {
            // Simulate outcome based on player indices (higher index more likely to win)
            let outcome = if i < j { 2 } else { 1 };
            builder = builder.add_match(players[i], players[j], outcome);
        }
    }
    
    let (system, manager, _) = builder.play_matches()?.build();
    Ok((system, manager))
}

/// Create a ladder tournament scenario
pub fn create_ladder_scenario(
    system_type: &str,
    num_players: u32,
) -> Result<(WasmRatingSystem, PlayerManager), JsValue> {
    let mut players = Vec::new();
    for i in 0..num_players {
        players.push(format!("player_{}", i));
    }
    
    let mut builder = ScenarioBuilder::new(system_type)?;
    
    // Add all players
    for player in &players {
        builder = builder.add_player(player)?;
    }
    
    // Ladder matches: each player plays the one above and below
    for i in 0..players.len()-1 {
        // Higher ranked player has 70% chance to win
        let outcome = if i % 10 < 7 { 1 } else { 2 };
        builder = builder.add_match(&players[i], &players[i+1], outcome);
    }
    
    let (system, manager, _) = builder.play_matches()?.build();
    Ok((system, manager))
}

/// Simulate a Swiss tournament
pub fn create_swiss_tournament(
    system_type: &str,
    num_players: u32,
    num_rounds: u32,
) -> Result<(WasmRatingSystem, PlayerManager), JsValue> {
    if num_players % 2 != 0 {
        return Err(JsValue::from_str("Swiss tournaments require even number of players"));
    }
    
    let mut players = Vec::new();
    for i in 0..num_players {
        players.push(format!("player_{}", i));
    }
    
    let mut builder = ScenarioBuilder::new(system_type)?;
    
    // Add all players
    for player in &players {
        builder = builder.add_player(player)?;
    }
    
    // Simple Swiss pairing simulation
    for round in 0..num_rounds {
        for i in 0..num_players/2 {
            let p1_idx = (i * 2) as usize;
            let p2_idx = (i * 2 + 1) as usize;
            
            // Mix up outcomes based on round
            let outcome = ((round + i) % 3).min(2).max(0) as u32;
            if outcome == 0 {
                builder = builder.add_match(&players[p1_idx], &players[p2_idx], 0);
            } else {
                builder = builder.add_match(&players[p1_idx], &players[p2_idx], outcome);
            }
        }
    }
    
    let (system, manager, _) = builder.play_matches()?.build();
    Ok((system, manager))
}

#[wasm_bindgen_test]
fn test_scenario_builder() {
    let (system, manager, players) = ScenarioBuilder::new("elo").unwrap()
        .add_player("alice").unwrap()
        .add_player("bob").unwrap()
        .add_match("alice", "bob", 1)
        .play_matches().unwrap()
        .build();
    
    assert_eq!(players.len(), 2);
    assert_eq!(manager.get_all_players().length(), 2);
    
    // Check match was recorded
    let alice_stats = manager.get_player_stats("alice").unwrap();
    let stats_obj = alice_stats.dyn_ref::<js_sys::Object>().unwrap();
    let wins = js_sys::Reflect::get(&stats_obj, &JsValue::from_str("wins")).unwrap();
    assert_eq!(wins.as_f64().unwrap(), 1.0);
}

#[wasm_bindgen_test]
fn test_round_robin_tournament() {
    let players = vec!["alice", "bob", "charlie", "diana"];
    let (system, manager) = create_round_robin_tournament("elo", players).unwrap();
    
    // Should have n*(n-1)/2 matches for n players
    // 4 players = 6 matches
    let all_matches = manager.get_match_history(None, None);
    assert_eq!(all_matches.length(), 6);
}

#[wasm_bindgen_test]
fn test_ladder_scenario() {
    let (system, manager) = create_ladder_scenario("glicko", 10).unwrap();
    
    // Should have n-1 matches for n players
    let all_matches = manager.get_match_history(None, None);
    assert_eq!(all_matches.length(), 9);
    
    // Check players were created
    assert_eq!(manager.get_all_players().length(), 10);
}

#[wasm_bindgen_test]
fn test_swiss_tournament() {
    let (system, manager) = create_swiss_tournament("trueskill", 8, 3).unwrap();
    
    // 8 players, 3 rounds, 4 matches per round = 12 total matches
    let all_matches = manager.get_match_history(None, None);
    assert_eq!(all_matches.length(), 12);
    
    // Odd number of players should fail
    let result = create_swiss_tournament("elo", 7, 3);
    assert!(result.is_err());
}