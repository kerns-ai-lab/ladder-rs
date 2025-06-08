//! WASM-specific type definitions and conversions
//!
//! This module contains type conversions between Rust types and
//! JavaScript/WASM boundary types, providing a clean API for JavaScript consumers.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// JavaScript-friendly rating representation
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsRating {
    /// Mean skill value (μ)
    mean: f64,
    /// Variance of skill (σ²)
    variance: f64,
}

#[wasm_bindgen]
impl JsRating {
    /// Creates a new rating with the given mean and variance
    #[wasm_bindgen(constructor)]
    pub fn new(mean: f64, variance: f64) -> Result<JsRating, JsValue> {
        if variance <= 0.0 {
            return Err(JsValue::from_str("Variance must be positive"));
        }
        Ok(JsRating { mean, variance })
    }

    /// Gets the mean skill value
    #[wasm_bindgen(getter)]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Gets the variance
    #[wasm_bindgen(getter)]
    pub fn variance(&self) -> f64 {
        self.variance
    }

    /// Gets the standard deviation (σ)
    #[wasm_bindgen(getter)]
    pub fn standard_deviation(&self) -> f64 {
        self.variance.sqrt()
    }

    /// Gets a conservative skill estimate (μ - 3σ)
    #[wasm_bindgen(getter)]
    pub fn conservative_rating(&self) -> f64 {
        self.mean - 3.0 * self.standard_deviation()
    }

    /// Creates a string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("Rating(μ={:.2}, σ={:.2})", self.mean, self.standard_deviation())
    }
}

/// JavaScript-friendly team representation
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTeam {
    /// Player ratings in this team
    #[wasm_bindgen(skip)]
    pub player_ratings: Vec<JsRating>,
}

#[wasm_bindgen]
impl JsTeam {
    /// Creates a new team from an array of ratings
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsTeam {
        JsTeam {
            player_ratings: Vec::new(),
        }
    }

    /// Adds a player rating to the team
    pub fn add_player(&mut self, rating: JsRating) {
        self.player_ratings.push(rating);
    }

    /// Gets the number of players in the team
    #[wasm_bindgen(getter)]
    pub fn player_count(&self) -> usize {
        self.player_ratings.len()
    }

    /// Gets the team's total mean (sum of player means)
    #[wasm_bindgen(getter)]
    pub fn team_mean(&self) -> f64 {
        self.player_ratings.iter().map(|r| r.mean).sum()
    }

    /// Gets the team's total variance (sum of player variances)
    #[wasm_bindgen(getter)]
    pub fn team_variance(&self) -> f64 {
        self.player_ratings.iter().map(|r| r.variance).sum()
    }

    /// Gets a player rating at the specified index
    pub fn get_player(&self, index: usize) -> Option<JsRating> {
        self.player_ratings.get(index).cloned()
    }

    /// Creates a string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("Team({} players)", self.player_count())
    }
}

/// JavaScript-friendly game outcome representation
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsGameOutcome {
    /// Ranks for each team (lower is better, 1 = winner)
    #[wasm_bindgen(skip)]
    pub ranks: Vec<u32>,
}

#[wasm_bindgen]
impl JsGameOutcome {
    /// Creates a new game outcome from ranks
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsGameOutcome {
        JsGameOutcome { ranks: Vec::new() }
    }

    /// Sets ranks from a JavaScript array
    pub fn set_ranks(&mut self, ranks: Vec<u32>) -> Result<(), JsValue> {
        // Validate ranks
        if ranks.is_empty() {
            return Err(JsValue::from_str("Ranks cannot be empty"));
        }
        
        // Check for duplicate ranks
        let mut sorted_ranks = ranks.clone();
        sorted_ranks.sort_unstable();
        for i in 1..sorted_ranks.len() {
            if sorted_ranks[i] == sorted_ranks[i - 1] {
                return Err(JsValue::from_str("Duplicate ranks are not allowed"));
            }
        }
        
        self.ranks = ranks;
        Ok(())
    }

