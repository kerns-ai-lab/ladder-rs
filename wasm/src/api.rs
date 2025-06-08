//! JavaScript API layer for ladder-rs rating systems
//!
//! This module provides a unified interface for all rating systems (Elo, Glicko, TrueSkill)
//! through the WasmRatingSystem struct, enabling JavaScript applications to easily work
//! with any of the supported rating algorithms.

use serde::Deserialize;
use serde_wasm_bindgen::from_value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use ladder_rs::{
    core::{GameOutcome, Rating, RatingSystem, TeamRating as TeamRatingTrait},
    elo::{EloRating, EloSystem, EloTeamRating},
    glicko::{Glicko, GlickoRating, GlickoTeamRating},
    trueskill::{TrueSkill, TrueSkillRating, TrueSkillTeam},
};

use crate::utils::js_error;

/// Player rating with ID for JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmRating {
    #[wasm_bindgen(getter_with_clone)]
    pub player_id: String,
    pub rating: f64,
    pub uncertainty: Option<f64>,
    pub volatility: Option<f64>,
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

/// Configuration for rating systems, parsed from JavaScript
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RatingSystemConfig {
    Elo {
        #[serde(default = "default_k_factor")]
        k_factor: f64,
    },
    Glicko {
        #[serde(default = "default_initial_volatility")]
        initial_volatility: f64,
    },
    TrueSkill {
        #[serde(default = "default_beta")]
        beta: f64,
        #[serde(default = "default_tau")]
        tau: f64,
    },
}

fn default_k_factor() -> f64 {
    32.0
}
fn default_initial_volatility() -> f64 {
    0.06
}
fn default_beta() -> f64 {
    25.0 / 6.0
}
fn default_tau() -> f64 {
    25.0 / 300.0
}

/// Internal enum to hold different rating system implementations
enum RatingSystemImpl {
    Elo(EloSystem),
    Glicko(Glicko),
    TrueSkill(TrueSkill),
}

/// Player data stored internally
struct PlayerData {
    elo_rating: Option<EloRating>,
    glicko_rating: Option<GlickoRating>,
    trueskill_rating: Option<TrueSkillRating>,
}

/// Unified rating system interface for JavaScript
///
/// This struct provides a consistent API for working with all supported rating systems.
/// It handles system-specific configuration and provides methods that work across all systems.
#[wasm_bindgen]
pub struct WasmRatingSystem {
    system: RatingSystemImpl,
    players: HashMap<String, PlayerData>,
}

#[wasm_bindgen]
impl WasmRatingSystem {
    /// Creates a new rating system instance
    ///
    /// # Arguments
    /// * `system_type` - The type of rating system: "elo", "glicko", or "trueskill"
    /// * `config` - JSON configuration object specific to the rating system
    ///
    /// # Returns
    /// A new WasmRatingSystem instance or an error if configuration is invalid
    #[wasm_bindgen(constructor)]
    pub fn new(system_type: &str, config: JsValue) -> Result<WasmRatingSystem, JsValue> {
        let config: RatingSystemConfig =
            from_value(config).map_err(|e| js_error(&format!("Invalid configuration: {}", e)))?;

        let system = match (system_type, config) {
            ("elo", RatingSystemConfig::Elo { k_factor }) => {
                let elo = EloSystem::with_parameters(k_factor, 1.0, 400.0, 1500.0);
                RatingSystemImpl::Elo(elo)
            }
            ("glicko", RatingSystemConfig::Glicko { .. }) => {
                // Use default Glicko configuration
                let glicko = Glicko::new();
                RatingSystemImpl::Glicko(glicko)
            }
            ("trueskill", RatingSystemConfig::TrueSkill { beta, .. }) => {
                // Calculate beta_squared from beta
                let beta_squared = beta * beta;
                let ts = TrueSkill::with_parameters(
                    25.0,                            // mu_0
                    (25.0 / 3.0) * (25.0 / 3.0),     // sigma_0_squared
                    beta_squared,                    // beta_squared
                    (25.0 / 300.0) * (25.0 / 300.0), // tau_squared
                    0.1,                             // draw_probability
                    ladder_rs::trueskill::TrueSkillImplementation::Simplified,
                )
                .map_err(|e| js_error(&format!("Failed to create TrueSkill: {}", e)))?;
                RatingSystemImpl::TrueSkill(ts)
            }
            _ => {
                return Err(js_error(&format!(
                    "Unsupported rating system: {}",
                    system_type
                )))
            }
        };

        Ok(WasmRatingSystem {
            system,
            players: HashMap::new(),
        })
    }

