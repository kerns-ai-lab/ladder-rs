//! Additional methods for the unified rating system

use wasm_bindgen::prelude::*;
use serde_json::json;

use ladder_rs::{
    core::{Rating, RatingSystem as CoreRatingSystem, TeamRating},
    elo::{EloRating, EloTeamRating},
    glicko::{GlickoRating, GlickoTeamRating},
    trueskill::{TrueSkillRating, TrueSkillTeamRating},
};

use crate::unified::{UnifiedRatingSystem, PlayerInfo, MatchResult, RatingSystemType, RatingStorage, UnifiedSystemState};
use crate::errors::{ErrorCode, WasmErrorBuilder, to_js_result};

#[wasm_bindgen]
impl UnifiedRatingSystem {
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
        
        match self.system_type {
            RatingSystemType::Elo => {
                let system = self.elo_system.as_ref().unwrap();
                
                let team1_ratings: Vec<EloRating> = team1_players.iter()
                    .map(|id| match &self.players[id].0 {
                        RatingStorage::Elo(r) => r.clone(),
                        _ => unreachable!()
                    })
                    .collect();
                
                let team2_ratings: Vec<EloRating> = team2_players.iter()
                    .map(|id| match &self.players[id].0 {
                        RatingStorage::Elo(r) => r.clone(),
                        _ => unreachable!()
                    })
                    .collect();
                
                let team1 = EloTeamRating::from_player_ratings(team1_ratings);
                let team2 = EloTeamRating::from_player_ratings(team2_ratings);
                
                to_js_result(system.calculate_match_quality(&[team1, team2]))
            },
            RatingSystemType::Glicko => {
                let system = self.glicko_system.as_ref().unwrap();
                
                let team1_ratings: Vec<GlickoRating> = team1_players.iter()
                    .map(|id| match &self.players[id].0 {
                        RatingStorage::Glicko(r) => r.clone(),
                        _ => unreachable!()
                    })
                    .collect();
                
                let team2_ratings: Vec<GlickoRating> = team2_players.iter()
                    .map(|id| match &self.players[id].0 {
                        RatingStorage::Glicko(r) => r.clone(),
                        _ => unreachable!()
                    })
                    .collect();
                
                let team1 = GlickoTeamRating::from_player_ratings(team1_ratings);
                let team2 = GlickoTeamRating::from_player_ratings(team2_ratings);
                
                to_js_result(system.calculate_match_quality(&[team1, team2]))
            },
            RatingSystemType::TrueSkill => {
                let system = self.trueskill_system.as_ref().unwrap();
                
                let team1_ratings: Vec<TrueSkillRating> = team1_players.iter()
                    .map(|id| match &self.players[id].0 {
                        RatingStorage::TrueSkill(r) => r.clone(),
                        _ => unreachable!()
                    })
                    .collect();
                
                let team2_ratings: Vec<TrueSkillRating> = team2_players.iter()
                    .map(|id| match &self.players[id].0 {
                        RatingStorage::TrueSkill(r) => r.clone(),
                        _ => unreachable!()
                    })
                    .collect();
                
                let team1 = TrueSkillTeamRating::from_player_ratings(team1_ratings);
                let team2 = TrueSkillTeamRating::from_player_ratings(team2_ratings);
                
                to_js_result(system.calculate_match_quality(&[team1, team2]))
            },
        }
    }
    
    /// Predict win probability for team 1
    pub fn predict_win_probability(
        &self,
        team1_players: Vec<String>,
        team2_players: Vec<String>
    ) -> Result<f64, JsValue> {
        // For most systems, match quality is the win probability
        // This might need adjustment for specific systems
        self.calculate_match_quality(team1_players, team2_players)
    }
    
    /// Get leaderboard sorted by rating
    pub fn get_leaderboard(&self, limit: Option<usize>) -> Vec<PlayerInfo> {
        let mut players: Vec<PlayerInfo> = self.players.iter()
            .map(|(id, (rating, matches))| rating.to_player_info(id, *matches))
            .collect();
        
        // Sort by rating (descending)
        players.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some(limit) = limit {
            players.truncate(limit);
        }
        
        players
    }
    
    /// Apply rating period decay (for Glicko)
    pub fn apply_rating_period_decay(&mut self) -> Result<(), JsValue> {
        match self.system_type {
            RatingSystemType::Glicko => {
                // In Glicko, ratings decay over time
                // This is a simplified implementation
                for (_, (rating, _)) in self.players.iter_mut() {
                    if let RatingStorage::Glicko(glicko_rating) = rating {
                        // Increase deviation slightly to simulate decay
                        let current_dev = glicko_rating.standard_deviation();
                        let new_dev = (current_dev * 1.01).min(350.0); // Cap at initial deviation
                        *glicko_rating = GlickoRating::new(glicko_rating.mean(), new_dev * new_dev);
                    }
                }
                Ok(())
            },
            _ => Err(WasmErrorBuilder::new(
                ErrorCode::InvalidInput,
                "Rating period decay is only supported for Glicko"
            )
            .build()
            .to_js_error()),
        }
    }
    
    /// Serialize the entire rating system state
    pub fn serialize(&self) -> Result<JsValue, JsValue> {
        let state = UnifiedSystemState {
            system_type: self.system_type,
            players: self.players.clone(),
            config: self.config.clone(),
        };
        
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
        let state: UnifiedSystemState = serde_wasm_bindgen::from_value(data)
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
            serde_wasm_bindgen::to_value(&state.config).unwrap()
        )?;
        
        // Restore players
        system.players = state.players;
        
        Ok(system)
    }
}