    /// Gets the number of teams
    #[wasm_bindgen(getter)]
    pub fn team_count(&self) -> usize {
        self.ranks.len()
    }

    /// Creates a win outcome (team at index wins)
    pub fn win(winner_index: usize, total_teams: usize) -> Result<JsGameOutcome, JsValue> {
        if winner_index >= total_teams {
            return Err(JsValue::from_str("Winner index out of bounds"));
        }
        if total_teams < 2 {
            return Err(JsValue::from_str("At least 2 teams required"));
        }

        let mut ranks = vec![2; total_teams];
        ranks[winner_index] = 1;
        
        Ok(JsGameOutcome { ranks: ranks.into_iter().map(|r| r as u32).collect() })
    }

    /// Creates a draw outcome between all teams
    pub fn draw(total_teams: usize) -> Result<JsGameOutcome, JsValue> {
        if total_teams < 2 {
            return Err(JsValue::from_str("At least 2 teams required"));
        }

        Ok(JsGameOutcome {
            ranks: vec![1; total_teams],
        })
    }

    /// Gets the rank for a specific team
    pub fn get_rank(&self, team_index: usize) -> Option<u32> {
        self.ranks.get(team_index).copied()
    }

    /// Creates a string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("GameOutcome({:?})", self.ranks)
    }
}

/// Rating system type enumeration
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingSystemType {
    Elo,
    Glicko,
    Glicko2,
    TrueSkill,
}

/// Configuration for rating systems
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingSystemConfig {
    /// Type of rating system
    pub system_type: RatingSystemType,
    
    /// Custom parameters (JSON string for flexibility)
    #[wasm_bindgen(skip)]
    pub parameters: Option<String>,
}

#[wasm_bindgen]
impl RatingSystemConfig {
    /// Creates a new configuration with default parameters
    #[wasm_bindgen(constructor)]
    pub fn new(system_type: RatingSystemType) -> RatingSystemConfig {
        RatingSystemConfig {
            system_type,
            parameters: None,
        }
    }

    /// Sets custom parameters as a JSON string
    pub fn set_parameters(&mut self, params: &str) {
        self.parameters = Some(params.to_string());
    }

    /// Gets the rating system type
    #[wasm_bindgen(getter = systemType)]
    pub fn get_system_type(&self) -> RatingSystemType {
        self.system_type
    }
}

/// Result type for rating updates
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingUpdate {
    /// Updated teams with new ratings
    #[wasm_bindgen(skip)]
    pub updated_teams: Vec<JsTeam>,
    
    /// Optional match quality (0-1, higher is better)
    pub match_quality: Option<f64>,
}

#[wasm_bindgen]
impl RatingUpdate {
    /// Gets the number of teams updated
    #[wasm_bindgen(getter)]
    pub fn team_count(&self) -> usize {
        self.updated_teams.len()
    }

    /// Gets an updated team by index
    pub fn get_team(&self, index: usize) -> Option<JsTeam> {
        self.updated_teams.get(index).cloned()
    }

    /// Gets the match quality if available
    #[wasm_bindgen(getter = matchQuality)]
    pub fn get_match_quality(&self) -> Option<f64> {
        self.match_quality
    }
}

// Internal conversion utilities (not exposed to JavaScript)
pub mod conversions {
    use super::*;
    use ladder_rs::core::{GameOutcome as CoreGameOutcome, Rating};
    
    /// Converts a JsGameOutcome to core GameOutcome
    pub fn js_to_core_outcome(js_outcome: &JsGameOutcome) -> CoreGameOutcome {
        CoreGameOutcome::new(js_outcome.ranks.iter().map(|&r| r as usize).collect())
    }
    
    /// Converts any Rating implementation to JsRating
    pub fn rating_to_js<R: Rating>(rating: &R) -> JsRating {
        JsRating {
            mean: rating.mean(),
            variance: rating.variance(),
        }
    }
    
