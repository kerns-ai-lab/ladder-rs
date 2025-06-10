// Rating System implementation with error handling for Task 1.2.5

use wasm_bindgen::prelude::*;
use crate::errors::*;
use crate::types::{JsRating, JsPlayer, JsMatchConfig, JsMatchResult, JsOutcome, JsEloConfig};
use crate::api::{WasmRatingSystem, WasmRating, WasmTeam};
use std::collections::HashMap;

/// Main rating system with error handling
#[wasm_bindgen]
pub struct JsRatingSystem {
    inner: WasmRatingSystem,
    players: HashMap<String, JsPlayer>,
}

#[wasm_bindgen]
impl JsRatingSystem {
    /// Create a new rating system
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsMatchConfig) -> Result<JsRatingSystem, JsValue> {
        // Validate the algorithm
        let algorithm = config.algorithm();
        match algorithm.as_str() {
            "elo" | "glicko" | "trueskill" => {},
            _ => {
                return Err(JsRatingError::configuration_error(&format!(
                    "Unknown algorithm: {}", algorithm
                )).to_js_value());
            }
        }
        
        // Create the inner system
        let inner = match WasmRatingSystem::new_with_config(config.algorithm().as_str(), config.config()) {
            Ok(system) => system,
            Err(e) => {
                return Err(JsRatingError::configuration_error(&format!(
                    "Failed to create rating system: {:?}", e
                )).to_js_value());
            }
        };
        
        Ok(JsRatingSystem {
            inner,
            players: HashMap::new(),
        })
    }
    
    /// Add a player to the system
    #[wasm_bindgen(js_name = addPlayer)]
    pub fn add_player(&mut self, player: JsPlayer) -> Result<(), JsValue> {
        // Validate player
        validate_non_empty(&player.id(), "Player ID")
            .map_err(|e| e.to_js_value())?;
            
        // Check for duplicate
        if self.players.contains_key(&player.id()) {
            return Err(JsRatingError::validation_error(&format!(
                "Player with ID '{}' already exists", player.id()
            )).with_recovery_suggestion("Use a different player ID or update the existing player")
            .to_js_value());
        }
        
        // Add to inner system
        self.inner.create_player(&player.id(), 
            player.rating().mean(),
            player.rating().variance()
        );
        
        // Store player
        self.players.insert(player.id(), player);
        
        Ok(())
    }
    
    /// Process a match between players
    #[wasm_bindgen(js_name = processMatch)]
    pub fn process_match(&self, players: Box<[JsPlayer]>, outcome: JsOutcome) -> Result<JsMatchResult, JsValue> {
        let players_slice = &*players;
        
        // Validate player count
        if players_slice.is_empty() {
            return Err(JsRatingError::validation_error("At least two players are required for a match")
                .with_context("player_count", "0")
                .to_js_value());
        }
        
        if players_slice.len() < 2 {
            return Err(JsRatingError::validation_error("At least two players are required for a match")
                .with_context("player_count", &players_slice.len().to_string())
                .to_js_value());
        }
        
        // Validate all players exist
        for player in players_slice.iter() {
            if !self.players.contains_key(&player.id()) {
                return Err(JsRatingError::validation_error(&format!(
                    "Player '{}' not found in the system", player.id()
                )).with_recovery_suggestion("Add the player to the system before processing matches")
                .to_js_value());
            }
        }
        
        // Create teams (assuming 1v1 for simplicity)
        let team1 = WasmTeam {
            player_ids: vec![players_slice[0].id()],
        };
        let team2 = WasmTeam {
            player_ids: vec![players_slice[1].id()],
        };
        
        // Convert outcome
        let outcome_bool = match outcome {
            JsOutcome::Win => true,
            JsOutcome::Loss => false,
            JsOutcome::Draw => {
                // For now, treat draw as a special case
                // In a real implementation, this would depend on the algorithm
                return Err(JsRatingError::validation_error("Draw outcomes are not yet implemented")
                    .with_recovery_suggestion("Use Win or Loss outcomes for now")
                    .to_js_value());
            }
        };
        
        // Process the match
        match self.inner.update_match(&team1, &team2, outcome_bool) {
            Ok(results) => {
                // Convert results to JsRatings
                let updated_ratings: Vec<JsRating> = results.into_iter()
                    .map(|r| JsRating::new(r.rating, r.uncertainty.unwrap_or(200.0)).unwrap())
                    .collect();
                    
                // Determine winner
                let winner = if outcome_bool { players_slice[0].id() } else { players_slice[1].id() };
                
                Ok(JsMatchResult::new(winner, updated_ratings))
            }
            Err(e) => {
                Err(JsRatingError::calculation_error(&format!("Failed to process match: {:?}", e))
                    .to_js_value())
            }
        }
    }
    
    /// Process a match with graceful degradation
    #[wasm_bindgen(js_name = processMatchSafe)]
    pub fn process_match_safe(&self, player_ids: Vec<String>, outcome: JsOutcome) -> Result<SafeMatchResult, JsValue> {
        // Try to get players
        let mut players = Vec::new();
        for id in &player_ids {
            match self.players.get(id) {
                Some(player) => players.push(player.clone()),
                None => {
                    let error = JsRatingError::validation_error(&format!("Player '{}' not found", id));
                    return Ok(SafeMatchResult::err(error));
                }
            }
        }
        
        // Try to process the match
        match self.process_match(players.into_boxed_slice(), outcome) {
            Ok(result) => Ok(SafeMatchResult::ok(serde_wasm_bindgen::to_value(&result).unwrap())),
            Err(e) => {
                let error = JsRatingError::calculation_error(&format!("Match processing failed: {:?}", e));
                Ok(SafeMatchResult::err(error))
            }
        }
    }
    
    /// Process multiple matches with partial failure handling
    #[wasm_bindgen(js_name = processBatchSafe)]
    pub fn process_batch_safe(&self, operations: Vec<JsValue>) -> BatchResult {
        let mut results = Vec::new();
        
        for op in operations {
            // Parse operation (simplified for now)
            // In a real implementation, this would deserialize the operation properly
            let safe_result = SafeMatchResult::err(
                JsRatingError::validation_error("Batch operation parsing not implemented")
            );
            results.push(safe_result);
        }
        
        BatchResult::new(results)
    }
    
    /// Get a player by ID
    #[wasm_bindgen(js_name = getPlayer)]
    pub fn get_player(&self, player_id: &str) -> Result<JsPlayer, JsValue> {
        self.players.get(player_id)
            .cloned()
            .ok_or_else(|| {
                JsRatingError::validation_error(&format!("Player '{}' not found", player_id))
                    .to_js_value()
            })
    }
    
    /// Get all players
    #[wasm_bindgen(js_name = getAllPlayers)]
    pub fn get_all_players(&self) -> Vec<JsPlayer> {
        self.players.values().cloned().collect()
    }
}