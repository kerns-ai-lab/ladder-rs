//! Custom assertions for rating system tests

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Trait for values that can be compared in assertions
pub trait RatingValue {
    fn mean(&self) -> f64;
    fn variance(&self) -> Option<f64>;
}

/// Assert that two rating values are approximately equal
pub fn assert_ratings_approximately_equal<T: RatingValue>(
    rating1: &T,
    rating2: &T,
    tolerance: f64,
) -> Result<(), String> {
    let mean_diff = (rating1.mean() - rating2.mean()).abs();
    if mean_diff > tolerance {
        return Err(format!(
            "Mean values differ by {}, expected difference < {}",
            mean_diff, tolerance
        ));
    }

    if let (Some(var1), Some(var2)) = (rating1.variance(), rating2.variance()) {
        let var_diff = (var1 - var2).abs();
        if var_diff > tolerance {
            return Err(format!(
                "Variance values differ by {}, expected difference < {}",
                var_diff, tolerance
            ));
        }
    }

    Ok(())
}

/// Assertion helper for WASM tests
#[wasm_bindgen]
pub struct AssertionHelper;

#[wasm_bindgen]
impl AssertionHelper {
    /// Assert that two values are equal
    pub fn assert_equals(actual: &JsValue, expected: &JsValue, message: &str) -> Result<(), JsValue> {
        if actual != expected {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected {:?}, got {:?}",
                message, expected, actual
            )));
        }
        Ok(())
    }

    /// Assert that a value is truthy
    pub fn assert_truthy(value: &JsValue, message: &str) -> Result<(), JsValue> {
        if !value.is_truthy() {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected truthy value, got {:?}",
                message, value
            )));
        }
        Ok(())
    }

    /// Assert that a value is falsy
    pub fn assert_falsy(value: &JsValue, message: &str) -> Result<(), JsValue> {
        if value.is_truthy() {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected falsy value, got {:?}",
                message, value
            )));
        }
        Ok(())
    }

    /// Assert that an array contains a value
    pub fn assert_contains(array: &js_sys::Array, value: &JsValue, message: &str) -> Result<(), JsValue> {
        let length = array.length();
        for i in 0..length {
            if array.get(i) == *value {
                return Ok(());
            }
        }
        Err(JsValue::from_str(&format!(
            "Assertion failed: {}. Array does not contain {:?}",
            message, value
        )))
    }

    /// Assert that a number is within a range
    pub fn assert_in_range(value: f64, min: f64, max: f64, message: &str) -> Result<(), JsValue> {
        if value < min || value > max {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Value {} is not in range [{}, {}]",
                message, value, min, max
            )));
        }
        Ok(())
    }

    /// Assert that two numbers are approximately equal
    pub fn assert_approx_equals(actual: f64, expected: f64, tolerance: f64, message: &str) -> Result<(), JsValue> {
        let diff = (actual - expected).abs();
        if diff > tolerance {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected {} ± {}, got {} (difference: {})",
                message, expected, tolerance, actual, diff
            )));
        }
        Ok(())
    }

    /// Assert that an object has a property
    pub fn assert_has_property(obj: &js_sys::Object, property: &str, message: &str) -> Result<(), JsValue> {
        if !js_sys::Reflect::has(obj, &JsValue::from_str(property))? {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Object does not have property '{}'",
                message, property
            )));
        }
        Ok(())
    }

    /// Assert that a value is of a specific type
    pub fn assert_type(value: &JsValue, expected_type: &str, message: &str) -> Result<(), JsValue> {
        let actual_type = match () {
            _ if value.is_undefined() => "undefined",
            _ if value.is_null() => "null",
            _ if value.is_string() => "string",
            _ if value.as_f64().is_some() => "number",
            _ if value.is_object() => {
                if value.is_array() {
                    "array"
                } else if value.is_function() {
                    "function"
                } else {
                    "object"
                }
            }
            _ => "unknown",
        };

        if actual_type != expected_type {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected type '{}', got '{}'",
                message, expected_type, actual_type
            )));
        }
        Ok(())
    }

    /// Assert that an array has a specific length
    pub fn assert_array_length(array: &js_sys::Array, expected_length: u32, message: &str) -> Result<(), JsValue> {
        let actual_length = array.length();
        if actual_length != expected_length {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected array length {}, got {}",
                message, expected_length, actual_length
            )));
        }
        Ok(())
    }

    /// Assert that a string matches a pattern
    pub fn assert_matches(value: &str, pattern: &str, message: &str) -> Result<(), JsValue> {
        if !value.contains(pattern) {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. String '{}' does not match pattern '{}'",
                message, value, pattern
            )));
        }
        Ok(())
    }

    /// Assert that a value is greater than another
    pub fn assert_greater_than(actual: f64, expected: f64, message: &str) -> Result<(), JsValue> {
        if actual <= expected {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected {} > {}, but {} <= {}",
                message, actual, expected, actual, expected
            )));
        }
        Ok(())
    }

    /// Assert that a value is less than another
    pub fn assert_less_than(actual: f64, expected: f64, message: &str) -> Result<(), JsValue> {
        if actual >= expected {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected {} < {}, but {} >= {}",
                message, actual, expected, actual, expected
            )));
        }
        Ok(())
    }

    /// Assert that a function throws an error
    pub fn assert_throws(func: &js_sys::Function, message: &str) -> Result<(), JsValue> {
        match func.call0(&JsValue::NULL) {
            Ok(_) => Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Expected function to throw an error, but it didn't",
                message
            ))),
            Err(_) => Ok(()),
        }
    }

    /// Assert that two arrays are equal (order matters)
    pub fn assert_arrays_equal(actual: &js_sys::Array, expected: &js_sys::Array, message: &str) -> Result<(), JsValue> {
        if actual.length() != expected.length() {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Arrays have different lengths: {} vs {}",
                message, actual.length(), expected.length()
            )));
        }

        for i in 0..actual.length() {
            if actual.get(i) != expected.get(i) {
                return Err(JsValue::from_str(&format!(
                    "Assertion failed: {}. Arrays differ at index {}: {:?} vs {:?}",
                    message, i, actual.get(i), expected.get(i)
                )));
            }
        }

        Ok(())
    }

    /// Assert that two objects are deep equal
    pub fn assert_deep_equals(actual: &JsValue, expected: &JsValue, message: &str) -> Result<(), JsValue> {
        // Use JSON stringify for deep comparison
        let actual_json = js_sys::JSON::stringify(actual)?;
        let expected_json = js_sys::JSON::stringify(expected)?;
        
        if actual_json != expected_json {
            return Err(JsValue::from_str(&format!(
                "Assertion failed: {}. Objects are not deep equal",
                message
            )));
        }
        
        Ok(())
    }
}

