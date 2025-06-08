//! Player management system for tracking players, profiles, and match history
//!
//! This module provides comprehensive player management capabilities including:
//! - Player registration and profile management
//! - Match history tracking
//! - Player statistics calculation
//! - Head-to-head records
//! - Player search and filtering
//! - Import/export functionality

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::utils::js_error;

/// Player profile information
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,

    #[wasm_bindgen(getter_with_clone)]
    pub name: Option<String>,

    #[wasm_bindgen(getter_with_clone)]
    pub email: Option<String>,

    pub created_at: f64, // Timestamp in milliseconds
    pub updated_at: f64,
    pub is_active: bool,
}

/// Match record for tracking game history
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,

    #[wasm_bindgen(skip)]
    pub team1_players: Vec<String>,

    #[wasm_bindgen(skip)]
    pub team2_players: Vec<String>,

    pub outcome: i32, // 0 = draw, 1 = team1 wins, 2 = team2 wins
    pub timestamp: f64,

    #[wasm_bindgen(getter_with_clone)]
    pub notes: Option<String>,
}

#[wasm_bindgen]
impl MatchRecord {
    #[wasm_bindgen(getter)]
    pub fn team1_player_count(&self) -> usize {
        self.team1_players.len()
    }

    #[wasm_bindgen(getter)]
    pub fn team2_player_count(&self) -> usize {
        self.team2_players.len()
    }

    pub fn get_team1_player(&self, index: usize) -> Option<String> {
        self.team1_players.get(index).cloned()
    }

    pub fn get_team2_player(&self, index: usize) -> Option<String> {
        self.team2_players.get(index).cloned()
    }
}

/// Player statistics
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    #[wasm_bindgen(getter_with_clone)]
    pub player_id: String,
    pub total_matches: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub win_rate: f64,
    pub current_streak: i32, // Positive for wins, negative for losses
    pub longest_win_streak: u32,
    pub longest_loss_streak: u32,
}

/// Head-to-head record between two players
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadToHeadRecord {
    #[wasm_bindgen(getter_with_clone)]
    pub player1_id: String,

    #[wasm_bindgen(getter_with_clone)]
    pub player2_id: String,

    pub total_matches: u32,
    pub player1_wins: u32,
    pub player2_wins: u32,
    pub draws: u32,
}

/// Player data stored internally
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerData {
    profile: PlayerProfile,
    match_ids: Vec<String>,
    aliases: Vec<String>,
}

/// Player manager for comprehensive player tracking
#[wasm_bindgen]
pub struct PlayerManager {
    players: HashMap<String, PlayerData>,
    matches: HashMap<String, MatchRecord>,
    alias_map: HashMap<String, String>, // alias -> player_id
}

