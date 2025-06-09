//! Core JavaScript interface types
//!
//! Provides idiomatic JavaScript interfaces for core rating system types.

use wasm_bindgen::prelude::*;
use js_sys::*;
use std::collections::HashMap;

/// JavaScript-friendly rating interface with idiomatic JS patterns
#[wasm_bindgen(js_name = "Rating")]
#[derive(Clone)]
pub struct JsRatingInterface {
    mean: f64,
    variance: f64,
}

#[wasm_bindgen(js_class = "Rating")]
impl JsRatingInterface {
    /// Creates a new rating with validation
    #[wasm_bindgen(constructor)]
    pub fn new(mean: f64, variance: f64) -> Result<JsRatingInterface, JsValue> {
        if variance <= 0.0 {
            return Err(JsValue::from_str("Variance must be positive"));
        }
        Ok(JsRatingInterface { mean, variance })
    }
    
    /// Gets the mean value (JavaScript property style)
    #[wasm_bindgen(getter)]
    pub fn mean(&self) -> f64 {
        self.mean
    }
    
    /// Gets the variance value (JavaScript property style)
    #[wasm_bindgen(getter)]
    pub fn variance(&self) -> f64 {
        self.variance
    }
    
    /// Gets the standard deviation (JavaScript property style)
    #[wasm_bindgen(getter, js_name = "standardDeviation")]
    pub fn standard_deviation(&self) -> f64 {
        self.variance.sqrt()
    }
    
    /// Gets conservative rating (JavaScript property style)
    #[wasm_bindgen(getter, js_name = "conservativeRating")]
    pub fn conservative_rating(&self) -> f64 {
        self.mean - 3.0 * self.standard_deviation()
    }
    
    // Explicit getter methods for backwards compatibility
    #[wasm_bindgen(js_name = "getMean")]
    pub fn get_mean(&self) -> f64 {
        self.mean()
    }
    
    #[wasm_bindgen(js_name = "getVariance")]
    pub fn get_variance(&self) -> f64 {
        self.variance()
    }
    
    #[wasm_bindgen(js_name = "getStandardDeviation")]
    pub fn get_standard_deviation(&self) -> f64 {
        self.standard_deviation()
    }
    
    #[wasm_bindgen(js_name = "getConservativeRating")]
    pub fn get_conservative_rating(&self) -> f64 {
        self.conservative_rating()
    }
    
    /// Fluent API: Adjust mean value
    #[wasm_bindgen(js_name = "adjustMean")]
    pub fn adjust_mean(&self, delta: f64) -> JsRatingInterface {
        let new_mean = self.mean + delta;
        JsRatingInterface {
            mean: new_mean,
            variance: self.variance,
        }
    }
    
    /// Fluent API: Adjust variance value
    #[wasm_bindgen(js_name = "adjustVariance")]
    pub fn adjust_variance(&self, delta: f64) -> JsRatingInterface {
        let new_variance = (self.variance + delta).max(0.001); // Ensure positive
        JsRatingInterface {
            mean: self.mean,
            variance: new_variance,
        }
    }
    
    /// Fluent API: Normalize rating to standard bounds
    pub fn normalize(&self) -> JsRatingInterface {
        let normalized_mean = self.mean.clamp(0.0, 3000.0);
        let normalized_variance = self.variance.clamp(0.001, 1000.0);
        JsRatingInterface {
            mean: normalized_mean,
            variance: normalized_variance,
        }
    }
    
    /// Convert to JSON string
    #[wasm_bindgen(js_name = "toJSON")]
    pub fn to_json(&self) -> String {
        format!("{{\"mean\":{},\"variance\":{}}}", self.mean, self.variance)
    }
    
    /// Create from JSON string
    #[wasm_bindgen(js_name = "fromJSON")]
    pub fn from_json(json: &str) -> Result<JsRatingInterface, JsValue> {
        // Simple JSON parsing for our specific format
        if let (Some(mean_start), Some(mean_end), Some(var_start), Some(var_end)) = (
            json.find("\"mean\":"),
            json.find(",\"variance\""),
            json.find("\"variance\":"),
            json.rfind("}"),
        ) {
            let mean_str = &json[mean_start + 7..mean_end];
            let var_str = &json[var_start + 11..var_end];
            
            if let (Ok(mean), Ok(variance)) = (mean_str.parse::<f64>(), var_str.parse::<f64>()) {
                return Self::new(mean, variance);
            }
        }
        Err(JsValue::from_str("Invalid JSON format"))
    }
    
