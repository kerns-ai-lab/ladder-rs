//! WebAssembly bindings for the ladder-rs rating system
//!
//! This crate provides a unified JavaScript interface for all rating systems,
//! optimized for minimal WASM bundle size.

use wasm_bindgen::prelude::*;

// Import the `console.log` function for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Simplified logging macro
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Use `wee_alloc` as the global allocator for smaller WASM binary size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Set up panic hook for better error messages in the browser
#[cfg(feature = "console_error_panic_hook")]
fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn wasm_main() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    console_log!("ladder-rs WASM module initialized");
}

// Basic test function
#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}! Welcome to ladder-rs WASM.", name);
}

// Module declarations
pub mod api;
pub mod types;
pub mod utils;
pub mod js_interface;
pub mod conversions;
pub mod errors;
pub mod unified;

// Include implementation modules
mod unified_impl;
mod unified_methods;
mod unified_constructor;

// Re-export main API
pub use api::{WasmRating, WasmRatingSystem, WasmTeam};

// Re-export unified interface
pub use unified::{UnifiedRatingSystem, RatingSystemType, PlayerInfo, MatchResult};

// Re-export JavaScript interface types
pub use js_interface::*;

// Re-export error types
pub use errors::{WasmError, ErrorCode};