#[wasm_bindgen]
impl PlayerManager {
    /// Creates a new player manager instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> PlayerManager {
        PlayerManager {
            players: HashMap::new(),
            matches: HashMap::new(),
            alias_map: HashMap::new(),
        }
    }

    /// Registers a new player
    pub fn register_player(
        &mut self,
        id: String,
        name: Option<String>,
        email: Option<String>,
    ) -> Result<PlayerProfile, JsValue> {
        // Check if player already exists
        if self.players.contains_key(&id) {
            return Err(js_error(&format!("Player with ID '{}' already exists", id)));
        }

        let now = Utc::now().timestamp_millis() as f64;
        let profile = PlayerProfile {
            id: id.clone(),
            name,
            email,
            created_at: now,
            updated_at: now,
            is_active: true,
        };

        let player_data = PlayerData {
            profile: profile.clone(),
            match_ids: Vec::new(),
            aliases: Vec::new(),
        };

        self.players.insert(id, player_data);
        Ok(profile)
    }

    /// Gets a player's profile
    pub fn get_player_profile(&self, id_or_alias: &str) -> Result<PlayerProfile, JsValue> {
        // Check if it's an alias
        let player_id = if let Some(id) = self.alias_map.get(id_or_alias) {
            id
        } else {
            id_or_alias
        };

        self.players
            .get(player_id)
            .map(|data| data.profile.clone())
            .ok_or_else(|| js_error(&format!("Player '{}' not found", id_or_alias)))
    }

    /// Updates a player's profile
    pub fn update_player_profile(
        &mut self,
        id: &str,
        name: Option<String>,
        email: Option<String>,
    ) -> Result<PlayerProfile, JsValue> {
        let player_data = self
            .players
            .get_mut(id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", id)))?;

        if name.is_some() {
            player_data.profile.name = name;
        }
        if email.is_some() {
            player_data.profile.email = email;
        }

        player_data.profile.updated_at = Utc::now().timestamp_millis() as f64;
        Ok(player_data.profile.clone())
    }

    /// Deactivates a player (soft delete)
    pub fn deactivate_player(&mut self, id: &str) -> Result<(), JsValue> {
        let player_data = self
            .players
            .get_mut(id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", id)))?;

        player_data.profile.is_active = false;
        player_data.profile.updated_at = Utc::now().timestamp_millis() as f64;
        Ok(())
    }

    /// Reactivates a player
    pub fn reactivate_player(&mut self, id: &str) -> Result<(), JsValue> {
        let player_data = self
            .players
            .get_mut(id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", id)))?;

        player_data.profile.is_active = true;
        player_data.profile.updated_at = Utc::now().timestamp_millis() as f64;
        Ok(())
    }

    /// Checks if a player is active
    pub fn is_player_active(&self, id: &str) -> bool {
        self.players
            .get(id)
            .map(|data| data.profile.is_active)
            .unwrap_or(false)
    }

    /// Adds a match record
    pub fn add_match_record(
        &mut self,
        team1_players: Vec<String>,
        team2_players: Vec<String>,
        outcome: i32,
        notes: Option<String>,
    ) -> Result<String, JsValue> {
        // Validate players exist
        for player_id in &team1_players {
            if !self.players.contains_key(player_id) {
                return Err(js_error(&format!("Player '{}' not found", player_id)));
            }
        }
        for player_id in &team2_players {
            if !self.players.contains_key(player_id) {
                return Err(js_error(&format!("Player '{}' not found", player_id)));
            }
        }

        // Validate outcome
        if outcome > 2 {
            return Err(js_error(
                "Invalid outcome: must be 0 (draw), 1 (team1 wins), or 2 (team2 wins)",
            ));
        }

        let match_id = Uuid::new_v4().to_string();
        let match_record = MatchRecord {
            id: match_id.clone(),
            team1_players: team1_players.clone(),
            team2_players: team2_players.clone(),
            outcome,
            timestamp: Utc::now().timestamp_millis() as f64,
            notes,
        };

        // Add match to all players' histories
        for player_id in &team1_players {
            if let Some(player_data) = self.players.get_mut(player_id) {
                player_data.match_ids.push(match_id.clone());
            }
        }
        for player_id in &team2_players {
            if let Some(player_data) = self.players.get_mut(player_id) {
                player_data.match_ids.push(match_id.clone());
            }
        }

        self.matches.insert(match_id.clone(), match_record);
        Ok(match_id)
    }

    /// Gets a player's match history
    pub fn get_player_match_history(
        &self,
        player_id: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MatchRecord>, JsValue> {
        let player_data = self
            .players
            .get(player_id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", player_id)))?;

        let mut matches: Vec<MatchRecord> = player_data
            .match_ids
            .iter()
            .filter_map(|match_id| self.matches.get(match_id))
            .cloned()
            .collect();

        // Sort by timestamp descending (newest first)
        matches.sort_by(|a, b| b.timestamp.partial_cmp(&a.timestamp).unwrap());

        // Apply pagination
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(matches.len());

        Ok(matches.into_iter().skip(offset).take(limit).collect())
    }

    /// Gets player statistics
    pub fn get_player_stats(&self, player_id: &str) -> Result<PlayerStats, JsValue> {
        let player_data = self
            .players
            .get(player_id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", player_id)))?;

        let mut wins = 0u32;
        let mut losses = 0u32;
        let mut draws = 0u32;
        let mut current_streak = 0i32;
        let mut longest_win_streak = 0u32;
        let mut longest_loss_streak = 0u32;
        let mut temp_win_streak = 0u32;
        let mut temp_loss_streak = 0u32;

        // Get matches sorted by timestamp
        let mut matches: Vec<&MatchRecord> = player_data
            .match_ids
            .iter()
            .filter_map(|match_id| self.matches.get(match_id))
            .collect();

        matches.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

        for match_record in matches {
            let player_in_team1 = match_record.team1_players.contains(&player_id.to_string());

            let result = match match_record.outcome {
                0 => "draw",
                1 if player_in_team1 => "win",
                1 => "loss",
                2 if player_in_team1 => "loss",
                2 => "win",
                _ => continue,
            };

            match result {
                "win" => {
                    wins += 1;
                    temp_win_streak += 1;
                    temp_loss_streak = 0;
                    if current_streak >= 0 {
                        current_streak += 1;
                    } else {
                        current_streak = 1;
                    }
                    longest_win_streak = longest_win_streak.max(temp_win_streak);
                }
                "loss" => {
                    losses += 1;
                    temp_loss_streak += 1;
                    temp_win_streak = 0;
                    if current_streak <= 0 {
                        current_streak -= 1;
                    } else {
                        current_streak = -1;
                    }
                    longest_loss_streak = longest_loss_streak.max(temp_loss_streak);
                }
                "draw" => {
                    draws += 1;
                    temp_win_streak = 0;
                    temp_loss_streak = 0;
                    current_streak = 0;
                }
                _ => {}
            }
        }

        let total_matches = wins + losses + draws;
        let win_rate = if total_matches > 0 {
            (wins as f64) / (total_matches as f64)
        } else {
            0.0
        };

        Ok(PlayerStats {
            player_id: player_id.to_string(),
            total_matches,
            wins,
            losses,
            draws,
            win_rate,
            current_streak,
            longest_win_streak,
            longest_loss_streak,
        })
    }

    /// Gets the total number of players
    #[wasm_bindgen(getter)]
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Gets all active players
    pub fn get_active_players(&self) -> Result<Vec<PlayerProfile>, JsValue> {
        Ok(self
            .players
            .values()
            .filter(|data| data.profile.is_active)
            .map(|data| data.profile.clone())
            .collect())
    }

    /// Searches for players by name or ID
    pub fn search_players(&self, query: &str) -> Result<Vec<PlayerProfile>, JsValue> {
        let query_lower = query.to_lowercase();

        Ok(self
            .players
            .values()
            .filter(|data| {
                data.profile.id.to_lowercase().contains(&query_lower)
                    || data
                        .profile
                        .name
                        .as_ref()
                        .map(|name| name.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .map(|data| data.profile.clone())
            .collect())
    }

    /// Imports players from JSON
    pub fn bulk_import_players(&mut self, json_data: &str) -> Result<u32, JsValue> {
        #[derive(Deserialize)]
        struct ImportPlayer {
            id: String,
            name: Option<String>,
            email: Option<String>,
        }

        let players: Vec<ImportPlayer> = serde_json::from_str(json_data)
            .map_err(|e| js_error(&format!("Invalid JSON: {}", e)))?;

        let mut imported = 0u32;
        for player in players {
            if !self.players.contains_key(&player.id) {
                self.register_player(player.id, player.name, player.email)?;
                imported += 1;
            }
        }

        Ok(imported)
    }

    /// Exports all players to JSON
    pub fn export_players(&self, include_inactive: bool) -> Result<String, JsValue> {
        let profiles: Vec<&PlayerProfile> = self
            .players
            .values()
            .filter(|data| include_inactive || data.profile.is_active)
            .map(|data| &data.profile)
            .collect();

        serde_json::to_string_pretty(&profiles)
            .map_err(|e| js_error(&format!("Export failed: {}", e)))
    }

    /// Gets head-to-head record between two players
    pub fn get_player_head_to_head(
        &self,
        player1_id: &str,
        player2_id: &str,
    ) -> Result<HeadToHeadRecord, JsValue> {
        // Validate players exist
        if !self.players.contains_key(player1_id) {
            return Err(js_error(&format!("Player '{}' not found", player1_id)));
        }
        if !self.players.contains_key(player2_id) {
            return Err(js_error(&format!("Player '{}' not found", player2_id)));
        }

        let mut player1_wins = 0u32;
        let mut player2_wins = 0u32;
        let mut draws = 0u32;

        // Check all matches involving both players
        for match_record in self.matches.values() {
            let p1_in_team1 = match_record.team1_players.contains(&player1_id.to_string());
            let p1_in_team2 = match_record.team2_players.contains(&player1_id.to_string());
            let p2_in_team1 = match_record.team1_players.contains(&player2_id.to_string());
            let p2_in_team2 = match_record.team2_players.contains(&player2_id.to_string());

            // Skip if players are on the same team or neither is in the match
            if (p1_in_team1 && p2_in_team1) || (p1_in_team2 && p2_in_team2) {
                continue;
            }
            if (!p1_in_team1 && !p1_in_team2) || (!p2_in_team1 && !p2_in_team2) {
                continue;
            }

            match match_record.outcome {
                0 => draws += 1,
                1 => {
                    if p1_in_team1 && p2_in_team2 {
                        player1_wins += 1;
                    } else if p2_in_team1 && p1_in_team2 {
                        player2_wins += 1;
                    }
                }
                2 => {
                    if p1_in_team2 && p2_in_team1 {
                        player1_wins += 1;
                    } else if p2_in_team2 && p1_in_team1 {
                        player2_wins += 1;
                    }
                }
                _ => {}
            }
        }

        Ok(HeadToHeadRecord {
            player1_id: player1_id.to_string(),
            player2_id: player2_id.to_string(),
            total_matches: player1_wins + player2_wins + draws,
            player1_wins,
            player2_wins,
            draws,
        })
    }

    /// Merges two player profiles (for handling duplicates)
    pub fn merge_players(&mut self, from_id: &str, to_id: &str) -> Result<(), JsValue> {
        if from_id == to_id {
            return Err(js_error("Cannot merge a player with itself"));
        }

        // Get the match IDs from the source player
        let from_match_ids = self
            .players
            .get(from_id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", from_id)))?
            .match_ids
            .clone();

        // Add match IDs to the target player
        let to_player = self
            .players
            .get_mut(to_id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", to_id)))?;

        for match_id in from_match_ids {
            if !to_player.match_ids.contains(&match_id) {
                to_player.match_ids.push(match_id);
            }
        }

        // Deactivate the source player
        self.deactivate_player(from_id)?;

        Ok(())
    }

    /// Adds an alias for a player
    pub fn add_player_alias(&mut self, player_id: &str, alias: &str) -> Result<(), JsValue> {
        // Check if player exists
        let player_data = self
            .players
            .get_mut(player_id)
            .ok_or_else(|| js_error(&format!("Player '{}' not found", player_id)))?;

        // Check if alias is already in use
        if self.alias_map.contains_key(alias) {
            return Err(js_error(&format!("Alias '{}' is already in use", alias)));
        }

        // Add alias
        player_data.aliases.push(alias.to_string());
        self.alias_map
            .insert(alias.to_string(), player_id.to_string());

        Ok(())
    }

    /// Gets all aliases for a player
    pub fn get_player_aliases(&self, player_id: &str) -> Result<Vec<String>, JsValue> {
        self.players
            .get(player_id)
            .map(|data| data.aliases.clone())
            .ok_or_else(|| js_error(&format!("Player '{}' not found", player_id)))
    }
}

impl Default for PlayerManager {
    fn default() -> Self {
        Self::new()
    }
}
