//! Simplified unified rating system interface - Elo only for now
//!
//! This is a working implementation with just Elo support.
//! Glicko and TrueSkill will be added later.

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem as CoreRatingSystem, TeamRating},
    elo::{EloRating, EloSystem, EloTeamRating},
};

use crate::errors::{ErrorCode, WasmErrorBuilder, to_js_result};

/// Rating system types
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingSystemType {
    Elo,
    Glicko,
    TrueSkill,
}

/// Player information exposed to JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    #[wasm_bindgen(skip)]
    pub id: String,
    #[wasm_bindgen(skip)]
    pub rating: f64,
    #[wasm_bindgen(skip)]
    pub uncertainty: f64,
    #[wasm_bindgen(skip)]
    pub conservative_rating: Option<f64>,
    #[wasm_bindgen(skip)]
    pub matches_played: u32,
}

#[wasm_bindgen]
impl PlayerInfo {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn rating(&self) -> f64 {
        self.rating
    }
    
    #[wasm_bindgen(getter)]
    pub fn uncertainty(&self) -> f64 {
        self.uncertainty
    }
    
    #[wasm_bindgen(getter)]
    pub fn conservative_rating(&self) -> Option<f64> {
        self.conservative_rating
    }
    
    #[wasm_bindgen(getter)]
    pub fn matches_played(&self) -> u32 {
        self.matches_played
    }
}

/// Match result information
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    #[wasm_bindgen(skip)]
    pub winner_team: u32,
    #[wasm_bindgen(skip)]
    pub updated_ratings: Vec<PlayerInfo>,
    #[wasm_bindgen(skip)]
    pub match_quality: f64,
}

#[wasm_bindgen]
impl MatchResult {
    #[wasm_bindgen(getter)]
    pub fn winner_team(&self) -> u32 {
        self.winner_team
    }
    
    #[wasm_bindgen(getter)]
    pub fn updated_ratings(&self) -> Vec<PlayerInfo> {
        self.updated_ratings.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn match_quality(&self) -> f64 {
        self.match_quality
    }
}

/// Simplified unified rating system - Elo only for now
#[wasm_bindgen]
pub struct UnifiedRatingSystem {
    system_type: RatingSystemType,
    elo_system: EloSystem,
    players: HashMap<String, (EloRating, u32)>, // (rating, matches_played)
    config: serde_json::Value,
}

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
        
        // Check system type
        let system_type = config_value["system"].as_str()
            .ok_or_else(|| {
                WasmErrorBuilder::new(
                    ErrorCode::InvalidConfiguration,
                    "Missing 'system' field in configuration"
                )
                .build()
                .to_js_error()
            })?;
        
        // For now, only support Elo
        if system_type != "elo" {
            return Err(WasmErrorBuilder::new(
                ErrorCode::InvalidConfiguration,
                "Only 'elo' system is currently supported"
            )
            .with_details(format!("Got: {}", system_type))
            .build()
            .to_js_error());
        }
        
        let k_factor = config_value["k_factor"].as_f64().unwrap_or(32.0);
        
        Ok(UnifiedRatingSystem {
            system_type: RatingSystemType::Elo,
            elo_system: EloSystem::with_parameters(k_factor, 1.0, 400.0, 1500.0),
            players: HashMap::new(),
            config: config_value,
        })
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
        
        let rating = self.elo_system.create_rating();
        let player_info = PlayerInfo {
            id: player_id.clone(),
            rating: rating.mean(),
            uncertainty: rating.standard_deviation(),
            conservative_rating: None,
            matches_played: 0,
        };
        
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
            .map(|(rating, matches)| PlayerInfo {
                id: player_id.clone(),
                rating: rating.mean(),
                uncertainty: rating.standard_deviation(),
                conservative_rating: None,
                matches_played: *matches,
            })
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
        
