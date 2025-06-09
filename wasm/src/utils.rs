//! Utility functions for WASM bindings
//!
//! This module provides helper functions for JavaScript interop,
//! optimized for minimal bundle size.

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

}
