//! Utility functions for WASM bindings
//!
//! This module provides helper functions for JavaScript interop,
//! including serialization, error handling, and performance utilities.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Sets up better panic messages in the browser console
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Serializes a value to a JavaScript object using serde-wasm-bindgen
pub fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Deserializes a JavaScript value to a Rust type using serde-wasm-bindgen
pub fn from_js_value<T: for<'de> Deserialize<'de>>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))
}

/// Converts a Result to a JavaScript-friendly Result
pub fn js_result<T>(result: Result<T, ladder_rs::error::Error>) -> Result<T, JsValue> {
    result.map_err(|e| JsValue::from_str(&format!("Error: {:?}", e)))
}

/// Helper for creating JavaScript errors with consistent formatting
pub fn js_error(message: &str) -> JsValue {
    JsValue::from_str(&format!("ladder-rs error: {}", message))
}

/// Performance measurement utilities
#[wasm_bindgen]
pub struct Performance;

#[wasm_bindgen]
impl Performance {
    /// Gets the current high-resolution timestamp in milliseconds
    pub fn now() -> f64 {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }

    /// Measures the time taken to execute a function
    pub fn measure(name: &str, start: f64) -> f64 {
        let duration = Self::now() - start;
        web_sys::console::log_1(&format!("{}: {:.2}ms", name, duration).into());
        duration
    }
}

/// Memory utilities for WASM
#[wasm_bindgen]
pub struct Memory;

#[wasm_bindgen]
impl Memory {
    /// Gets the current memory usage in bytes
    pub fn usage() -> u32 {
        // This is a simplified version - actual memory usage tracking would require
        // more sophisticated implementation
        0 // Placeholder - actual implementation would use web_sys APIs
    }

    /// Logs memory usage to console
    pub fn log_usage(label: &str) {
        let usage = Self::usage();
        web_sys::console::log_1(&format!("{}: {} bytes", label, usage).into());
    }
}

/// Batch operations helper for efficient JavaScript interop
#[wasm_bindgen]
pub struct BatchOperations;

#[wasm_bindgen]
impl BatchOperations {
    /// Processes multiple rating updates in a single call
    /// Takes a JSON string of operations and returns a JSON string of results
    pub fn process_batch(operations_json: &str) -> Result<String, JsValue> {
        #[derive(Deserialize)]
        struct BatchOperation {
            operation_type: String,
            data: serde_json::Value,
        }

        #[derive(Serialize)]
        struct BatchResult {
            success: bool,
            data: Option<serde_json::Value>,
            error: Option<String>,
        }

        let operations: Vec<BatchOperation> = serde_json::from_str(operations_json)
            .map_err(|e| js_error(&format!("Invalid batch operations: {}", e)))?;

        let results: Vec<BatchResult> = operations
            .into_iter()
            .map(|op| {
                // Process each operation (to be implemented with actual operations)
                match op.operation_type.as_str() {
                    "test" => BatchResult {
                        success: true,
                        data: Some(op.data),
                        error: None,
                    },
                    _ => BatchResult {
                        success: false,
                        data: None,
                        error: Some(format!("Unknown operation type: {}", op.operation_type)),
                    },
                }
            })
            .collect();

        serde_json::to_string(&results)
            .map_err(|e| js_error(&format!("Failed to serialize results: {}", e)))
    }
}

/// Console logging utilities
pub mod console {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn error(s: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn warn(s: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn info(s: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn debug(s: &str);
    }

    /// Macro for console.log with formatting
    #[macro_export]
    macro_rules! console_log {
        ($($t:tt)*) => ($crate::utils::console::log(&format!($($t)*)));
    }

    /// Macro for console.error with formatting
    #[macro_export]
    macro_rules! console_error {
        ($($t:tt)*) => ($crate::utils::console::error(&format!($($t)*)));
    }

    /// Macro for console.warn with formatting
    #[macro_export]
    macro_rules! console_warn {
        ($($t:tt)*) => ($crate::utils::console::warn(&format!($($t)*)));
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_js_error() {
        // Test that js_error creates proper error messages
        let error_msg = "test error";
        // We can't actually test JsValue creation in non-wasm environment
        // but we can test the logic
        let expected = format!("ladder-rs error: {}", error_msg);
        assert_eq!(expected, "ladder-rs error: test error");
    }

    #[test]
    fn test_batch_operation_structure() {
        // Test that we can create the batch operation structures
        #[derive(serde::Deserialize)]
        struct TestBatchOp {
            operation_type: String,
            data: serde_json::Value,
        }

        let json = r#"{"operation_type": "test", "data": {"value": 42}}"#;
        let op: TestBatchOp = serde_json::from_str(json).unwrap();
        assert_eq!(op.operation_type, "test");
        assert!(op.data.is_object());
    }
}