    /// Convert to binary representation (for efficient storage/transfer)
    #[wasm_bindgen(js_name = "toBinary")]
    pub fn to_binary(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.mean.to_le_bytes());
        bytes.extend_from_slice(&self.variance.to_le_bytes());
        bytes
    }
    
    /// Create from binary representation
    #[wasm_bindgen(js_name = "fromBinary")]
    pub fn from_binary(bytes: &[u8]) -> Result<JsRatingInterface, JsValue> {
        if bytes.len() != 16 {
            return Err(JsValue::from_str("Invalid binary data length"));
        }
        
        let mean_bytes: [u8; 8] = bytes[0..8].try_into()
            .map_err(|_| JsValue::from_str("Invalid mean bytes"))?;
        let variance_bytes: [u8; 8] = bytes[8..16].try_into()
            .map_err(|_| JsValue::from_str("Invalid variance bytes"))?;
        
        let mean = f64::from_le_bytes(mean_bytes);
        let variance = f64::from_le_bytes(variance_bytes);
        
        Self::new(mean, variance)
    }
    
    /// JavaScript toString method
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        format!("Rating(μ={:.2}, σ={:.2})", self.mean(), self.standard_deviation())
    }
    
    /// Compare with another rating (-1, 0, 1)
    #[wasm_bindgen(js_name = "compareTo")]
    pub fn compare_to(&self, other: &JsRatingInterface) -> i32 {
        let self_conservative = self.conservative_rating();
        let other_conservative = other.conservative_rating();
        
        if self_conservative < other_conservative {
            -1
        } else if self_conservative > other_conservative {
            1
        } else {
            0
        }
    }
}

/// JavaScript-friendly team interface
#[wasm_bindgen(js_name = "Team")]
#[derive(Clone)]
pub struct JsTeamInterface {
    players: Vec<JsRatingInterface>,
    metadata: HashMap<String, String>,
}

#[wasm_bindgen(js_class = "Team")]
impl JsTeamInterface {
    /// Creates a new empty team
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsTeamInterface {
        JsTeamInterface {
            players: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Adds a player to the team
    #[wasm_bindgen(js_name = "addPlayer")]
    pub fn add_player(&mut self, rating: JsRatingInterface) {
        self.players.push(rating);
    }
    
    /// Adds a player and returns self for chaining
    #[wasm_bindgen(js_name = "addPlayerChained")]
    pub fn add_player_chained(mut self, rating: JsRatingInterface) -> JsTeamInterface {
        self.players.push(rating);
        self
    }
    
    /// Gets the number of players
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.players.len()
    }
    
    /// Gets a player by index
    #[wasm_bindgen(js_name = "getPlayer")]
    pub fn get_player(&self, index: usize) -> Option<JsRatingInterface> {
        self.players.get(index).cloned()
    }
    
    /// Gets all players as a JavaScript array
    #[wasm_bindgen(js_name = "getAllPlayers")]
    pub fn get_all_players(&self) -> Array {
        let array = Array::new();
        for player in &self.players {
            array.push(&JsValue::from(player.clone()));
        }
        array
    }
    
    /// Gets total team mean
    #[wasm_bindgen(js_name = "totalMean")]
    pub fn total_mean(&self) -> f64 {
        self.players.iter().map(|p| p.mean()).sum()
    }
    
    /// Gets total team variance
    #[wasm_bindgen(js_name = "totalVariance")]
    pub fn total_variance(&self) -> f64 {
        self.players.iter().map(|p| p.variance()).sum()
    }
    
    /// Calculate team strength (conservative estimate)
    #[wasm_bindgen(js_name = "calculateStrength")]
    pub fn calculate_strength(&self) -> f64 {
        let mean = self.total_mean();
        let variance = self.total_variance();
        mean - 3.0 * variance.sqrt()
    }
    
    /// Set team metadata
    #[wasm_bindgen(js_name = "setMetadata")]
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get team metadata
    #[wasm_bindgen(js_name = "getMetadata")]
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
    
    /// JavaScript toString method
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        format!("Team({} players, strength: {:.1})", 
                self.size(), self.calculate_strength())
    }
}

/// JavaScript-friendly game outcome interface
#[wasm_bindgen(js_name = "GameOutcome")]
#[derive(Clone)]
pub struct JsGameOutcomeInterface {
    ranks: Vec<u32>,
}

#[wasm_bindgen(js_class = "GameOutcome")]
impl JsGameOutcomeInterface {
    /// Creates a win outcome
    #[wasm_bindgen(js_name = "createWin")]
    pub fn create_win(winner_index: usize, team_count: usize) -> Result<JsGameOutcomeInterface, JsValue> {
        if winner_index >= team_count {
            return Err(JsValue::from_str("Winner index out of bounds"));
        }
        if team_count < 2 {
            return Err(JsValue::from_str("At least 2 teams required"));
        }
        
        let mut ranks = vec![2u32; team_count];
        ranks[winner_index] = 1;
        
        Ok(JsGameOutcomeInterface { ranks })
    }
    