        // Check all players exist and get their ratings
        let team1_ratings: Vec<EloRating> = team1_players.iter()
            .map(|id| {
                self.players.get(id)
                    .map(|(r, _)| r.clone())
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::InvalidInput,
                            "Player not found"
                        )
                        .with_details(format!("Player ID: {}", id))
                        .build()
                        .to_js_error()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        let team2_ratings: Vec<EloRating> = team2_players.iter()
            .map(|id| {
                self.players.get(id)
                    .map(|(r, _)| r.clone())
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::InvalidInput,
                            "Player not found"
                        )
                        .with_details(format!("Player ID: {}", id))
                        .build()
                        .to_js_error()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        // Create teams
        let team1 = EloTeamRating::from_player_ratings(team1_ratings);
        let team2 = EloTeamRating::from_player_ratings(team2_ratings);
        
        // Calculate match quality before the match
        let match_quality = to_js_result(
            self.elo_system.calculate_match_quality(&[team1.clone(), team2.clone()])
        )?;
        
        // Create outcome
        let outcome = if winner_team == 1 {
            GameOutcome::new(vec![1, 2])
        } else {
            GameOutcome::new(vec![2, 1])
        };
        
        // Rate match
        let updated_teams = to_js_result(
            self.elo_system.rate(&[team1, team2], &outcome)
        )?;
        
        // Update storage and collect results
        let mut updated_players = Vec::new();
        
        for (i, player_id) in team1_players.iter().enumerate() {
            let new_rating = updated_teams[0].player_ratings()[i].clone();
            let matches = self.players.get_mut(player_id).unwrap().1 + 1;
            self.players.insert(player_id.clone(), (new_rating.clone(), matches));
            
            updated_players.push(PlayerInfo {
                id: player_id.clone(),
                rating: new_rating.mean(),
                uncertainty: new_rating.standard_deviation(),
                conservative_rating: None,
                matches_played: matches,
            });
        }
        
        for (i, player_id) in team2_players.iter().enumerate() {
            let new_rating = updated_teams[1].player_ratings()[i].clone();
            let matches = self.players.get_mut(player_id).unwrap().1 + 1;
            self.players.insert(player_id.clone(), (new_rating.clone(), matches));
            
            updated_players.push(PlayerInfo {
                id: player_id.clone(),
                rating: new_rating.mean(),
                uncertainty: new_rating.standard_deviation(),
                conservative_rating: None,
                matches_played: matches,
            });
        }
        
        Ok(MatchResult {
            winner_team,
            updated_ratings: updated_players,
            match_quality,
        })
    }
    
    /// Process multiple matches in batch
    pub fn process_matches(
        &mut self,
        matches: Vec<JsValue>
    ) -> Result<Vec<MatchResult>, JsValue> {
        let mut results = Vec::new();
        
        for match_data in matches {
            let data: serde_json::Value = serde_wasm_bindgen::from_value(match_data)
                .map_err(|e| {
                    WasmErrorBuilder::new(
                        ErrorCode::SerializationError,
                        "Failed to parse match data"
                    )
                    .with_details(format!("Error: {}", e))
                    .build()
                    .to_js_error()
                })?;
            
            let team1: Vec<String> = serde_json::from_value(data["team1"].clone())
                .map_err(|_| {
                    WasmErrorBuilder::new(
                        ErrorCode::InvalidInput,
                        "Invalid team1 data"
                    )
                    .build()
                    .to_js_error()
                })?;
            
            let team2: Vec<String> = serde_json::from_value(data["team2"].clone())
                .map_err(|_| {
                    WasmErrorBuilder::new(
                        ErrorCode::InvalidInput,
                        "Invalid team2 data"
                    )
                    .build()
                    .to_js_error()
                })?;
            
            let winner: u32 = data["winner"].as_u64()
                .ok_or_else(|| {
                    WasmErrorBuilder::new(
                        ErrorCode::InvalidInput,
                        "Invalid winner data"
                    )
                    .build()
                    .to_js_error()
                })? as u32;
            
            let result = self.process_match(team1, team2, winner)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Calculate match quality between two teams
    pub fn calculate_match_quality(
        &self,
        team1_players: Vec<String>,
        team2_players: Vec<String>
    ) -> Result<f64, JsValue> {
        // Get ratings for both teams
        let team1_ratings: Vec<EloRating> = team1_players.iter()
            .map(|id| {
                self.players.get(id)
                    .map(|(r, _)| r.clone())
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::InvalidInput,
                            "Player not found"
                        )
                        .with_details(format!("Player ID: {}", id))
                        .build()
                        .to_js_error()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        let team2_ratings: Vec<EloRating> = team2_players.iter()
            .map(|id| {
                self.players.get(id)
                    .map(|(r, _)| r.clone())
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::InvalidInput,
                            "Player not found"
                        )
                        .with_details(format!("Player ID: {}", id))
                        .build()
                        .to_js_error()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        let team1 = EloTeamRating::from_player_ratings(team1_ratings);
        let team2 = EloTeamRating::from_player_ratings(team2_ratings);
        
        to_js_result(self.elo_system.calculate_match_quality(&[team1, team2]))
    }
    
    /// Predict win probability for team 1
    pub fn predict_win_probability(
        &self,
        team1_players: Vec<String>,
        team2_players: Vec<String>
    ) -> Result<f64, JsValue> {
        // For Elo, match quality is the win probability for team 1
        self.calculate_match_quality(team1_players, team2_players)
    }
    
    /// Get leaderboard sorted by rating
    pub fn get_leaderboard(&self, limit: Option<usize>) -> Vec<PlayerInfo> {
        let mut players: Vec<PlayerInfo> = self.players.iter()
            .map(|(id, (rating, matches))| PlayerInfo {
                id: id.clone(),
                rating: rating.mean(),
                uncertainty: rating.standard_deviation(),
                conservative_rating: None,
                matches_played: *matches,
            })
            .collect();
        
        // Sort by rating (descending)
        players.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(limit) = limit {
            players.truncate(limit);
        }
        
        players
    }
    
    /// Apply rating period decay (not supported for Elo)
    pub fn apply_rating_period_decay(&mut self) -> Result<(), JsValue> {
        Err(WasmErrorBuilder::new(
            ErrorCode::InvalidInput,
            "Rating period decay is only supported for Glicko"
        )
        .build()
        .to_js_error())
    }
    
    /// Serialize the entire rating system state
    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        let state = serde_json::json!({
            "system_type": self.system_type,
            "players": self.players.iter().map(|(id, (rating, matches))| {
                serde_json::json!({
                    "id": id,
                    "rating": rating.mean(),
                    "uncertainty": rating.standard_deviation(),
                    "matches_played": matches,
                })
            }).collect::<Vec<_>>(),
            "config": self.config,
        });
        
        serde_wasm_bindgen::to_value(&state)
            .map_err(|e| {
                WasmErrorBuilder::new(
                    ErrorCode::SerializationError,
                    "Failed to serialize system state"
                )
                .with_details(format!("Error: {}", e))
                .build()
                .to_js_error()
            })
    }
    
    /// Deserialize and restore rating system state
    pub fn deserialize(data: JsValue) -> Result<UnifiedRatingSystem, JsValue> {
        let state: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| {
                WasmErrorBuilder::new(
                    ErrorCode::SerializationError,
                    "Failed to deserialize system state"
                )
                .with_details(format!("Error: {}", e))
                .build()
                .to_js_error()
            })?;
        
        // Recreate the system with the saved configuration
        let mut system = UnifiedRatingSystem::new(
            serde_wasm_bindgen::to_value(&state["config"]).unwrap()
        )?;
        
        // Restore players
        if let Some(players) = state["players"].as_array() {
            for player_data in players {
                let id = player_data["id"].as_str()
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::SerializationError,
                            "Invalid player ID in saved state"
                        )
                        .build()
                        .to_js_error()
                    })?;
                
                let rating_mean = player_data["rating"].as_f64()
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::SerializationError,
                            "Invalid rating in saved state"
                        )
                        .build()
                        .to_js_error()
                    })?;
                
                let uncertainty = player_data["uncertainty"].as_f64()
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::SerializationError,
                            "Invalid uncertainty in saved state"
                        )
                        .build()
                        .to_js_error()
                    })?;
                
                let matches = player_data["matches_played"].as_u64()
                    .ok_or_else(|| {
                        WasmErrorBuilder::new(
                            ErrorCode::SerializationError,
                            "Invalid matches_played in saved state"
                        )
                        .build()
                        .to_js_error()
                    })? as u32;
                
                let rating = system.elo_system.create_rating_with_values(rating_mean, uncertainty * uncertainty);
                system.players.insert(id.to_string(), (rating, matches));
            }
        }
        
        Ok(system)
    }
}