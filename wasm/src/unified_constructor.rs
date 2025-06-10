//! Constructor and basic methods for the unified rating system

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use ladder_rs::{
    core::{Rating, RatingSystem as CoreRatingSystem},
    elo::{EloSystem},
    glicko::{GlickoSystem},
    trueskill::{TrueSkillSystem},
};

use crate::unified::{UnifiedRatingSystem, PlayerInfo, MatchResult, RatingSystemType, RatingStorage, SystemConfig};
use crate::errors::{ErrorCode, WasmErrorBuilder};

#[wasm_bindgen]
impl UnifiedRatingSystem {
    /// Create a new unified rating system
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<UnifiedRatingSystem, JsValue> {
        let config_value: serde_json::Value = serde_wasm_bindgen::from_value(config.clone())
            .map_err(|e| {
                WasmErrorBuilder::new(
                    ErrorCode::SerializationError,
                    "Failed to parse configuration"
                )
                .with_details(format!("Deserialization error: {}", e))
                .build()
                .to_js_error()
            })?;
        
        let system_config: SystemConfig = serde_json::from_value(config_value.clone())
            .map_err(|e| {
                WasmErrorBuilder::new(
                    ErrorCode::InvalidConfiguration,
                    "Invalid system configuration"
                )
                .with_details(format!("Configuration error: {}", e))
                .build()
                .to_js_error()
            })?;
        
        let mut system = UnifiedRatingSystem {
            system_type: RatingSystemType::Elo,
            players: HashMap::new(),
            elo_system: None,
            glicko_system: None,
            trueskill_system: None,
            config: config_value,
        };
        
        match system_config.system.as_str() {
            "elo" => {
                let k_factor = system_config.k_factor.unwrap_or(32.0);
                system.system_type = RatingSystemType::Elo;
                system.elo_system = Some(EloSystem::with_parameters(k_factor, 1.0, 400.0, 1500.0));
            },
            "glicko" => {
                let initial_rating = system_config.initial_rating.unwrap_or(1500.0);
                let initial_deviation = system_config.initial_deviation.unwrap_or(350.0);
                let volatility = system_config.volatility.unwrap_or(0.06);
                system.system_type = RatingSystemType::Glicko;
                system.glicko_system = Some(GlickoSystem::new(initial_rating, initial_deviation, volatility));
            },
            "trueskill" => {
                let mu = system_config.mu.unwrap_or(25.0);
                let sigma = system_config.sigma.unwrap_or(8.333);
                let beta = system_config.beta.unwrap_or(4.166);
                let tau = system_config.tau.unwrap_or(0.083);
                let draw_prob = system_config.draw_probability.unwrap_or(0.1);
                system.system_type = RatingSystemType::TrueSkill;
                system.trueskill_system = Some(TrueSkillSystem::new(mu, sigma, beta, tau, draw_prob));
            },
            _ => {
                return Err(WasmErrorBuilder::new(
                    ErrorCode::InvalidConfiguration,
                    "Unknown rating system type"
                )
                .with_details(format!("System '{}' is not supported. Use 'elo', 'glicko', or 'trueskill'", system_config.system))
                .build()
                .to_js_error());
            }
        }
        
        Ok(system)
    }
    
    /// Get the type of rating system
    #[wasm_bindgen(getter)]
    pub fn system_type(&self) -> RatingSystemType {
        self.system_type
    }
    
    /// Create a new player with default rating
    pub fn create_player(&mut self, player_id: String) -> Result<PlayerInfo, JsValue> {
        if player_id.is_empty() {
            return Err(WasmErrorBuilder::new(
                ErrorCode::InvalidInput,
                "Player ID cannot be empty"
            )
            .build()
            .to_js_error());
        }
        
        if self.players.contains_key(&player_id) {
            return Err(WasmErrorBuilder::new(
                ErrorCode::InvalidInput,
                "Player already exists"
            )
            .with_details(format!("Player ID: {}", player_id))
            .build()
            .to_js_error());
        }
        
        let rating = match self.system_type {
            RatingSystemType::Elo => {
                let system = self.elo_system.as_ref().unwrap();
                RatingStorage::Elo(system.create_rating())
            },
            RatingSystemType::Glicko => {
                let system = self.glicko_system.as_ref().unwrap();
                RatingStorage::Glicko(system.create_rating())
            },
            RatingSystemType::TrueSkill => {
                let system = self.trueskill_system.as_ref().unwrap();
                RatingStorage::TrueSkill(system.create_rating())
            },
        };
        
        let player_info = rating.to_player_info(&player_id, 0);
        self.players.insert(player_id, (rating, 0));
        
        Ok(player_info)
    }
    
    /// Create multiple players at once
    pub fn create_players(&mut self, player_ids: Vec<String>) -> Result<Vec<PlayerInfo>, JsValue> {
        let mut created_players = Vec::new();
        
        for id in player_ids {
            let player = self.create_player(id)?;
            created_players.push(player);
        }
        
        Ok(created_players)
    }
    
    /// Get player information
    pub fn get_player(&self, player_id: String) -> Result<PlayerInfo, JsValue> {
        self.players.get(&player_id)
            .map(|(rating, matches)| rating.to_player_info(&player_id, *matches))
            .ok_or_else(|| {
                WasmErrorBuilder::new(
                    ErrorCode::InvalidInput,
                    "Player not found"
                )
                .with_details(format!("Player ID: {}", player_id))
                .build()
                .to_js_error()
            })
    }
    
    /// Process a match between two teams
    pub fn process_match(
        &mut self,
        team1_players: Vec<String>,
        team2_players: Vec<String>,
        winner_team: u32
    ) -> Result<MatchResult, JsValue> {
        // Validate inputs
        if team1_players.is_empty() || team2_players.is_empty() {
            return Err(WasmErrorBuilder::new(
                ErrorCode::InvalidInput,
                "Teams cannot be empty"
            )
            .build()
            .to_js_error());
        }
        
        if winner_team != 1 && winner_team != 2 {
            return Err(WasmErrorBuilder::new(
                ErrorCode::InvalidInput,
                "Winner team must be 1 or 2"
            )
            .with_details(format!("Got: {}", winner_team))
            .build()
            .to_js_error());
        }
        
        // Check all players exist
        for player_id in team1_players.iter().chain(team2_players.iter()) {
            if !self.players.contains_key(player_id) {
                return Err(WasmErrorBuilder::new(
                    ErrorCode::InvalidInput,
                    "Player not found"
                )
                .with_details(format!("Player ID: {}", player_id))
                .build()
                .to_js_error());
            }
        }
        
        // Process match based on system type
        let updated_ratings = match self.system_type {
            RatingSystemType::Elo => self.process_elo_match(&team1_players, &team2_players, winner_team)?,
            RatingSystemType::Glicko => self.process_glicko_match(&team1_players, &team2_players, winner_team)?,
            RatingSystemType::TrueSkill => self.process_trueskill_match(&team1_players, &team2_players, winner_team)?,
        };
        
        // Calculate match quality
        let match_quality = self.calculate_match_quality(team1_players, team2_players)?;
        
        Ok(MatchResult {
            winner_team,
            updated_ratings,
            match_quality,
        })
    }
}