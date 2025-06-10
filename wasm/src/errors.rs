// Error Handling Framework for Task 1.2.5

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

/// Error severity levels
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Main error type for the rating system
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsRatingError {
    message: String,
    error_type: String,
    code: Option<String>,
    context: HashMap<String, String>,
    stack_trace: Option<String>,
    cause: Option<Box<JsRatingError>>,
    level: ErrorLevel,
    recovery_suggestion: Option<String>,
}

#[wasm_bindgen]
impl JsRatingError {
    /// Create a validation error
    #[wasm_bindgen(js_name = validationError)]
    pub fn validation_error(message: &str) -> JsRatingError {
        JsRatingError {
            message: message.to_string(),
            error_type: "ValidationError".to_string(),
            code: None,
            context: HashMap::new(),
            stack_trace: None,
            cause: None,
            level: ErrorLevel::Error,
            recovery_suggestion: None,
        }
    }

    /// Create a calculation error
    #[wasm_bindgen(js_name = calculationError)]
    pub fn calculation_error(message: &str) -> JsRatingError {
        JsRatingError {
            message: message.to_string(),
            error_type: "CalculationError".to_string(),
            code: None,
            context: HashMap::new(),
            stack_trace: None,
            cause: None,
            level: ErrorLevel::Error,
            recovery_suggestion: None,
        }
    }

    /// Create a configuration error
    #[wasm_bindgen(js_name = configurationError)]
    pub fn configuration_error(message: &str) -> JsRatingError {
        JsRatingError {
            message: message.to_string(),
            error_type: "ConfigurationError".to_string(),
            code: None,
            context: HashMap::new(),
            stack_trace: None,
            cause: None,
            level: ErrorLevel::Error,
            recovery_suggestion: None,
        }
    }

    /// Create a convergence error
    #[wasm_bindgen(js_name = convergenceError)]
    pub fn convergence_error(message: &str, iterations: u32) -> JsRatingError {
        let mut error = JsRatingError {
            message: message.to_string(),
            error_type: "ConvergenceError".to_string(),
            code: None,
            context: HashMap::new(),
            stack_trace: None,
            cause: None,
            level: ErrorLevel::Error,
            recovery_suggestion: None,
        };
        error.context.insert("iterations".to_string(), iterations.to_string());
        error
    }

    /// Get the error message
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Get the error type
    #[wasm_bindgen(getter, js_name = errorType)]
    pub fn error_type(&self) -> String {
        self.error_type.clone()
    }

    /// Get the error code if present
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> Option<String> {
        self.code.clone()
    }