    /// Creates a new player with the default rating for the system
    ///
    /// # Arguments
    /// * `player_id` - Unique identifier for the player
    ///
    /// # Returns
    /// A JsRating object representing the new player's rating
    pub fn create_player(&mut self, player_id: String) -> Result<WasmRating, JsValue> {
        let mut player_data = PlayerData {
            elo_rating: None,
            glicko_rating: None,
            trueskill_rating: None,
        };

        let js_rating = match &self.system {
            RatingSystemImpl::Elo(elo) => {
                let rating = elo.create_rating();
                player_data.elo_rating = Some(rating.clone());
                WasmRating {
                    player_id: player_id.clone(),
                    rating: rating.mean(),
                    uncertainty: None,
                    volatility: None,
                }
            }
            RatingSystemImpl::Glicko(glicko) => {
                let rating = glicko.create_rating();
                player_data.glicko_rating = Some(rating.clone());
                WasmRating {
                    player_id: player_id.clone(),
                    rating: rating.mu,
                    uncertainty: Some(rating.rd),
                    volatility: None,
                }
            }
            RatingSystemImpl::TrueSkill(trueskill) => {
                let rating = trueskill.create_rating();
                player_data.trueskill_rating = Some(rating.clone());
                WasmRating {
                    player_id: player_id.clone(),
                    rating: rating.mean(),
                    uncertainty: Some(rating.std_dev()),
                    volatility: None,
                }
            }
        };

        self.players.insert(player_id, player_data);
        Ok(js_rating)
    }