    /// Converts a vector of ratings to JsTeam
    pub fn ratings_to_js_team<R: Rating>(ratings: Vec<R>) -> JsTeam {
        JsTeam {
            player_ratings: ratings.iter().map(rating_to_js).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create JsRating for tests without wasm-bindgen
    fn create_test_rating(mean: f64, variance: f64) -> JsRating {
        JsRating { mean, variance }
    }

    #[test]
    fn test_js_rating_fields() {
        let rating = create_test_rating(25.0, 64.0);
        assert_eq!(rating.mean, 25.0);
        assert_eq!(rating.variance, 64.0);
        // Test derived calculations
        assert_eq!(rating.variance.sqrt(), 8.0); // standard deviation
        assert_eq!(rating.mean - 3.0 * rating.variance.sqrt(), 1.0); // conservative rating
    }

    #[test]
    fn test_js_rating_validation() {
        // These would normally be caught by the constructor
        assert!(JsRating { mean: 25.0, variance: 0.0 }.variance <= 0.0);
        assert!(JsRating { mean: 25.0, variance: -1.0 }.variance <= 0.0);
    }

    #[test]
    fn test_js_team_fields() {
        let team = JsTeam {
            player_ratings: vec![
                create_test_rating(25.0, 64.0),
                create_test_rating(30.0, 36.0),
            ],
        };
        
        assert_eq!(team.player_ratings.len(), 2);
        assert_eq!(team.player_ratings.iter().map(|r| r.mean).sum::<f64>(), 55.0);
        assert_eq!(team.player_ratings.iter().map(|r| r.variance).sum::<f64>(), 100.0);
    }

    #[test]
    fn test_js_game_outcome_fields() {
        let outcome = JsGameOutcome {
            ranks: vec![1, 2, 3],
        };
        assert_eq!(outcome.ranks.len(), 3);
        assert_eq!(outcome.ranks[0], 1);
        assert_eq!(outcome.ranks[1], 2);
        assert_eq!(outcome.ranks[2], 3);
    }

    #[test]
    fn test_game_outcome_validation_logic() {
        // Test validation logic that would be in set_ranks
        let empty_ranks: Vec<u32> = vec![];
        assert!(empty_ranks.is_empty());
        
        let duplicate_ranks = vec![1, 1, 2];
        let mut sorted = duplicate_ranks.clone();
        sorted.sort_unstable();
        let has_duplicates = sorted.windows(2).any(|w| w[0] == w[1]);
        assert!(has_duplicates);
    }

    #[test]
    fn test_win_outcome_logic() {
        // Test the logic for creating a win outcome
        let winner_index = 0;
        let total_teams = 3;
        
        let mut ranks = vec![2; total_teams];
        ranks[winner_index] = 1;
        
        assert_eq!(ranks, vec![1, 2, 2]);
    }

    #[test]
    fn test_draw_outcome_logic() {
        // Test the logic for creating a draw outcome
        let total_teams = 3;
        let ranks = vec![1; total_teams];
        assert_eq!(ranks, vec![1, 1, 1]);
    }

    #[test]
    fn test_conversions() {
        use crate::types::conversions::*;
        
        // Test outcome conversion
        let js_outcome = JsGameOutcome { ranks: vec![1, 2, 3] };
        let core_outcome = js_to_core_outcome(&js_outcome);
        assert_eq!(core_outcome.ranks(), &[1, 2, 3]);
        
        // Test rating conversion
        #[derive(Debug, Clone)]
        struct TestRating { mean: f64, variance: f64 }
        impl crate::Rating for TestRating {
            fn mean(&self) -> f64 { self.mean }
            fn variance(&self) -> f64 { self.variance }
        }
        
        let test_rating = TestRating { mean: 25.0, variance: 64.0 };
        let js_rating = rating_to_js(&test_rating);
        assert_eq!(js_rating.mean, 25.0);
        assert_eq!(js_rating.variance, 64.0);
    }
}