/// Custom assertions for rating systems
pub struct RatingAssertions;

impl RatingAssertions {
    /// Assert that a rating is within expected bounds
    pub fn assert_rating_in_bounds(rating: f64, min: f64, max: f64) -> Result<(), String> {
        if rating < min || rating > max {
            return Err(format!(
                "Rating {} is outside expected bounds [{}, {}]",
                rating, min, max
            ));
        }
        Ok(())
    }

    /// Assert that ratings are properly ordered
    pub fn assert_ratings_ordered(ratings: &[f64], descending: bool) -> Result<(), String> {
        for i in 1..ratings.len() {
            let properly_ordered = if descending {
                ratings[i - 1] >= ratings[i]
            } else {
                ratings[i - 1] <= ratings[i]
            };

            if !properly_ordered {
                return Err(format!(
                    "Ratings not properly ordered at index {}: {} vs {}",
                    i, ratings[i - 1], ratings[i]
                ));
            }
        }
        Ok(())
    }

    /// Assert match quality is valid (0-1)
    pub fn assert_valid_match_quality(quality: f64) -> Result<(), String> {
        if quality < 0.0 || quality > 1.0 {
            return Err(format!(
                "Match quality {} is outside valid range [0, 1]",
                quality
            ));
        }
        Ok(())
    }

    /// Assert win probability is valid (0-1)
    pub fn assert_valid_probability(probability: f64) -> Result<(), String> {
        if probability < 0.0 || probability > 1.0 {
            return Err(format!(
                "Probability {} is outside valid range [0, 1]",
                probability
            ));
        }
        Ok(())
    }

    /// Assert that rating changes are reasonable
    pub fn assert_reasonable_rating_change(old_rating: f64, new_rating: f64, max_change: f64) -> Result<(), String> {
        let change = (new_rating - old_rating).abs();
        if change > max_change {
            return Err(format!(
                "Rating change {} exceeds maximum allowed change {}",
                change, max_change
            ));
        }
        Ok(())
    }
}