    /// Creates a draw outcome
    #[wasm_bindgen(js_name = "createDraw")]
    pub fn create_draw(team_count: usize) -> Result<JsGameOutcomeInterface, JsValue> {
        if team_count < 2 {
            return Err(JsValue::from_str("At least 2 teams required"));
        }
        
        Ok(JsGameOutcomeInterface {
            ranks: vec![1u32; team_count]
        })
    }
    
    /// Creates outcome from custom ranks
    #[wasm_bindgen(js_name = "fromRanks")]
    pub fn from_ranks(ranks: Vec<u32>) -> Result<JsGameOutcomeInterface, JsValue> {
        if ranks.is_empty() {
            return Err(JsValue::from_str("Ranks cannot be empty"));
        }
        
        // Validate no duplicate ranks
        let mut sorted_ranks = ranks.clone();
        sorted_ranks.sort_unstable();
        for window in sorted_ranks.windows(2) {
            if window[0] == window[1] {
                return Err(JsValue::from_str("Duplicate ranks not allowed"));
            }
        }
        
        Ok(JsGameOutcomeInterface { ranks })
    }
    
    /// Gets the number of teams
    #[wasm_bindgen(js_name = "teamCount")]
    pub fn team_count(&self) -> usize {
        self.ranks.len()
    }
    
    /// Gets the winner index (if any)
    #[wasm_bindgen(js_name = "getWinnerIndex")]
    pub fn get_winner_index(&self) -> Option<usize> {
        self.ranks.iter()
            .position(|&rank| rank == 1)
    }
    
    /// Gets rank for specific team
    #[wasm_bindgen(js_name = "getRank")]
    pub fn get_rank(&self, team_index: usize) -> u32 {
        self.ranks.get(team_index).copied().unwrap_or(u32::MAX)
    }
    
    /// Checks if outcome is a draw
    #[wasm_bindgen(js_name = "isDraw")]
    pub fn is_draw(&self) -> bool {
        self.ranks.iter().all(|&rank| rank == 1)
    }
    
    /// JavaScript toString method
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        format!("GameOutcome({:?})", self.ranks)
    }
}

/// Rating collection interface
#[wasm_bindgen(js_name = "RatingCollection")]
pub struct JsRatingCollectionInterface {
    items: Vec<JsRatingInterface>,
}

#[wasm_bindgen(js_class = "RatingCollection")]
impl JsRatingCollectionInterface {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }
    
    pub fn add(&mut self, item: JsRatingInterface) {
        self.items.push(item);
    }
    
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.items.len()
    }
    
    pub fn get(&self, index: usize) -> Option<JsRatingInterface> {
        self.items.get(index).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rating_interface_creation() {
        let rating = JsRatingInterface::new(1500.0, 200.0).unwrap();
        assert_eq!(rating.mean(), 1500.0);
        assert_eq!(rating.variance(), 200.0);
    }
    
    #[test]
    fn test_team_interface_operations() {
        let mut team = JsTeamInterface::new();
        let rating = JsRatingInterface::new(1500.0, 200.0).unwrap();
        team.add_player(rating);
        
        assert_eq!(team.size(), 1);
        assert_eq!(team.total_mean(), 1500.0);
    }
    
    #[test]
    fn test_outcome_interface_creation() {
        let outcome = JsGameOutcomeInterface::create_win(0, 3).unwrap();
        assert_eq!(outcome.get_winner_index(), Some(0));
        assert!(!outcome.is_draw());
        
        let draw = JsGameOutcomeInterface::create_draw(3).unwrap();
        assert!(draw.is_draw());
    }
    
    #[test]
    fn test_fluent_api() {
        let rating = JsRatingInterface::new(1500.0, 200.0).unwrap();
        let adjusted = rating.adjust_mean(50.0).adjust_variance(-20.0);
        
        assert_eq!(adjusted.mean(), 1550.0);
        assert_eq!(adjusted.variance(), 180.0);
    }
    
    #[test]
    fn test_binary_serialization() {
        let rating = JsRatingInterface::new(1500.0, 200.0).unwrap();
        let binary = rating.to_binary();
        let restored = JsRatingInterface::from_binary(&binary).unwrap();
        
        assert_eq!(restored.mean(), 1500.0);
        assert_eq!(restored.variance(), 200.0);
    }
}