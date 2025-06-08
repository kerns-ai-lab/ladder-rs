//! WebAssembly bindings for the ladder-rs matchmaking library
//!
//! This crate provides JavaScript-accessible bindings for the core rating systems
//! implemented in ladder-rs, including Elo, Glicko, and TrueSkill algorithms.

use wasm_bindgen::prelude::*;

// Import the `console.log` function from the `console` module for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!(..)` style syntax for `console.log` logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Set up panic hook for better error messages in the browser
#[cfg(feature = "console_error_panic_hook")]
fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// Use `wee_alloc` as the global allocator for smaller WASM binary size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Initialize the WASM module - renamed to avoid conflict with test main
#[wasm_bindgen(start)]
pub fn wasm_main() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    console_log!("ladder-rs WASM module initialized");
}

// Basic test function to verify WASM bindings work
#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}! Welcome to ladder-rs WASM.", name);
}

// Re-export ladder-rs core types for internal use
pub use ladder_rs::core::{Outcome, Rating, RatingSystem, TeamRating};
pub use ladder_rs::error::Error as LadderError;

// Module declarations
pub mod api;
pub mod player_management;
pub mod test_utils;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use api::{WasmRating, WasmRatingSystem, WasmTeam};
pub use player_management::{
    HeadToHeadRecord, MatchRecord, PlayerManager, PlayerProfile, PlayerStats,
};
pub use test_utils::{
    AssertionHelper, BrowserEnvironment, MockDataGenerator, PerformanceTimer, TestFixture,
    TestLogger, TestSnapshot,
};
pub use types::{
    JsGameOutcome, JsRating, JsTeam, RatingSystemConfig, RatingSystemType, RatingUpdate,
};
