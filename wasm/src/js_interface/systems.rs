//! Rating system interfaces for JavaScript
//!
//! Provides factory methods and system-specific interfaces.

use crate::js_interface::core::*;
use wasm_bindgen::prelude::*;
use js_sys::*;
use wasm_bindgen_futures::*;

/// Rating system factory for creating different algorithm implementations
#[wasm_bindgen(js_name = "RatingSystemFactory")]
pub struct JsRatingSystemFactory;

#[wasm_bindgen(js_class = "RatingSystemFactory")]
impl JsRatingSystemFactory {
    /// Creates an Elo rating system
    #[wasm_bindgen(js_name = "createElo")]
    pub fn create_elo(_config: JsEloConfigInterface) -> JsRatingSystemInterface {
        JsRatingSystemInterface::new("elo".to_string())
    }
    
    /// Creates a Glicko rating system
    #[wasm_bindgen(js_name = "createGlicko")]
    pub fn create_glicko(_config: JsGlickoConfigInterface) -> JsRatingSystemInterface {
        JsRatingSystemInterface::new("glicko".to_string())
    }
    
    /// Creates a TrueSkill rating system
    #[wasm_bindgen(js_name = "createTrueSkill")]
    pub fn create_trueskill(_config: JsTrueSkillConfigInterface) -> JsRatingSystemInterface {
        JsRatingSystemInterface::new("trueskill".to_string())
    }
    
    /// Get list of available rating systems
    #[wasm_bindgen(js_name = "getAvailableSystems")]
    pub fn get_available_systems() -> Array {
        let systems = Array::new();
        systems.push(&JsValue::from_str("elo"));
        systems.push(&JsValue::from_str("glicko"));
        systems.push(&JsValue::from_str("trueskill"));
        systems
    }
}

/// Generic rating system interface
#[wasm_bindgen(js_name = "RatingSystem")]
pub struct JsRatingSystemInterface {
    system_type: String,
}

#[wasm_bindgen(js_class = "RatingSystem")]
impl JsRatingSystemInterface {
    /// Creates a new rating system
    pub fn new(system_type: String) -> Self {
        Self { system_type }
    }
    
    /// Gets the system type
    #[wasm_bindgen(js_name = "getSystemType")]
    pub fn get_system_type(&self) -> String {
        self.system_type.clone()
    }
    
    /// Calculate match quality between teams
    #[wasm_bindgen(js_name = "calculateMatchQuality")]
    pub fn calculate_match_quality(&self, teams: Vec<JsTeamInterface>) -> f64 {
        // Simplified match quality calculation
        if teams.len() != 2 {
            return 0.0;
        }
        
        let team1_strength = teams[0].calculate_strength();
        let team2_strength = teams[1].calculate_strength();
        let strength_diff = (team1_strength - team2_strength).abs();
        
        // Convert to 0-1 scale (closer teams = higher quality)
        let max_diff = 500.0; // Reasonable maximum difference
        let quality = 1.0 - (strength_diff / max_diff).min(1.0);
        quality.max(0.0)
    }
    
    /// Update ratings synchronously (simplified version)
    #[wasm_bindgen(js_name = "updateRatings")]
    pub fn update_ratings(&self, teams: Vec<JsTeamInterface>, outcome: JsGameOutcomeInterface) -> Result<Array, JsValue> {
        // This is a simplified implementation
        // In a real implementation, this would delegate to the specific algorithm
        
        let updated_teams = Array::new();
        
        for (i, team) in teams.iter().enumerate() {
            let rank = outcome.get_rank(i);
            let mut updated_team = JsTeamInterface::new();
            
            // Simple rating adjustment based on rank
            for j in 0..team.size() {
                if let Some(player) = team.get_player(j) {
                    let rating_change = if rank == 1 { 25.0 } else { -25.0 };
                    let variance_change = -5.0; // Reduce uncertainty
                    
                    let updated_player = player
                        .adjust_mean(rating_change)
                        .adjust_variance(variance_change);
                    
                    updated_team.add_player(updated_player);
                }
            }
            
            updated_teams.push(&JsValue::from(updated_team));
        }
        
        Ok(updated_teams)
    }
    
    /// Create default rating for this system
    #[wasm_bindgen(js_name = "createDefaultRating")]
    pub fn create_default_rating(&self) -> Result<JsRatingInterface, JsValue> {
        match self.system_type.as_str() {
            "elo" => JsRatingInterface::new(1500.0, 300.0),
            "glicko" => JsRatingInterface::new(1500.0, 122500.0), // 350^2
            "trueskill" => JsRatingInterface::new(25.0, 69.44), // 8.333^2
            _ => Err(JsValue::from_str("Unknown system type"))
        }
    }
}

/// Configuration interface trait (for internal use)
pub trait JsConfigInterface {
    fn validate(&self) -> bool;
    fn get_system_type(&self) -> &str;
}

/// Elo configuration interface
#[wasm_bindgen(js_name = "EloConfig")]
pub struct JsEloConfigInterface {
    k_factor: f64,
    initial_rating: f64,
    initial_variance: f64,
}

