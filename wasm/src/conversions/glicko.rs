//! Glicko-specific type conversions
//!
//! This module provides conversions between ladder_rs Glicko types
//! and JavaScript-friendly WASM types.

use crate::types::*;
use wasm_bindgen::prelude::*;

// Placeholder for Glicko conversions

/// Create default Glicko rating from config
pub fn create_default_glicko_rating(config: &JsGlickoConfig) -> JsRating {
    let variance = config.initial_deviation() * config.initial_deviation();
    JsRating::new_unchecked(config.initial_rating(), variance)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glicko_config_usage() {
        let js_config = JsGlickoConfig::new(1500.0, 350.0, 15.0);
        let default_rating = create_default_glicko_rating(&js_config);
        assert_eq!(default_rating.mean(), 1500.0);
        assert_eq!(default_rating.variance(), 122500.0); // 350^2
    }
}