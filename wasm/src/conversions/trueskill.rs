//! TrueSkill-specific type conversions
//!
//! This module provides conversions between ladder_rs TrueSkill types
//! and JavaScript-friendly WASM types.

use crate::types::*;
use wasm_bindgen::prelude::*;

// Placeholder for TrueSkill conversions

/// Create default TrueSkill rating from config
pub fn create_default_trueskill_rating(config: &JsTrueSkillConfig) -> JsRating {
    let variance = config.initial_std_dev() * config.initial_std_dev();
    JsRating::new_unchecked(config.initial_mean(), variance)
}

/// Calculate conservative rating (μ - k*σ) for TrueSkill
pub fn calculate_conservative_rating(js_rating: &JsRating, k: f64) -> f64 {
    let std_dev = js_rating.variance().sqrt();
    js_rating.mean() - k * std_dev
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trueskill_config_usage() {
        let js_config = JsTrueSkillConfig::new(25.0, 8.333, 4.166, 0.083, 0.1);
        let default_rating = create_default_trueskill_rating(&js_config);
        assert_eq!(default_rating.mean(), 25.0);
        assert!((default_rating.variance() - 69.439).abs() < 0.01); // 8.333^2
    }
    
    #[test]
    fn test_conservative_rating() {
        let js_rating = JsRating::new_unchecked(25.0, 64.0); // std_dev 8.0
        let conservative = calculate_conservative_rating(&js_rating, 3.0);
        assert_eq!(conservative, 1.0); // 25.0 - 3.0 * 8.0
    }
}