#[wasm_bindgen(js_class = "EloConfig")]
impl JsEloConfigInterface {
    /// Creates new Elo configuration
    #[wasm_bindgen(constructor)]
    pub fn new(k_factor: f64, initial_rating: f64, initial_variance: f64) -> Self {
        Self {
            k_factor,
            initial_rating,
            initial_variance,
        }
    }
    
    /// Validates configuration
    pub fn validate(&self) -> bool {
        self.k_factor > 0.0 && 
        self.initial_variance > 0.0 &&
        self.initial_rating >= 0.0
    }
    
    #[wasm_bindgen(getter, js_name = "kFactor")]
    pub fn k_factor(&self) -> f64 {
        self.k_factor
    }
    
    #[wasm_bindgen(getter, js_name = "initialRating")]
    pub fn initial_rating(&self) -> f64 {
        self.initial_rating
    }
    
    #[wasm_bindgen(getter, js_name = "initialVariance")]
    pub fn initial_variance(&self) -> f64 {
        self.initial_variance
    }
}

impl JsConfigInterface for JsEloConfigInterface {
    fn validate(&self) -> bool {
        self.validate()
    }
    
    fn get_system_type(&self) -> &str {
        "elo"
    }
}

/// Glicko configuration interface
#[wasm_bindgen(js_name = "GlickoConfig")]
pub struct JsGlickoConfigInterface {
    initial_rating: f64,
    initial_deviation: f64,
    c: f64,
}

#[wasm_bindgen(js_class = "GlickoConfig")]
impl JsGlickoConfigInterface {
    /// Creates new Glicko configuration
    #[wasm_bindgen(constructor)]
    pub fn new(initial_rating: f64, initial_deviation: f64, c: f64) -> Self {
        Self {
            initial_rating,
            initial_deviation,
            c,
        }
    }
    
    /// Validates configuration
    pub fn validate(&self) -> bool {
        self.initial_deviation > 0.0 && 
        self.c > 0.0 &&
        self.initial_rating >= 0.0
    }
    
    #[wasm_bindgen(getter, js_name = "initialRating")]
    pub fn initial_rating(&self) -> f64 {
        self.initial_rating
    }
    
    #[wasm_bindgen(getter, js_name = "initialDeviation")]
    pub fn initial_deviation(&self) -> f64 {
        self.initial_deviation
    }
    
    #[wasm_bindgen(getter)]
    pub fn c(&self) -> f64 {
        self.c
    }
}

impl JsConfigInterface for JsGlickoConfigInterface {
    fn validate(&self) -> bool {
        self.validate()
    }
    
    fn get_system_type(&self) -> &str {
        "glicko"
    }
}

/// TrueSkill configuration interface
#[wasm_bindgen(js_name = "TrueSkillConfig")]
pub struct JsTrueSkillConfigInterface {
    initial_mean: f64,
    initial_std_dev: f64,
    beta: f64,
    tau: f64,
    draw_probability: f64,
}

#[wasm_bindgen(js_class = "TrueSkillConfig")]
impl JsTrueSkillConfigInterface {
    /// Creates new TrueSkill configuration
    #[wasm_bindgen(constructor)]
    pub fn new(
        initial_mean: f64,
        initial_std_dev: f64,
        beta: f64,
        tau: f64,
        draw_probability: f64,
    ) -> Self {
        Self {
            initial_mean,
            initial_std_dev,
            beta,
            tau,
            draw_probability,
        }
    }
    
    /// Validates configuration
    pub fn validate(&self) -> bool {
        self.initial_std_dev > 0.0 && 
        self.beta > 0.0 &&
        self.tau >= 0.0 &&
        self.draw_probability >= 0.0 &&
        self.draw_probability <= 1.0
    }
    
    #[wasm_bindgen(getter, js_name = "initialMean")]
    pub fn initial_mean(&self) -> f64 {
        self.initial_mean
    }
    
    #[wasm_bindgen(getter, js_name = "initialStdDev")]
    pub fn initial_std_dev(&self) -> f64 {
        self.initial_std_dev
    }
    
    #[wasm_bindgen(getter)]
    pub fn beta(&self) -> f64 {
        self.beta
    }
    
    #[wasm_bindgen(getter)]
    pub fn tau(&self) -> f64 {
        self.tau
    }
    
    #[wasm_bindgen(getter, js_name = "drawProbability")]
    pub fn draw_probability(&self) -> f64 {
        self.draw_probability
    }
}

impl JsConfigInterface for JsTrueSkillConfigInterface {
    fn validate(&self) -> bool {
        self.validate()
    }
    
    fn get_system_type(&self) -> &str {
        "trueskill"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_elo_config_validation() {
        let valid_config = JsEloConfigInterface::new(32.0, 1500.0, 300.0);
        assert!(valid_config.validate());
        
        let invalid_config = JsEloConfigInterface::new(-32.0, 1500.0, 300.0);
        assert!(!invalid_config.validate());
    }
    
    #[test]
    fn test_system_factory() {
        let systems = JsRatingSystemFactory::get_available_systems();
        assert_eq!(systems.length(), 3);
    }
    
    #[test]
    fn test_rating_system_creation() {
        let elo_config = JsEloConfigInterface::new(32.0, 1500.0, 300.0);
        let system = JsRatingSystemFactory::create_elo(elo_config);
        assert_eq!(system.get_system_type(), "elo");
    }
}