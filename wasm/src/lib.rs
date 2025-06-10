//! Optimized WebAssembly bindings for the ladder-rs Elo rating system
//!
//! This crate provides a lightweight JavaScript interface for Elo rating calculations,
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

    console_log!("ladder-rs WASM (Elo-optimized) module initialized");
}

// Basic test function
#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}! Welcome to ladder-rs Elo WASM.", name);
}

// Module declarations - minimal set
pub mod api;
pub mod types;
pub mod utils;
pub mod js_interface;
pub mod conversions;
pub mod errors;

// Re-export optimized API
pub use api::{WasmRating, WasmRatingSystem, WasmTeam};

// Re-export JavaScript interface types
pub use js_interface::*;