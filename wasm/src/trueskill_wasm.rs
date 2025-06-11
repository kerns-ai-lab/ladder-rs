//! WASM bindings for the TrueSkill rating system
//!
//! This module provides JavaScript-friendly wrappers around the core TrueSkill
//! rating system, with optimizations for WASM usage.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use ladder_rs::trueskill::{TrueSkillRating as CoreTrueSkillRating, TrueSkill as CoreTrueSkillSystem, TrueSkillTeam as CoreTrueSkillTeam};
use ladder_rs::core::{GameOutcome, Outcome, RatingSystem, TeamRating};

/// WASM-friendly TrueSkill rating wrapper
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrueSkillRating {
    mean: f64,
    variance: f64,
}

#[wasm_bindgen]
impl TrueSkillRating {
    /// Creates a new TrueSkill rating with the specified values
    #[wasm_bindgen(constructor)]
    pub fn new(mean: f64, variance: f64) -> Result<TrueSkillRating, JsValue> {
        if variance <= 0.0 {
            return Err(JsValue::from_str("Variance must be positive"));
        }
        Ok(Self { mean, variance })
    }

    /// Gets the rating mean (μ)
    #[wasm_bindgen(getter)]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Gets the rating variance (σ²)
    #[wasm_bindgen(getter)]
    pub fn variance(&self) -> f64 {
        self.variance
    }

    /// Gets the standard deviation (σ)
    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }

    /// Gets the conservative rating (μ - 3σ)
    pub fn conservative_rating(&self) -> f64 {
        self.mean - 3.0 * self.std_dev()
    }

    /// Serializes the rating to a string
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserializes a rating from a string
    pub fn deserialize(data: &str) -> Result<TrueSkillRating, JsValue> {
        serde_json::from_str(data)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))
    }

    /// Converts to JSON string for JavaScript interop
    pub fn to_json(&self) -> String {
        self.serialize()
    }

    /// Creates from JSON string
    pub fn from_json(json: &str) -> Result<TrueSkillRating, JsValue> {
        Self::deserialize(json)
    }
}

/// WASM-friendly TrueSkill team wrapper
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct TrueSkillTeam {
    ratings: Vec<TrueSkillRating>,
    weights: Option<Vec<f64>>,
}

#[wasm_bindgen]
impl TrueSkillTeam {
    /// Creates a team from player ratings
    pub fn from_ratings(ratings: Vec<TrueSkillRating>) -> Result<TrueSkillTeam, JsValue> {
        if ratings.is_empty() {
            return Err(JsValue::from_str("Team must have at least one player"));
        }
        Ok(Self {
            ratings,
            weights: None,
        })
    }

    /// Creates a team with partial play weights
    pub fn from_ratings_with_weights(
        ratings: Vec<TrueSkillRating>,
        weights: Vec<f64>,
    ) -> Result<TrueSkillTeam, JsValue> {
        if ratings.is_empty() {
            return Err(JsValue::from_str("Team must have at least one player"));
        }
        if ratings.len() != weights.len() {
            return Err(JsValue::from_str("Ratings and weights must have same length"));
        }
        if weights.iter().any(|&w| w < 0.0 || w > 1.0) {
            return Err(JsValue::from_str("Weights must be between 0 and 1"));
        }
        Ok(Self {
            ratings,
            weights: Some(weights),
        })
    }

    /// Gets the number of players in the team
    pub fn size(&self) -> usize {
        self.ratings.len()
    }

    /// Gets the sum of all player means
    pub fn mean_sum(&self) -> f64 {
        self.ratings.iter().map(|r| r.mean).sum()
    }

    /// Gets the sum of all player variances
    pub fn variance_sum(&self) -> f64 {
        self.ratings.iter().map(|r| r.variance).sum()
    }

    /// Gets a copy of the ratings
    pub fn ratings(&self) -> Vec<TrueSkillRating> {
        self.ratings.clone()
    }

