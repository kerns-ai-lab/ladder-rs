//! Optimized JavaScript API for ladder-rs Elo rating system
//!
//! This module provides a minimal interface for Elo rating calculations,
//! optimized for smallest possible WASM bundle size.

use serde::Deserialize;
use serde_wasm_bindgen::from_value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating as TeamRatingTrait},
    elo::{EloRating, EloSystem, EloTeamRating},
};

use crate::utils::js_error;

/// Player rating for JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmRating {
    #[wasm_bindgen(getter_with_clone)]
    pub player_id: String,
    pub rating: f64,
}

/// Team representation for JavaScript  
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmTeam {
    #[wasm_bindgen(skip)]
    pub players: Vec<WasmRating>,
    pub score: f64,
}

#[wasm_bindgen]
impl WasmTeam {
    #[wasm_bindgen(constructor)]
    pub fn new(score: f64) -> WasmTeam {
        WasmTeam {
            players: Vec::new(),
            score,
        }
    }

    pub fn add_player(&mut self, player: WasmRating) {
        self.players.push(player);
    }

    #[wasm_bindgen(getter)]
    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}

/// Elo system configuration
#[derive(Debug, Deserialize)]
struct EloConfig {
    #[serde(default = "default_k_factor")]
    k_factor: f64,
}

fn default_k_factor() -> f64 {
    32.0
}

/// Optimized Elo rating system for JavaScript
///
/// This provides a minimal API surface for Elo rating calculations
/// to achieve the smallest possible WASM bundle size.
#[wasm_bindgen]
pub struct WasmRatingSystem {
    system: EloSystem,
    players: HashMap<String, EloRating>,
}

#[wasm_bindgen]
impl WasmRatingSystem {
    /// Creates a new Elo rating system
    ///
    /// # Arguments
    /// * `config` - JSON configuration object with optional k_factor
    ///
    /// # Returns
    /// A new WasmRatingSystem instance
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmRatingSystem, JsValue> {
        let config: EloConfig = from_value(config)
            .unwrap_or_else(|_| EloConfig { k_factor: 32.0 });

        let system = EloSystem::with_parameters(config.k_factor, 1.0, 400.0, 1500.0);

        Ok(WasmRatingSystem {
            system,
            players: HashMap::new(),
        })
    }

    /// Creates a new player with default Elo rating (1500)
    ///
    /// # Arguments
    /// * `player_id` - Unique identifier for the player
    ///
    /// # Returns
    /// A WasmRating object representing the new player's rating
    pub fn create_player(&mut self, player_id: String) -> Result<WasmRating, JsValue> {
        let rating = self.system.create_rating();
        
        let js_rating = WasmRating {
            player_id: player_id.clone(),
            rating: rating.mean(),
        };

        self.players.insert(player_id, rating);
        Ok(js_rating)
    }

    /// Updates ratings for a 1v1 match
    ///
    /// # Arguments
    /// * `player1_id` - ID of first player
    /// * `player2_id` - ID of second player  
    /// * `player1_wins` - true if player1 wins, false if player2 wins
    ///
    /// # Returns
    /// Array with updated ratings for both players
    pub fn update_match(&mut self, player1_id: String, player2_id: String, player1_wins: bool) -> Result<Vec<WasmRating>, JsValue> {
        // Get or create ratings
        let rating1 = self.players.get(&player1_id)
            .cloned()
            .unwrap_or_else(|| self.system.create_rating());
            
        let rating2 = self.players.get(&player2_id)
            .cloned()
            .unwrap_or_else(|| self.system.create_rating());

        // Create teams
        let team1 = EloTeamRating::new(rating1);
        let team2 = EloTeamRating::new(rating2);

        // Create outcome (rank 1 wins, rank 2 loses)
        let outcome = if player1_wins {
            GameOutcome::new(vec![1, 2])
        } else {
            GameOutcome::new(vec![2, 1])
        };

        // Update ratings
        let updated = self.system
            .rate(&[team1, team2], &outcome)
            .map_err(|e| js_error(&format!("Rating update failed: {}", e)))?;

        // Extract updated ratings
        let updated_rating1 = &updated[0].player_ratings()[0];
        let updated_rating2 = &updated[1].player_ratings()[0];

        // Store updated ratings
        self.players.insert(player1_id.clone(), updated_rating1.clone());
        self.players.insert(player2_id.clone(), updated_rating2.clone());

        // Return updated ratings
        Ok(vec![
            WasmRating {
                player_id: player1_id,
                rating: updated_rating1.mean(),
            },
            WasmRating {
                player_id: player2_id,
                rating: updated_rating2.mean(),
            }
        ])
    }

    /// Calculates expected win probability for player1 vs player2
    ///
    /// # Arguments
    /// * `player1_id` - ID of first player
    /// * `player2_id` - ID of second player
    ///
    /// # Returns
    /// Probability (0.0 to 1.0) that player1 wins
    pub fn get_win_probability(&self, player1_id: String, player2_id: String) -> Result<f64, JsValue> {
        let rating1 = self.players.get(&player1_id)
            .cloned()
            .unwrap_or_else(|| self.system.create_rating());
            
        let rating2 = self.players.get(&player2_id)
            .cloned()
            .unwrap_or_else(|| self.system.create_rating());

        let team1 = EloTeamRating::new(rating1);
        let team2 = EloTeamRating::new(rating2);

        self.system.calculate_match_quality(&[team1, team2])
            .map_err(|e| js_error(&format!("Win probability calculation failed: {}", e)))
    }

    /// Gets a player's current rating
    ///
    /// # Arguments
    /// * `player_id` - ID of the player
    ///
    /// # Returns
    /// Current rating value, or default (1500) if player not found
    pub fn get_rating(&self, player_id: String) -> f64 {
        self.players.get(&player_id)
            .map(|r| r.mean())
            .unwrap_or(1500.0)
    }

    /// Returns all players sorted by rating (highest first)
    ///
    /// # Returns
    /// Array of WasmRating objects sorted by rating descending
    pub fn get_leaderboard(&self) -> Vec<WasmRating> {
        let mut leaderboard: Vec<WasmRating> = self.players
            .iter()
            .map(|(id, rating)| WasmRating {
                player_id: id.clone(),
                rating: rating.mean(),
            })
            .collect();

        leaderboard.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        leaderboard
    }

    /// Gets the number of tracked players
    pub fn player_count(&self) -> usize {
        self.players.len()
    }
}