    /// Get the error context
    #[wasm_bindgen(getter)]
    pub fn context(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.context).unwrap_or(JsValue::NULL)
    }

    /// Get the error level
    #[wasm_bindgen(getter)]
    pub fn level(&self) -> ErrorLevel {
        self.level
    }

    /// Get the recovery suggestion if present
    #[wasm_bindgen(getter, js_name = recoverySuggestion)]
    pub fn recovery_suggestion(&self) -> Option<String> {
        self.recovery_suggestion.clone()
    }

    /// Check if there is a cause
    #[wasm_bindgen(getter)]
    pub fn cause(&self) -> Option<JsRatingError> {
        self.cause.as_ref().map(|c| (**c).clone())
    }

    /// Add context to the error
    #[wasm_bindgen(js_name = withContext)]
    pub fn with_context(mut self, key: &str, value: &str) -> JsRatingError {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// Add an error code
    #[wasm_bindgen(js_name = withCode)]
    pub fn with_code(mut self, code: &str) -> JsRatingError {
        self.code = Some(code.to_string());
        self
    }

    /// Add a cause to the error
    #[wasm_bindgen(js_name = withCause)]
    pub fn with_cause(mut self, cause: Box<JsRatingError>) -> JsRatingError {
        self.cause = Some(cause);
        self
    }

    /// Set the error level
    #[wasm_bindgen(js_name = withLevel)]
    pub fn with_level(mut self, level: ErrorLevel) -> JsRatingError {
        self.level = level;
        self
    }

    /// Add a recovery suggestion
    #[wasm_bindgen(js_name = withRecoverySuggestion)]
    pub fn with_recovery_suggestion(mut self, suggestion: &str) -> JsRatingError {
        self.recovery_suggestion = Some(suggestion.to_string());
        self
    }

    /// Convert to a JavaScript value
    #[wasm_bindgen(js_name = toJsValue)]
    pub fn to_js_value(&self) -> JsValue {
        // Create a JavaScript Error-like object
        let obj = js_sys::Object::new();
        
        // Set standard Error properties
        js_sys::Reflect::set(&obj, &"message".into(), &self.message.clone().into()).unwrap();
        js_sys::Reflect::set(&obj, &"name".into(), &self.error_type.clone().into()).unwrap();
        
        // Set custom properties
        if let Some(code) = &self.code {
            js_sys::Reflect::set(&obj, &"code".into(), &code.clone().into()).unwrap();
        }
        
        js_sys::Reflect::set(&obj, &"level".into(), &format!("{:?}", self.level).into()).unwrap();
        
        if let Some(suggestion) = &self.recovery_suggestion {
            js_sys::Reflect::set(&obj, &"recoverySuggestion".into(), &suggestion.clone().into()).unwrap();
        }
        
        // Add context
        let context_obj = js_sys::Object::new();
        for (key, value) in &self.context {
            js_sys::Reflect::set(&context_obj, &key.clone().into(), &value.clone().into()).unwrap();
        }
        js_sys::Reflect::set(&obj, &"context".into(), &context_obj.into()).unwrap();
        
        // Add cause if present
        if let Some(cause) = &self.cause {
            js_sys::Reflect::set(&obj, &"cause".into(), &cause.to_js_value()).unwrap();
        }
        
        // Set toString method
        let to_string = js_sys::Function::new_with_args(
            "",
            &format!("return '{}: {}'", self.error_type, self.message)
        );
        js_sys::Reflect::set(&obj, &"toString".into(), &to_string.into()).unwrap();
        
        obj.into()
    }

    /// Convert to JSON string
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|_| {
            format!("{{\"error\": \"Failed to serialize error\", \"message\": \"{}\"}}", self.message)
        })
    }

    /// Log the error to console
    pub fn log(&self) {
        match self.level {
            ErrorLevel::Debug => web_sys::console::debug_1(&self.to_js_value()),
            ErrorLevel::Info => web_sys::console::info_1(&self.to_js_value()),
            ErrorLevel::Warning => web_sys::console::warn_1(&self.to_js_value()),
            ErrorLevel::Error => web_sys::console::error_1(&self.to_js_value()),
            ErrorLevel::Critical => {
                web_sys::console::error_1(&format!("CRITICAL: {}", self.to_json()).into());
            }
        }
    }
}

impl fmt::Display for JsRatingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for JsRatingError {}

/// Result type for operations that can succeed partially
#[wasm_bindgen]
#[derive(Clone)]
pub struct SafeMatchResult {
    success: bool,
    error: Option<JsRatingError>,
    result: Option<JsValue>,
}

#[wasm_bindgen]
impl SafeMatchResult {
    /// Create a successful result
    pub fn ok(result: JsValue) -> SafeMatchResult {
        SafeMatchResult {
            success: true,
            error: None,
            result: Some(result),
        }
    }