    /// Updates ratings based on match results
    ///
    /// # Arguments
    /// * `teams` - Array of JsTeam objects representing the match participants and results
    ///
    /// # Returns
    /// Updated teams with new ratings for all players
    pub fn update_ratings(&mut self, teams: Vec<WasmTeam>) -> Result<Vec<WasmTeam>, JsValue> {
        // Validate input
        if teams.len() < 2 {
            return Err(js_error("At least 2 teams are required for a match"));
        }

        for team in &teams {
            if team.players.is_empty() {
                return Err(js_error("Teams cannot be empty"));
            }
        }

        // Create outcome based on scores
        let mut score_indices: Vec<(usize, f64)> = teams
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.score))
            .collect();

        // Sort by score descending (lower score = better rank)
        score_indices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Convert scores to ranks
        let mut rank_map = vec![0; teams.len()];
        let mut current_rank = 1;
        for i in 0..score_indices.len() {
            if i > 0 && score_indices[i].1 != score_indices[i - 1].1 {
                current_rank = i + 1;
            }
            rank_map[score_indices[i].0] = current_rank;
        }
        let ranks = rank_map;

        let outcome = GameOutcome::new(ranks);

        // Process based on rating system type
        match &self.system {
            RatingSystemImpl::Elo(_) => self.update_elo_ratings(&teams, &outcome),
            RatingSystemImpl::Glicko(_) => self.update_glicko_ratings(&teams, &outcome),
            RatingSystemImpl::TrueSkill(_) => self.update_trueskill_ratings(&teams, &outcome),
        }
    }

    /// Calculates the match quality for a proposed match
    ///
    /// # Arguments
    /// * `teams` - Array of JsTeam objects representing the proposed match participants
    ///
    /// # Returns
    /// A quality score between 0 and 1, where 1 indicates a perfectly balanced match
    pub fn get_match_quality(&self, teams: Vec<WasmTeam>) -> Result<f64, JsValue> {
        match &self.system {
            RatingSystemImpl::Elo(elo) => {
                // Elo only supports 1v1
                if teams.len() != 2 || teams[0].players.len() != 1 || teams[1].players.len() != 1 {
                    return Err(js_error("Elo only supports 1v1 matches"));
                }

                let rating1 = self
                    .players
                    .get(&teams[0].players[0].player_id)
                    .and_then(|p| p.elo_rating.as_ref())
                    .map(|r| r.clone())
                    .unwrap_or_else(|| elo.create_rating());

                let rating2 = self
                    .players
                    .get(&teams[1].players[0].player_id)
                    .and_then(|p| p.elo_rating.as_ref())
                    .map(|r| r.clone())
                    .unwrap_or_else(|| elo.create_rating());

                let team1 = EloTeamRating::new(rating1);
                let team2 = EloTeamRating::new(rating2);

                elo.calculate_match_quality(&[team1, team2])
                    .map_err(|e| js_error(&format!("Match quality calculation failed: {}", e)))
            }
            RatingSystemImpl::Glicko(glicko) => {
                // Glicko only supports 1v1
                if teams.len() != 2 || teams[0].players.len() != 1 || teams[1].players.len() != 1 {
                    return Err(js_error("Glicko only supports 1v1 matches"));
                }

                let rating1 = self
                    .players
                    .get(&teams[0].players[0].player_id)
                    .and_then(|p| p.glicko_rating.as_ref())
                    .map(|r| r.clone())
                    .unwrap_or_else(|| glicko.create_rating());

                let rating2 = self
                    .players
                    .get(&teams[1].players[0].player_id)
                    .and_then(|p| p.glicko_rating.as_ref())
                    .map(|r| r.clone())
                    .unwrap_or_else(|| glicko.create_rating());

                let team1 =
                    <GlickoTeamRating as TeamRatingTrait>::from_player_ratings(vec![rating1]);
                let team2 =
                    <GlickoTeamRating as TeamRatingTrait>::from_player_ratings(vec![rating2]);

                glicko
                    .calculate_match_quality(&[team1, team2])
                    .map_err(|e| js_error(&format!("Match quality calculation failed: {}", e)))
            }
            RatingSystemImpl::TrueSkill(trueskill) => {
                // Convert to TrueSkill teams and calculate quality
                let mut ts_teams = Vec::new();

                for team in &teams {
                    let mut ts_players = Vec::new();
                    for player in &team.players {
                        if let Some(player_data) = self.players.get(&player.player_id) {
                            if let Some(rating) = &player_data.trueskill_rating {
                                ts_players.push(rating.clone());
                            } else {
                                ts_players.push(trueskill.create_rating());
                            }
                        } else {
                            ts_players.push(trueskill.create_rating());
                        }
                    }
                    ts_teams.push(TrueSkillTeam::from_player_ratings(ts_players));
                }

                trueskill
                    .calculate_match_quality(&ts_teams)
                    .map_err(|e| js_error(&format!("Match quality calculation failed: {}", e)))
            }
        }
    }

    /// Returns a sorted leaderboard of all tracked players
    ///
    /// # Returns
    /// Array of JsRating objects sorted by rating (highest first)
    pub fn get_leaderboard(&self) -> Result<Vec<WasmRating>, JsValue> {
        let mut leaderboard = Vec::new();

        for (player_id, player_data) in &self.players {
            let js_rating = match &self.system {
                RatingSystemImpl::Elo(_) => {
                    if let Some(rating) = &player_data.elo_rating {
                        WasmRating {
                            player_id: player_id.clone(),
                            rating: rating.mean(),
                            uncertainty: None,
                            volatility: None,
                        }
                    } else {
                        continue;
                    }
                }
                RatingSystemImpl::Glicko(_) => {
                    if let Some(rating) = &player_data.glicko_rating {
                        WasmRating {
                            player_id: player_id.clone(),
                            rating: rating.mu,
                            uncertainty: Some(rating.rd),
                            volatility: None,
                        }
                    } else {
                        continue;
                    }
                }
                RatingSystemImpl::TrueSkill(_) => {
                    if let Some(rating) = &player_data.trueskill_rating {
                        WasmRating {
                            player_id: player_id.clone(),
                            rating: rating.mean(),
                            uncertainty: Some(rating.std_dev()),
                            volatility: None,
                        }
                    } else {
                        continue;
                    }
                }
            };
            leaderboard.push(js_rating);
        }

        // Sort by rating descending
        leaderboard.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());

        Ok(leaderboard)
    }
}

