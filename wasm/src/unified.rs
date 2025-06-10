//! Unified rating system interface for WASM bindings
//!
//! This module provides a consistent API across all rating systems (Elo, Glicko, TrueSkill)
//! with support for serialization and team-based ratings.

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use ladder_rs::{
    core::{Rating, RatingSystem as CoreRatingSystem},
    elo::{EloRating, EloSystem},
    glicko::{GlickoRating, Glicko},
    trueskill::{TrueSkillRating, TrueSkillSystem},
};

use crate::errors::{ErrorCode, WasmErrorBuilder};

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

/// Configuration for creating rating systems
#[derive(Debug, Deserialize)]
pub(crate) struct SystemConfig {
    pub system: String,
    // Elo parameters
    pub k_factor: Option<f64>,
    // Glicko parameters
    pub initial_rating: Option<f64>,
    pub initial_deviation: Option<f64>,
    pub volatility: Option<f64>,
    pub rating_period_duration: Option<u64>,
    // TrueSkill parameters
    pub mu: Option<f64>,
    pub sigma: Option<f64>,
    pub beta: Option<f64>,
    pub tau: Option<f64>,
    pub draw_probability: Option<f64>,
}

/// Internal rating storage that can hold any rating type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RatingStorage {
    Elo(EloRating),
    Glicko(GlickoRating),
    TrueSkill(TrueSkillRating),
}

impl RatingStorage {
    pub fn to_player_info(&self, id: &str, matches_played: u32) -> PlayerInfo {
        match self {
            RatingStorage::Elo(r) => PlayerInfo {
                id: id.to_string(),
                rating: r.mean(),
                uncertainty: r.standard_deviation(),
                conservative_rating: None,
                matches_played,
            },
            RatingStorage::Glicko(r) => PlayerInfo {
                id: id.to_string(),
                rating: r.mean(),
                uncertainty: r.standard_deviation(),
                conservative_rating: None,
                matches_played,
            },
            RatingStorage::TrueSkill(r) => PlayerInfo {
                id: id.to_string(),
                rating: r.mean(),
                uncertainty: r.standard_deviation(),
                conservative_rating: Some(r.conservative_rating()),
                matches_played,
            },
        }
    }
}

/// Serializable state for the entire rating system
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedSystemState {
    pub system_type: RatingSystemType,
    pub players: HashMap<String, (RatingStorage, u32)>, // (rating, matches_played)
    pub config: serde_json::Value,
}

/// Unified rating system that can use any of the available rating algorithms
#[wasm_bindgen]
pub struct UnifiedRatingSystem {
    pub(crate) system_type: RatingSystemType,
    pub(crate) players: HashMap<String, (RatingStorage, u32)>,
    // Store system-specific data
    pub(crate) elo_system: Option<EloSystem>,
    pub(crate) glicko_system: Option<Glicko>,
    pub(crate) trueskill_system: Option<TrueSkillSystem>,
    // Configuration for serialization
    pub(crate) config: serde_json::Value,
}

// Main implementation methods are in unified_impl.rs, unified_constructor.rs and unified_methods.rs