    /// Create a failed result
    pub fn err(error: JsRatingError) -> SafeMatchResult {
        SafeMatchResult {
            success: false,
            error: Some(error),
            result: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<JsRatingError> {
        self.error.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn result(&self) -> Option<JsValue> {
        self.result.clone()
    }
}

/// Result type for batch operations
#[wasm_bindgen]
pub struct BatchResult {
    results: Vec<SafeMatchResult>,
    successful_count: u32,
    failed_count: u32,
}

#[wasm_bindgen]
impl BatchResult {
    pub fn new(results: Vec<SafeMatchResult>) -> BatchResult {
        let successful_count = results.iter().filter(|r| r.success).count() as u32;
        let failed_count = results.len() as u32 - successful_count;
        
        BatchResult {
            results,
            successful_count,
            failed_count,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn results(&self) -> Vec<SafeMatchResult> {
        self.results.clone()
    }

    #[wasm_bindgen(getter, js_name = successfulCount)]
    pub fn successful_count(&self) -> u32 {
        self.successful_count
    }

    #[wasm_bindgen(getter, js_name = failedCount)]
    pub fn failed_count(&self) -> u32 {
        self.failed_count
    }

    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> u32 {
        self.results.len() as u32
    }
}

// Validation helper functions
pub fn validate_finite(value: f64, field_name: &str) -> Result<(), JsRatingError> {
    if !value.is_finite() {
        Err(JsRatingError::validation_error(&format!(
            "{} must be a finite number", field_name
        )).with_recovery_suggestion(&format!(
            "Ensure {} is not NaN or Infinity", field_name
        )))
    } else {
        Ok(())
    }
}

pub fn validate_positive(value: f64, field_name: &str) -> Result<(), JsRatingError> {
    if value <= 0.0 {
        Err(JsRatingError::validation_error(&format!(
            "{} must be positive", field_name
        )).with_recovery_suggestion(&format!(
            "Use a value greater than 0 for {}", field_name
        )))
    } else {
        Ok(())
    }
}

pub fn validate_finite_positive(value: f64, field_name: &str) -> Result<(), JsRatingError> {
    validate_finite(value, field_name)?;
    validate_positive(value, field_name)?;
    Ok(())
}

pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), JsRatingError> {
    if value.trim().is_empty() {
        Err(JsRatingError::validation_error(&format!(
            "{} cannot be empty", field_name
        )).with_recovery_suggestion(&format!(
            "Provide a non-empty value for {}", field_name
        )))
    } else {
        Ok(())
    }
}

pub fn validate_probability(value: f64, field_name: &str) -> Result<(), JsRatingError> {
    validate_finite(value, field_name)?;
    if value < 0.0 || value > 1.0 {
        Err(JsRatingError::validation_error(&format!(
            "{} must be between 0 and 1", field_name
        )).with_recovery_suggestion(&format!(
            "Use a value between 0.0 and 1.0 for {}", field_name
        )))
    } else {
        Ok(())
    }
}

// Conversion from ladder_rs errors
impl From<ladder_rs::error::LadderError> for JsRatingError {
    fn from(err: ladder_rs::error::LadderError) -> Self {
        match err {
            ladder_rs::error::LadderError::InvalidInput(msg) => {
                JsRatingError::validation_error(&msg)
            }
            ladder_rs::error::LadderError::InvalidMatchOutcome(msg) => {
                JsRatingError::validation_error(&msg)
                    .with_code("INVALID_OUTCOME")
            }
            ladder_rs::error::LadderError::ConvergenceFailed { iterations, .. } => {
                JsRatingError::convergence_error(
                    "Algorithm failed to converge within maximum iterations",
                    iterations
                )
            }
            ladder_rs::error::LadderError::ConfigurationError(msg) => {
                JsRatingError::configuration_error(&msg)
            }
            ladder_rs::error::LadderError::UnsupportedTeamSize(size) => {
                JsRatingError::validation_error(&format!(
                    "Team size {} is not supported", size
                )).with_recovery_suggestion(
                    "Use teams with at least 1 player each"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        // Test validation error
        let error = JsRatingError::validation_error("Invalid rating value");
        assert_eq!(error.error_type(), "ValidationError");
        assert!(error.message().contains("Invalid rating value"));
        
        // Test calculation error
        let error = JsRatingError::calculation_error("Division by zero");
        assert_eq!(error.error_type(), "CalculationError");
        
        // Test configuration error
        let error = JsRatingError::configuration_error("Invalid K-factor");
        assert_eq!(error.error_type(), "ConfigurationError");
        
        // Test convergence error
        let error = JsRatingError::convergence_error("Failed to converge", 100);
        assert_eq!(error.error_type(), "ConvergenceError");
        assert!(error.context.contains_key("iterations"));
    }

    #[test]
    fn test_error_context_and_chaining() {
        let base_error = JsRatingError::validation_error("Invalid player ID");
        let enhanced_error = base_error
            .with_context("player_id", "player_123")
            .with_context("operation", "create_player");
            
        assert_eq!(enhanced_error.context.get("player_id").unwrap(), "player_123");
        assert_eq!(enhanced_error.context.get("operation").unwrap(), "create_player");
        
        // Test error chaining
        let cause = JsRatingError::validation_error("Negative variance");
        let error = JsRatingError::calculation_error("Cannot calculate rating")
            .with_cause(Box::new(cause));
            
        assert!(error.cause().is_some());
    }

    #[test]
    fn test_error_levels() {
        let debug_error = JsRatingError::validation_error("Minor issue")
            .with_level(ErrorLevel::Debug);
        assert_eq!(debug_error.level(), ErrorLevel::Debug);
        
        let critical_error = JsRatingError::calculation_error("System failure")
            .with_level(ErrorLevel::Critical);
        assert_eq!(critical_error.level(), ErrorLevel::Critical);
    }

    #[test]
    fn test_recovery_suggestions() {
        let error = JsRatingError::validation_error("Invalid variance")
            .with_recovery_suggestion("Variance must be positive. Try using a value greater than 0.");
            
        assert!(error.recovery_suggestion().is_some());
        assert!(error.recovery_suggestion().unwrap().contains("positive"));
    }

    #[test]
    fn test_validation_helpers() {
        // Test finite validation
        assert!(validate_finite(1500.0, "rating").is_ok());
        assert!(validate_finite(f64::NAN, "rating").is_err());
        assert!(validate_finite(f64::INFINITY, "rating").is_err());
        
        // Test positive validation
        assert!(validate_positive(100.0, "variance").is_ok());
        assert!(validate_positive(0.0, "variance").is_err());
        assert!(validate_positive(-100.0, "variance").is_err());
        
        // Test finite positive validation
        assert!(validate_finite_positive(200.0, "variance").is_ok());
        assert!(validate_finite_positive(-100.0, "variance").is_err());
        assert!(validate_finite_positive(f64::INFINITY, "variance").is_err());
        
        // Test non-empty validation
        assert!(validate_non_empty("player1", "player_id").is_ok());
        assert!(validate_non_empty("", "player_id").is_err());
        assert!(validate_non_empty("   ", "player_id").is_err());
        
        // Test probability validation
        assert!(validate_probability(0.5, "draw_probability").is_ok());
        assert!(validate_probability(0.0, "draw_probability").is_ok());
        assert!(validate_probability(1.0, "draw_probability").is_ok());
        assert!(validate_probability(-0.1, "draw_probability").is_err());
        assert!(validate_probability(1.1, "draw_probability").is_err());
    }

    #[test]
    fn test_error_serialization() {
        let error = JsRatingError::validation_error("Test error")
            .with_context("field", "rating")
            .with_code("E001")
            .with_level(ErrorLevel::Warning);
            
        // Test JSON serialization
        let json = error.to_json();
        assert!(json.contains("ValidationError"));
        assert!(json.contains("Test error"));
        assert!(json.contains("E001"));
        assert!(json.contains("Warning"));
    }

    #[test]
    fn test_safe_match_result() {
        // Test successful result
        let ok_result = SafeMatchResult::ok(JsValue::from_str("success"));
        assert!(ok_result.success());
        assert!(ok_result.error().is_none());
        assert!(ok_result.result().is_some());
        
        // Test failed result
        let error = JsRatingError::validation_error("Test failure");
        let err_result = SafeMatchResult::err(error);
        assert!(!err_result.success());
        assert!(err_result.error().is_some());
        assert!(err_result.result().is_none());
    }

    #[test]
    fn test_batch_result() {
        let results = vec![
            SafeMatchResult::ok(JsValue::from_str("result1")),
            SafeMatchResult::err(JsRatingError::validation_error("error")),
            SafeMatchResult::ok(JsValue::from_str("result2")),
        ];
        
        let batch = BatchResult::new(results);
        assert_eq!(batch.total_count(), 3);
        assert_eq!(batch.successful_count(), 2);
        assert_eq!(batch.failed_count(), 1);
    }
}