// Private helper methods
impl WasmRatingSystem {
    fn update_elo_ratings(
        &mut self,
        teams: &[WasmTeam],
        outcome: &GameOutcome,
    ) -> Result<Vec<WasmTeam>, JsValue> {
        let elo = match &self.system {
            RatingSystemImpl::Elo(elo) => elo,
            _ => unreachable!(),
        };
        // Elo only supports 1v1
        if teams.len() != 2 || teams[0].players.len() != 1 || teams[1].players.len() != 1 {
            return Err(js_error("Elo only supports 1v1 matches"));
        }

        let player1_id = teams[0].players[0].player_id.clone();
        let player2_id = teams[1].players[0].player_id.clone();

        // Get or create ratings
        let rating1 = self
            .players
            .get(&player1_id)
            .and_then(|p| p.elo_rating.as_ref())
            .map(|r| r.clone())
            .unwrap_or_else(|| elo.create_rating());

        let rating2 = self
            .players
            .get(&player2_id)
            .and_then(|p| p.elo_rating.as_ref())
            .map(|r| r.clone())
            .unwrap_or_else(|| elo.create_rating());

        // Create teams
        let team1 = EloTeamRating::new(rating1);
        let team2 = EloTeamRating::new(rating2);

        // Update ratings
        let updated = elo
            .rate(&[team1, team2], outcome)
            .map_err(|e| js_error(&format!("Rating update failed: {}", e)))?;

        // Extract updated ratings
        let updated_rating1 = &updated[0].player_ratings()[0];
        let updated_rating2 = &updated[1].player_ratings()[0];

        // Update stored ratings
        self.players
            .entry(player1_id.clone())
            .or_insert(PlayerData {
                elo_rating: None,
                glicko_rating: None,
                trueskill_rating: None,
            })
            .elo_rating = Some(updated_rating1.clone());

        self.players
            .entry(player2_id.clone())
            .or_insert(PlayerData {
                elo_rating: None,
                glicko_rating: None,
                trueskill_rating: None,
            })
            .elo_rating = Some(updated_rating2.clone());

        // Create result teams
        let mut result_teams = Vec::new();

        let mut team1_result = WasmTeam::new(teams[0].score);
        team1_result.players.push(WasmRating {
            player_id: player1_id,
            rating: updated_rating1.mean(),
            uncertainty: None,
            volatility: None,
        });
        result_teams.push(team1_result);

        let mut team2_result = WasmTeam::new(teams[1].score);
        team2_result.players.push(WasmRating {
            player_id: player2_id,
            rating: updated_rating2.mean(),
            uncertainty: None,
            volatility: None,
        });
        result_teams.push(team2_result);

        Ok(result_teams)
    }