    /// Converts to JSON for serialization
    pub fn to_json(&self) -> Result<String, JsValue> {
        #[derive(Serialize)]
        struct TeamData {
            ratings: Vec<TrueSkillRating>,
            weights: Option<Vec<f64>>,
        }
        
        let data = TeamData {
            ratings: self.ratings.clone(),
            weights: self.weights.clone(),
        };
        
        serde_json::to_string(&data)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

/// WASM-friendly TrueSkill system wrapper
#[wasm_bindgen]
pub struct TrueSkillSystem {
    mu: f64,
    sigma: f64,
    beta: f64,
    tau: f64,
    draw_probability: f64,
    inner: CoreTrueSkillSystem,
}

#[wasm_bindgen]
impl TrueSkillSystem {
    /// Creates a new TrueSkill system with default parameters
    /// Default: mu = 25.0, sigma = 8.333, beta = 4.166, tau = 0.0833, draw_prob = 0.1
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let inner = CoreTrueSkillSystem::new();
        Self {
            mu: 25.0,
            sigma: 25.0 / 3.0,
            beta: 25.0 / 6.0,
            tau: 25.0 / 300.0,
            draw_probability: 0.1,
            inner,
        }
    }

    /// Creates a new TrueSkill system with custom parameters
    pub fn with_parameters(
        mu: f64,
        sigma: f64,
        beta: f64,
        tau: f64,
        draw_probability: f64,
    ) -> Result<TrueSkillSystem, JsValue> {
        if sigma <= 0.0 {
            return Err(JsValue::from_str("Sigma must be positive"));
        }
        if beta <= 0.0 {
            return Err(JsValue::from_str("Beta must be positive"));
        }
        if tau < 0.0 {
            return Err(JsValue::from_str("Tau must be non-negative"));
        }
        if draw_probability < 0.0 || draw_probability > 1.0 {
            return Err(JsValue::from_str("Draw probability must be between 0 and 1"));
        }

        // Build custom TrueSkill system
        let inner = CoreTrueSkillSystem::builder()
            .initial_mean(mu)
            .initial_variance(sigma * sigma)
            .performance_variance(beta * beta)
            .dynamics_factor(tau * tau)
            .draw_probability(draw_probability)
            .build();

        Ok(Self {
            mu,
            sigma,
            beta,
            tau,
            draw_probability,
            inner,
        })
    }

    /// Gets the initial mean (μ)
    #[wasm_bindgen(getter)]
    pub fn mu(&self) -> f64 {
        self.mu
    }

    /// Gets the initial standard deviation (σ)
    #[wasm_bindgen(getter)]
    pub fn sigma(&self) -> f64 {
        self.sigma
    }

    /// Gets the performance variance parameter (β)
    #[wasm_bindgen(getter)]
    pub fn beta(&self) -> f64 {
        self.beta
    }

    /// Gets the dynamics factor (τ)
    #[wasm_bindgen(getter)]
    pub fn tau(&self) -> f64 {
        self.tau
    }

    /// Gets the draw probability
    #[wasm_bindgen(getter)]
    pub fn draw_probability(&self) -> f64 {
        self.draw_probability
    }

    /// Gets the calculated draw margin
    pub fn draw_margin(&self) -> f64 {
        self.inner.draw_margin()
    }

    /// Creates a new rating with the default values
    pub fn create_rating(&self) -> TrueSkillRating {
        TrueSkillRating {
            mean: self.mu,
            variance: self.sigma * self.sigma,
        }
    }

    /// Creates a rating with specific values
    pub fn create_rating_with_values(&self, mean: f64, variance: f64) -> Result<TrueSkillRating, JsValue> {
        TrueSkillRating::new(mean, variance)
    }

    /// Processes a match and returns updated teams
    /// ranks: array where ranks[i] is the rank of team i (1 = first place, 2 = second, etc.)
    pub fn process_match(
        &self,
        teams: &js_sys::Array,
        ranks: &js_sys::Array,
    ) -> Result<js_sys::Array, JsValue> {
        // Convert JS arrays to Rust types
        let mut rust_teams = Vec::new();
        let mut rust_ranks = Vec::new();

        // Process teams
        for i in 0..teams.length() {
            let team = teams.get(i);
            let trueskill_team = team.dyn_into::<TrueSkillTeam>()
                .map_err(|_| JsValue::from_str("Invalid team in array"))?;
            
            // Convert to core team
            let mut core_ratings = Vec::new();
            for rating in &trueskill_team.ratings {
                let core_rating = CoreTrueSkillRating::new(rating.mean, rating.variance)
                    .map_err(|e| JsValue::from_str(&format!("Invalid rating: {}", e)))?;
                core_ratings.push(core_rating);
            }
            
            let core_team = CoreTrueSkillTeam::from_player_ratings(core_ratings);
            rust_teams.push(core_team);
        }

        // Process ranks
        for i in 0..ranks.length() {
            let rank = ranks.get(i).as_f64()
                .ok_or_else(|| JsValue::from_str("Rank must be a number"))?;
            rust_ranks.push(rank as u32);
        }

        // Create game outcome
        let outcome = GameOutcome::from_ranks(&rust_ranks)
            .map_err(|e| JsValue::from_str(&format!("Invalid ranks: {}", e)))?;

        // Process the match
        let results = self.inner
            .rate(&rust_teams, &outcome)
            .map_err(|e| JsValue::from_str(&format!("Rating error: {}", e)))?;

        // Convert results back to JS
        let js_results = js_sys::Array::new();
        for team_result in results {
            let mut ratings = Vec::new();
            for player_rating in team_result.player_ratings() {
                ratings.push(TrueSkillRating {
                    mean: player_rating.mean,
                    variance: player_rating.variance,
                });
            }
            
            let result_team = TrueSkillTeam {
                ratings,
                weights: None, // Weights don't change
            };
            js_results.push(&JsValue::from(result_team));
        }

        Ok(js_results)
    }

    /// Calculates win probabilities for each team
    pub fn win_probability(&self, teams: &js_sys::Array) -> Result<js_sys::Array, JsValue> {
        // Convert teams and calculate match quality
        let mut rust_teams = Vec::new();
        
        for i in 0..teams.length() {
            let team = teams.get(i);
            let trueskill_team = team.dyn_into::<TrueSkillTeam>()
                .map_err(|_| JsValue::from_str("Invalid team in array"))?;
            
            let mut core_ratings = Vec::new();
            for rating in &trueskill_team.ratings {
                let core_rating = CoreTrueSkillRating::new(rating.mean, rating.variance)
                    .map_err(|e| JsValue::from_str(&format!("Invalid rating: {}", e)))?;
                core_ratings.push(core_rating);
            }
            
            let core_team = CoreTrueSkillTeam::from_player_ratings(core_ratings);
            rust_teams.push(core_team);
        }

        // For 2-team matches, calculate win probability
        if rust_teams.len() == 2 {
            let quality = self.inner.calculate_match_quality(&rust_teams)
                .map_err(|e| JsValue::from_str(&format!("Quality calculation error: {}", e)))?;
            
            // Approximate win probabilities based on team strengths
            let team1_strength: f64 = rust_teams[0].player_ratings().iter().map(|r| r.mean).sum();
            let team2_strength: f64 = rust_teams[1].player_ratings().iter().map(|r| r.mean).sum();
            
            let total_variance: f64 = rust_teams[0].player_ratings().iter().map(|r| r.variance).sum::<f64>()
                + rust_teams[1].player_ratings().iter().map(|r| r.variance).sum::<f64>()
                + 2.0 * self.beta * self.beta * rust_teams.len() as f64;
            
            let strength_diff = team1_strength - team2_strength;
            let win_prob = 1.0 / (1.0 + (-strength_diff / total_variance.sqrt()).exp());
            
            let probs = js_sys::Array::new();
            probs.push(&JsValue::from(win_prob));
            probs.push(&JsValue::from(1.0 - win_prob));
            
            Ok(probs)
        } else {
            // For multi-team matches, return equal probabilities as approximation
            let prob = 1.0 / rust_teams.len() as f64;
            let probs = js_sys::Array::new();
            for _ in 0..rust_teams.len() {
                probs.push(&JsValue::from(prob));
            }
            Ok(probs)
        }
    }

    /// Calculates match quality (0-1, higher is better)
    pub fn match_quality(&self, teams: &js_sys::Array) -> Result<f64, JsValue> {
        // Convert teams
        let mut rust_teams = Vec::new();
        
        for i in 0..teams.length() {
            let team = teams.get(i);
            let trueskill_team = team.dyn_into::<TrueSkillTeam>()
                .map_err(|_| JsValue::from_str("Invalid team in array"))?;
            
            let mut core_ratings = Vec::new();
            for rating in &trueskill_team.ratings {
                let core_rating = CoreTrueSkillRating::new(rating.mean, rating.variance)
                    .map_err(|e| JsValue::from_str(&format!("Invalid rating: {}", e)))?;
                core_ratings.push(core_rating);
            }
            
            let core_team = CoreTrueSkillTeam::from_player_ratings(core_ratings);
            rust_teams.push(core_team);
        }

        self.inner
            .calculate_match_quality(&rust_teams)
            .map_err(|e| JsValue::from_str(&format!("Quality calculation error: {}", e)))
    }

    /// Serializes the system configuration
    pub fn serialize(&self) -> String {
        serde_json::to_string(&SystemConfig {
            mu: self.mu,
            sigma: self.sigma,
            beta: self.beta,
            tau: self.tau,
            draw_probability: self.draw_probability,
        })
        .unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserializes system configuration
    pub fn deserialize(data: &str) -> Result<TrueSkillSystem, JsValue> {
        let config: SystemConfig = serde_json::from_str(data)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        Self::with_parameters(config.mu, config.sigma, config.beta, config.tau, config.draw_probability)
    }
}

#[derive(Serialize, Deserialize)]
struct SystemConfig {
    mu: f64,
    sigma: f64,
    beta: f64,
    tau: f64,
    draw_probability: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trueskill_rating_creation() {
        let rating = TrueSkillRating::new(25.0, 64.0).unwrap();
        assert_eq!(rating.mean(), 25.0);
        assert_eq!(rating.variance(), 64.0);
        assert_eq!(rating.std_dev(), 8.0);
        assert_eq!(rating.conservative_rating(), 1.0);
    }

    #[test]
    fn test_trueskill_team_creation() {
        let r1 = TrueSkillRating::new(25.0, 64.0).unwrap();
        let r2 = TrueSkillRating::new(30.0, 36.0).unwrap();
        let team = TrueSkillTeam::from_ratings(vec![r1, r2]).unwrap();
        assert_eq!(team.size(), 2);
        assert_eq!(team.mean_sum(), 55.0);
        assert_eq!(team.variance_sum(), 100.0);
    }

    #[test]
    fn test_trueskill_system_defaults() {
        let system = TrueSkillSystem::new();
        assert_eq!(system.mu(), 25.0);
        assert!((system.sigma() - 8.333).abs() < 0.001);
        assert!((system.beta() - 4.166).abs() < 0.001);
        assert!((system.tau() - 0.0833).abs() < 0.001);
        assert_eq!(system.draw_probability(), 0.1);
    }

    #[test]
    fn test_serialization() {
        let rating = TrueSkillRating::new(30.0, 100.0).unwrap();
        let serialized = rating.serialize();
        let deserialized = TrueSkillRating::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.mean(), 30.0);
        assert_eq!(deserialized.variance(), 100.0);
    }
}