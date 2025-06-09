//! Data validation interface for JavaScript

use wasm_bindgen::prelude::*;

/// Data validator interface
#[wasm_bindgen(js_name = "DataValidator")]
pub struct JsDataValidatorInterface;

#[wasm_bindgen(js_class = "DataValidator")]
impl JsDataValidatorInterface {
    /// Creates a new validator
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }
    
    /// Validate rating parameters
    #[wasm_bindgen(js_name = "validateRating")]
    pub fn validate_rating(&self, mean: f64, variance: f64) -> bool {
        variance > 0.0 && mean.is_finite() && variance.is_finite()
    }
    
    /// Validate team size
    #[wasm_bindgen(js_name = "validateTeamSize")]
    pub fn validate_team_size(&self, size: usize) -> bool {
        size > 0 && size <= 100  // Reasonable limits
    }
    
    /// Validate outcome ranks
    #[wasm_bindgen(js_name = "validateOutcome")]
    pub fn validate_outcome(&self, ranks: &[u32]) -> bool {
        if ranks.is_empty() {
            return false;
        }
        
        let mut sorted = ranks.to_vec();
        sorted.sort_unstable();
        
        // Check for duplicates
        for window in sorted.windows(2) {
            if window[0] == window[1] {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rating_validation() {
        let validator = JsDataValidatorInterface::new();
        assert!(validator.validate_rating(1500.0, 200.0));
        assert!(!validator.validate_rating(1500.0, -100.0));
    }
}