    fn update_glicko_ratings(
        &mut self,
        teams: &[WasmTeam],
        outcome: &GameOutcome,
    ) -> Result<Vec<WasmTeam>, JsValue> {
        let glicko = match &self.system {
            RatingSystemImpl::Glicko(glicko) => glicko,
            _ => unreachable!(),
        };
        // Glicko only supports 1v1
        if teams.len() != 2 || teams[0].players.len() != 1 || teams[1].players.len() != 1 {
            return Err(js_error("Glicko only supports 1v1 matches"));
        }

        let player1_id = teams[0].players[0].player_id.clone();
        let player2_id = teams[1].players[0].player_id.clone();

        // Get or create ratings
        let rating1 = self
            .players
            .get(&player1_id)
            .and_then(|p| p.glicko_rating.as_ref())
            .map(|r| r.clone())
            .unwrap_or_else(|| glicko.create_rating());

        let rating2 = self
            .players
            .get(&player2_id)
            .and_then(|p| p.glicko_rating.as_ref())
            .map(|r| r.clone())
            .unwrap_or_else(|| glicko.create_rating());

        // Create teams
        let team1 = <GlickoTeamRating as TeamRatingTrait>::from_player_ratings(vec![rating1]);
        let team2 = <GlickoTeamRating as TeamRatingTrait>::from_player_ratings(vec![rating2]);

        // Update ratings
        let updated = glicko
            .rate(&[team1, team2], outcome)
            .map_err(|e| js_error(&format!("Rating update failed: {}", e)))?;

        // Extract updated ratings
        let updated_rating1 = &updated[0].player_ratings()[0];
        let updated_rating2 = &updated[1].player_ratings()[0];

        // Update stored ratings
        self.players
            .entry(player1_id.clone())
            .or_insert(PlayerData {
                elo_rating: None,
                glicko_rating: None,
                trueskill_rating: None,
            })
            .glicko_rating = Some(updated_rating1.clone());

        self.players
            .entry(player2_id.clone())
            .or_insert(PlayerData {
                elo_rating: None,
                glicko_rating: None,
                trueskill_rating: None,
            })
            .glicko_rating = Some(updated_rating2.clone());

        // Create result teams
        let mut result_teams = Vec::new();

        let mut team1_result = WasmTeam::new(teams[0].score);
        team1_result.players.push(WasmRating {
            player_id: player1_id,
            rating: updated_rating1.mu,
            uncertainty: Some(updated_rating1.rd),
            volatility: None,
        });
        result_teams.push(team1_result);

        let mut team2_result = WasmTeam::new(teams[1].score);
        team2_result.players.push(WasmRating {
            player_id: player2_id,
            rating: updated_rating2.mu,
            uncertainty: Some(updated_rating2.rd),
            volatility: None,
        });
        result_teams.push(team2_result);

        Ok(result_teams)
    }

    fn update_trueskill_ratings(
        &mut self,
        teams: &[WasmTeam],
        outcome: &GameOutcome,
    ) -> Result<Vec<WasmTeam>, JsValue> {
        let trueskill = match &self.system {
            RatingSystemImpl::TrueSkill(trueskill) => trueskill,
            _ => unreachable!(),
        };
        let mut ts_teams = Vec::new();
        let mut player_ids: Vec<Vec<String>> = Vec::new();

        // Convert to TrueSkill teams
        for team in teams {
            let mut ts_players = Vec::new();
            let mut team_player_ids = Vec::new();

            for player in &team.players {
                team_player_ids.push(player.player_id.clone());

                let rating = self
                    .players
                    .get(&player.player_id)
                    .and_then(|p| p.trueskill_rating.as_ref())
                    .map(|r| r.clone())
                    .unwrap_or_else(|| trueskill.create_rating());

                ts_players.push(rating);
            }

            ts_teams.push(TrueSkillTeam::from_player_ratings(ts_players));
            player_ids.push(team_player_ids);
        }

        // Update ratings
        let updated = trueskill
            .rate(&ts_teams, outcome)
            .map_err(|e| js_error(&format!("Rating update failed: {}", e)))?;

        // Convert back and store
        let mut result_teams = Vec::new();
        for (team_idx, updated_team) in updated.iter().enumerate() {
            let mut js_team = WasmTeam::new(teams[team_idx].score);

            for (player_idx, rating) in updated_team.player_ratings().iter().enumerate() {
                let player_id: String = player_ids[team_idx][player_idx].clone();

                // Update stored rating
                self.players
                    .entry(player_id.clone())
                    .or_insert(PlayerData {
                        elo_rating: None,
                        glicko_rating: None,
                        trueskill_rating: None,
                    })
                    .trueskill_rating = Some(rating.clone());

                js_team.players.push(WasmRating {
                    player_id,
                    rating: rating.mean(),
                    uncertainty: Some(rating.std_dev()),
                    volatility: None,
                });
            }
            result_teams.push(js_team);
        }

        Ok(result_teams)
    }
}
