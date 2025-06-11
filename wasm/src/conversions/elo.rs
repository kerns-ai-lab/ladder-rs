//! Elo-specific type conversions
//!
//! This module provides conversions between ladder_rs Elo types
//! and JavaScript-friendly WASM types.

use crate::types::*;

// Placeholder for Elo conversions - would require importing EloRating when available
// For now, we provide the interface definitions

/// Create default Elo rating from config
pub fn create_default_elo_rating(config: &JsEloConfig) -> JsRating {
    JsRating::new_unchecked(config.initial_rating(), config.initial_variance())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_elo_config_usage() {
        let js_config = JsEloConfig::new(32.0, 1500.0, 300.0);
        let default_rating = create_default_elo_rating(&js_config);
        assert_eq!(default_rating.mean(), 1500.0);
        assert_eq!(default_rating.variance(), 300.0);
    }
}