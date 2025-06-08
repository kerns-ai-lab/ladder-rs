//! Utility functions for WASM module
//!
//! This module contains helper functions and utilities for the WASM bindings.

use wasm_bindgen::prelude::*;

/// Set up panic hook for better error messages in the browser
#[wasm_bindgen]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Get the version of the ladder-rs-wasm package
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check if the WASM module is properly initialized
#[wasm_bindgen]
pub fn is_initialized() -> bool {
    true
}